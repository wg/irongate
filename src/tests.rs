// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use chrono::{DateTime, UTC, TimeZone};

const REFERENCE: &'static str = "2000-01-02 03:04:05 +00:00";

#[test]
fn timestamp_ok() {
    assert_eq!(timestamp("Jan  2 03:04:05"), UTC.ymd(2000, 01, 02).and_hms(03, 04, 05));
    assert_eq!(timestamp("Jan 22 03:04:05"), UTC.ymd(2000, 01, 22).and_hms(03, 04, 05));
}

#[test]
fn timestamp_relative_year() {
    assert_eq!(timestamp("Dec  2 03:04:05"), UTC.ymd(1999, 12, 02).and_hms(03, 04, 05));
    assert_eq!(timestamp("Dec 31 23:23:59"), UTC.ymd(1999, 12, 31).and_hms(23, 23, 59));
    assert_eq!(timestamp("Jan  1 00:00:00"), UTC.ymd(2000, 01, 01).and_hms(00, 00, 00));
}

#[test]
fn timestamp_invalid() {
    let reference = DateTime::parse_from_str(REFERENCE, "%F %T %z").unwrap();
    let invalid   = "Boomtime Chaos 2nd, YOLD 3166";
    assert!(super::timestamp(invalid, reference.with_timezone(&UTC)).is_err());
}

fn timestamp(s: &str) -> DateTime<UTC> {
    let reference = DateTime::parse_from_str(REFERENCE, "%F %T %z").unwrap();
    super::timestamp(s, reference.with_timezone(&UTC)).unwrap()
}

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn sshd_log_match() {
    let events = [
        "sshd[24019]: Failed keyboard-interactive/pam for invalid user admin from 192.254.79.89 port 2971 ssh2",
        "sshd[37405]: Failed keyboard-interactive/pam for root from 193.107.17.72 port 2436 ssh2",
        "sshd[89737]: Received disconnect from 103.41.124.25: 11:  [preauth]",
        "sshd[92498]: Received disconnect from 8.254.73.28: 11: Bye Bye [preauth]",
        "sshd[92736]: Invalid user postgres from 8.254.73.28",
        "sshd[78190]: Failed keyboard-interactive/pam for invalid user admin from 2404:6800:4004:814::200e port 50534 ssh2",
    ];
    let sep = regex!("( |: )");

    for event in &events {
        let addr = sep.split(event).map(|w| w.parse::<IpAddr>()).find(|r| r.is_ok()).unwrap().unwrap();
        assert_eq!(Some(addr), super::matches(event));
    }
}

#[test]
fn sshd_log_nomatch() {
    let events = [
        "sshd[92368]: Did not receive identification string from 128.61.240.66",
        "sshd[92736]: input_userauth_request: invalid user postgres [preauth]",
        "sshd[56043]: Received disconnect from 128.61.240.66: 11: disconnected by user",
    ];

    for event in &events {
        assert_eq!(None, super::matches(event));
    }
}

#[test]
fn ip_addr() {
    let v4 = match "1.2.3.4".parse() {
        Ok(IpAddr::V4(addr)) => addr,
        _                    => panic!("invalid ipv4 addr"),
    };

    let v6 = match "2404:6800:4004:814::200e".parse() {
        Ok(IpAddr::V6(addr)) => addr,
        _                    => panic!("invalid ipv6 addr"),
    };

    assert_eq!("1.2.3.4", v4.to_string());
    assert_eq!("2404:6800:4004:814::200e", v6.to_string());

    assert_eq!(true, v4.is_global());
    assert_eq!(true, v6.is_global());

    assert_eq!(false, "10.0.0.1".parse::<Ipv4Addr>().unwrap().is_global());
    assert_eq!(false,  "::1".parse::<Ipv6Addr>().unwrap().is_global());
}
