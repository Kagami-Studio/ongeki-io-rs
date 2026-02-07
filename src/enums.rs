//! Enumerations for button types and result codes

/// Operator panel buttons
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpBtn {
    /// Test button
    Test = 0x01,
    /// Service button
    Service = 0x02,
    /// Coin insert button
    Coin = 0x04,
}

/// Game buttons (3 main buttons + side + menu for each side)
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GameBtn {
    /// Button 1
    Btn1 = 0x01,
    /// Button 2
    Btn2 = 0x02,
    /// Button 3
    Btn3 = 0x04,
    /// Side button
    Side = 0x08,
    /// Menu button
    Menu = 0x10,
}

/// Result codes for IO operations
#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum HResult {
    /// Success
    Ok = 0x00,
    /// Failure
    Bad = 0x01,
}