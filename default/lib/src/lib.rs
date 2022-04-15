#![no_std]

extern crate alloc;

#[macro_use]
extern crate nx;

use nx::diag::log;

pub fn hello() {
    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Hello world!");
}