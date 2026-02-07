use crate::{config::KeyBoardConfig, enums::{GameBtn, HResult, OpBtn}};
use super::{ButtonDriver, Driver, PollDriver};

use dyn_dyn::dyn_dyn_impl;
use windows::Win32::UI::Input::KeyboardAndMouse;


#[derive(Debug)]
pub struct KeyBoardIO {
    op_btns: u8,
    left_btns: u8,
    right_btns: u8,
    config: KeyBoardConfig
}

impl KeyBoardIO {
    pub fn new(config: KeyBoardConfig) -> Self {
        Self {
            op_btns: 0,
            left_btns: 0,
            right_btns: 0,
            config,
        }
    }
}

#[dyn_dyn_impl(Driver, PollDriver, ButtonDriver)]
impl Driver for KeyBoardIO {}

/// Helper function to check if a key is pressed
///
/// # Safety
/// This function calls Windows API GetAsyncKeyState which is safe to call
/// with any virtual key code. The function properly handles the return value.
fn is_key_pressed(key: i32) -> bool {
    unsafe { KeyboardAndMouse::GetAsyncKeyState(key) != 0 }
}

/// Helper function to set button state based on key press
///
/// # Arguments
/// * `buttons` - Mutable reference to button state bitmap
/// * `key` - Virtual key code to check
/// * `button_flag` - Button flag to set if key is pressed
fn set_button_if_pressed(buttons: &mut u8, key: i32, button_flag: u8) {
    if is_key_pressed(key) {
        *buttons |= button_flag;
    }
}

impl PollDriver for KeyBoardIO {
    fn poll(&mut self) -> HResult {
        self.op_btns = 0;

        // Check operator buttons
        set_button_if_pressed(&mut self.op_btns, self.config.test, OpBtn::Test as u8);
        set_button_if_pressed(&mut self.op_btns, self.config.service, OpBtn::Service as u8);
        set_button_if_pressed(&mut self.op_btns, self.config.coin, OpBtn::Coin as u8);

        self.left_btns = 0;
        self.right_btns = 0;

        // Left side buttons (A, S, D keys)
        set_button_if_pressed(&mut self.left_btns, 0x41, GameBtn::Btn1 as u8); // A
        set_button_if_pressed(&mut self.left_btns, 0x53, GameBtn::Btn2 as u8); // S
        set_button_if_pressed(&mut self.left_btns, 0x44, GameBtn::Btn3 as u8); // D
        set_button_if_pressed(&mut self.left_btns, 0x55, GameBtn::Menu as u8);  // U
        set_button_if_pressed(&mut self.left_btns, 0x01, GameBtn::Side as u8);  // Mouse Left
        
        // Right side buttons (J, K, L keys)
        set_button_if_pressed(&mut self.right_btns, 0x4A, GameBtn::Btn1 as u8); // J
        set_button_if_pressed(&mut self.right_btns, 0x4B, GameBtn::Btn2 as u8); // K
        set_button_if_pressed(&mut self.right_btns, 0x4C, GameBtn::Btn3 as u8); // L
        set_button_if_pressed(&mut self.right_btns, 0x4F, GameBtn::Menu as u8);  // O
        set_button_if_pressed(&mut self.right_btns, 0x02, GameBtn::Side as u8);  // Mouse Right

        HResult::Ok
    }
}

impl ButtonDriver for KeyBoardIO {
    fn op_btns(&self) -> u8 {
        self.op_btns
    }

    fn left_btns(&self) -> u8 {
        self.left_btns
    }

    fn right_btns(&self) -> u8 {
        self.right_btns
    }
}

