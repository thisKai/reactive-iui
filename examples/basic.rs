use reactive_iui::*;

struct Main {
    is_true: bool,
}
impl Main {
    view! {
        Group {
            title: "state",
            margined: true,
            child: Button {
                text: self.is_true.to_string(),
                #[on] clicked: Self::on_clicked,
            },
        }
    }
    fn on_clicked(&mut self) {
        self.is_true = !self.is_true;
    }
}

fn main() {
    App::new(|| {
        Group {
            title: String::from("Hello"),
            margined: true,
            child: Some(Button { text: String::from("world!") }.boxed()),
        }
    }).run();
}
