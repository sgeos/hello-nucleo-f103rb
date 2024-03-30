// src/main.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln,rtt_init_print};

const BOARD: &str = "Nucleo-F103RB";
const DELAY: usize = 100_000;

#[entry]
fn main() -> ! {
  rtt_init_print!();
  rprintln!("Hello, {}!", BOARD);
  let mut counter: usize = 0;
  loop {
    counter += 1;
    if DELAY < counter {
      counter = 0;
    }
    match counter {
      0 => rprintln!("Echo..."),
      _ => nop(),
    }
  }
}

