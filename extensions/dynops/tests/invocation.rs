#[cfg(test)]
mod integration {

    use std::ffi::{CString};
    #[test]
    fn version() {
        let mut c_chars = vec![i8::from(0); 1024];
        dynops::RVExtensionVersion(c_chars.as_mut_ptr(), 1024);
    }

    #[test]
    fn datetime() {
        let mut c_chars = vec![i8::from(0); 1024];
        let function =CString::new("datetime").unwrap();
        dynops::RVExtension(c_chars.as_mut_ptr(), 1024, function.as_ptr());
    }

    #[test]
    fn echo() {
        let mut c_chars = vec![i8::from(0); 1024];
        let function =CString::new("datetime").unwrap();
        // dynops::RVExtensionArgs(c_chars.as_mut_ptr(), 1024, function.as_ptr());
    }
}
