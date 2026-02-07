use super::{Driver, LEDriver, LEDriverNew};

use dyn_dyn::dyn_dyn_impl;

/// LED bit positions for debugging output
const LED_BIT_POSITIONS: [u32; 18] = [
    23, 19, 22, 20, 21, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6,
];

#[derive(Debug, Default)]
pub struct LEDebug;

impl LEDebug {
    pub fn new() -> Self {
        Self
    }
}

#[dyn_dyn_impl(Driver, LEDriver, LEDriverNew)]
impl Driver for LEDebug {}

impl LEDriver for LEDebug {
    fn set_led(&mut self, data: u32) {
        print!("Ongeki IO: Set LED\n");
        for (i, &pos) in LED_BIT_POSITIONS.iter().enumerate() {
            let value = if (data >> pos) & 1 == 1 { 255 } else { 0 };
            print!("{} ", value);
            // Add comma separator after every 3 values, except after the last one
            if (i + 1) % 3 == 0 && i + 1 < LED_BIT_POSITIONS.len() {
                print!(", ");
            }
        }
        println!();
    }
}


impl LEDriverNew for LEDebug {
    fn set_led_new(&mut self, board: u8, rgb: &[rgb::RGB8]) {
        println!("Ongeki IO: Set LED Board {board}");
        for rgb in rgb {
            print!("{:X} ", rgb);
        }
        println!();
    }
}

