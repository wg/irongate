// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use std::io::Write;
use std::fs::{self, File};
use tempdir::TempDir;
use super::*;

#[test]
fn dir_must_exist() {
    let dir = TempDir::new("test").unwrap();
    let file = dir.path().join("data.txt");
    dir.close().unwrap();
    assert!(Tailer::new(&file).is_err());
}

#[test]
fn find_created() {
    let dir = TempDir::new("test").unwrap();
    let file = dir.path().join("data.txt");

    let mut tailer = Tailer::new(&file).unwrap();
    writeln!(&mut File::create(&file).unwrap(), "foo").unwrap();

    loop {
        if let Some(line) = tailer.next_line(None).unwrap() {
            assert_eq!("foo", line);
            break;
        }
    }
}

#[test]
fn find_existing() {
    let dir = TempDir::new("test").unwrap();
    let file = dir.path().join("data.txt");

    let created = &mut File::create(&file).unwrap();
    writeln!(created, "foo").unwrap();
    writeln!(created, "bar").unwrap();
    writeln!(created, "baz").unwrap();

    let mut tailer = Tailer::new(&file).unwrap();
    assert_eq!(Some("foo"), tailer.next_line(None).unwrap());
    assert_eq!(Some("bar"), tailer.next_line(None).unwrap());
    assert_eq!(Some("baz"), tailer.next_line(None).unwrap());
}

#[test]
fn follow_rename() {
    let dir = TempDir::new("test").unwrap();
    let file = dir.path().join("data.txt");
    let mut tailer = Tailer::new(&file).unwrap();

    let write = |file, value| {
        let file = &mut File::create(&file).unwrap();
        file.write_fmt(format_args!("{}\n", value)).unwrap();
    };

    write(&file, "foo");
    assert_eq!(None,        tailer.next_line(None).unwrap());
    assert_eq!(Some("foo"), tailer.next_line(None).unwrap());
    fs::rename(&file, &dir.path().join("rename.txt")).unwrap();
    write(&file, "bar");
    assert_eq!(None,        tailer.next_line(None).unwrap());
    assert_eq!(Some("bar"), tailer.next_line(None).unwrap());
}
