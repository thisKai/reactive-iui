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

#[derive(PartialEq)]
struct Main {
    is_true: bool,
}
impl Main {
    fn on_clicked(&mut self) {
        self.is_true = !self.is_true;
    }
}
impl Component for Main {
    fn view(&self) -> Box<dyn BaseVirtualControl> {
        Group {
            title: String::from("Hello"),
            margined: true,
        }
        .child(Handler {
            child: Button {
                text: self.is_true.to_string(),
            },
            event: Button::Clicked,
            handler: Self::on_clicked,
        })
        .boxed()
    }
}

fn main() {
    App::new(Main { is_true: false }).run();
}
