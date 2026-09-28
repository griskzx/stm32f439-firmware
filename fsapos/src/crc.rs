pub trait Crc {
    type Error;
    fn crc32(&self, buf: &[u8]) -> Result<u32, Self::Error>;
}

pub struct Crcer<C> {
    crc: C,
}
impl<C: Crc> Crcer<C> {
    pub fn new(crc: C) -> Self {
        Self { crc }
    }
    pub fn crc32(&self, buf: &[u8]) -> Result<u32, C::Error> {
        self.crc.crc32(buf)
    }
}
