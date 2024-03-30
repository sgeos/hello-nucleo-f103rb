// examples/serial_echo.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

/// This Rust program runs on the Nucleo-F103RB board and demonstrates key concepts
/// of embedded systems programming without a standard library, targeting the
/// STM32F103RB microcontroller. It showcases:
///
/// 1. Reading from USART to receive commands from a serial terminal, responding to
///    commands to change text conversion modes (normal, upper case, lower case,
///    inverted case), and displaying help information.
///
/// 2. Using a push-button to cycle through text conversion modes, affecting how
///    text received from USART is echoed back:
///    - Normal Case: Echoes text unchanged.
///    - Force Upper Case: Converts all alphabetic characters to uppercase.
///    - Force Lower Case: Converts all alphabetic characters to lowercase.
///    - Inverted Case: Inverts the case of alphabetic characters.
///
/// 3. Controlling an LED based on the current text mode:
///    - Off in Normal Case mode.
///    - On in Force Upper Case mode.
///    - Blinking in Force Lower Case mode.
///    - Strobing in Inverted Case mode.
///
/// 4. Implementing software-based delay to control execution rate and LED patterns.
///
/// The main loop handles reading from USART, interpreting button presses, echoing
/// characters per the current text mode, and controlling the LED state. It also
/// includes functionality to flush the USART buffer and send strings to the serial
/// terminal, providing user feedback.
///
/// Constants like BLINK_MS, STROBE_MS, DELAY_MS, and DELAY_COUNTER_MAX define the
/// timing for LED control. Modular arithmetic determines the LED's state (on, off,
/// blink, strobe) based on `counter`, while the main loop's timing and `counter`
/// increment rely on the hardware timer (TIM2) for precise delay intervals. This
/// ensures accurate control over periodic events like LED blinking and strobing,
/// alongside the program's execution rate.
///
/// This example demonstrates handling of peripheral I/O (USART and GPIO), conditional
/// logic based on external inputs (USART commands and button state), and basic use
/// of Rust's type system (enums, match statements) in an embedded context without
/// the standard library.

use core::{fmt::Write, str};
use cortex_m_rt::entry;
use heapless::String;
use nb::block;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{ErasedPin, Output},
    pac,
    pac::USART2,
    prelude::*,
    serial::{Config, Serial, Tx},
};

const BOARD: &str = "Nucleo-F103RB";
const BUFFER_SIZE: usize = 128;
const CASE_OFFSET: u8 = 0x20;
const BLINK_MS: u32 = 500;
const STROBE_MS: u32 = 50;
const DELAY_MS: u32 = if BLINK_MS < STROBE_MS {
    BLINK_MS
} else {
    STROBE_MS
};
const DELAY_COUNTER_MAX: u32 = 2 * if STROBE_MS < BLINK_MS {
    BLINK_MS
} else {
    STROBE_MS
};

#[derive(PartialEq)]
enum TextMode {
    NormalCase,
    ForceUpper,
    ForceLower,
    InvertedCase,
}

#[derive(PartialEq)]
enum LedMode {
    Off,
    On,
    Blink(u32), // Value represents the blink period in milliseconds
}

impl LedMode {
    pub fn control_led(&self, led: &mut ErasedPin<Output>, counter: u32) {
        match *self {
            LedMode::Off => led.set_low(),
            LedMode::On => led.set_high(),
            LedMode::Blink(period) => {
                if (counter / period) % 2 == 0 {
                    led.set_high();
                } else {
                    led.set_low();
                }
            }
        }
    }
}

impl From<&TextMode> for LedMode {
    fn from(text_mode: &TextMode) -> Self {
        match text_mode {
            TextMode::NormalCase => LedMode::Off,
            TextMode::ForceUpper => LedMode::On,
            TextMode::ForceLower => LedMode::Blink(BLINK_MS),
            TextMode::InvertedCase => LedMode::Blink(STROBE_MS),
        }
    }
}

fn is_lowercase(c: u8) -> bool {
    (b'a'..=b'z').contains(&c)
}

fn is_uppercase(c: u8) -> bool {
    (b'A'..=b'Z').contains(&c)
}

fn convert_case(c: u8, text_mode: &TextMode) -> u8 {
    let mut result = c;
    result = result
        + match text_mode {
            TextMode::ForceLower | TextMode::InvertedCase if is_uppercase(c) => CASE_OFFSET,
            _ => 0,
        };
    result = result
        - match text_mode {
            TextMode::ForceUpper | TextMode::InvertedCase if is_lowercase(c) => CASE_OFFSET,
            _ => 0,
        };
    result
}

fn flush_buffer(
    tx: &mut Tx<USART2>,
    buffer: &[u8],
    index: usize,
    text_mode: &TextMode,
) -> nb::Result<(), core::fmt::Error> {
    block!(tx.write(b'\r')).ok();
    for c in &buffer[..index] {
        block!(tx.write(convert_case(*c, text_mode))).ok();
    }
    block!(tx.flush()).ok();
    Ok(())
}

fn send_string(tx: &mut Tx<USART2>, string: &str) -> nb::Result<(), core::fmt::Error> {
    rprintln!("{}", string);
    write!(tx, "\r{}\r\n", string).ok();
    block!(tx.flush()).ok();
    Ok(())
}

fn send_start_message(tx: &mut Tx<USART2>) -> nb::Result<(), core::fmt::Error> {
    let mut buffer: String<BUFFER_SIZE> = String::new();
    write!(buffer, "Hello, {}!", BOARD).ok();
    send_string(tx, &buffer)
}

fn send_help_text(tx: &mut Tx<USART2>) -> nb::Result<(), core::fmt::Error> {
    let help_text = "\
Press user button B1 to cycle through text conversion modes.\r\n\
The following text conversion commands can be sent of USART:\r\n\
= : Echo lines unchanged.\r\n\
+ : Echo lines in upper case.\r\n\
- : Echo lines in lower case.\r\n\
~ : Echo lines in inverted case.\r\n\
? : Display this help message.\
";
    send_string(tx, help_text)
}

#[entry]
fn main() -> ! {
    // Access device specific peripherals.
    let dp = pac::Peripherals::take().unwrap();

    // Configure GPIO pins as push-pull output.
    // For pins 0-7, use `crl`, and for pins 8-15, use `crh`.
    let mut gpioa = dp.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl).erase(); // On Board LED LD2

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

    let _ = send_start_message(&mut tx);
    let _ = send_help_text(&mut tx);

    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut index: usize = 0;

    let mut counter: u32 = 0;
    let mut button_down = false;
    let mut text_mode = TextMode::NormalCase;
    let mut mode_change: bool = false;
    let mut do_flush_buffer: bool = false;
    let mut reset_buffer: bool = false;
    loop {
        match rx.read() {
            Ok(b'?') => {
                let _ = send_help_text(&mut tx);
            }
            Ok(b'=') => {
                if TextMode::NormalCase != text_mode {
                    text_mode = TextMode::NormalCase;
                    mode_change = true;
                }
            }
            Ok(b'+') => {
                if TextMode::ForceUpper != text_mode {
                    text_mode = TextMode::ForceUpper;
                    mode_change = true;
                }
            }
            Ok(b'-') => {
                if TextMode::ForceLower != text_mode {
                    text_mode = TextMode::ForceLower;
                    mode_change = true;
                }
            }
            Ok(b'~') => {
                if TextMode::InvertedCase != text_mode {
                    text_mode = TextMode::InvertedCase;
                    mode_change = true;
                }
            }
            Ok(b'\r') => {
                do_flush_buffer = true;
                reset_buffer = true;
            }
            Ok(c) => {
                if index < BUFFER_SIZE {
                    buffer[index] = c;
                    index += 1;
                    // Echo back the received character.
                    block!(tx.write(convert_case(c, &text_mode))).ok();
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
                TextMode::ForceLower => TextMode::InvertedCase,
                TextMode::InvertedCase => TextMode::NormalCase,
            };
            mode_change = true;
        }
        button_down = button_state;
        let led_mode: LedMode = (&text_mode).into();
        led_mode.control_led(&mut led, counter);
        if mode_change {
            let _ = match text_mode {
                TextMode::NormalCase => send_string(&mut tx, "Use normal case."),
                TextMode::ForceUpper => send_string(&mut tx, "Force upper case."),
                TextMode::ForceLower => send_string(&mut tx, "Force lower case."),
                TextMode::InvertedCase => send_string(&mut tx, "Use inverted case."),
            };
            mode_change = false;
            do_flush_buffer = true;
        }
        if do_flush_buffer && 0 < index {
            let _ = flush_buffer(&mut tx, &buffer, index, &text_mode);
        }
        do_flush_buffer = false;
        if reset_buffer && 0 < index {
            index = 0; // Reset buffer index
            block!(tx.write(b'\r')).ok();
            block!(tx.write(b'\n')).ok();
        }
        reset_buffer = false;

        // Simple rate limiting
        delay.delay_ms(DELAY_MS);
        counter = (counter + DELAY_MS) % DELAY_COUNTER_MAX;
    }
}
