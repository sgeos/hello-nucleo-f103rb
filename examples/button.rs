// examples/button.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{ErasedPin, Output},
    pac,
    prelude::*,
};

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
    let mut led_set = [
        gpioa.pa5.into_push_pull_output(&mut gpioa.crl).erase(), // On Board LED LD2
        // Optionally, connect some LEDs to GPIO.
        // Wire external LEDs as follows.
        //   GPIO Pin >---|>|---[R]--- GND
        //                LED   Resistor
        gpioa.pa7.into_push_pull_output(&mut gpioa.crl).erase(), // Arduino D11/PWM/MOSI
        gpiob.pb6.into_push_pull_output(&mut gpiob.crl).erase(), // Arduino D10/PWM/CS
        gpioc.pc7.into_push_pull_output(&mut gpioc.crl).erase(), // Arduino D9/PWM
        gpioa.pa9.into_push_pull_output(&mut gpioa.crh).erase(), // Arduino D8
        gpioa.pa8.into_push_pull_output(&mut gpioa.crh).erase(), // Arduino D7
        gpiob.pb10.into_push_pull_output(&mut gpiob.crh).erase(), // Arduino D6/PWM
        gpiob.pb5.into_push_pull_output(&mut gpiob.crl).erase(), // Arduino D4
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
    rprintln!("Hold user button B1 to strobe LEDs.");

    let mut strobe: bool = false;
    let mut led_on: bool = false;
    loop {
        if button.is_high() {
            delay.delay_ms(BLINK_MS);
            if strobe {
                rprintln!("Blinking...");
            }
            strobe = false;
        } else {
            delay.delay_ms(STROBE_MS);
            if !strobe {
                rprintln!("Strobe!");
            }
            strobe = true;
        }
        set_leds(&mut led_set, led_on);
        led_on = !led_on;
    }
}
