use stream_ui::*;


fn main() {
    App::new(|| {
        Group {
            title: String::from("Hello"),
            margined: true,
            child: Some(Button { text: String::from("world!") }.boxed()),
        }
    }).run();
}
