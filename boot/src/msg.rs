pub const BOOT_CONTEXT_MAGIC: u32 = 0x9c3ab47f;
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Partition {
    A,
    B,
}

pub struct Context {
    magic: u32,
    ///代表app正在被使用的分区
    valid: Partition,
}

impl Context {
    pub fn new() -> Self {
        Self {
            magic: BOOT_CONTEXT_MAGIC,
            valid: Partition::B,
        }
    }
    pub fn valid(&self) -> Partition {
        self.valid
    }
    pub fn set_valid(&mut self, partition: Partition) {
        self.valid = partition;
    }
}
