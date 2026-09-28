#![no_std]

pub mod crc;
pub mod rand;
pub mod region;
pub mod storage_header;
pub mod stroage;

use embedded_storage::nor_flash::NorFlash;
use log::debug;

pub struct FlashDriver<F> {
    flash: F,
}

impl<F: NorFlash> FlashDriver<F> {
    pub fn new(flash: F) -> Self {
        Self { flash }
    }
    #[inline]
    pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), F::Error> {
        debug!("execute read flash");
        self.flash.read(offset, bytes)
    }
    #[inline]
    pub fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), F::Error> {
        debug!("execute write flash");
        self.flash.write(offset, bytes)
    }
    #[inline]
    pub fn erase(&mut self, from: u32, to: u32) -> Result<(), F::Error> {
        debug!("execute erase !!!");
        self.flash.erase(from, to)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SectorId {
    Sector12 = 12,
    Sector13,
    Sector14,
    Sector15,
    Sector16,
}

impl SectorId {
    pub(crate) const fn size(&self) -> usize {
        match self {
            SectorId::Sector12 | SectorId::Sector13 | SectorId::Sector14 | SectorId::Sector15 => {
                16 * 1024
            }
            SectorId::Sector16 => 64 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SectorAddr {
    Sector12 = 0x0810_0000, // 16kb
    Sector13 = 0x0810_4000, // 16kb
    Sector14 = 0x0810_8000, // 16kb
    Sector15 = 0x0810_c000, // 16kb
    Sector16 = 0x0811_0000, // 64kb
}

pub struct Sector {
    id: SectorId,
    addr: SectorAddr,
}

pub const SECTOR_12: Sector = Sector {
    id: SectorId::Sector12,
    addr: SectorAddr::Sector12,
};
pub const SECTOR_13: Sector = Sector {
    id: SectorId::Sector13,
    addr: SectorAddr::Sector13,
};
pub const SECTOR_14: Sector = Sector {
    id: SectorId::Sector14,
    addr: SectorAddr::Sector14,
};
pub const SECTOR_15: Sector = Sector {
    id: SectorId::Sector15,
    addr: SectorAddr::Sector15,
};
pub const SECTOR_16: Sector = Sector {
    id: SectorId::Sector16,
    addr: SectorAddr::Sector16,
};

pub enum Error {
    InvalidRange,
}

pub struct StorageEngine {
    primary: (Sector, Sector),
    secondary: (Sector, Sector),
}

impl StorageEngine {
    /// Creates storage from inclusive sector ranges whose address ranges do not overlap.
    pub fn create(primary: (Sector, Sector), secondary: (Sector, Sector)) -> Result<Self, Error> {
        let primary_start = primary.0.addr as usize;
        let secondary_start = secondary.0.addr as usize;

        if primary_start > primary.1.addr as usize {
            return Err(Error::InvalidRange);
        }
        if secondary_start > secondary.1.addr as usize {
            return Err(Error::InvalidRange);
        }

        // Address ranges are [start, end), including the entire final sector.
        let primary_end = primary.1.addr as usize + primary.1.id.size();
        let secondary_end = secondary.1.addr as usize + secondary.1.id.size();

        if primary_start < secondary_end && secondary_start < primary_end {
            return Err(Error::InvalidRange);
        }

        Ok(Self { primary, secondary })
    }
    pub fn primary(&self) -> &(Sector, Sector) {
        &self.primary
    }
    pub fn secondary(&self) -> &(Sector, Sector) {
        &self.secondary
    }

    pub fn init(&self) -> Result<(), Error> {
        unimplemented!()
    }
}
