use embedded_storage::nor_flash::NorFlash;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    FlashDriver,
    rand::{Rander, Random},
};

const STORAGE_INDEX_HEAD_SIZE: usize = core::mem::size_of::<StorageListHeader>();
const STORAGE_INDEX_BODY_SIZE: usize = core::mem::size_of::<StorageListBody>();
const STORAGE_INDEX_SIZE: usize = core::mem::size_of::<StorageList>();

pub enum StorageType {
    Index,
    Data,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct StorageListHeader {
    id: [u8; 256],
    next: u32,
    crc: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct StorageListBody {
    len: u32,
    next: u32,
    crc: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct StorageList {
    head: StorageListHeader,
    body: StorageListBody,
}

impl StorageListHeader {
    pub fn create<R: Random>(storage_type: StorageType, r: R) -> Result<Self, R::Error> {
        match storage_type {
            StorageType::Index => Ok(Self {
                id: [0x05; 256],
                next: 0xffff_ffff,
                crc: 0,
            }),
            StorageType::Data => {
                let rand = Rander::new(r);
                let mut id = [0; 256];
                rand.rand(&mut id)?;
                Ok(Self {
                    id,
                    next: 0xffff_ffff,
                    crc: 0,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StorageTableList {
    id: [u8; 256],
    len: usize,
    ///相对偏移量
    next: u32,
}

#[derive(Debug)]
pub struct StorageTable<'a> {
    table: &'a mut [StorageTableList],
}

impl<'a> StorageTable<'a> {
    pub fn new(table: &'a mut [StorageTableList]) -> Self {
        Self { table }
    }
    pub fn traverse<F: NorFlash, C: StorageConfig>(
        &mut self,
        flash_drive: &mut FlashDriver<F>,
    ) -> Result<usize, Error> {
        let mut bytes = [0u8; STORAGE_INDEX_SIZE];
        let mut curr_addr = C::START_ADDR;
        let mut count = 0;

        // 链表上的一个不同id才是数据的开始，eg： a1->b1->b2->a2->c中a1 和a2才是相同数据的开始。但在flash中排列可能为 b2 c a2 a1 b1
        while curr_addr != 0xffff_ffff {
            flash_drive
                .read(curr_addr, &mut bytes)
                .map_err(|_| Error::Flash)?;

            let (_head, _body) = bytes.split_at_mut(STORAGE_INDEX_HEAD_SIZE);
            // TO DO!!! 注意后面处理unwrap
            let head = StorageListHeader::read_from_bytes(_head).unwrap();

            // 查重：仅比对已记录有效项的 id
            let exists = self.table[..count].iter().any(|v| v.id == head.id);
            if !exists {
                // 如果发现新的不同 ID，但 table 已满，则返回错误通知调用者扩容
                if count >= self.table.len() {
                    return Err(Error::TableTooSmall);
                }
                self.table[count].id = head.id;
                self.table[count].next = curr_addr; // 记录当前首次出现的不同 ID 节点的起始地址
                count += 1;
            }

            curr_addr = head.next;
        }

        Ok(count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Flash,
    /// 传入的 table 容量小于 Flash 中实际存储的不同 ID 类型数量
    TableTooSmall,
}

pub trait StorageConfig {
    ///存储区域偏移地址
    const START_ADDR: u32;
}
