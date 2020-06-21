//! Overriding an exception handler
//!
//! You can override an exception handler using the [`#[exception]`][1] attribute.
//!
//! [1]: https://rust-embedded.github.io/cortex-m-rt/0.6.1/cortex_m_rt_macros/fn.exception.html
//!
//! ---
#![feature(alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;
use panic_halt as _;

use core::sync::atomic::AtomicBool;
use core::alloc::Layout;
use core::cell::Cell;
use alloc::boxed::Box;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::Peripherals;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::{debug, hprint, hprintln};
use alloc_cortex_m::CortexMHeap;

use futures::task::{Context, Poll};
use futures::future::lazy;
use futures::task::{LocalSpawn, Waker};
use embedded_futures::CortexMExecutor;


// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024 * 2; // in bytes

static mut MULTIPLE_OF_1: Option<Waker> = None;
static mut MULTIPLE_OF_2: Option<Waker> = None;
static mut MULTIPLE_OF_3: Option<Waker> = None;

async fn wait_on_multiple_of_one(cx: &mut Context<'_>) -> Poll<()> {
    unsafe {
        MULTIPLE_OF_1 = Some(cx.waker().clone());
    }
    loop {
        hprintln!("x1").unwrap();
        return Poll::Pending;
    }
}

async fn wait_on_multiple_of_two(cx: &mut Context<'_>) -> Poll<()> {
    unsafe {
        MULTIPLE_OF_2 = Some(cx.waker().clone());
    }
    loop {
        hprintln!("x2").unwrap();
        return Poll::Pending;
    }
}

fn wait_on_multiple_of_three(cx: &mut Context<'_>) -> Poll<()> {
    unsafe {
        MULTIPLE_OF_3 = Some(cx.waker().clone());
    }
    loop {
        hprintln!("x3").unwrap();
        return Poll::Pending;
    }
}

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let mut pool = CortexMExecutor::new();

    let p = Peripherals::take().unwrap();
    let mut syst = p.SYST;

    // configures the system timer to trigger a SysTick exception every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(8_000_000); // period = 1s
    syst.enable_counter();
    syst.enable_interrupt();


    let spawn = pool.spawner();
    spawn.spawn_local_obj(Box::pin(lazy(move |cx| {
        async {
            wait_on_multiple_of_one(cx).await;
        }
    })).into()).unwrap();

    pool.run();

    // debug::exit(debug::EXIT_SUCCESS);
    loop {}
}

#[exception]
fn SysTick() {
    static mut COUNT: u32 = 0;

    hprint!(".").unwrap();
    *COUNT += 1;
    if *COUNT >= 10 {
        debug::exit(debug::EXIT_SUCCESS);
    }

    unsafe {
        if *COUNT % 1 == 0 {
            MULTIPLE_OF_1.as_ref().map(|w| w.wake_by_ref());
        } else if *COUNT % 2 == 0 {
            MULTIPLE_OF_2.as_ref().map(|w| w.wake_by_ref());
        } else if *COUNT % 3 == 0 {
            MULTIPLE_OF_3.as_ref().map(|w| w.wake_by_ref());
        }
    }
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    hprintln!("ALLOC ERR").unwrap();
    debug::exit(debug::EXIT_FAILURE);

    loop {}
}