// src/main.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln,rtt_init_print};
use stm32f1xx_hal::{pac, prelude::*};

const BOARD: &str = "Nucleo-F103RB";
const DELAY: usize = 100_000;

#[entry]
fn main() -> ! {
  // Get access to the device specific peripherals, and acquire GPIOA.
  let dp = pac::Peripherals::take().unwrap();
  let mut gpioa = dp.GPIOA.split();

  // Configure GPIO A pin 5 as a push-pull output with the `crl` register.
  // For pins 8-15, use `crh` instead.
  let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);

  rtt_init_print!();
  rprintln!("Hello, {}!", BOARD);

  let mut x: usize = 0;
  loop {
    // Pre-increment to avoid blinking on first tick.
    x += 1;
    if 2 * DELAY < x {
      x = 0;
    }
    match x {
      0 => {
        rprintln!("Blink off...");
        led.set_low();
      },
      DELAY => {
        rprintln!("Blink on...");
        led.set_high();
      },
      // Use ASM nop to slow the cycle down.
      _ => nop(),
    }
  }
}

