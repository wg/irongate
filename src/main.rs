// Copyright (C) 2015 - Will Glozer.  All rights reserved.

#![feature(convert, core, plugin, collections, ip, ip_addr, libc, duration)]
#![plugin(docopt_macros, regex_macros)]
#![allow(dead_code)]

mod cms;
mod kqueue;
mod pf;
mod posix;
mod tail;
#[macro_use]
mod log;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate docopt;
extern crate libc;
extern crate rustc_serialize;

extern crate time;
extern crate chrono;
extern crate regex;
extern crate sketchy;

use std::net::IpAddr;
use std::path::Path;
use chrono::*;
use regex::Regex;
use cms::TimeWindowCMS;
use pf::{Pf, Addr, Table};
use tail::Tailer;

docopt!(Args derive Debug, "
Usage: irongate [options] <logfile>
       irongate --help

Options:
  -l, --limit <limit>    Maximum attempts per period [default: 3].
  -p, --period <period>  Attempt monitoring period in minutes [default: 1].
  -t, --table <table>    Add addresses to this table.
", flag_limit: u64, flag_period: u64);

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let file = args.arg_logfile;
    let gate = IronGate {
        table:  &args.flag_table,
        limit:  args.flag_limit,
        period: Duration::minutes(args.flag_period as i64),
    };

    match gate.monitor(Path::new(&file)) {
        Err(e) => println!("{}", e),
        Ok(..) => (),
    }
}

struct IronGate<'a> {
    limit:  u64,
    period: Duration,
    table:  &'a str,
}

impl<'a> IronGate<'a> {
    fn monitor(&self, path: &Path) -> Result<(), tail::Error> {
        let pf = try!(Pf::new());
        let mut tailer = try!(Tailer::new(path.as_os_str()));
        let resolution = |d: &Duration| { d.num_seconds() };
        let mut cms = TimeWindowCMS::new(self.period, &resolution);

        try!(pf.add_tables(&vec![Table::new(self.table)]));

        loop {
            if let Ok(Some(line)) = tailer.next_line(None) {
                if let Ok(timestamp) = timestamp(line, Local::now()) {
                    if let Some(addr) = matches(line) {
                        if is_global(addr) && cms.add(timestamp, addr) > self.limit {
                            let addrs = vec![Addr::new(addr)];
                            if try!(pf.add_addrs(self.table, &addrs)) == 1 {
                                syslog!("Address added to table '{}': {}", self.table,  addr);
                            }
                        }
                    }
                }
            }
        }
    }
}

static PATTERNS: [Regex; 3] = [
    regex!(r"sshd\[\d+\]: Invalid user (\w+) from (?P<addr>.+)"),
    regex!(r"sshd\[\d+\]: Failed (.+) for( invalid user)? (\w+) from (?P<addr>.+) port"),
    regex!(r"sshd\[\d+\]: Received disconnect from (?P<addr>.+?):.+\[preauth\]"),
];

fn matches(line: &str) -> Option<IpAddr> {
    for regex in PATTERNS.iter() {
        if let Some(cap) = regex.captures(line) {
            return cap.name("addr").and_then(|addr| addr.parse().ok());
        }
    }
    None
}

pub fn timestamp<Z: TimeZone>(line: &str, now: DateTime<Z>) -> Result<DateTime<Z>, time::ParseError> {
    let tm  = try!(time::strptime(&line[..15], "%b %e %T"));
    let (year, month) = match tm.tm_mon as u32 + 1 {
        month if month > now.month() => (now.year() - 1, month),
        month                        => (now.year(),     month),
    };
    let day    = tm.tm_mday as u32;
    let hour   = tm.tm_hour as u32;
    let minute = tm.tm_min  as u32;
    let second = tm.tm_sec  as u32;
    Ok(now.timezone().ymd(year, month, day).and_hms(hour, minute, second))
}

fn is_global(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(addr) => addr.is_global(),
        IpAddr::V6(addr) => addr.is_global(),
    }
}

#[cfg(test)] extern crate tempdir;
#[cfg(test)] mod tests;
