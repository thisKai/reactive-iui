use {super::*, std::sync::mpsc::channel};

pub trait Component: as_any::AsAny {
    fn view(&self) -> Box<dyn BoxedPrimitiveWidget>;
}
impl as_any::Downcast for dyn Component {}
impl as_any::Downcast for dyn Component + Send {}
impl as_any::Downcast for dyn Component + Sync {}
impl as_any::Downcast for dyn Component + Send + Sync {}

#[derive(Clone, PartialEq)]
pub struct ComponentWidget<C: Component + Any + Clone + PartialEq>(pub C);
impl<C: Component + Any + Clone + PartialEq> ComponentWidget<C> {
    fn state(&self) -> ComponentState {
        ComponentState(Fragile::new(Box::new(self.0.clone())))
    }
}
impl<C: Component + Any + Clone + PartialEq> PrimitiveWidget for ComponentWidget<C> {
    type Control = controls::Control;

    type Event = ();

    fn create_entity(&self, ctx: &UI, world: &mut World, _event_sender: EventSender) -> Entity {
        let (sender, receiver) = channel();
        let (root_entity, root_control) = self.0.view().create_control(ctx, world, sender.clone());
        let entity = world.insert(
            (),
            Some((
                self.state(),
                Root(root_entity),
                Fragile::new(root_control),
                Fragile::new(sender.clone()),
                Fragile::new(receiver),
            )),
        )[0];

        entity
    }
}
pub struct ComponentState(Fragile<Box<dyn Component>>);
impl ComponentState {
    pub fn edit(&mut self) -> &mut dyn Component {
        &mut **self.0.get_mut()
    }
    pub fn view(&self) -> Box<dyn BoxedPrimitiveWidget> {
        self.0.get().view()
    }
}
pub struct Root(Entity);
