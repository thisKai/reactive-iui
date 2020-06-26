use iui::prelude::*;
use iui::controls::*;

fn main() {
    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut win = Window::new(&ui, "Test App", 200, 200, WindowType::NoMenubar);

    let label = Label::new(&ui, "Hello world!");
    win.set_child(&ui, label);

    win.show(&ui);

    ui.main();
}
