pub mod widgets;

pub use {
    codegen::view,
    widgets::{BoxedPrimitiveWidget, PrimitiveWidget, Button, Group, SingleChildParent},
};
use {
    iui::{controls, UI},
    legion::prelude::*,
    std::sync::mpsc::channel,
};

pub trait Component {
    fn view(&self) -> Box<dyn BoxedPrimitiveWidget>;
}

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
                .create_control(&ctx, &mut world, sender.clone()),
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
                            .create_control(&ctx, &mut world, sender.clone()),
                    );
                }
            }
        });

        event_loop.run(&ctx);
    }
}
