#![feature(allocator_api)]

use std::alloc::{Layout, System};
use std::io::Write;
use std::mem::MaybeUninit;
use std::sync::Once;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};
use std::time::Instant;
use std::alloc::{GlobalAlloc, AllocError};
use std::ptr::NonNull;

static INIT: Once = Once::new();
static ALLOC_COUNTER: AtomicU32 = AtomicU32::new(0);
static DEALLOC_COUNTER: AtomicU32 = AtomicU32::new(0);

static INIT_START: Once = Once::new();
static mut START: MaybeUninit<Instant> = MaybeUninit::uninit();

struct TrackingAllocator<W: Write>(Mutex<MaybeUninit<W>>);

// `std::panic::Location::caller();` and `#[track_caller]` doesn't work well with `#[global_allocator]` this prevents to track the calling location.

unsafe impl std::alloc::GlobalAlloc for TrackingAllocator<Vec<u8, System>> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Init program start instant
        INIT_START.call_once(|| {
            unsafe {
                START.write(Instant::now());
            }
        });
        
        // Set write target in allocator.
        INIT.call_once(|| {
            self.0
                .lock()
                .unwrap()
                .write(Vec::<u8, System>::new_in(SYSTEM));
        });

        let ptr = std::alloc::System.alloc(layout);

        ALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let mut guard = self.0.lock().unwrap();
        let init = guard.assume_init_mut();
        let dur = START.assume_init_ref().elapsed();
        writeln!(init, "alloc {}:{} + {:?} {}",
            dur.as_secs(),
            dur.subsec_nanos(),
            ptr,
            layout.size()
        ).unwrap();

        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Init program start instant
        INIT_START.call_once(|| {
            unsafe {
                START.write(Instant::now());
            }
        });

        // Set write target in allocator.
        INIT.call_once(|| {
            self.0
                .lock()
                .unwrap()
                .write(Vec::<u8, System>::new_in(SYSTEM));
        });

        std::alloc::System.dealloc(ptr, layout);

        DEALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let mut guard = self.0.lock().unwrap();
        let init = guard.assume_init_mut();
        let dur = START.assume_init_ref().elapsed();
        writeln!(init, "dealloc {}:{} - {:?} {}",
            dur.as_secs(),
            dur.subsec_nanos(),
            ptr,
            layout.size()
        ).unwrap();
    }
}
unsafe impl std::alloc::Allocator for TrackingAllocator<Vec<u8, System>> {
    #[track_caller]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>,AllocError> {
        // Init program start instant
        INIT_START.call_once(|| {
            unsafe {
                START.write(Instant::now());
            }
        });
        
        // Set write target in allocator.
        INIT.call_once(|| {
            self.0
                .lock()
                .unwrap()
                .write(Vec::<u8, System>::new_in(SYSTEM));
        });

        let ptr = unsafe { std::alloc::System.alloc(layout) };

        ALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let pos = std::panic::Location::caller();

        let mut guard = self.0.lock().unwrap();
        let init = unsafe { guard.assume_init_mut() };
        let dur = unsafe { START.assume_init_ref().elapsed() };
        writeln!(init, "allocate {}:{} + {:?} {} | {:?}",
            dur.as_secs(),
            dur.subsec_nanos(),
            ptr,
            layout.size(),
            pos
        ).unwrap();

        Ok(NonNull::slice_from_raw_parts(NonNull::new(ptr).unwrap(),layout.size()))
    }
    #[track_caller]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // Init program start instant
        INIT_START.call_once(|| {
            unsafe {
                START.write(Instant::now());
            }
        });

        // Set write target in allocator.
        INIT.call_once(|| {
            self.0
                .lock()
                .unwrap()
                .write(Vec::<u8, System>::new_in(SYSTEM));
        });

        std::alloc::System.dealloc(ptr.as_ptr(), layout);

        DEALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);

        let pos = std::panic::Location::caller();

        let mut guard = self.0.lock().unwrap();
        let init = guard.assume_init_mut();
        let dur = START.assume_init_ref().elapsed();
        writeln!(init, "deallocate {}:{} - {:?} {} | {:?}",
            dur.as_secs(),
            dur.subsec_nanos(),
            ptr,
            layout.size(),
            pos
        ).unwrap();
    }
}

type Tracking = TrackingAllocator<Vec<u8, System>>;
static SYSTEM: System = System;
static TRACKING: Tracking = TrackingAllocator(Mutex::new(MaybeUninit::uninit()));
// // TODO Use [`std::cell::OnceCell`] instead when it is stabilized.
// // We use the raw system allocator for the buffer we use to track all other allocations.
// #[global_allocator]
// static mut GLOBAL: TrackingAllocator<Vec<u8, System>> =
//     TrackingAllocator(Mutex::new(MaybeUninit::uninit()));

fn main() {
    // Init program start instant
    INIT_START.call_once(|| {
        unsafe {
            START.write(Instant::now());
        }
    });

    // Allocate some arbitrary data.
    let mut x: Vec<usize,&'static Tracking> = Vec::new_in(&TRACKING);
    x.extend([1,2,3]);

    let mut y = Vec::new_in(&TRACKING);
    for i in x {
        y.push(Vec::<u8,&'static Tracking>::with_capacity_in(i,&TRACKING));
    }

    println!(
        "{} | {}",
        ALLOC_COUNTER.load(Ordering::SeqCst),
        DEALLOC_COUNTER.load(Ordering::SeqCst)
    );

    unsafe {
        TRACKING.0.lock().unwrap().assume_init_mut().flush().unwrap();

        // We need to clone here, else it may deadlock as we are preventing it acquring the lock
        // needed to do another allocation (if we where to acquire the lock th).
        let clone = TRACKING.0.lock().unwrap().assume_init_ref().clone();
        let s = std::str::from_utf8(&clone).unwrap();
        println!("{s}");
        let mut file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open("foo.txt").unwrap();
        file.write_all(&clone).unwrap();
        
    }
    println!(
        "{} | {}",
        ALLOC_COUNTER.load(Ordering::SeqCst),
        DEALLOC_COUNTER.load(Ordering::SeqCst)
    );
}
