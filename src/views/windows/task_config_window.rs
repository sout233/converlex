use vizia::prelude::*;

use crate::{
    controllers::main::app_event::AppEvent, views::pages::task_config_page,
};

pub fn popup(cx: &mut Context) -> Handle<Window> {
    Window::popup(cx, true, |cx| {
        task_config_page::new(cx);
    })
    .title("Converlex - Config")
    .on_close(|cx| {
        cx.emit(AppEvent::ConfigWindowClosing);
    })
}
