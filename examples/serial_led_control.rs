// examples/serial_led_control.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
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
const BLINK_MS: u32 = 500;
const STROBE_MS: u32 = 50;
const BUFFER_SIZE: usize = 128;

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

fn send_string(tx: &mut Tx<USART2>, string: &str) {
    rprintln!("{}", string);
    write!(tx, "\r{}\r\n", string).unwrap();
    block!(tx.flush()).unwrap();
}

fn send_start_message(tx: &mut Tx<USART2>) {
    let mut buffer: String<BUFFER_SIZE> = String::new();
    write!(buffer, "Hello, {}!", BOARD).unwrap();
    send_string(tx, &buffer);
}

fn send_help_text(tx: &mut Tx<USART2>) {
    let help_text = "\
Hold user button B1 to activate controlled LED when enabled.\r\n\
The following LED control commands can be sent of USART:\r\n\
0 - Disable all LEDs\r\n\
1 - Enable all LEDs\r\n\
2 - Toggle static LED\r\n\
3 - Toggle blinking LED\r\n\
4 - Toggle strobing LED\r\n\
5 - Toggle controlled LED\r\n\
9 - Toggle LED control inversion\r\n\
? - Display this help message\
";
    send_string(tx, help_text);
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
        gpioa.pa8.into_push_pull_output(&mut gpioa.crh).erase(),  // Arduino D7
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

    let mut counter: u32 = 0;
    let static_on: bool = true;
    let mut static_enable: bool = true;
    let mut blink_on: bool;
    let mut blink_enable: bool = true;
    let mut strobe_on: bool = false;
    let mut strobe_enable: bool = true;
    let mut controlled_on: bool = false;
    let mut controlled_enable: bool = true;
    let mut controlled_inversion: bool = false;
    loop {
        match rx.read() {
            Ok(b'?') => {
                send_help_text(&mut tx);
            }
            Ok(b'0') => {
                if static_enable || blink_enable || strobe_enable || controlled_enable {
                    static_enable = false;
                    blink_enable = false;
                    strobe_enable = false;
                    controlled_enable = false;
                    send_string(&mut tx, "All LEDs disabled.");
                }
            }
            Ok(b'1') => {
                if !static_enable || !blink_enable || !strobe_enable || !controlled_enable {
                    static_enable = true;
                    blink_enable = true;
                    strobe_enable = true;
                    controlled_enable = true;
                    send_string(&mut tx, "All LEDs enabled.");
                }
            }
            Ok(b'2') => {
                static_enable = !static_enable;
                if static_enable {
                    send_string(&mut tx, "Static enabled.");
                } else {
                    send_string(&mut tx, "Static disabled.");
                }
            }
            Ok(b'3') => {
                blink_enable = !blink_enable;
                if blink_enable {
                    send_string(&mut tx, "Blink enabled.");
                } else {
                    send_string(&mut tx, "Blink disabled.");
                }
            }
            Ok(b'4') => {
                strobe_enable = !strobe_enable;
                if strobe_enable {
                    send_string(&mut tx, "Strobe enabled.");
                } else {
                    send_string(&mut tx, "Strobe disabled.");
                }
            }
            Ok(b'5') => {
                controlled_enable = !controlled_enable;
                if controlled_enable {
                    send_string(&mut tx, "Controlled enabled.");
                } else {
                    send_string(&mut tx, "Controlled disabled.");
                }
            }
            Ok(b'9') => {
                controlled_inversion = !controlled_inversion;
                if controlled_inversion {
                    send_string(&mut tx, "LED control inversion enabled.");
                } else {
                    send_string(&mut tx, "LED control inversion disabled.");
                }
            }
            Ok(_) => (),
            Err(nb::Error::WouldBlock) => (),
            Err(_) => (),
        }
        delay.delay_ms(STROBE_MS);
        counter = counter + STROBE_MS;
        if 2 * BLINK_MS < counter {
            counter = 0;
        }
        strobe_on = !strobe_on;
        blink_on = BLINK_MS <= counter;

        // Apply inversion logic here.
        let button_state = button.is_low() != controlled_inversion;
        if button_state {
            if controlled_enable && !controlled_on {
                send_string(&mut tx, "Controlled LED on.");
            }
            controlled_on = true;
        } else {
            if controlled_enable && controlled_on {
                send_string(&mut tx, "Controlled LED off.");
            }
            controlled_on = false;
        }
        set_leds(&mut leds_static, static_enable && static_on);
        set_leds(&mut leds_blink, blink_enable && blink_on);
        set_leds(&mut leds_strobe, strobe_enable && strobe_on);

        // The LED is inverted even if disabled, but the inverted state is baked in above.
        // This results in boolean logic that is hard to follow.
        if !controlled_inversion {
            set_leds(&mut leds_controlled, controlled_enable && controlled_on);
        } else {
            // Note that inversion logic is applied above, so this may appear wrong at first glance.
            set_leds(&mut leds_controlled, !controlled_enable || controlled_on);
        }
    }
}
