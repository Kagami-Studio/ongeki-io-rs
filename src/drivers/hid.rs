use std::io::{Cursor, Write};

use crate::{
    config::HIDConfig,
    enums::{GameBtn, HResult},
};

use super::{ButtonDriver, Driver, LEDriver, LeverDriver, PollDriver, LEDriverNew};

use byteorder::WriteBytesExt;
use dyn_dyn::dyn_dyn_impl;
use hidapi::{HidApi, HidDevice};

// HID buffer and data constants
const HID_BUFFER_SIZE: usize = 64;
const HID_REPORT_SIZE: usize = 65;
const LED_REPORT_ID: u8 = 0;
const LED_COMMAND: u8 = 100;
const LED_COUNT_BOARD_1: usize = 6;

pub struct HidIO {
    lever: i16,
    left_btns: u8,
    right_btns: u8,
    config: HIDConfig,
    device: Option<HidDevice>,
}

#[dyn_dyn_impl(Driver, PollDriver, ButtonDriver, LeverDriver, LEDriver, LEDriverNew)]
impl Driver for HidIO {}
unsafe impl Sync for HidIO {}

impl HidIO {
    pub fn new(config: HIDConfig) -> Self {
        let mut s = HidIO {
            lever: 0,
            left_btns: 0,
            right_btns: 0,
            config,
            device: None,
        };
        s.try_connect_device();
        s
    }

    fn try_connect_device(&mut self) {
        let api = HidApi::new().unwrap();
        self.device = api
            .device_list()
            .filter(|d| d.vendor_id() == self.config.vid && d.product_id() == self.config.pid)
            .find_map(|d| {
                if d.interface_number() == self.config.interface {
                    println!(
                        "Ongeki IO HID: {} 已连接",
                        d.product_string().unwrap_or_default()
                    );
                    return d.open_device(&api).inspect(|d| {
                        d.set_blocking_mode(false).unwrap();
                    }).ok();
                }
                None
            });
    }
}

impl PollDriver for HidIO {
    fn poll(&mut self) -> HResult {
        let Some(ref device) = self.device else {
            self.try_connect_device();
            return HResult::Ok;
        };

        let mut data = [0u8; HID_BUFFER_SIZE];
        self.left_btns = 0;
        self.right_btns = 0;
        if let Err(e) = device.read(&mut data) {
            println!("Ongeki IO HID: 设备断开 {e}");
            self.device = None;
            return HResult::Ok;
        }

        // Map button data to left buttons
        const LEFT_BTN_MAP: [(usize, GameBtn); 5] = [
            (0, GameBtn::Btn1),
            (1, GameBtn::Btn2),
            (2, GameBtn::Btn3),
            (3, GameBtn::Side),
            (4, GameBtn::Menu),
        ];
        
        for (idx, btn) in LEFT_BTN_MAP {
            if data[idx] == 1 {
                self.left_btns |= btn as u8;
            }
        }

        // Map button data to right buttons
        const RIGHT_BTN_MAP: [(usize, GameBtn); 5] = [
            (5, GameBtn::Btn1),
            (6, GameBtn::Btn2),
            (7, GameBtn::Btn3),
            (8, GameBtn::Side),
            (9, GameBtn::Menu),
        ];
        
        for (idx, btn) in RIGHT_BTN_MAP {
            if data[idx] == 1 {
                self.right_btns |= btn as u8;
            }
        }

        // self.lever = -20 * i16::from_be_bytes([data[10], data[11]]);
        // Auto Calculation
        let lever_meta = i16::from_be_bytes([data[10], data[11]]);
        if self.config.lever_left > self.config.lever_right {
            if lever_meta > self.config.lever_left {
                self.config.lever_left = lever_meta;
            }
            if lever_meta < self.config.lever_right {
                self.config.lever_right = lever_meta;
            }
        } else {
            if lever_meta < self.config.lever_left {
                self.config.lever_left = lever_meta;
            }
            if lever_meta > self.config.lever_right {
                self.config.lever_right = lever_meta;
            }
        }

        // 映射前动态交换左右边界，确保方向正确
        let (in_min, in_max) = if self.config.lever_left < self.config.lever_right {
            (self.config.lever_left, self.config.lever_right)
        } else {
            // 如果校准值方向颠倒，交换它们
            (self.config.lever_right, self.config.lever_left)
        };

        if self.config.lever_right != self.config.lever_left {
            self.lever = map(
                i32::from(lever_meta),
                i32::from(in_min),
                i32::from(in_max),
                -32768,
                32768,
            ) as i16;
        }

        HResult::Ok
    }
}

pub(crate) fn map(x: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    // 自动处理反向输入（例如 in_min > in_max）
    let numerator = (x - in_min) * (out_max - out_min);
    let denominator = in_max - in_min;
    if denominator == 0 {
        return out_min; // 避免除以零
    }
    numerator / denominator + out_min
}

/// Helper function to extract bit from data and convert to LED brightness
/// Extracts a specific bit from the data and returns 255 if bit is 1, 0 otherwise
fn extract_led_bit(data: u32, bit_position: u32) -> u8 {
    if (data >> bit_position) & 1 == 1 {
        255
    } else {
        0
    }
}

/// LED bit positions for buttons (from MSB to LSB)
const LED_BIT_POSITIONS: [u32; 18] = [
    23, 19, 22, 20, 21, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6,
];

impl LeverDriver for HidIO {
    fn lever(&self) -> i16 {
        self.lever
    }
}

impl ButtonDriver for HidIO {
    fn op_btns(&self) -> u8 {
        0
    }

    fn left_btns(&self) -> u8 {
        self.left_btns
    }

    fn right_btns(&self) -> u8 {
        self.right_btns
    }
}

impl LEDriver for HidIO {
    fn set_led(&mut self, data: u32) {
        let Some(ref device) = self.device else {
            self.try_connect_device();
            return;
        };

        let mut buf = Cursor::new([0u8; HID_REPORT_SIZE]);
        buf.set_position(1);
        buf.write_u8(LED_REPORT_ID).unwrap();
        buf.write_u8(LED_COMMAND).unwrap();
        
        // Convert bit flags to LED brightness values
        let led_data: Vec<u8> = LED_BIT_POSITIONS
            .iter()
            .map(|&pos| extract_led_bit(data, pos))
            .collect();
        buf.write_all(&led_data).unwrap();

        if let Err(e) = device.write(buf.get_ref()) {
            println!("Ongeki IO HID: 设备断开 {e}");
            self.device = None;
        }
    }
}


impl LEDriverNew for HidIO {
    fn set_led_new(&mut self, board: u8, rgb: &[rgb::RGB8]) {
        if board == 1 {
            let Some(ref device) = self.device else {
                self.try_connect_device();
                return;
            };
    
            // Validate we have enough RGB data
            if rgb.len() < LED_COUNT_BOARD_1 {
                eprintln!("Ongeki IO HID: 警告 - RGB 数据不足，期望 {} 个，实际 {}", LED_COUNT_BOARD_1, rgb.len());
                return;
            }

            let mut buf = Cursor::new([0u8; HID_REPORT_SIZE]);
            buf.set_position(1);
            buf.write_u8(LED_REPORT_ID).unwrap();
            buf.write_u8(LED_COMMAND).unwrap();
            
            // Write RGB data for board 1 (6 LEDs = 18 bytes)
            for i in 0..LED_COUNT_BOARD_1 {
                buf.write_all(&[rgb[i].r, rgb[i].g, rgb[i].b]).unwrap();
            }

            if let Err(e) = device.write(buf.get_ref()) {
                println!("Ongeki IO HID: 设备断开 {e}");
                self.device = None;
            }
        }
    }
}


/// test
#[cfg(test)]
mod hid_test {
    use super::map;
    #[test]
    fn map_test() {
        let temp = map(12, -280, 280, -20000, 20000);
        assert_eq!(temp, 857);
    }
}