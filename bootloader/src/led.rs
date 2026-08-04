use core::convert::Infallible;

use stm32f4xx_hal::hal_02::digital::v2::OutputPin;

pub struct Led<P>
where
    P: OutputPin<Error = Infallible>,
{
    pin: P,
}

impl<P> Led<P>
where
    P: OutputPin<Error = Infallible>,
{
    pub fn new(pin: P) -> Self {
        Self { pin }
    }
    pub fn on(&mut self) -> Result<(), P::Error> {
        self.pin.set_high()
    }
    pub fn off(&mut self) -> Result<(), P::Error> {
        self.pin.set_low()
    }
}
