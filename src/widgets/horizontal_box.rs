use super::*;

#[derive(Clone, PartialEq)]
pub struct HorizontalBox {
    pub padded: bool,
}
impl Parent<HorizontalBox> for HorizontalBox {
    type List = Hlist!();

    fn child<C>(self, child: C) -> ConnectChildren<Self, Hlist!(C)>
    where
        Self: Sized,
    {
        ConnectChildren {
            parent: self,
            children: hlist!(child),
        }
    }
}
impl PrimitiveWidget for HorizontalBox {
    type Control = controls::HorizontalBox;

    type Event = ();

    fn create_entity(&self, ctx: &UI, world: &mut World, _event_sender: EventSender) -> Entity {
        let mut horizontal_box = controls::HorizontalBox::new(ctx);

        horizontal_box.set_padded(ctx, self.padded);

        let entity = world.insert((), Some((self.clone(), Fragile::new(horizontal_box))))[0];

        entity
    }
}

impl ParentControl for controls::HorizontalBox {
    fn append_child(&mut self, ctx: &UI, child: controls::Control) {
        controls::HorizontalBox::append(self, ctx, child, controls::LayoutStrategy::Stretchy);
    }
    fn box_clone(&self) -> Box<dyn ParentControl> {
        Box::new(Clone::clone(self))
    }
}
