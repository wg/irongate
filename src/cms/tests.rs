// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use chrono::{Duration, UTC};
use super::TimeWindowCMS;

#[test]
fn count_ok() {
    let resolution = |d: &Duration| { d.num_seconds() };
    let now = UTC::now();
    let mut cms = TimeWindowCMS::new(Duration::seconds(10), &resolution);

    assert_eq!(1, cms.add(now + Duration::seconds(0),  "foo"));
    assert_eq!(2, cms.add(now + Duration::seconds(0),  "foo"));
    assert_eq!(1, cms.add(now + Duration::seconds(0),  "bar"));
}


#[test]
fn count_across_segments() {
    let resolution = |d: &Duration| { d.num_seconds() };
    let now = UTC::now();
    let mut cms = TimeWindowCMS::new(Duration::seconds(10), &resolution);

    assert_eq!(1, cms.add(now + Duration::seconds(0),  "foo"));
    assert_eq!(2, cms.add(now + Duration::seconds(1),  "foo"));
    assert_eq!(3, cms.add(now + Duration::seconds(2),  "foo"));
    assert_eq!(1, cms.add(now + Duration::seconds(12), "foo"));
}
