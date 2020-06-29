use reactive_iui::*;

// struct Main {
//     is_true: bool,
// }
// impl Main {
//     view! {
//         Group {
//             title: "state",
//             margined: true,
//             child: Button {
//                 text: self.is_true.to_string(),
//                 #[on] clicked: Self::on_clicked,
//             },
//         }
//     }
//     fn on_clicked(&mut self) {
//         self.is_true = !self.is_true;
//     }
// }

#[derive(Clone, PartialEq)]
struct Main;
impl Component for Main {
    fn view(&self) -> Box<dyn BoxedPrimitiveWidget> {
        Group {
            title: String::from("Hello"),
            margined: true,
        }
        .child(
            HorizontalBox { padded: true }
                .child(ComponentWidget(BooleanButton { value: false }))
                .child(ComponentWidget(BooleanButton { value: false })),
        )
        .boxed()
    }
}

#[derive(Clone, PartialEq)]
struct BooleanButton {
    value: bool,
}
impl BooleanButton {
    fn on_clicked(&mut self) {
        self.value = !self.value;
    }
}
impl Component for BooleanButton {
    fn view(&self) -> Box<dyn BoxedPrimitiveWidget> {
        Button {
            text: self.value.to_string(),
        }
        .on_clicked(Self::on_clicked)
        .boxed()
    }
}

fn main() {
    App::new(Main).run();
}
