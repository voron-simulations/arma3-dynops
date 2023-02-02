// This file contains wrappers interfacing with ArmA 3's RealVirtuality engine

mod cluster;
//mod kdtree;
mod misc;
mod shape;
mod bounding;

use libc::{c_char, c_int, strncpy};
use std::ffi::{CStr, CString};
use std::panic::catch_unwind;
use std::result::Result;
use std::slice::from_raw_parts;

// Write to C-string
// TODO: better error handling
fn write_output(value: &str, output: *mut c_char, output_size: c_int) {
    let data = CString::new(value).unwrap_or(CString::new("Failed to allocate string").unwrap());
    unsafe {
        strncpy(output, data.as_ptr(), output_size as usize);
    }
}

fn exec_function(function: &str, args: &[String]) -> Result<String, String> {
    let result = catch_unwind(|| {
        return match function {
            "cluster" => cluster::entrypoint(&args[0]),
            "datetime" => Ok(misc::get_current_datetime()),
            "echo" => Ok(misc::echo(args)),
            "uuid" => Ok(misc::uuid()),
            "panic" => panic!("Test panic"),
            _ => Err(format!("Unknown function: {function}")),
        };
    });

    // Panic handling
    match result {
        Ok(result) => result, // return value is already a Result
        Err(reason) => {
            // Try to extract error message
            match reason.downcast::<&str>() {
                Ok(panic_msg) => {
                    return Err(format!("Panic: {panic_msg}"));
                }
                Err(_) => Err("Panic: unknown".to_string()),
            }
        }
    }
}

/// # Examples
/// ```
/// use std::ffi::{CString,CStr};
/// let mut c_chars = vec![0; 1024];
/// let function = CString::new("uuid").unwrap();
/// unsafe {
///     dynops::RVExtension(
///         c_chars.as_mut_ptr(),
///         c_chars.len() as i32,
///         function.as_ptr(),
///     );
/// }
/// let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
/// ```
///
/// # Safety
///
/// This function uses raw string pointers and so relies on passed buffers being correctly formed
#[no_mangle]
pub unsafe extern "C" fn RVExtension(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
) {
    let fun = unsafe { CStr::from_ptr(function).to_str().unwrap_or_default() };
    let args: Vec<String> = Vec::new();

    let result = exec_function(fun, args.as_slice());
    let outstr = result.unwrap_or_else(|err| err);
    write_output(outstr.as_str(), output, output_size);
}

/// # Safety
///
/// This function uses raw string pointers and so relies on passed buffers being correctly formed
#[no_mangle]
pub unsafe extern "C" fn RVExtensionArgs(
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
    retval
}

/// # Examples
///
/// ```
/// use std::ffi::CStr;
/// let mut c_chars = vec![0; 1024];
/// unsafe { dynops::RVExtensionVersion(c_chars.as_mut_ptr(), c_chars.len() as i32) };
/// let result = unsafe { CStr::from_ptr(c_chars.as_ptr()).to_str().unwrap() };
/// assert!(result.starts_with("Dynamic Operations"));
/// ```
///
/// # Safety
///
/// This function uses raw string pointers and so relies on passed buffers being correctly formed
#[no_mangle]
pub unsafe extern "C" fn RVExtensionVersion(output: *mut c_char, output_size: c_int) {
    let version = "Dynamic Operations v0.1";
    write_output(version, output, output_size);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_echo() {
        let args: Vec<String> = vec!["A".to_owned(), "B".to_owned()];
        assert_eq!("echo(A, B)", crate::exec_function("echo", &args).unwrap());
    }

    #[test]
    fn test_panic() {
        let args: Vec<String> = Vec::new();
        assert!(crate::exec_function("panic", &args).is_err());
    }
}
