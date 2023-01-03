use embedded_hal::blocking::i2c::WriteRead;

const SECONDARY_KB_ADDRESS: u8 = 0x08;

pub struct SecondarySideI2C<I2C>
where
    I2C: WriteRead,
{
    i2c: I2C
}

impl<I2C> SecondarySideI2C<I2C>
where
    I2C: WriteRead,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c}
    }

    pub fn read_keys(&mut self) -> Result<[u8;crate::KB_N_BYTES], I2C::Error> {
        let mut data = [0u8;crate::KB_N_BYTES];
        self.i2c.write_read(SECONDARY_KB_ADDRESS, &[0x00], &mut data)?;
        Ok(data)
    }
}