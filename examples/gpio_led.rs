// examples/gpio_led.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln,rtt_init_print};
use stm32f1xx_hal::{gpio::{ErasedPin, Output}, pac, prelude::*};

const BOARD: &str = "Nucleo-F103RB";
const BLINK_MS: u32 = 500;
const STROBE_MS: u32 = 50;

fn set_leds(led_set: &mut [ErasedPin<Output>], led_on: bool) {
  if led_on {
    for led in led_set {
      led.set_high();
    }
  } else {
    for led in led_set {
      led.set_low();
    }
  }
}

#[entry]
fn main() -> ! {
  // Access device specific peripherals, and acquire GPIOA, GPIOB GPIOC.
  let dp = pac::Peripherals::take().unwrap();
  let mut gpioa = dp.GPIOA.split();
  let mut gpiob = dp.GPIOB.split();
  let mut gpioc = dp.GPIOC.split();

  // Configure GPIO pins as push-pull output.
  // For pins 0-7, use `crl`, and for pins 8-15, use `crh`.
  // `erase()` removes the type so different pins can be collected in an array.
  //   Wire external LEDs as follows.
  //     GPIO Pin >---|>|---[R]--- GND
  //                  LED   Resistor
  let mut leds_static = [
    gpioa.pa7.into_push_pull_output(&mut gpioa.crl).erase(), // Arduino D11/PWM/MOSI
    gpiob.pb6.into_push_pull_output(&mut gpiob.crl).erase(), // Arduino D10/PWM/CS
    gpioc.pc7.into_push_pull_output(&mut gpioc.crl).erase(), // Arduino D9/PWM
  ];
  let mut leds_blink = [
    gpiob.pb10.into_push_pull_output(&mut gpiob.crh).erase(), // Arduino D6/PWM
    gpioa.pa8.into_push_pull_output(&mut gpioa.crh).erase(), // Arduino D7
  ];
  let mut leds_strobe = [
    gpioa.pa9.into_push_pull_output(&mut gpioa.crh).erase(), // Arduino D8
    gpiob.pb5.into_push_pull_output(&mut gpiob.crl).erase(), // Arduino D4
  ];
  let mut leds_controlled = [
    gpioa.pa5.into_push_pull_output(&mut gpioa.crl).erase(), // On Board LED LD2
    gpioa.pa10.into_push_pull_output(&mut gpioa.crh).erase(), // Arduino D2
  ];

  // Acquire read-only user button B1, not mutable.
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

  rtt_init_print!();
  rprintln!("Hello, {}!", BOARD);
  rprintln!("Hold user button B1 to activate controlled LED.");

  let mut counter: u32 = 0;
  let static_on: bool = true;
  let mut blink_on: bool;
  let mut strobe_on: bool = false;
  let mut controlled_on: bool = false;
  loop {
    delay.delay_ms(STROBE_MS);
    counter = counter + STROBE_MS;
    if 2 * BLINK_MS < counter {
      counter = 0;
    }
    strobe_on = !strobe_on;
    blink_on = BLINK_MS <= counter;
    if button.is_low() {
      if !controlled_on {
        rprintln!("On");
      }
      controlled_on = true;
    } else {
      if controlled_on {
        rprintln!("Off");
      }
      controlled_on = false;
    }
    set_leds(&mut leds_static, static_on);
    set_leds(&mut leds_blink, blink_on);
    set_leds(&mut leds_strobe, strobe_on);
    set_leds(&mut leds_controlled, controlled_on);
  }
}

