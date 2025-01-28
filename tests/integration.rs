#[cfg(test)]
mod integration {
    use std::ffi::{c_char, c_int, CStr, CString};

    fn test_map_data(data: &str) {
        let mut c_chars = vec![0; 1024 * 128];
        let function = CString::new("cluster").unwrap();
        let input = CString::new(data).unwrap();
        let mut args: Vec<*mut c_char> = vec![input.into_raw()];
        let retval = unsafe {
            /*
              pub unsafe extern "stdcall" fn RVExtensionArgs(output: *mut arma_rs_libc::c_char,
                size: arma_rs_libc::size_t, function: *mut arma_rs_libc::c_char,
                args: *mut *mut arma_rs_libc::c_char, arg_count: arma_rs_libc::c_int) -> arma_rs_libc::c_int
            */
            dynops::RVExtensionArgs(
                c_chars.as_mut_ptr(),
                c_chars.len(),
                function.into_raw(),
                args.as_mut_ptr(),
                args.len() as c_int,
            )
        };
        let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
        assert!(retval != 0, "{}", result);
    }

    #[test]
    #[ignore]
    fn test_map_altis() {
        test_map_data(include_str!("../data/objects.Altis.txt"));
    }

    #[test]
    fn test_map_stratis() {
        test_map_data(include_str!("../data/objects.Stratis.txt"));
    }

    #[test]
    #[ignore]
    fn test_map_livonia() {
        test_map_data(include_str!("../data/objects.Livonia.txt"));
    }

    #[test]
    #[ignore]
    fn test_map_tanoa() {
        test_map_data(include_str!("../data/objects.Tanoa.txt"));
    }

    #[test]
    #[ignore]
    fn test_map_malden() {
        test_map_data(include_str!("../data/objects.Malden.txt"));
    }

    #[test]
    #[ignore]
    fn test_map_chernarus() {
        test_map_data(include_str!("../data/objects.Chernarus2020.txt"));
    }
}
