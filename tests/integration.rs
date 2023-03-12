#[cfg(test)]
mod integration {
    use libc::c_char;
    use std::ffi::{CStr, CString};

    // fn test_map_data(data: &str) {
    //     let mut c_chars = vec![0; 1024 * 128];
    //     let function = CString::new("cluster").unwrap();
    //     let input = CString::new(data).unwrap();
    //     let args: Vec<*const c_char> = vec![input.as_ptr()];

    //     let retval = unsafe {
    //         dynops::RVExtensionArgs(
    //             c_chars.as_mut_ptr(),
    //             c_chars.len() as i32,
    //             function.as_ptr(),
    //             args.as_ptr(),
    //             args.len() as i32,
    //         )
    //     };
    //     let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
    //     assert!(retval != 0, "{}", result);
    // }

    #[test]
    fn test_map_altis() {
        //test_map_data(include_str!("../data/objects.Altis.txt"));
    }

    #[test]
    fn test_map_stratis() {
        //test_map_data(include_str!("../data/objects.Stratis.txt"));
    }

    #[test]
    fn test_map_livonia() {
        //test_map_data(include_str!("../data/objects.Livonia.txt"));
    }

    #[test]
    fn test_map_tanoa() {
        //test_map_data(include_str!("../data/objects.Tanoa.txt"));
    }

    #[test]
    fn test_map_malden() {
        //test_map_data(include_str!("../data/objects.Malden.txt"));
    }

    #[test]
    fn test_map_chernarus() {
        //test_map_data(include_str!("../data/objects.Chernarus2020.txt"));
    }
}
