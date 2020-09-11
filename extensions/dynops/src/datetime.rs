use chrono::Utc;

pub fn get_current_datetime() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
