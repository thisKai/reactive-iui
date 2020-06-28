pub use codegen::view;
use {
    fragile::Fragile,
    iui::{controls, UI},
    legion::prelude::*,
    std::any::Any,
};

pub trait Component {
    fn view(&self) -> Box<dyn BaseVirtualControl>;
}

pub struct App<C: Component + 'static> {
    ctx: UI,
    window: controls::Window,
    component: C,
}
impl<C: Component + 'static> App<C> {
    pub fn new(component: C) -> Self {
        let ctx = UI::init().expect("Couldn't initialize UI library");

        let window =
            controls::Window::new(&ctx, "Test App", 200, 200, controls::WindowType::NoMenubar);

        Self {
            ctx,
            window,
            component,
        }
    }

    pub fn run(self) {
        let Self {
            ctx,
            mut window,
            component,
        } = self;

        let universe = Universe::new();
        let mut world = universe.create_world();

        window.set_child(&ctx, component.view().create_control(&ctx, &mut world));
        window.show(&ctx);

        ctx.main();
    }
}

pub trait VirtualControl: Any + PartialEq {
    type Control: Clone + Into<controls::Control>;

    type Event;

    fn boxed(self) -> Box<dyn BaseVirtualControl>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity;

    fn create_control<'a, 'b>(&'a self, ctx: &UI, world: &'b mut World) -> Self::Control {
        let entity = self.create_entity(ctx, world);
        world.get_component::<Fragile<Self::Control>>(entity).unwrap().get().clone()
    }
}
pub trait BaseVirtualControl {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn BaseVirtualControl) -> bool;
    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity;
    fn create_control(&self, ctx: &UI, world: &mut World) -> controls::Control;
}
impl<T: VirtualControl> BaseVirtualControl for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn BaseVirtualControl) -> bool {
        other.as_any().downcast_ref() == Some(self)
    }
    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity {
        VirtualControl::create_entity(self, ctx, world)
    }
    fn create_control(&self, ctx: &UI, world: &mut World) -> controls::Control {
        VirtualControl::create_control(self, ctx, world).into()
    }
}
impl PartialEq for dyn BaseVirtualControl {
    fn eq(&self, other: &Self) -> bool {
        BaseVirtualControl::eq(self, other)
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
impl<V: VirtualControl, SelfTy: PartialEq + 'static> VirtualControl for Handler<V, SelfTy> {
    type Control = V::Control;

    type Event = V::Event;

    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity {
        self.child.create_entity(ctx, world)
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

    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity {
        let parent = self.parent.create_entity(ctx, world);
        let child = self.child.create_entity(ctx, world);

        let parent_control = world.get_component::<Fragile<P::Control>>(parent).unwrap();
        let child_control = world.get_component::<Fragile<C::Control>>(child).unwrap();

        parent_control.get().clone().set_child(ctx, child_control.get().clone());

        parent
    }
}

pub struct Clicked;

#[derive(Clone, PartialEq)]
pub struct Button {
    pub text: String,
}
impl Button {
    #[allow(non_upper_case_globals)]
    pub const Clicked: Clicked = Clicked;

    pub fn on_click<SelfTy>(self, handler: fn(&mut SelfTy)) -> Handler<Self, SelfTy> {
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

    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity {
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

    fn create_entity(&self, ctx: &UI, world: &mut World) -> Entity {
        let mut group = controls::Group::new(ctx, &self.title);

        group.set_margined(ctx, self.margined);

        let entity = world.insert((), Some((self.clone(), Fragile::new(group))))[0];

        entity
    }
}
