<!-- README.md -->

# Hello, Nucleo-F103RB!

This project was made while following along with the
[Embedded Rust Setup Explained](https://www.youtube.com/watch?v=TOAynddiu5M) video.
The examples use the [stm32f1xx-hal crate](https://crates.io/crates/stm32f1xx-hal)
hardware abstraction layer (HAL) to do things like blink the LED and get button input.

This repository can likely be used as a starting point for any STM32 board,
but non F1XX boards will require more tinkering with the dependencies.
Also, note that blinking the led on your board may require different a pin.

Assuming all the tooling is installed, connect the board to the development host
machine and run commands to flash the code to the development target.

```sh
cargo embed
cargo embed --example button
```

## Basic Tooling

First, make sure [rustup](https://rustup.rs) is installed.
Use the following code on *nix or macOS, of follow the link for Windows.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Update `rustup`.

```sh
rustup update
```

Feel free to get your editor of choice set up according to the video.
The repo author uses `vim` from the command line.

## Hardware

The author went to Akihabara in Tokyo, and bought a cheap STM32 Nucleo from
[Akizukidenshi](https://akizukidenshi.com/).
It turned out to be a NUCLEO-F103RB, and that is what this repo is written for.
The assumption is that the reader has a random STM32 board, and the repo can
be made to work with minor modifications.

Finding the architecture for the board was not immediately obvious to the author,
but STM's page on the
[STM32F103](https://www.st.com/en/microcontrollers-microprocessors/stm32f103.html)
series of processors was located after some searching.
There is a page for the
[STM32F103RB](https://www.st.com/en/microcontrollers-microprocessors/stm32f103rb.html) 
(click in the grid).
Note that it is a Cortex-M3 with no FPU. Both
[STM](https://www.st.com/content/st_com/en/arm-32-bit-microcontrollers/arm-cortex-m3.html)
and [ARM](https://developer.arm.com/Processors/Cortex-M3)
have documentation for this processor.
The [UM1724 User manual](https://www.st.com/resource/en/user_manual/um1724-stm32-nucleo64-boards-mb1136-stmicroelectronics.pdf)
for **STM32 Nucleo-64 boards (MB1136)** was also located.

## Cross-Compilation Tooling

According to the
[Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
page, `thumbv7m-none-eabi` is the correct target for the bare metal Armv7-M architecture.
If you are using a different board, your target may be different.

Add the cross-compilation target with `rustup`.

```sh
rustup target add thumbv7m-none-eabi
```

Installed targets can be checked.

```sh
rustup show
```

Install other tools with `rustup` for interacting with bare metal.

```sh
rustup component add llvm-tools
cargo install cargo-binutils
```

Test one of the `binutils`.

```sh
cargo build
cargo size -- -Ax
```

# Bare Metal Rust Project

Install `cargo-embed`.

```sh
cargo install cargo-embed
```

**memory.x** is setup for the STM32F103 according to the datasheet.
**.cargo/config.toml** contains the target and compiler flags.
**Embed.toml** is also setup to use the STM32F103.
These files may need to be modified if you are using a different board.
If you need to update **Embed.toml**, note that the following command
can be used to search for a specific chip for `cargo-embed` support.

```sh
CHIP="STM32F103RB"
cargo embed --list-chips | grep -i ${CHIP}
```

If your board is connected to the development machine, you should be
able to flash the project with `cargo-embed`.

```sh
cargo embed
```

Real time transfer (RTT) messages should be printed to the terminal.
Hit **Control + C** to exit the RTT interface.

## GDB

Install `arm-none-eabi-gdb` or `gdb-multiarch` for your platform.
The video goes into details.  So far as the author can tell,
`arm-none-eabi-gdb` is not supported for **arm64** development
machines.  The GDB-related lines have been commented out in
**Embed.toml**, but debugging with GDB theoretically works if it
is installed on the host machine.

Finally, note that GDB will interfere with RTT.
**main.rs** is currently written to work with either RTT or GDB
debugging.


