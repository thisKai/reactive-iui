use {
    fragile::Fragile,
    iui::{controls, UI},
    legion::prelude::*,
    std::{any::Any, sync::mpsc::Sender},
};

pub trait PrimitiveWidget: Any + PartialEq {
    type Control: Clone + Into<controls::Control>;

    type Event: Clone;

    fn boxed(self) -> Box<dyn BoxedPrimitiveWidget>
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
pub trait BoxedPrimitiveWidget {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn BoxedPrimitiveWidget) -> bool;
    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity;
    fn create_control(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> controls::Control;
}
impl<T: PrimitiveWidget> BoxedPrimitiveWidget for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn BoxedPrimitiveWidget) -> bool {
        other.as_any().downcast_ref() == Some(self)
    }
    fn create_entity(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> Entity {
        PrimitiveWidget::create_entity(self, ctx, world, event_sender)
    }
    fn create_control(&self, ctx: &UI, world: &mut World, event_sender: EventSender) -> controls::Control {
        PrimitiveWidget::create_control(self, ctx, world, event_sender).into()
    }
}
impl PartialEq for dyn BoxedPrimitiveWidget {
    fn eq(&self, other: &Self) -> bool {
        BoxedPrimitiveWidget::eq(self, other)
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
    pub fn handle(&self, component: &mut dyn Any) {
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

pub trait ControlEventListener<V: PrimitiveWidget, SelfTy> {
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
pub struct Handler<V: PrimitiveWidget, SelfTy> {
    pub handler: fn(&mut SelfTy),
    pub event: V::Event,
    pub child: V,
}
impl<V: PrimitiveWidget, SelfTy> PartialEq for Handler<V, SelfTy> {
    fn eq(&self, other: &Self) -> bool {
        self.child == other.child
    }
}
impl<V, SelfTy> PrimitiveWidget for Handler<V, SelfTy>
where
    V: PrimitiveWidget,
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

pub trait SingleChildParent: PrimitiveWidget
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
impl<P, C> PrimitiveWidget for ConnectSingleChild<P, C>
where
    P: SingleChildParent + PrimitiveWidget,
    C: PrimitiveWidget,
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
impl PrimitiveWidget for Button {
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
impl PrimitiveWidget for Group {
    type Control = controls::Group;

    type Event = ();

    fn create_entity(&self, ctx: &UI, world: &mut World, _event_sender: EventSender) -> Entity {
        let mut group = controls::Group::new(ctx, &self.title);

        group.set_margined(ctx, self.margined);

        let entity = world.insert((), Some((self.clone(), Fragile::new(group))))[0];

        entity
    }
}
