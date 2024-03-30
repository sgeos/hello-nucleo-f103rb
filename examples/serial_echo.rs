// examples/serial_echo.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::{String};
use nb::block;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{pac, pac::USART2, prelude::*, serial::{Config, Serial, Tx}};

const BOARD: &str = "Nucleo-F103RB";
const BUFFER_SIZE: usize = 128;
const DELAY_MS: u32 = 50;

#[derive(PartialEq)]
enum TextMode {
  NormalCase,
  ForceUpper,
  ForceLower,
}

fn send_string(tx: &mut Tx<USART2>, string: &str) {
  rprintln!("{}", string);
  block!(tx.write(b'\r')).ok();
  for c in string.as_bytes() {
    block!(tx.write(*c)).ok();
  }
  block!(tx.write(b'\r')).ok();
  block!(tx.write(b'\n')).ok();
}

fn send_start_message(tx: &mut Tx<USART2>) {
  let mut buffer: String<BUFFER_SIZE> = String::new();
  write!(buffer, "Hello, {}!", BOARD).unwrap();
  send_string(tx, &buffer);
}

fn send_help_text(tx: &mut Tx<USART2>) {
  let help_text = "\
Press user button B1 to cycle through text conversion modes.\r\n\
The following text conversion commands can be sent of USART:\r\n\
= : Echo lines unchanged.\r\n\
+ : Echo lines in upper case.\r\n\
- : Echo lines in lower case.\r\n\
? : Display this help message.\
";
  send_string(tx, help_text);
}

#[entry]
fn main() -> ! {
  // Access device specific peripherals.
  let dp = pac::Peripherals::take().unwrap();

  // Configure GPIO pins as push-pull output.
  // For pins 0-7, use `crl`, and for pins 8-15, use `crh`.
  let mut gpioa = dp.GPIOA.split();
  let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl); // On Board LED LD2

  // Acquire read-only user button B1, not mutable.
  let gpioc = dp.GPIOC.split();
  let button = gpioc.pc13;

  // Take ownership of raw flash and rcc devices.
  let mut flash = dp.FLASH.constrain();
  let rcc = dp.RCC.constrain();

  // Set up system clock and configure delay provider.
  let clocks = rcc
    .cfgr
    .use_hse(8.MHz())
    .sysclk(48.MHz())
    .freeze(&mut flash.acr);
  let mut delay = dp.TIM2.delay_us(&clocks);

  // Acquire alternate function input/output (AFIO).
  let mut afio = dp.AFIO.constrain();

  // Prepare Tx and Rx pins, and setup ST-Link connected USART2.
  let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
  let rx = gpioa.pa3;
  let serial = Serial::new(
    dp.USART2,
    (tx, rx),
    &mut afio.mapr,
    Config::default().baudrate(115200.bps()),
    &clocks,
  );
  let (mut tx, mut rx) = serial.split();

  // Use RTT because `cargo embed` expects it.
  // Also using RTT in when writing text to USART.
  rtt_init_print!();

  send_start_message(&mut tx);
  send_help_text(&mut tx);

  let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
  let mut index: usize = 0;

  let mut button_down = false;
  let mut text_mode = TextMode::NormalCase;
  let mut mode_change: bool = false;
  let mut flush_buffer: bool = false;
  let mut reset_buffer: bool = false;
  loop {
    match rx.read() {
      Ok(b'?') => {
        send_help_text(&mut tx);
      },
      Ok(b'=') => {
        if TextMode::NormalCase != text_mode {
          text_mode = TextMode::NormalCase;
          mode_change = true;
        }
      },
      Ok(b'+') => {
        if TextMode::ForceUpper != text_mode {
          text_mode = TextMode::ForceUpper;
          mode_change = true;
        }
      },
      Ok(b'-') => {
        if TextMode::ForceLower != text_mode {
          text_mode = TextMode::ForceLower;
          mode_change = true;
        }
      },
      Ok(b'\r') => {
        flush_buffer = true;
        reset_buffer = true;
      },
      Ok(c) => {
        if index < BUFFER_SIZE {
          buffer[index] = c;
          index += 1;
          block!(tx.write(c)).ok(); // Echo back the received character
        }
      }
      Err(nb::Error::WouldBlock) => (),
      Err(_) => (),
    }
    let button_state = button.is_low();
    if button_state && !button_down {
      // Button was just pressed. Cycle through the text modes.
      text_mode = match text_mode {
        TextMode::NormalCase => TextMode::ForceUpper,
        TextMode::ForceUpper => TextMode::ForceLower,
        TextMode::ForceLower => TextMode::NormalCase,
      };
      mode_change = true;
    }
    button_down = button_state;
    if mode_change {
      if TextMode::NormalCase == text_mode {
        led.set_low();
      } else {
        led.set_high();
      }
      match text_mode {
        TextMode::NormalCase => send_string(&mut tx, "Use normal case."),
        TextMode::ForceUpper => send_string(&mut tx, "Force upper case."),
        TextMode::ForceLower => send_string(&mut tx, "Force lower case."),
      }
      mode_change = false;
      flush_buffer = true;
    }
    if flush_buffer && 0 < index {
      // Echo the buffer contents up to the current index
      block!(tx.write(b'\r')).ok();
      for character in &buffer[0..index] {
        let mut c = *character;
        c = match text_mode {
          TextMode::NormalCase => c,
          TextMode::ForceUpper => if (b'a'..=b'z').contains(&c) { c - 32 } else { c },
          TextMode::ForceLower => if (b'A'..=b'Z').contains(&c) { c + 32 } else { c },
        };
        block!(tx.write(c)).ok();
      }
    }
    flush_buffer = false;
    if reset_buffer && 0 < index {
      index = 0; // Reset buffer index
      block!(tx.write(b'\r')).ok();
      block!(tx.write(b'\n')).ok();
    }
    reset_buffer = false;
    // Simple rate limiting
    delay.delay_ms(DELAY_MS);
  }
}

