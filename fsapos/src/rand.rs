pub trait Random {
    type Error;
    fn rand(&self, buf: &mut [u8]) -> Result<(), Self::Error>;
}

pub struct Rander<R> {
    rand: R,
}
impl<R: Random> Rander<R> {
    pub fn new(rand: R) -> Self {
        Self { rand }
    }
    pub fn rand(&self, buf: &mut [u8]) -> Result<(), R::Error> {
        self.rand.rand(buf)
    }
}
