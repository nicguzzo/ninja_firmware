use embedded_hal::blocking::i2c::WriteRead;

const SECONDARY_KB_ADDRESS: u8 = 0x08;
const SECONDARY_KB_N_BYTES:usize = 3;
pub struct RightSideI2C<I2C>
where
    I2C: WriteRead,
{
    i2c: I2C,
}

impl<I2C> RightSideI2C<I2C>
where
    I2C: WriteRead,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn read_keys(&mut self) -> Result<[u8;SECONDARY_KB_N_BYTES], I2C::Error> {
        let mut data = [0u8;SECONDARY_KB_N_BYTES];
        self.i2c.write_read(SECONDARY_KB_ADDRESS, &[0x00], &mut data)?;
        Ok(data)
    }
}