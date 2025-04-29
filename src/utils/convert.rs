fn convert() {
    let tasks = self.tasks.clone();
    let indices = self.indices.clone();
    for index in indices {
        let task = &tasks[index];
        if !task.done {
            if let Some(input_path) = &task.name {
                let output_path = get_output_path(input_path, "mp3");

                if input_path == &output_path {
                    println!("输入输出路径相同，跳过！");
                    continue;
                }

                let choice = MessageDialog::new()
                    .set_level(MessageLevel::Info)
                    .set_title("转换确认")
                    .set_description(&format!(
                        "是否删除源文件？\n\n源文件:\n{}\n输出文件:\n{}",
                        input_path, output_path
                    ))
                    .set_buttons(MessageButtons::YesNoCancel)
                    .show();

                match choice {
                    MessageDialogResult::Yes => {
                        if convert_media(input_path, &output_path).is_ok() {
                            let _ = std::fs::remove_file(input_path);
                            println!("转换成功并删除源文件：{}", input_path);
                        } else {
                            println!("任务转换失败：{}", input_path);
                        }
                    }
                    MessageDialogResult::No => {
                        if convert_media(input_path, &output_path).is_err() {
                            println!("任务转换失败：{}", input_path);
                        }
                    }
                    MessageDialogResult::Cancel => {
                        println!("取消转换：{}", input_path);
                    }
                    MessageDialogResult::Ok => todo!(),
                    MessageDialogResult::Custom(_) => todo!(),
                }
            }
        }
    }
}
