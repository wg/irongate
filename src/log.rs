// Copyright (C) 2015 - Will Glozer.  All rights reserved.

macro_rules! syslog {
    ($($arg:tt)*) => {{
        use std::ffi::CString;
        use libc::types::os::arch::c95::{c_int, c_char};
        
        extern {
            fn syslog(priority: c_int, message: *const c_char, ...);
        }

        let cstr = CString::new(format!($($arg)*)).unwrap();
        unsafe { syslog(5, cstr.as_ptr()) };
    }}
}
