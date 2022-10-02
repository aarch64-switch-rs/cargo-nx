#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

extern crate alloc;

use nx::result::*;
use nx::util;
use nx::svc;
use nx::diag::abort;
use nx::diag::log;

use core::panic;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    diag_log!(log::lm::LmLogger { log::LogSeverity::Trace, false } => "Hello world!");

    loop {
        // Sleep 10ms (aka 10'000'000 ns)
        svc::sleep_thread(10_000_000)?;
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::lm::LmLogger>(info, abort::AbortLevel::FatalThrow())
}