/// All possible errors in this crate
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
    /// CRC checksum mismatch (PEC)
    ChecksumMismatch,
    /// Invalid input data
    InvalidInputData,
    /// Bad eeprom write
    BadEepromWrite,
    /// Bad temperature reading
    BadRead(Temperature),
}

/// IC marker
pub mod ic {
    /// MLX90614 IC marker
    pub struct Mlx90614;
    /// MLX90615 IC marker
    pub struct Mlx90615;
}

/// Possible slave addresses
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlaveAddr {
    /// Default slave address
    Default,
    /// Alternative slave address
    Alternative(u8),
}

impl Default for SlaveAddr {
    /// Default slave address
    fn default() -> Self {
        SlaveAddr::Default
    }
}

/// Temperature value
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature(
    /// Raw temperature value
    pub u16,
);

impl Temperature {
    /// Temperature in kelvin
    pub fn kelvin(&self) -> f32 {
        self.0 as f32 * 0.02
    }

    /// Temperature in celsius
    pub fn celsius(&self) -> f32 {
        self.kelvin() - 273.15
    }

    /// Temperature in fahrenheit
    pub fn fahrenheit(&self) -> f32 {
        self.kelvin() * 9.0 / 5.0 - 459.67
    }

    /// Temperature in millikelvin
    pub fn millikelvin(&self) -> u32 {
        self.0 as u32 * 20
    }

    /// Temperature in millicelsius
    pub fn millicelsius(&self) -> i32 {
        self.millikelvin() as i32 - 273150
    }

    /// Temperature in millifahrenheit
    pub fn millifahrenheit(&self) -> i32 {
        self.millikelvin() as i32 * 9 / 5 - 459670
    }
}
