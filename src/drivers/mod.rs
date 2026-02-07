//! Driver management module
//!
//! This module provides a unified interface for managing multiple input/output drivers.
//! Drivers are loaded based on configuration and polled together.

use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast};
use std::fs::{self, File};
use std::io::Write;
use std::ops::{Deref, DerefMut};

use crate::config::Config;
use crate::drivers::hid::HidIO;
use crate::enums::HResult;

#[dyn_dyn_base]
trait Driver: Sync + Send {}



pub mod hid;
mod keyboard;
mod led_debug;
mod mouse;

use self::keyboard::KeyBoardIO;
use self::led_debug::LEDebug;
use self::mouse::MouseIO;

trait PollDriver {
    fn poll(&mut self) -> HResult;
}

trait ButtonDriver {
    fn op_btns(&self) -> u8;
    fn left_btns(&self) -> u8;
    fn right_btns(&self) -> u8;
}

trait LeverDriver {
    fn lever(&self) -> i16;
}

trait LEDriver {
    fn set_led(&mut self, data: u32);
}

trait LEDriverNew {
    fn set_led_new(&mut self, board: u8, rgb: &[rgb::RGB8]);
}

pub struct Drivers(Vec<Box<dyn Driver>>);

impl Drivers {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn init(&mut self) {
        const CONFIG_PATH: &str = "ongeki-io.toml";
        let config = Self::load_or_create_config(CONFIG_PATH);

        if config.keyboard.enabled {
            self.0
                .push(Box::new(KeyBoardIO::new(config.keyboard.clone())));
        }
        if config.mouse.enabled {
            self.0.push(Box::new(MouseIO::new()));
        }
        if config.led_debug.enabled {
            self.0.push(Box::new(LEDebug::new()));
        }
        if config.hid.enabled {
            self.0.push(Box::new(HidIO::new(config.hid.clone())));
        }
    }

    /// Load configuration from file or create a default one
    fn load_or_create_config(path: &str) -> Config {
        match fs::read_to_string(path) {
            Ok(content) => {
                match toml::from_str(&content) {
                    Ok(config) => {
                        println!("Ongeki IO: 使用配置文件\n{:#?}", config);
                        config
                    }
                    Err(e) => {
                        eprintln!("Ongeki IO: 配置文件解析失败: {}, 使用默认配置", e);
                        Self::create_default_config(path)
                    }
                }
            }
            Err(_) => {
                println!("Ongeki IO: 未发现配置文件，创建默认配置");
                Self::create_default_config(path)
            }
        }
    }

    /// Create and save a default configuration file
    fn create_default_config(path: &str) -> Config {
        let config = Config::default();
        if let Ok(toml_string) = toml::to_string_pretty(&config) {
            if let Ok(mut f) = File::create(path) {
                let _ = f.write_all(toml_string.as_bytes());
            }
        }
        println!("Ongeki IO: 使用默认配置\n{:#?}", config);
        config
    }

    pub fn poll(&mut self) {
        for driver in self.0.iter_mut() {
            if let Ok(d) = dyn_dyn_cast!(mut Driver => PollDriver, driver.deref_mut()) {
                d.poll();
            }
        }
    }

    pub fn op_btns(&self) -> u8 {
        self.0
            .iter()
            .filter_map(|d| dyn_dyn_cast!(Driver => ButtonDriver, d.deref()).ok())
            .map(|d| d.op_btns())
            .fold(0, |r, v| r | v)
    }

    pub fn left_btns(&self) -> u8 {
        self.0
            .iter()
            .filter_map(|d| dyn_dyn_cast!(Driver => ButtonDriver, d.deref()).ok())
            .map(|d| d.left_btns())
            .fold(0, |r, v| r | v)
    }

    pub fn right_btns(&self) -> u8 {
        self.0
            .iter()
            .filter_map(|d| dyn_dyn_cast!(Driver => ButtonDriver, d.deref()).ok())
            .map(|d| d.right_btns())
            .fold(0, |r, v| r | v)
    }

    pub fn lever(&self) -> Option<i16> {
        self.0
            .iter()
            .filter_map(|d| dyn_dyn_cast!(Driver => LeverDriver, d.deref()).ok())
            .map(|d| d.lever())
            .next()
    }

    pub fn set_led(&mut self, data: u32) {
        for driver in self.0.iter_mut() {
            if let Ok(d) = dyn_dyn_cast!(mut Driver => LEDriver, driver.deref_mut()) {
                d.set_led(data);
            }
        }
    }

    pub fn set_led_new(&mut self, board: u8, rgb: &[rgb::RGB8]) {
        for driver in self.0.iter_mut() {
            if let Ok(d) = dyn_dyn_cast!(mut Driver => LEDriverNew, driver.deref_mut()) {
                d.set_led_new(board, rgb);
            }
        }
    }
}
