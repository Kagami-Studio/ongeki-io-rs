use dyn_dyn::dyn_dyn_impl;
use windows::Win32::UI::WindowsAndMessaging::{self, SM_CXSCREEN};
use windows::Win32::Foundation::POINT;

use crate::drivers::{Driver, LeverDriver, PollDriver};
use crate::enums::HResult;

use super::hid;

/// Lever range minimum value
const LEVER_MIN: i32 = -32768;
/// Lever range maximum value
const LEVER_MAX: i32 = 32768;

#[derive(Debug, Default)]
pub struct MouseIO {
    lever: i16,
}

impl MouseIO {
    pub fn new() -> Self {
        Self::default()
    }
}

#[dyn_dyn_impl(Driver, PollDriver, LeverDriver)]
impl Driver for MouseIO {}

impl PollDriver for MouseIO {
    fn poll(&mut self) -> HResult {
        unsafe {
            let mut p = POINT::default();
            WindowsAndMessaging::GetCursorPos(&mut p as *mut POINT).unwrap();
            let screen_width = WindowsAndMessaging::GetSystemMetrics(SM_CXSCREEN);
            
            // Clamp mouse position to screen bounds
            let mouse_x = p.x.clamp(0, screen_width);

            // Map mouse position to lever range
            let mapped_x = hid::map(mouse_x, 0, screen_width, LEVER_MIN, LEVER_MAX);
            self.lever = mapped_x as i16;
        }
        HResult::Ok
    }
}

impl LeverDriver for MouseIO {
    fn lever(&self) -> i16 {
        self.lever
    }
}

