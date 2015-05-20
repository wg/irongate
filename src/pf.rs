// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use std::cmp;
use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::intrinsics::{bswap16, copy};
use std::io::Error;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::unix::io::AsRawFd;
use std::str;

use libc::consts::os::bsd44::{AF_INET, AF_INET6};
use libc::funcs::bsd44::ioctl;
use libc::types::common::c95::c_void;
use libc::types::common::c99::{int8_t, uint8_t, int32_t, uint32_t};
use libc::types::os::arch::c95::{c_char, c_int, c_ulong};

const DIOCRADDTABLES: c_ulong = 0xC450443D;
const DIOCRGETTABLES: c_ulong = 0xC450443F;
const DIOCRADDADDRS:  c_ulong = 0xC4504443;
const DIOCRDELADDRS:  c_ulong = 0xC4504444;
const DIOCRGETADDRS:  c_ulong = 0xc4504446;

pub struct Pf {
    file: File
}

impl Pf {
    pub fn new() -> Result<Self, Error> {
        match OpenOptions::new().read(true).write(true).open("/dev/pf") {
            Ok(file) => Ok(Pf { file: file }),
            Err(err) => Err(Error::new(err.kind(), "failed to open /dev/pf")),
        }
    }

    pub fn tables(&self) -> Result<Vec<Table>, Error> {
        self.get(DIOCRGETTABLES, &mut Command::new())
    }

    pub fn add_tables(&self, tables: &[Table]) -> Result<isize, Error> {
        let mut cmd = Command::new();
        match self.modify(DIOCRADDTABLES, &mut cmd, tables) {
            Ok(()) => Ok(cmd.nadd as isize),
            Err(e) => Err(e),
        }
    }

    pub fn addrs(&self, table: &str) -> Result<Vec<Addr>, Error> {
        self.get(DIOCRGETADDRS, &mut Command::with_table(table))
    }

    pub fn add_addrs(&self, table: &str, addrs: &[Addr]) -> Result<isize, Error> {
        let mut cmd = Command::with_table(table);
        match self.modify(DIOCRADDADDRS, &mut cmd, addrs) {
            Ok(()) => Ok(cmd.nadd as isize),
            Err(e) => Err(e),
        }
    }

    pub fn del_addrs(&self, table: &str, addrs: &[Addr]) -> Result<isize, Error> {
        let mut cmd = Command::with_table(table);
        match self.modify(DIOCRDELADDRS, &mut cmd, addrs) {
            Ok(()) => Ok(cmd.ndel as isize),
            Err(e) => Err(e),
        }
    }

    fn get<T>(&self, request: c_ulong, cmd: &mut Command) -> Result<Vec<T>, Error> {
        let fd = self.file.as_raw_fd();
        unsafe {
            cmd.esize = mem::size_of::<T>() as c_int;

            if ioctl(fd, request, cmd as *mut _ as *mut c_void) == -1 {
                return Err(Error::last_os_error());
            }

            let mut vec: Vec<T> = Vec::with_capacity(cmd.size as usize);
            cmd.buffer = vec.as_mut_ptr() as *mut c_void;

            if ioctl(fd, request, cmd as *mut _ as *mut c_void) == -1 {
                return Err(Error::last_os_error());
            }

            vec.set_len(cmd.size as usize);

            Ok(vec)
        }
    }

    fn modify<T>(&self, request: c_ulong, cmd: &mut Command, slice: &[T]) -> Result<(), Error> {
        let fd = self.file.as_raw_fd();
        unsafe {
            cmd.esize  = mem::size_of::<T>() as c_int;
            cmd.buffer = slice.as_ptr() as *mut c_void;
            cmd.size   = slice.len() as c_int;

            match ioctl(fd, request, cmd as *mut _ as *mut c_void) {
                -1 => Err(Error::last_os_error()),
                 _ => Ok(()),
            }
        }
    }
}

#[repr(C)]
#[derive(Copy)]
pub struct Table {
    anchor: [c_char; 1024],
    name:   [c_char; 32],
    flags:  int32_t,
    fback:  int8_t,
}

impl<'a> Table {
    pub fn new(name: &str) -> Self {
        unsafe {
            let mut table: Self = mem::zeroed();
            let len = cmp::min(table.name.len() - 1, name.len());
            copy(mem::transmute(name.as_ptr()), table.name.as_mut_ptr(), len);
            table
        }
    }

    pub fn name(&self) -> &'a str {
        unsafe {
            let name = CStr::from_ptr(self.name.as_ptr());
            str::from_utf8(name.to_bytes()).unwrap()
        }
    }
}

impl Clone for Table { fn clone(&self) -> Table { *self } }

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Addr {
    addr:  [u8; 16],
    af:    uint8_t,
    net:   uint8_t,
    not:   uint8_t,
    fback: uint8_t,
}

impl Addr {
    pub fn new(ip: IpAddr) -> Self {
        unsafe {
            let mut addr: Self = mem::zeroed();
            match ip {
                IpAddr::V4(v4) => {
                    addr.af  = AF_INET as uint8_t;
                    addr.net = 32;
                    copy(v4.octets().as_ptr(), addr.addr.as_mut_ptr(), 4);
                },
                IpAddr::V6(v6) => {
                    addr.af  = AF_INET6 as uint8_t;
                    addr.net = 128;
                    addr.addr = mem::transmute(Addr::bswap(v6.segments()));
                },
            }
            addr
        }
    }

    pub fn as_ip_addr(&self) -> IpAddr {
        match self.af as i32 {
            AF_INET => {
                let w = &self.addr;
                IpAddr::V4(Ipv4Addr::new(w[0], w[1], w[2], w[3]))
            },
            AF_INET6 => unsafe {
                let w = Addr::bswap(mem::transmute_copy(&self.addr));
                IpAddr::V6(Ipv6Addr::new(w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]))
            },
            _ => panic!("Unknown address family"),
        }
    }

    #[cfg(target_endian = "little")]
    unsafe fn bswap(mut words: [u16; 8]) -> [u16; 8] {
        for word in words.iter_mut() {
            *word = bswap16(*word);
        }
        words
    }
}

#[repr(C)]
struct Command {
    table:   Table,
    buffer:  *mut c_void,
    esize:   c_int,
    size:    c_int,
    size2:   c_int,
    nadd:    c_int,
    ndel:    c_int,
    nchange: c_int,
    flags:   c_int,
    ticket:  uint32_t,
}

impl Command {
    fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    fn with_table(table: &str) -> Self {
        let mut cmd = Command::new();
        unsafe {
            let ptr = cmd.table.name.as_mut_ptr();
            let len = cmp::min(cmd.table.name.len() - 1, table.len());
            copy(mem::transmute(table.as_ptr()), ptr, len);
        }
        cmd
    }
}
