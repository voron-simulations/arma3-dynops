use chrono::Utc;
use uuid::Uuid;

pub fn get_current_datetime() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn echo(args: &[String]) -> std::string::String {
    format!("echo({})", args.join(", "))
}

pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_uuid() {
        crate::misc::uuid();
    }

    #[test]
    fn test_echo() {
        let data = vec!["a".to_owned(), "b".to_owned()];
        assert_eq!("echo(a, b)", crate::misc::echo(&data));
    }

    #[test]
    fn test_datetime() {
        crate::misc::get_current_datetime();
    }
}
