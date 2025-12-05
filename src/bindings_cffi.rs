// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

use std::ffi::{c_char, c_int, CString};
use std::ptr::null_mut;
use std::slice;

use super::{compute_loads, Country, Load, Location, Logical};

fn set_err(out_error: *mut *mut c_char, msg: &str) {
    if out_error.is_null() {
        return;
    }
    let cmsg = CString::new(msg).unwrap_or_else(|_| {
        CString::new("error creating error message").unwrap()
    });

    // nosem: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        *out_error = cmsg.into_raw();
    }
}

// nosem: rust.lang.security.unsafe-usage.unsafe-usage
#[no_mangle]
pub extern "C" fn compute_loads_cffi(
    logicals_ptr: *const Logical,
    logicals_len: usize,
    status_file_ptr: *const u8,
    status_file_len: usize,
    user_location_ptr: *const Location,
    user_country_ptr: *const [u8; 2],
    loads: *mut Load,
    error: *mut *mut c_char,
) -> c_int {
    if !error.is_null() {
        // nosem: rust.lang.security.unsafe-usage.unsafe-usage
        unsafe {
            *error = null_mut();
        }
    }

    if logicals_ptr.is_null() || status_file_ptr.is_null() || loads.is_null() {
        set_err(error, "null pointer for required parameter");
        return -1;
    }

    // nosem: rust.lang.security.unsafe-usage.unsafe-usage
    let logicals = unsafe { slice::from_raw_parts(logicals_ptr, logicals_len) };

    let status_file =
        // nosem: rust.lang.security.unsafe-usage.unsafe-usage
        unsafe { slice::from_raw_parts(status_file_ptr, status_file_len) };

    // nosem: rust.lang.security.unsafe-usage.unsafe-usage
    let user_location: Option<Location> = unsafe {
        if user_location_ptr.is_null() {
            None
        } else {
            Some((*user_location_ptr).clone())
        }
    };

    // nosem: rust.lang.security.unsafe-usage.unsafe-usage
    let user_country: Option<Country> = unsafe {
        if user_country_ptr.is_null() {
            None
        } else {
            match Country::try_from(&*user_country_ptr) {
                Ok(c) => Some(c),
                Err(err) => {
                    set_err(error, &err.to_string());
                    return -2;
                }
            }
        }
    };

    let output_slice =
        // nosem: rust.lang.security.unsafe-usage.unsafe-usage
        unsafe { slice::from_raw_parts_mut(loads, logicals_len) };

    if let Err(e) = compute_loads(
        output_slice,
        &logicals,
        status_file,
        &user_location,
        &user_country,
    ) {
        set_err(error, &e.to_string());
        return -3;
    }

    0
}

// nosem: rust.lang.security.unsafe-usage.unsafe-usage
#[unsafe(no_mangle)]
pub extern "C" fn free_c_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    // nosem: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        drop(CString::from_raw(s));
    }
}
