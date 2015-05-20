// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use libc::types::common::c95::c_void;
use libc::types::common::c99::{int16_t, uint16_t, uint32_t};
use libc::types::os::arch::c95::c_int;
use libc::types::os::arch::c99::{intptr_t, uintptr_t};
use libc::types::os::common::posix01::timespec;

use std::ptr;

#[repr(C)]
#[derive(Copy)]
pub struct Kevent {
    pub ident:  uintptr_t,
    pub filter: int16_t,
    pub flags:  uint16_t,
    pub fflags: uint32_t,
    pub data:   intptr_t,
    pub udata:  *const c_void,
}

pub const EVFILT_READ:  int16_t = -1;
pub const EVFILT_WRITE: int16_t = -2;
pub const EVFILT_AIO:   int16_t = -3;
pub const EVFILT_VNODE: int16_t = -4;

bitflags! {
    flags Flags: uint16_t {
        const EV_ADD      = 0x0001,
        const EV_DELETE   = 0x0002,
        const EV_ENABLE   = 0x0004,
        const EV_DISABLE  = 0x0008,
        const EV_RECEIPT  = 0x0040,
        const EV_ONESHOT  = 0x0010,
        const EV_CLEAR    = 0x0020,
        const EV_ERROR	  = 0x4000,
    }
}

bitflags! {
    flags FilterFlags: uint32_t {
        const NOTE_DELETE = 0x00000001,
        const NOTE_WRITE  = 0x00000002,
        const NOTE_EXTEND = 0x00000004,
        const NOTE_ATTRIB = 0x00000008,
        const NOTE_LINK   = 0x00000010,
        const NOTE_RENAME = 0x00000020,
        const NOTE_REVOKE = 0x00000040,
        const NOTE_NONE   = 0x00000080,
    }
}

extern {
    pub fn kqueue() -> c_int;
    pub fn kevent(
        kq:         c_int,
        changelist: *const Kevent,
        nchanges:   c_int,
        eventlist:  *mut Kevent,
        nevents:    c_int,
        timeout:    *const timespec) -> c_int;
}

impl Kevent {
    pub fn empty() -> Kevent {
        Kevent {
            ident:  0,
            filter: 0,
            flags:  0,
            fflags: 0,
            data:   0,
            udata:  ptr::null(),
        }
    }
}

impl Clone for Kevent {
    fn clone(&self) -> Kevent { *self }
}
