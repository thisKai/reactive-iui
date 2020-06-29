pub mod widgets;

pub use {
    codegen::view,
    widgets::{
        BoxedPrimitiveWidget, Button, Component, ComponentWidget, Group, HorizontalBox,
        PrimitiveWidget, SingleChildParent, SingleChildParentControl, Parent,
    },
};
use {
    fragile::Fragile,
    iui::{controls, UI},
    legion::prelude::*,
    std::sync::mpsc::channel,
    widgets::{ComponentState, EventReceiver, EventSender},
};

pub struct App<C: Component + 'static> {
    ctx: UI,
    window: controls::Window,
    root_component: C,
}
impl<C: Component + 'static> App<C> {
    pub fn new(root_component: C) -> Self {
        let ctx = UI::init().expect("Couldn't initialize UI library");

        let window =
            controls::Window::new(&ctx, "Test App", 200, 200, controls::WindowType::NoMenubar);

        Self {
            ctx,
            window,
            root_component,
        }
    }

    pub fn run(self) {
        let Self {
            ctx,
            mut window,
            mut root_component,
        } = self;

        let universe = Universe::new();
        let mut world = universe.create_world();

        let (sender, receiver) = channel();

        window.set_child(
            &ctx,
            root_component
                .view()
                .create_control(&ctx, &mut world, sender.clone())
                .1,
        );
        window.show(&ctx);

        let mut event_loop = ctx.event_loop();

        event_loop.on_tick(&ctx, {
            let ctx = ctx.clone();

            move || {
                if let Ok(event) = receiver.try_recv() {
                    event.handle(&mut root_component);

                    window.set_child(
                        &ctx,
                        root_component
                            .view()
                            .create_control(&ctx, &mut world, sender.clone())
                            .1,
                    );
                }

                let components = <(
                    Write<ComponentState>,
                    Read<Fragile<EventSender>>,
                    Read<Fragile<EventReceiver>>,
                    Write<Fragile<controls::Control>>,
                    Write<Fragile<Box<dyn SingleChildParentControl>>>,
                )>::query();

                let mut updates = vec![];

                for (entity, (mut state, event_sender, event_receiver, control, parent_control)) in
                    components.iter_entities_mut(&mut world)
                {
                    if let Ok(event) = event_receiver.get().try_recv() {
                        event.handle(state.edit());
                        updates.push((
                            entity,
                            state.view(),
                            event_sender.get().clone(),
                            control.get().clone(),
                            parent_control.get().box_clone(),
                        ));
                    }
                }

                for (entity, view, event_sender, control, mut parent_control) in updates {
                    let (new_entity, new_control) =
                        view.create_control(&ctx, &mut world, event_sender);
                    parent_control.set_child(&ctx, new_control.clone());
                    unsafe { control.destroy() }
                    let _ = world.add_component(entity, Fragile::new(new_control));
                }
            }
        });

        event_loop.run(&ctx);
    }
}

struct NeedsUpdate;
