// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use std::mem::transmute;
use std::intrinsics::copy;
use std::str::{from_utf8, Utf8Error};
use std::io::Error;

use libc::funcs::posix88::unistd;
use libc::types::common::c95::c_void;
use libc::types::os::arch::c95::{c_int, size_t};

pub struct Buf {
    bytes:  Vec<u8>,
    cursor: usize,
    sep:    u8,
}

impl Buf {
    pub fn new(sep: u8) -> Buf {
        Buf {
            bytes:  Vec::new(),
            cursor: 0,
            sep:    sep,
        }
    }

    pub fn fill(&mut self, fd: c_int, len: usize) -> Result<(), Error> {
        self.compact();
        self.bytes.reserve(len);
        unsafe {
            let pos = self.bytes.len();
            let ptr = self.bytes.as_ptr().offset(pos as isize);
            let n = unistd::read(fd, ptr as *mut c_void, len as size_t);
            if n == -1 {
                return Err(Error::last_os_error())
            }
            self.bytes.set_len(pos + n as usize);
        }
        Ok(())
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.compact();
        self.bytes.push_all(bytes);
    }

    pub fn next_line<'a>(&mut self) -> Result<Option<&'a str>, Utf8Error> {
        Ok(match self.bytes[self.cursor..].position_elem(&self.sep) {
            Some(n) => {
                let line = &self.bytes[self.cursor..self.cursor+n];
                self.cursor += n + 1;
                Some(unsafe { try!(from_utf8(transmute(line))) })
            },
            None => None,
        })
    }

    fn compact(&mut self) {
        if self.cursor > 0 {
            let src = self.bytes[self.cursor..].as_ptr();
            let len = self.bytes.len() - self.cursor;
            unsafe {
                copy(src, self.bytes.as_mut_ptr(), len);
                self.bytes.set_len(len);
            }
            self.cursor = 0;
        }
    }
}
