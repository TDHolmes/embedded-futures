//! How to use the heap and a dynamic memory allocator
//!
//! This example depends on the alloc-cortex-m crate so you'll have to add it to your Cargo.toml:
//!
//! ``` text
//! # or edit the Cargo.toml file manually
//! $ cargo add alloc-cortex-m
//! ```
//!
//! ---

#![feature(alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;
use panic_halt as _;

use core::alloc::Layout;

use alloc::rc::Rc;
use alloc::boxed::Box;
use core::cell::Cell;

use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

use futures::future::lazy;
use futures::task::LocalSpawn;
use embedded_futures::CortexMExecutor;


// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024 * 2; // in bytes

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let mut pool = CortexMExecutor::new();

    const ITER: usize = 20;

    let cnt = Rc::new(Cell::new(0));

    let spawn = pool.spawner();

    for _ in 0..ITER {
        let cnt = cnt.clone();
        spawn.spawn_local_obj(Box::pin(lazy(move |_| {
            cnt.set(cnt.get() + 1);
        })).into()).unwrap();
    }

    pool.run();

    if cnt.get() == ITER {
        hprintln!("future worked!").unwrap();
    } else {
        hprintln!("something went wrong...").unwrap();
    }

    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    hprintln!("ALLOC ERR").unwrap();
    debug::exit(debug::EXIT_FAILURE);

    loop {}
}
