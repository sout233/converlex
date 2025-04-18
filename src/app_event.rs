use crate::task::Task;

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(usize),
    UpdateTask(usize, Task),
    ToggleAutoRename(usize),
    ChangeOutputFormat(usize, usize),
    StartConvert,
    RemoveAll,
    ToggleConifgWindow(usize),
}