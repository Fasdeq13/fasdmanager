use std::time::SystemTime;

pub fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::DECIMAL)
}

pub fn format_timestamp(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return String::from("—");
    };
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M").to_string()
}
