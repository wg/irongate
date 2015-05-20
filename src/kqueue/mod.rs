// Copyright (C) 2015 - Will Glozer.  All rights reserved.

mod ffi;

use libc::types::os::arch::c95::c_int;
use libc::types::os::common::posix01::timespec;
use libc::funcs::posix88::unistd::close;

use std::io::Error;
use std::ptr;
use std::time::Duration;
use std::mem;

pub use self::ffi::*;

pub struct Kqueue {
    fd: c_int,
}

impl Kqueue {
    pub fn new() -> Result<Kqueue, Error> {
        match unsafe { kqueue() } {
            -1 => Err(Error::last_os_error()),
            fd => Ok(Kqueue { fd: fd })
        }
    }

    pub fn change(&mut self, changes: &mut [Kevent]) -> Result<usize, Error> {
        let events: &mut [Kevent] = unsafe { mem::transmute(&mut changes[..]) };
        let n = try!(self.kevent(changes, events, None));
        for event in &events[0..n] {
            let flags = Flags::from_bits(event.flags).unwrap();
            if flags.contains(EV_ERROR) && event.data != 0 {
                return Err(Error::from_raw_os_error(event.data as i32));
            }
        }
        Ok(n)
    }

    pub fn wait(&self, events: &mut [Kevent], timeout: Option<Duration>) -> Result<usize, Error> {
        self.kevent(&[], events, timeout)
    }

    fn kevent(&self, changes: &[Kevent], events: &mut [Kevent], timeout: Option<Duration>) -> Result<usize, Error> {
        let timeout: *const timespec = match timeout {
            Some(d) => {
                let secs  = Duration::new(d.secs(), 0);
                let nanos = d - secs;
                &timespec {
                    tv_sec:  secs.secs() as i64,
                    tv_nsec: nanos.extra_nanos() as i64,
                }
            },
            None => ptr::null()
        };

        let n = unsafe {
            let nchanges = changes.len() as c_int;
            let nevents  = events.len() as c_int;
            kevent(self.fd, changes.as_ptr(), nchanges, events.as_mut_ptr(), nevents, timeout)
        };

        match n {
            -1 => Err(Error::last_os_error()),
             0 => Ok(0),
             n => Ok(n as usize),
        }
    }
}

impl Drop for Kqueue {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}
