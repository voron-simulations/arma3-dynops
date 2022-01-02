#[cfg(test)]
mod invocation {
    use libc::c_char;
    use std::ffi::{CStr, CString};

    #[test]
    fn version() {
        let mut c_chars = vec![i8::from(0); 1024];
        dynops::RVExtensionVersion(c_chars.as_mut_ptr(), c_chars.len() as i32);
    }

    #[test]
    fn datetime() {
        let mut c_chars = vec![i8::from(0); 1024];
        let function = CString::new("datetime").unwrap();
        dynops::RVExtension(
            c_chars.as_mut_ptr(),
            c_chars.len() as i32,
            function.as_ptr(),
        );
    }

    #[test]
    fn echo() {
        let mut c_chars = vec![i8::from(0); 1024];
        let function = CString::new("echo").unwrap();
        let inputs = vec![
            CString::new("A").unwrap(),
            CString::new("B").unwrap(),
            CString::new("C").unwrap(),
        ];
        let args: Vec<*const c_char> = inputs
            .iter()
            .map(|str| -> *const c_char { str.as_ptr() })
            .collect();

        dynops::RVExtensionArgs(
            c_chars.as_mut_ptr(),
            c_chars.len() as i32,
            function.as_ptr(),
            args.as_ptr(),
            args.len() as i32,
        );
        let result: CString = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_owned() };
        assert_eq!("echo(A, B, C)", result.to_str().unwrap())
    }
}
