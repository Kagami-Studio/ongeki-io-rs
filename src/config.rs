//! Configuration structures for various input and output drivers
//!
//! This module defines the configuration options for all supported drivers.
//! Configurations are typically loaded from a TOML file.

use serde::{Deserialize, Serialize};

/// Keyboard driver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBoardConfig {
    /// Whether the keyboard driver is enabled
    pub enabled: bool,
    /// Virtual key code for Test button
    pub test: i32,
    /// Virtual key code for Service button
    pub service: i32,
    /// Virtual key code for Coin button
    pub coin: i32,
}

/// Mouse driver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfig {
    /// Whether the mouse driver is enabled
    pub enabled: bool,
}

/// LED debug output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LEDebugConfig {
    /// Whether LED debug output is enabled
    pub enabled: bool,
}

/// HID (USB) controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HIDConfig {
    /// Whether the HID driver is enabled
    pub enabled: bool,
    /// USB Vendor ID
    pub vid: u16,
    /// USB Product ID
    pub pid: u16,
    /// USB Interface number to use
    pub interface: i32,
    /// Left boundary for lever calibration
    pub lever_left: i16,
    /// Right boundary for lever calibration
    pub lever_right: i16,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keyboard: KeyBoardConfig,
    pub mouse: MouseConfig,
    pub hid: HIDConfig,
    pub led_debug: LEDebugConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keyboard: KeyBoardConfig {
                enabled: true,
                test: 0x31,
                service: 0x32,
                coin: 0x33,
            },
            mouse: MouseConfig { enabled: true },
            hid: HIDConfig {
                enabled: false,
                vid: 0x2341,
                pid: 0x8036,
                interface: 1,
                lever_left: i16::MIN,
                lever_right: i16::MAX,
            },
            led_debug: LEDebugConfig { enabled: true },
        }
    }
}
