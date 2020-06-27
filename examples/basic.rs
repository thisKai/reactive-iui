use {
    iui::{prelude::*},
    stream_ui::*,
};


fn main() {
    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut win = Window::new(&ui, "Test App", 200, 200, WindowType::NoMenubar);

    let view = stream_ui::Group {
        title: String::from("Hello"),
        margined: true,
        child: Some(stream_ui::Button { text: String::from("world!") }.boxed())
    };

    win.set_child(&ui, view.control(&ui));

    win.show(&ui);

    ui.main();
}
