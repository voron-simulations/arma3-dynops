// This file contains wrappers interfacing with ArmA 3's RealVirtuality engine

mod cluster;
mod datetime;
mod echo;

use libc::{c_char, c_int, strncpy};
use std::ffi::{CStr, CString};
use std::panic::catch_unwind;
use std::result::Result;
use std::slice::from_raw_parts;

// Write to C-string
fn write_output(value: &str, output: *mut c_char, output_size: c_int) {
    let data = CString::new(value)
        .unwrap_or(CString::new("ERROR: Failed to pass output to game").unwrap());
    unsafe {
        strncpy(output, data.as_ptr(), output_size as usize);
    }
}

fn exec_function(function: &str, args: &[String]) -> Result<String, String> {
    let result = catch_unwind(|| {
        return match function {
            "datetime" => datetime::get_current_datetime(),
            "echo" => echo::echo(args),
            "cluster" => cluster::entrypoint(args),
            "panic" => panic!("Test panic"),
            _ => format!("ERROR: Unknown function: {}", function).to_owned(),
        };
    });
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err("Library panicked".to_owned())
    }
}

#[no_mangle]
pub extern "C" fn RVExtension(output: *mut c_char, output_size: c_int, function: *const c_char) {
    let fun = unsafe { CStr::from_ptr(function).to_str().unwrap_or_default() };
    let args: Vec<String> = Vec::new();

    let result = exec_function(fun, args.as_slice());
    let outstr = result.unwrap_or_else(|err| err);
    write_output(outstr.as_str(), output, output_size);
}

#[no_mangle]
pub extern "C" fn RVExtensionArgs(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
    argv: *const *const c_char,
    argc: c_int,
) -> i32 {
    let fun = unsafe { CStr::from_ptr(function).to_str().unwrap_or_default() };
    let arglen = argc as usize;

    let args: Vec<String> = unsafe {
        from_raw_parts(argv, arglen)
            .iter()
            .map(|arg| {
                CStr::from_ptr(*arg)
                    .to_string_lossy()
                    .to_owned()
                    .to_string()
            })
            .collect()
    };

    let result = exec_function(fun, args.as_slice());
    let retval = result.is_err() as i32; // return 1 if error
    let outstr = result.unwrap_or_else(|err| err);
    write_output(outstr.as_str(), output, output_size);
    return retval;
}

#[no_mangle]
pub extern "C" fn RVExtensionVersion(output: *mut c_char, output_size: c_int) {
    let version = "Test Extension 0.0.1";
    write_output(version, output, output_size);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_datetime() {
        let args: Vec<String> = Vec::new();
        crate::exec_function("datetime", &args).unwrap();
    }

    #[test]
    fn test_echo() {
        let args: Vec<String> = vec!["A".to_owned(), "B".to_owned(), "C".to_owned()];
        assert_eq!(
            "echo(A, B, C)",
            crate::exec_function("echo", &args).unwrap()
        );
    }
}
