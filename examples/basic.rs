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
struct MainNoMacro {
    is_true: bool,
}
impl MainNoMacro {
    fn on_clicked(&mut self) {
        self.is_true = !self.is_true;
    }
}
impl Component for MainNoMacro {
    fn view(&self) -> Box<dyn BaseVirtualControl> {
        Group {
            title: String::from("Hello"),
            margined: true,
            child: Some(Handler {
                child: Button { text: self.is_true.to_string() },
                event: Button::Clicked,
                handler: Self::on_clicked,
            }.boxed()),
        }.boxed()
    }
}

fn main() {
    App::new(MainNoMacro { is_true: false }).run();
}
