//! # Ongeki IO Library
//!
//! This library provides an IO interface for ONGEKI arcade game controllers,
//! compatible with segatools. It supports multiple input drivers (keyboard, mouse, HID)
//! and LED output for controller feedback.
//!
//! ## Supported Drivers
//! - **Keyboard**: Maps keyboard keys to game buttons
//! - **Mouse**: Maps mouse X position to lever position
//! - **HID**: Support for custom HID controllers
//! - **LED Debug**: Console output for LED debugging
//!
//! ## Configuration
//! Configuration is loaded from `ongeki-io.toml` in the working directory.
//! If the file doesn't exist, a default configuration is created.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use drivers::Drivers;
use lazy_static::lazy_static;
use rgb::Rgb;
use std::sync::RwLock;
use windows::Win32::System::Console;

use enums::HResult;

mod config;
mod drivers;
mod enums;

lazy_static! {
    static ref DRIVERS: RwLock<Drivers> = RwLock::new(Drivers::new());
}

#[no_mangle]
/// Get the API version supported by this library
/// Returns: 0x0101 (version 1.1)
pub extern "C" fn mu3_io_get_api_version() -> u16 {
    0x0101
}

#[no_mangle]
/// Initialize the IO library
/// This function must be called before any other IO functions
/// Returns: HResult::Ok on success
pub extern "C" fn mu3_io_init() -> HResult {
    unsafe {
        let _ = Console::AttachConsole(Console::ATTACH_PARENT_PROCESS);
    }
    color_backtrace::install();

    println!("Ongeki IO: 启动！");

    let mut drivers = DRIVERS.write().unwrap();
    drivers.init();

    HResult::Ok
}

#[no_mangle]
/// Poll all input devices for current state
/// This should be called regularly to update button and lever states
/// Returns: HResult::Ok on success
pub extern "C" fn mu3_io_poll() -> HResult {
    let mut drivers = DRIVERS.write().unwrap();
    drivers.poll();

    HResult::Ok
}

#[no_mangle]
/// Get the state of operator buttons (test, service, coin)
/// 
/// # Arguments
/// * `option_btns` - Pointer to u8 that will receive the button state bitmap
pub extern "C" fn mu3_io_get_opbtns(option_btns: *mut u8) {
    let drivers = DRIVERS.read().unwrap();
    if !option_btns.is_null() {
        unsafe { *option_btns = drivers.op_btns() }
    }
}

#[no_mangle]
/// Get the state of game buttons for both sides
/// 
/// # Arguments
/// * `left` - Pointer to u8 that will receive the left side button state
/// * `right` - Pointer to u8 that will receive the right side button state
pub extern "C" fn mu3_io_get_gamebtns(left: *mut u8, right: *mut u8) {
    let drivers = DRIVERS.read().unwrap();
    if !left.is_null() {
        unsafe { *left = drivers.left_btns() }
    }
    if !right.is_null() {
        unsafe { *right = drivers.right_btns() }
    }
}

#[no_mangle]
/// Get the current lever position
/// 
/// # Arguments
/// * `pos` - Pointer to i16 that will receive the lever position (-32768 to 32767)
pub extern "C" fn mu3_io_get_lever(pos: *mut i16) {
    let drivers = DRIVERS.read().unwrap();
    if !pos.is_null() {
        unsafe { *pos = drivers.lever().unwrap_or(0) }
    }
}

#[no_mangle]
/// Initialize LED support
/// Returns: HResult::Ok on success
pub extern "C" fn mu3_io_led_init() -> HResult {
    HResult::Ok
}

#[no_mangle]
/// Set LED state using legacy bit-field format
/// 
/// # Arguments
/// * `data` - 32-bit value containing LED on/off states
pub extern "C" fn mu3_io_set_led(data: u32) {
    let mut drivers = DRIVERS.write().unwrap();
    drivers.set_led(data);
}

/// Update the RGB LEDs. rgb is a pointer to an array of up to 61 * 3 = 183 bytes.
///
/// ONGEKI uses one board with WS2811 protocol (each logical led corresponds to 3
/// physical leds). Board 0 is used for all cab lights and both WAD button lights.
///
/// Board 0 has 61 LEDs:
///    [0]-[1]: left side button
///    [2]-[8]: left pillar lower LEDs
///    [9]-[17]: left pillar center LEDs
///    [18]-[24]: left pillar upper LEDs
///    [25]-[35]: billboard LEDs
///    [36]-[42]: right pillar upper LEDs
///    [43]-[51]: right pillar center LEDs
///    [52]-[58]: right pillar lower LEDs
///    [59]-[60]: right side button
///
/// Board 1 has 6 LEDs:
///    [0]-[5]: 3 left and 3 right controller buttons
///
/// Each rgb value is comprised of 3 bytes in R,G,B order. The tricky part is
/// that the board 0 is called from mu3 and the board 1 is called from amdaemon.
/// So the library must be able to handle both calls, using shared memory f.e.
/// This is up to the developer to decide how to handle this, recommended way is
/// to use the amdaemon process as the main one and the mu3 process as a sub one.
///
/// Minimum API version: 0x0101
#[no_mangle]
pub extern "C" fn mu3_io_led_set_colors(board: u8, rgb: *mut u8) {
    let mut drivers = DRIVERS.write().unwrap();
    let len = if board == 0 { 183 } else { 18 };
    let data = unsafe { std::slice::from_raw_parts(rgb, len) };
    let led_colors: Vec<Rgb<u8>> = (0..len)
        .step_by(3)
        .map(|i| Rgb::new(data[i], data[i + 1], data[i + 2]))
        .collect();

    drivers.set_led_new(board, &led_colors);
}

#[cfg(test)]
mod tests {

    use super::drivers::hid;

    #[test]
    fn map_test() {
        let temp = hid::map(12, -280, 280, -20000, 20000);
        assert_eq!(temp, 857);
    }
}
