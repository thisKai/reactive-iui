pub use codegen::view;
use {
    fragile::Fragile,
    iui::{controls, UI},
    legion::prelude::*,
    std::{any::Any, sync::mpsc::{channel, Sender}},
};

pub trait Component {
    fn view(&self) -> Box<dyn BaseVirtualControl>;
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

        window.set_child(&ctx, root_component.view().create_control(&ctx, &mut world, sender.clone()));
        window.show(&ctx);

        let mut event_loop = ctx.event_loop();

        event_loop.on_tick(&ctx, {
            let ctx = ctx.clone();

            move || {
                if let Ok(event) = receiver.try_recv() {
                    event.handle(&mut root_component);

                    window.set_child(&ctx, root_component.view().create_control(&ctx, &mut world, sender.clone()));
                }
            }
        });

        event_loop.run(&ctx);
    }
}

pub trait VirtualControl: Any + PartialEq {
    type Control: Clone + Into<controls::Control>;

    type Event: Clone;

    fn boxed(self) -> Box<dyn BaseVirtualControl>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity;

    fn create_control<'a, 'b>(&'a self, ctx: &UI, world: &'b mut World, event_sender: EventSender) -> Self::Control {
        let entity = self.create_entity(ctx, world, event_sender);
        world
            .get_component::<Fragile<Self::Control>>(entity)
            .unwrap()
            .get()
            .clone()
    }
}
pub trait BaseVirtualControl {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn BaseVirtualControl) -> bool;
    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity;
    fn create_control(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> controls::Control;
}
impl<T: VirtualControl> BaseVirtualControl for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn BaseVirtualControl) -> bool {
        other.as_any().downcast_ref() == Some(self)
    }
    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity {
        VirtualControl::create_entity(self, ctx, world, event_sender)
    }
    fn create_control(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> controls::Control {
        VirtualControl::create_control(self, ctx, world, event_sender).into()
    }
}
impl PartialEq for dyn BaseVirtualControl {
    fn eq(&self, other: &Self) -> bool {
        BaseVirtualControl::eq(self, other)
    }
}

pub struct Event {
    handler: Box<dyn HandleEvent + Send + Sync + 'static>,
}
impl Event {
    fn new<SelfTy: 'static>(handler: fn(&mut SelfTy)) -> Self {
        Self {
            handler: Box::new(TypedEvent { handler }),
        }
    }
    fn handle(&self, component: &mut dyn Any) {
        self.handler.handle(component)
    }
}

type EventSender = Sender<Event>;

pub trait HandleEvent {
    fn handle(&self, component: &mut dyn Any);
}
pub struct TypedEvent<SelfTy: 'static> {
    pub handler: fn(&mut SelfTy),
}
impl<SelfTy: Any> TypedEvent<SelfTy> {
    fn handle_typed(&self, component: &mut SelfTy) {
        (self.handler)(component)
    }
}
impl<SelfTy: Any> HandleEvent for TypedEvent<SelfTy> {
    fn handle(&self, component: &mut dyn Any) {
        self.handle_typed(component.downcast_mut().unwrap())
    }
}

pub trait ControlEventListener<V: VirtualControl, SelfTy> {
    fn on_event(
        &mut self,
        ctx: &UI,
        event: V::Event,
        handler: fn(&mut SelfTy),
        event_sender: EventSender,
    );
}
impl<SelfTy: 'static> ControlEventListener<Button, SelfTy> for controls::Button {
    fn on_event(
        &mut self,
        ctx: &UI,
        _event: Clicked,
        handler: fn(&mut SelfTy),
        event_sender: EventSender,
    ) {
        self.on_clicked(ctx, move |_| {
            let _ = event_sender.send(Event::new(handler));
        });
    }
}

#[derive(Clone)]
pub struct Handler<V: VirtualControl, SelfTy> {
    pub handler: fn(&mut SelfTy),
    pub event: V::Event,
    pub child: V,
}
impl<V: VirtualControl, SelfTy> PartialEq for Handler<V, SelfTy> {
    fn eq(&self, other: &Self) -> bool {
        self.child == other.child
    }
}
impl<V, SelfTy> VirtualControl for Handler<V, SelfTy>
where
    V: VirtualControl,
    SelfTy: PartialEq + 'static,
    V::Control: ControlEventListener<V, SelfTy>,
{
    type Control = V::Control;

    type Event = V::Event;

    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity {
        let entity = self.child.create_entity(ctx, world, event_sender.clone());

        let mut control = world
            .get_component::<Fragile<Self::Control>>(entity)
            .unwrap()
            .get()
            .clone();

        control.on_event(&ctx, self.event.clone(), self.handler, event_sender);

        entity
    }
}

pub trait SingleChildParent: VirtualControl
where
    Self::Control: SingleChildParentControl,
{
    fn child<C>(self, child: C) -> ConnectSingleChild<Self, C>
    where
        Self: Sized;
}
pub trait SingleChildParentControl: Into<controls::Control> {
    fn set_child<C: Into<controls::Control>>(&mut self, ctx: &UI, child: C);
}
impl SingleChildParentControl for controls::Group {
    fn set_child<C: Into<controls::Control>>(&mut self, ctx: &UI, child: C) {
        controls::Group::set_child(self, ctx, child)
    }
}

#[derive(Clone)]
pub struct ConnectSingleChild<P, C> {
    parent: P,
    child: C,
}
impl<P: PartialEq, C> PartialEq for ConnectSingleChild<P, C> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent
    }
}
impl<P, C> VirtualControl for ConnectSingleChild<P, C>
where
    P: SingleChildParent + VirtualControl,
    C: VirtualControl,
    P::Control: SingleChildParentControl,
{
    type Control = P::Control;

    type Event = P::Event;

    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity {
        let parent = self.parent.create_entity(ctx, world, event_sender.clone());
        let child = self.child.create_entity(ctx, world, event_sender);

        let parent_control = world.get_component::<Fragile<P::Control>>(parent).unwrap();
        let child_control = world.get_component::<Fragile<C::Control>>(child).unwrap();

        parent_control
            .get()
            .clone()
            .set_child(ctx, child_control.get().clone());

        parent
    }
}

#[derive(Copy, Clone)]
pub struct Clicked;

#[derive(Clone, PartialEq)]
pub struct Button {
    pub text: String,
}
impl Button {
    #[allow(non_upper_case_globals)]
    pub const Clicked: Clicked = Clicked;

    pub fn on_clicked<SelfTy>(self, handler: fn(&mut SelfTy)) -> Handler<Self, SelfTy> {
        Handler {
            child: self,
            event: Self::Clicked,
            handler,
        }
    }
}
impl VirtualControl for Button {
    type Control = controls::Button;

    type Event = Clicked;

    fn create_entity(&self, ctx: &UI, world: &mut World, _event_sender: EventSender) -> Entity {
        let button = Fragile::new(controls::Button::new(ctx, &self.text));

        let entity = world.insert((), Some((self.clone(), button)))[0];

        entity
    }
}

#[derive(Clone, PartialEq)]
pub struct Group {
    pub title: String,
    pub margined: bool,
}
impl SingleChildParent for Group {
    fn child<C>(self, child: C) -> ConnectSingleChild<Self, C>
    where
        Self: Sized,
    {
        ConnectSingleChild {
            parent: self,
            child,
        }
    }
}
impl VirtualControl for Group {
    type Control = controls::Group;

    type Event = ();

    fn create_entity(&self, ctx: &UI, world: &mut World, _event_sender: EventSender) -> Entity {
        let mut group = controls::Group::new(ctx, &self.title);

        group.set_margined(ctx, self.margined);

        let entity = world.insert((), Some((self.clone(), Fragile::new(group))))[0];

        entity
    }
}
