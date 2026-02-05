/// All possible errors in this crate
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
    /// CRC checksum mismatch (PEC)
    ChecksumMismatch,
    /// Invalid input data
    InvalidInputData,
}

/// IC marker
pub mod ic {
    /// MLX90614 IC marker
    pub struct Mlx90614;
    /// MLX90615 IC marker
    pub struct Mlx90615;
}

/// Possible slave addresses
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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

/// IIR filter settings (Bits 0-2)
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
