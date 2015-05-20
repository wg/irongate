// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use std::ffi::OsStr;
use std::io::Error;

use libc::consts::os::c95::SEEK_CUR;
use libc::consts::os::posix88::O_RDONLY;
use libc::funcs::posix88::{fcntl, unistd};
use libc::types::os::arch::c95::c_int;
use libc::types::os::arch::posix88::off_t;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Fd(pub c_int);

impl Fd {
    pub fn open(path: &OsStr) -> Result<Fd, Error> {
        let path = path.to_cstring().unwrap();
        unsafe {
            match fcntl::open(path.as_ptr(), O_RDONLY, 0) {
                -1 => Err(Error::last_os_error()),
                fd => Ok(Fd(fd))
            }
        }
    }

    pub fn close(self) -> Result<(), Error> {
        unsafe {
            match unistd::close(self.0 as c_int) {
                -1 => Err(Error::last_os_error()),
                 _ => Ok(())
            }
        }
    }

    pub fn seek(self, offset: off_t) -> Result<(), Error> {
        unsafe {
            match unistd::lseek(self.0 as c_int, offset, SEEK_CUR) {
                -1 => Err(Error::last_os_error()),
                 _ => Ok(())
            }
        }
    }
}
