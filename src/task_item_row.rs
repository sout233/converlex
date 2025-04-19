

pub struct TaskItemRow {
    task_name: String,
    selected_format: usize,
    format_to_list: Vec<Audio>,
}

impl View for TaskItemRow {}

impl TaskItemRow {
    fn new<L>(cx: &mut Context, task_name: L) -> Handle<Self>
    where
        L: Lens<Target = String>,
    {
        Self {
            task_name: task_name.get(cx),
            selected_format: 0,
            format_to_list: vec![Audio::Mp3, Audio::Wav, Audio::Flac],
        }
        .build(cx, |cx| {
            HStack::new(cx, |cx| {
                // Checkbox::new(cx, task_done).on_toggle(move |cx| {
                //     cx.emit(AppEvent::ToggleTask(cx.index())); // 使用 cx.index() 获取索引
                // });
                Label::new(cx, task_name);
                ComboBox::new(cx, AppData::format_to_list, AppData::selected_format);
            });
        })
    }
}
