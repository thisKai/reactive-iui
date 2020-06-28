use super::*;

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

#[derive(Copy, Clone)]
pub struct Clicked;

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
