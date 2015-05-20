// Copyright (C) 2015 - Will Glozer.  All rights reserved.

use sketchy::CountMinSketch;
use chrono::{DateTime, Duration, TimeZone, UTC};
use std::collections::VecDeque;
use std::hash::Hash;

pub struct TimeWindowCMS<'a, T: Copy + Hash> {
    resolution: &'a Fn(&Duration) -> i64,
    last:       DateTime<UTC>,
    sketches:   VecDeque<CountMinSketch<T>>,
}

impl<'a, T: Copy + Hash> TimeWindowCMS<'a, T> {
    pub fn new(period: Duration, resolution: &'a Fn(&Duration) -> i64) -> TimeWindowCMS<'a, T> {
        let segments = resolution(&period);

        let mut sketches = VecDeque::with_capacity(segments as usize);
        for _ in 0..segments {
            sketches.push_back(CountMinSketch::with_confidence(0.001, 0.99));
        }

        TimeWindowCMS {
            resolution: resolution,
            last:       UTC::now(),
            sketches:   sketches,
        }
    }

    pub fn add<Z: TimeZone>(&mut self, when: DateTime<Z>, event: T) -> u64 {
        let resolution = &self.resolution;
        let when = when.with_timezone(&UTC);

        let elapsed = Duration::seconds(when.timestamp() - self.last.timestamp());
        let segments = resolution(&elapsed);
        if segments > 0 {
            for _ in 0..segments {
                let empty = CountMinSketch::with_confidence(0.001, 0.99);
                self.sketches.pop_front();
                self.sketches.push_back(empty);
            }
            self.last = when;
        }

        self.sketches.back_mut().map(|s| s.insert(event));
        self.sketches.iter().fold(0, |n, s| n + s.estimate(&event))
    }
}

#[cfg(test)]
mod tests;
