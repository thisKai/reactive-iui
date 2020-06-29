use super::*;

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

impl SingleChildParentControl for controls::Group {
    fn set_child(&mut self, ctx: &UI, child: controls::Control) {
        controls::Group::set_child(self, ctx, child)
    }
    fn box_clone(&self) -> Box<dyn SingleChildParentControl> {
        Box::new(Clone::clone(self))
    }
}
