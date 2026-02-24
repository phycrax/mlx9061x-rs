//! MLX90614-specific functions

use crate::{
    ic,
    register_access::mlx90614::{self, Register, DEV_ADDR},
    Error, Mlx9061x, SlaveAddr, Temperature,
};
use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, digital::OutputPin, i2c::I2c};

impl<E, I2C> Mlx9061x<I2C, ic::Mlx90614>
where
    I2C: I2c<Error = E>,
{
    /// Create new instance of the MLX90614 device.
    ///
    /// The slave address must match the address stored in the device EEPROM.
    /// To change it you need to connect first and then change it with `set_address()`.
    /// An invalid alternative slave address will return `Error::InvalidInputData`.
    ///
    /// When writing to the EEPROM waiting a certain amount of time is necessary.
    /// This delay is configured through the `eeprom_write_delay_ms` parameter
    /// in milliseconds.
    pub fn new_mlx90614(
        i2c: I2C,
        address: SlaveAddr,
        eeprom_write_delay_ms: u8,
    ) -> Result<Self, Error<E>> {
        let address = Self::get_address(address, DEV_ADDR)?;
        Ok(Mlx9061x {
            i2c,
            eeprom_write_delay_ms,
            address,
            _ic: PhantomData,
        })
    }

    /// Read the ambient temperature
    pub fn ambient_temperature(&mut self) -> Result<Temperature, Error<E>> {
        Self::convert_to_temp(self.read_u16(Register::TA)?)
    }

    /// Read the object 1 temperature
    pub fn object1_temperature(&mut self) -> Result<Temperature, Error<E>> {
        Self::convert_to_temp(self.read_u16(Register::TOBJ1)?)
    }

    /// Read the object 2 temperature
    ///
    /// Note that this is only available in dual-zone thermopile device variants.
    pub fn object2_temperature(&mut self) -> Result<Temperature, Error<E>> {
        Self::convert_to_temp(self.read_u16(Register::TOBJ2)?)
    }

    fn convert_to_temp(raw: u16) -> Result<Temperature, Error<E>> {
        if raw & 0x8000 != 0 {
            return Err(Error::BadRead(Temperature(raw & 0x7FFF)));
        }
        Ok(Temperature(raw))
    }

    /// Read the channel 1 raw IR data
    pub fn raw_ir_channel1(&mut self) -> Result<i16, Error<E>> {
        self.read_i16(Register::RAW_IR1)
    }

    /// Read the channel 2 raw IR data
    pub fn raw_ir_channel2(&mut self) -> Result<i16, Error<E>> {
        self.read_i16(Register::RAW_IR2)
    }

    /// Get emissivity epsilon
    pub fn emissivity(&mut self) -> Result<f32, Error<E>> {
        let raw = self.read_u16(Register::EMISSIVITY)?;
        Ok(f32::from(raw) / 65535.0)
    }

    /// Set emissivity epsilon [0.1-1.0]
    ///
    /// Wrong values will return `Error::InvalidInputData`.
    pub fn set_emissivity<D: DelayNs>(
        &mut self,
        epsilon: f32,
        delay: &mut D,
    ) -> Result<(), Error<E>> {
        if epsilon < 0.1 || epsilon > 1.0 {
            return Err(Error::InvalidInputData);
        }
        let eps = (epsilon * 65535.0 + 0.5) as u16;
        if eps < 6553 {
            return Err(Error::InvalidInputData);
        }
        self.write_u16_eeprom(Register::EMISSIVITY, eps, delay)
    }

    /// Get the configuration register 1
    pub fn config_1(&mut self) -> Result<Config, Error<E>> {
        self.read_u16(Register::CONFIG_1)
            .map(|bits| Config::from_bits(bits))
    }

    /// Set the configuration register 1
    pub fn set_config_1<D: DelayNs>(
        &mut self,
        config: Config,
        delay: &mut D,
    ) -> Result<(), Error<E>> {
        self.write_u16_eeprom(Register::CONFIG_1, 0, delay)?;
        delay.delay_ms(u32::from(self.eeprom_write_delay_ms));
        self.write_u16_eeprom(Register::CONFIG_1, config.as_bits(), delay)?;
        delay.delay_ms(u32::from(self.eeprom_write_delay_ms));
        if config == self.config_1()? {
            Ok(())
        } else {
            Err(Error::BadEepromWrite)
        }
    }

    /// Get the device ID
    pub fn device_id(&mut self) -> Result<u64, Error<E>> {
        let mut id = 0;
        for i in 0..4 {
            let part = self.read_u16(Register::ID0 + i)?;
            let part = u64::from(part) << (16 * (3 - i));
            id |= part;
        }
        Ok(id)
    }
}

/// Wake device from sleep mode.
///
/// Note that this includes a 33ms delay.
pub fn wake_mlx90614<E, SclPin: OutputPin<Error = E>, SdaPin: OutputPin<Error = E>, D: DelayNs>(
    scl: &mut SclPin,
    sda: &mut SdaPin,
    delay: &mut D,
) -> Result<(), E> {
    scl.set_high()?;
    sda.set_low()?;
    delay.delay_ms(u32::from(mlx90614::WAKE_DELAY_MS));
    sda.set_high()
}

/// IIR filter settings (Bits 0-2)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Iir {
    /// 50% - a1=0.5, b=0.5
    Step50 = 0b000,
    /// 25% - a1=0.25, b=0.75
    Step25 = 0b001,
    /// 17% - a1=0.166(6), b=0.833(3)
    Step17 = 0b010,
    /// 13% - a1=0.125, b=0.875
    Step13 = 0b011,
    /// 100% - a1=1, b=0
    Step100 = 0b100,
    /// 80% - a1=0.8, b=0.2
    Step80 = 0b101,
    /// 67% - a1=0.666(6), b=0.333(3)
    Step67 = 0b110,
    /// 57% - a1=0.571, b=0.428
    Step57 = 0b111,
}

/// PWM mode configuration (Bits 4-5)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PwmMode {
    /// PWM mode - Ta, Tobj1
    TaTobj1 = 0b00,
    /// PWM mode - Ta, Tobj2
    TaTobj2 = 0b01,
    /// PWM mode - Tobj2 only
    Tobj2 = 0b10,
    /// PWM mode - Tobj1, Tobj2 (Undefined in table but logically 11)
    Tobj1Tobj2 = 0b11,
}

/// FIR filter settings (Bits 8-10)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Fir {
    /// 8
    Step8 = 0b000,
    /// 16
    Step16 = 0b001,
    /// 32
    Step32 = 0b010,
    /// 64
    Step64 = 0b011,
    /// 128
    Step128 = 0b100,
    /// 256
    Step256 = 0b101,
    /// 512
    Step512 = 0b110,
    /// 1024
    Step1024 = 0b111,
}

/// Amplifier gain settings (Bits 11-13)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Gain {
    /// Gain = 1
    Gain1 = 0b000,
    /// Gain = 3
    Gain3 = 0b001,
    /// Gain = 6
    Gain6 = 0b010,
    /// Gain = 12.5
    Gain12_5 = 0b011,
    /// Gain = 25
    Gain25 = 0b100,
    /// Gain = 50
    Gain50 = 0b101,
    /// Gain = 100
    Gain100 = 0b110,
    /// Gain = 100
    Gain100Alt = 0b111,
}

/// Configuration register 1
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// IIR filter settings
    pub iir: Iir,
    /// Repeat sensor selftest
    pub repeat_sensor_selftest: bool,
    /// PWM mode configuration
    pub pwm_mode: PwmMode,
    /// Single (0) or Dual (1) IR sensor
    pub dual_ir_sensor: bool,
    /// Ks sign (0 = positive, 1 = negative)
    pub ks_sign_negative: bool,
    /// FIR filter settings
    pub fir: Fir,
    /// Amplifier gain settings
    pub gain: Gain,
    /// Kt2 sign (0 = positive, 1 = negative)
    pub kt2_sign_negative: bool,
    /// Sensor selftest (0 = enabled, 1 = disabled)
    pub sensor_selftest_disabled: bool,
}

impl Config {
    /// Convert from raw bits
    pub fn from_bits(bits: u16) -> Self {
        let iir = match bits & 0b111 {
            0b000 => Iir::Step50,
            0b001 => Iir::Step25,
            0b010 => Iir::Step17,
            0b011 => Iir::Step13,
            0b100 => Iir::Step100,
            0b101 => Iir::Step80,
            0b110 => Iir::Step67,
            0b111 => Iir::Step57,
            _ => unreachable!(),
        };

        let repeat_sensor_selftest = (bits & (1 << 3)) != 0;

        let pwm_mode = match (bits >> 4) & 0b11 {
            0b00 => PwmMode::TaTobj1,
            0b01 => PwmMode::TaTobj2,
            0b10 => PwmMode::Tobj2,
            0b11 => PwmMode::Tobj1Tobj2,
            _ => unreachable!(),
        };

        let dual_ir_sensor = (bits & (1 << 6)) != 0;
        let ks_sign_negative = (bits & (1 << 7)) != 0;

        let fir = match (bits >> 8) & 0b111 {
            0b000 => Fir::Step8,
            0b001 => Fir::Step16,
            0b010 => Fir::Step32,
            0b011 => Fir::Step64,
            0b100 => Fir::Step128,
            0b101 => Fir::Step256,
            0b110 => Fir::Step512,
            0b111 => Fir::Step1024,
            _ => unreachable!(),
        };

        let gain = match (bits >> 11) & 0b111 {
            0b000 => Gain::Gain1,
            0b001 => Gain::Gain3,
            0b010 => Gain::Gain6,
            0b011 => Gain::Gain12_5,
            0b100 => Gain::Gain25,
            0b101 => Gain::Gain50,
            0b110 => Gain::Gain100,
            0b111 => Gain::Gain100Alt,
            _ => unreachable!(),
        };

        let kt2_sign_negative = (bits & (1 << 14)) != 0;
        let sensor_selftest_disabled = (bits & (1 << 15)) != 0;

        Config {
            iir,
            repeat_sensor_selftest,
            pwm_mode,
            dual_ir_sensor,
            ks_sign_negative,
            fir,
            gain,
            kt2_sign_negative,
            sensor_selftest_disabled,
        }
    }

    /// Convert to raw bits
    pub fn as_bits(&self) -> u16 {
        let mut bits = 0u16;

        bits |= self.iir as u16;
        if self.repeat_sensor_selftest {
            bits |= 1 << 3;
        }
        bits |= (self.pwm_mode as u16) << 4;
        if self.dual_ir_sensor {
            bits |= 1 << 6;
        }
        if self.ks_sign_negative {
            bits |= 1 << 7;
        }
        bits |= (self.fir as u16) << 8;
        bits |= (self.gain as u16) << 11;
        if self.kt2_sign_negative {
            bits |= 1 << 14;
        }
        if self.sensor_selftest_disabled {
            bits |= 1 << 15;
        }

        bits
    }
}
