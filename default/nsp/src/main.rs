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

// Using 128KB custom heap
const CUSTOM_HEAP_LEN: usize = 0x20000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_LEN] = [0; CUSTOM_HEAP_LEN];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP.len())
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
    util::simple_panic_handler::<log::lm::LmLogger>(info, abort::AbortLevel::SvcBreak())
}