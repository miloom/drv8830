use rppal::i2c::I2c;
use rppal::i2c::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub trait WriteRegister {
    fn write(&self, i2c: &mut I2c) -> Result<()>;
}
pub trait ReadRegister {
    fn new(i2c: &mut I2c) -> Result<Self> where Self: Sized;
}

pub struct Control {
    in1: bool,
    in2: bool,
}
impl Control {
    const ADDRESS: u8 = 0x00;
    pub const COAST: Self = Self { in1: false, in2: false };
    pub const REVERSE: Self = Self { in1: false, in2: true };
    pub const FORWARD: Self = Self { in1: true, in2: false };
    pub const BRAKE: Self = Self { in1: true, in2: true };
}
impl WriteRegister for Control {
    fn write(&self, i2c: &mut I2c) -> Result<()> {
        let write_reg = (u8::from(self.in2) << 1) | u8::from(self.in1);
        i2c.smbus_write_byte(Self::ADDRESS, write_reg)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Fault {
    // When written to 1, clears the fault status bits
    pub clear: bool,
    // If set, indicates the fault was caused by an extended current limit even
    i_limit: bool,
    // If set, indicates that the fault was caused by an overtemperature (OTS) condition
    ots: bool,
    // If set, indicates the fault was caused by an undervoltage lockou
    uvlo: bool,
    // If set, indicates the fault was caused by an overcurrent (OCP) event
    ocp: bool,
    // Set if any fault condition exists
    fault: bool,
}
impl Fault {
    const ADDRESS: u8 = 0x01;
}
impl ReadRegister for Fault {
    fn new(i2c: &mut I2c) -> Result<Self> where Self: Sized {
        let read_buf = i2c.smbus_read_byte(Self::ADDRESS)?;
        Ok(Self { clear: (read_buf >> 7) != 0, i_limit: ((read_buf >> 4) & 1) != 0, ots: ((read_buf >> 3) & 1) != 0, uvlo: ((read_buf >> 2) & 1) != 0, ocp: ((read_buf >> 1) & 1) != 0, fault: (read_buf & 1) != 0 })
    }
}
impl WriteRegister for Fault {
    fn write(&self, i2c: &mut I2c) -> Result<()> {
        let write_buf = (u8::from(self.clear) << 7) | (u8::from(self.i_limit) << 4) | (u8::from(self.ots) << 3) | (u8::from(self.uvlo) << 2) | (u8::from(self.ocp) << 1) | u8::from(self.fault);
        i2c.smbus_write_byte(Self::ADDRESS, write_buf)?;
        Ok(())
    }
}



