// Copyright (C) 2015 - Will Glozer.  All rights reserved.

mod buf;

use std::convert::From;
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};
use std::ptr;
use std::str::Utf8Error;
use std::time::Duration;

use libc::types::common::c99::int16_t;
use libc::types::os::arch::c95::c_int;
use libc::types::os::arch::c99::{intptr_t, uintptr_t};

use kqueue::*;
use posix::Fd;
use self::buf::Buf;

pub struct Tailer {
    kqueue:  Kqueue,
    path:    PathBuf,
    dir:     Fd,
    file:    Fd,
    events:  [Kevent; 16],
    buf:     Buf,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    Utf8Error(Utf8Error),
}

impl Tailer {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<Tailer, Error> {
        let mut kqueue = try!(Kqueue::new());
        let path = PathBuf::from(path);
        let dir = try!(Fd::open(path.parent().unwrap().as_os_str()));

        try!(kqueue.change(&mut[vnode_event(dir)]));
        let file = try!(Tailer::open(&mut kqueue, &path));

        Ok(Tailer {
            kqueue: kqueue,
            dir:    dir,
            path:   path,
            file:   file,
            events: [Kevent::empty(); 16],
            buf:    Buf::new(b'\n'),
        })
    }

    pub fn next_line<'a>(&'a mut self, timeout: Option<Duration>) -> Result<Option<&'a str>, Error> {
        match self.buf.next_line() {
            Ok(Some(str)) => Ok(Some(str)),
            _             => self.wait(timeout),
        }
    }

    pub fn wait<'a>(&'a mut self, timeout: Option<Duration>) -> Result<Option<&'a str>, Error> {
        let n = try!(self.kqueue.wait(&mut self.events[..], timeout));
        for kevent in &self.events[0..n] {
            let fflags = FilterFlags::from_bits(kevent.fflags).unwrap();
            let fd = Fd(kevent.ident as c_int);

            match kevent.filter {
                EVFILT_READ if fd == self.file && kevent.data > 0 => {
                    let len = kevent.data as usize;
                    try!(self.buf.fill(fd.0, len));
                },
                EVFILT_READ if fd == self.file => {
                    try!(fd.seek(kevent.data));
                },
                EVFILT_VNODE if fd == self.file && fflags.intersects(NOTE_DELETE | NOTE_RENAME) => {
                    try!(fd.close());
                    self.file = try!(Tailer::open(&mut self.kqueue, &self.path));
                },
                EVFILT_VNODE if fd == self.dir && self.file.0 == -1 && fflags.contains(NOTE_WRITE) => {
                    self.file = try!(Tailer::open(&mut self.kqueue, &self.path));
                },
                _ => (),
            }
        }
        Ok(try!(self.buf.next_line()))
    }

    fn open(kqueue: &mut Kqueue, path: &Path) -> Result<Fd, io::Error> {
        let fd = match Fd::open(path.as_os_str()) {
            Ok(fd) => fd,
            Err(_) => return Ok(Fd(-1))
        };
        try!(kqueue.change(&mut[vnode_event(fd), read_event(fd)]));
        Ok(fd)
    }
}

impl Drop for Tailer {
    fn drop(&mut self) {
        self.file.close().unwrap();
        self.dir.close().unwrap();
    }
}

fn vnode_event(fd: Fd) -> Kevent {
    event(fd, EVFILT_VNODE, NOTE_DELETE | NOTE_EXTEND | NOTE_RENAME | NOTE_WRITE)
}

fn read_event(fd: Fd) -> Kevent {
    event(fd, EVFILT_READ, FilterFlags::empty())
}

fn event(fd: Fd, filter: int16_t, fflags: FilterFlags) -> Kevent {
    let flags = EV_ADD | EV_CLEAR | EV_RECEIPT;
    Kevent {
        ident:  fd.0 as uintptr_t,
        filter: filter,
        flags:  flags.bits(),
        fflags: fflags.bits(),
        data:   0 as intptr_t,
        udata:  ptr::null()
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            &Error::IoError(ref err)    => err.fmt(fmt),
            &Error::Utf8Error(ref err)  => err.fmt(fmt),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8Error(err)
    }
}

#[cfg(test)]
mod tests;
