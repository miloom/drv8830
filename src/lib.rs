#![no_std]
use embedded_hal::i2c::I2c;

pub trait WriteRegister {
    // #[cfg(feature = "rpi")]
    // fn write(&self, i2c: &mut I2c) -> Result<()>;
    fn write<I: I2c>(&self, i2c: &mut I, chip_addr: u8) -> Result<(), I::Error>;
}
pub trait ReadRegister {

    fn new<I: I2c>(i2c: &mut I, chip_addr: u8) -> Result<Self, I::Error>
    where
        Self: Sized;
}

#[derive(Debug, Default)]
pub struct Control {
    in1: bool,
    in2: bool,
    // Output voltage in volts that the driver will attempt to match (1.29V - 5.06V)
    pub speed_mult: f32,
}
impl Control {
    const ADDRESS: u8 = 0x00;
    const MAX_VOLTAGE: f32 = 5.06;
    const MIN_VOLTAGE: f32 = 0.8;
    pub const COAST: Self = Self {
        in1: false,
        in2: false,
        speed_mult: 1.0,
    };
    pub const REVERSE: Self = Self {
        in1: false,
        in2: true,
        speed_mult: 1.0,
    };
    pub const FORWARD: Self = Self {
        in1: true,
        in2: false,
        speed_mult: 1.0,
    };
    pub const BRAKE: Self = Self {
        in1: true,
        in2: true,
        speed_mult: 1.0,
    };
}
impl WriteRegister for Control {
    fn write<I: I2c>(&self, i2c: &mut I, chip_addr: u8) -> Result<(), I::Error> {
        // VOUT = 4 x VREF x (VSET +1) / 64, where VREF is the internal 1.285-V
        let vout = (Self::MAX_VOLTAGE - Self::MIN_VOLTAGE) * self.speed_mult.clamp(0.0, 100.0) + Self::MIN_VOLTAGE;
        let voltage_enc = (vout / 0.0803) as u8;
        let write_reg = (voltage_enc << 2) | (u8::from(self.in2) << 1) | u8::from(self.in1);
        i2c.write(chip_addr, &[Self::ADDRESS, write_reg])?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Fault {
    // When written to 1, clears the fault status bits
    pub clear: bool,
    // If set, indicates the fault was caused by an extended current limit even
    pub i_limit: bool,
    // If set, indicates that the fault was caused by an overtemperature (OTS) condition
    pub ots: bool,
    // If set, indicates the fault was caused by an undervoltage lockout
    pub uvlo: bool,
    // If set, indicates the fault was caused by an overcurrent (OCP) event
    pub ocp: bool,
    // Set if any fault condition exists
    pub fault: bool,
}
impl Fault {
    const ADDRESS: u8 = 0x01;
}
impl ReadRegister for Fault {

    fn new<I: I2c>(i2c: &mut I, chip_addr: u8) -> Result<Self, I::Error>
    where
        Self: Sized,
    {
        let mut read_buf = [0u8; 1];
        i2c.write_read(chip_addr, &[Self::ADDRESS], &mut read_buf)?;
        let read_buf = read_buf[0];
        Ok(Self {
            clear: (read_buf >> 7) != 0,
            i_limit: ((read_buf >> 4) & 1) != 0,
            ots: ((read_buf >> 3) & 1) != 0,
            uvlo: ((read_buf >> 2) & 1) != 0,
            ocp: ((read_buf >> 1) & 1) != 0,
            fault: (read_buf & 1) != 0,
        })
    }
}
impl WriteRegister for Fault {

    fn write<I: I2c>(&self, i2c: &mut I, chip_addr: u8) -> Result<(), I::Error> {
        let write_buf = (u8::from(self.clear) << 7)
            | (u8::from(self.i_limit) << 4)
            | (u8::from(self.ots) << 3)
            | (u8::from(self.uvlo) << 2)
            | (u8::from(self.ocp) << 1)
            | u8::from(self.fault);
        i2c.write(chip_addr, &[Self::ADDRESS, write_buf])?;
        Ok(())
    }
}
