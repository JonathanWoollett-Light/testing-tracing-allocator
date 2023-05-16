
use alloc_track::{AllocTrack, BacktraceMode};
use std::alloc::System;

#[global_allocator]
static GLOBAL_ALLOC: AllocTrack<System> = AllocTrack::new(System, BacktraceMode::Short);

fn main() {
    // Allocate some arbitrary data.
    let x = vec![1, 2, 3];
    let _y = x
        .into_iter()
        .map(Vec::<u8>::with_capacity)
        .collect::<Vec<_>>();
}