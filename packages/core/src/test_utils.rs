use std::{fs::File, io::Write, time::UNIX_EPOCH};

pub fn write_and_edit(file_ext: &str, content: &str) {
    let tmp_path = std::env::temp_dir().join(format!(
        "debug-{}{}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        file_ext
    ));
    File::create(&tmp_path)
        .unwrap()
        .write_fmt(format_args!("{}", content))
        .unwrap();
    std::process::Command::new("code")
        .arg(&tmp_path)
        .spawn()
        .unwrap();
}
