pub struct Region {
    addr: usize,
    len: usize,
}
/// 左闭右开
impl Region {
    pub const fn new(addr: usize, len: usize) -> Self {
        Self { addr, len }
    }

    pub const fn end(&self) -> usize {
        self.addr + self.len
    }

    pub const fn start(&self) -> usize {
        self.addr
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}
