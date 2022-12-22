#[cfg(test)]
mod integration {
    use libc::c_char;
    use std::ffi::{CStr, CString};

    #[test]
    fn version() {
        let mut c_chars = vec![0; 1024];
        unsafe { dynops::RVExtensionVersion(c_chars.as_mut_ptr(), c_chars.len() as i32) };
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        assert!(result.starts_with("Dynamic Operations"));
    }

    #[test]
    fn datetime() {
        let mut c_chars = vec![0; 1024];
        let function = CString::new("datetime").unwrap();
        unsafe {
            dynops::RVExtension(
                c_chars.as_mut_ptr(),
                c_chars.len() as i32,
                function.as_ptr(),
            );
        }
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        assert!(result.len() > 5);
    }

    #[test]
    fn uuid() {
        let mut c_chars = vec![0; 1024];
        let function = CString::new("uuid").unwrap();
        unsafe {
            dynops::RVExtension(
                c_chars.as_mut_ptr(),
                c_chars.len() as i32,
                function.as_ptr(),
            );
        }
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        assert!(result.len() > 5);
    }

    #[test]
    fn panic() {
        let mut c_chars = vec![0; 1024];
        let function = CString::new("panic").unwrap();
        unsafe {
            dynops::RVExtension(
                c_chars.as_mut_ptr(),
                c_chars.len() as i32,
                function.as_ptr(),
            );
        }
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        assert_eq!("Panic: Test panic", result);
    }

    #[test]
    fn echo() {
        let mut c_chars = vec![0; 1024];
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

        let retval = unsafe {
            dynops::RVExtensionArgs(
                c_chars.as_mut_ptr(),
                c_chars.len() as i32,
                function.as_ptr(),
                args.as_ptr(),
                args.len() as i32,
            )
        };
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        assert_eq!(0, retval);
        assert_eq!("echo(A, B, C)", result)
    }

    fn test_map_data(data: &str) {
        let mut c_chars = vec![0; 1024 * 128];
        let function = CString::new("cluster").unwrap();
        let input = CString::new(data).unwrap();
        let args: Vec<*const c_char> = vec![input.as_ptr()];

        let retval = unsafe {
            dynops::RVExtensionArgs(
                c_chars.as_mut_ptr(),
                c_chars.len() as i32,
                function.as_ptr(),
                args.as_ptr(),
                args.len() as i32,
            )
        };
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        if retval != 0 {
            println!("{}", result);
            assert!(false);
        }
    }

    #[test]
    fn test_map_altis() {
        test_map_data(include_str!("../data/objects.Altis.txt"));
    }

    #[test]
    fn test_map_stratis() {
        test_map_data(include_str!("../data/objects.Stratis.txt"));
    }

    #[test]
    fn test_map_livonia() {
        test_map_data(include_str!("../data/objects.Livonia.txt"));
    }

    #[test]
    fn test_map_tanoa() {
        test_map_data(include_str!("../data/objects.Tanoa.txt"));
    }

    #[test]
    fn test_map_malden() {
        test_map_data(include_str!("../data/objects.Malden.txt"));
    }

    #[test]
    fn test_map_chernarus() {
        test_map_data(include_str!("../data/objects.Chernarus2020.txt"));
    }
}
