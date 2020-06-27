use {
    iui::{controls, UI},
    std::any::Any,
};

pub trait VirtualControl: Any + PartialEq {
    type Control: Into<controls::Control>;

    type UpdateCtx;

    const TYPE_NAME: &'static str;

    fn boxed(self) -> Box<dyn BaseVirtualControl> where Self: Sized {
        Box::new(self)
    }

    fn create(&self, ctx: &UI) -> (Self::Control, Self::UpdateCtx);

    fn update(
        &self,
        previous: Self,
        control: &mut Self::Control,
        update_ctx: Self::UpdateCtx,
        ctx: &UI,
    ) -> Self::UpdateCtx;
}
pub trait BaseVirtualControl {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn BaseVirtualControl) -> bool;
    fn type_name(&self) -> &'static str;
    fn control(&self, ctx: &UI) -> controls::Control;
}
impl<T: VirtualControl> BaseVirtualControl for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn eq(&self, other: &dyn BaseVirtualControl) -> bool {
        other.as_any().downcast_ref() == Some(self)
    }
    fn type_name(&self) -> &'static str {
        <Self as VirtualControl>::TYPE_NAME
    }
    fn control(&self, ctx: &UI) -> controls::Control {
        self.create(ctx).0.into()
    }
}
impl PartialEq for dyn BaseVirtualControl {
    fn eq(&self, other: &Self) -> bool {
        BaseVirtualControl::eq(self, other)
    }
}

#[derive(PartialEq)]
pub struct Button {
    pub text: String,
}
impl VirtualControl for Button {
    type Control = controls::Button;

    type UpdateCtx = ();

    const TYPE_NAME: &'static str = "Button";

    fn create(&self, ctx: &UI) -> (Self::Control, Self::UpdateCtx) {
        (controls::Button::new(ctx, &self.text), ())
    }

    fn update(
        &self,
        previous: Self,
        control: &mut Self::Control,
        _update_ctx: Self::UpdateCtx,
        ctx: &UI,
    ) -> Self::UpdateCtx {
        if self.text != previous.text {
            control.set_text(ctx, &self.text);
        }
    }
}

#[derive(PartialEq)]
pub struct Group {
    pub title: String,
    pub margined: bool,
    pub child: Option<Box<dyn BaseVirtualControl>>,
}
impl VirtualControl for Group {
    type Control = controls::Group;

    type UpdateCtx = GroupUpdateCtx;

    const TYPE_NAME: &'static str = "Group";

    fn create(&self, ctx: &UI) -> (Self::Control, Self::UpdateCtx) {
        let mut group = controls::Group::new(ctx, &self.title);

        group.set_margined(ctx, self.margined);

        let update_ctx = GroupUpdateCtx {
            child: self.child.as_ref().map(|child| child.control(ctx)),
        };
        if let Some(child) = &update_ctx.child {
            group.set_child(ctx, child.clone());
        }

        (group, update_ctx)
    }

    fn update(
        &self,
        previous: Self,
        control: &mut Self::Control,
        update_ctx: Self::UpdateCtx,
        ctx: &UI,
    ) -> Self::UpdateCtx {
        if self.title != previous.title {
            control.set_title(ctx, &self.title);
        }
        if self.margined != previous.margined {
            control.set_margined(ctx, self.margined);
        }
        if self.child == previous.child {
            update_ctx
        } else {
            let new_update_ctx = GroupUpdateCtx {
                child: self.child.as_ref().map(|child| child.control(ctx)),
            };
            match new_update_ctx.child.as_ref() {
                Some(child) => {
                    control.set_child(ctx, child.clone());
                }
                None => {
                    if let Some(child) = update_ctx.child {
                        unsafe {
                            child.destroy();
                        }
                    }
                }
            }
            new_update_ctx
        }
    }
}

pub struct GroupUpdateCtx {
    child: Option<controls::Control>,
}
