// src/main.rs

#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln,rtt_init_print};

#[entry]
fn main() -> ! {
  let board = "Nucleo-F103RB";
  rtt_init_print!();
  rprintln!("Hello, {}!", board);
  let mut x: usize = 0;
  loop {
    x += 1;
    if 100_000 < x {
      x = 0;
    }
    match x {
      0 => rprintln!("Echo..."),
      _ => nop(),
    }
  }
}

