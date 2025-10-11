#![no_main]
#![no_std]

use libc::{handle::Handle, main};

#[main]
fn main(_: Handle) {}

// include!(concat!(env!("OUT_DIR"), "/hello.rs"));
