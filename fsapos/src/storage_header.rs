//! 有问题！！！
//! 存储信息表，此表为总表，存放所有数据的位置信息，读写数据通过查此表找到对应真正存放数据的区域，进行增删改查
//! 采用链表形式存放到flash
//! 链头固定在存储区域首地址
//! eg: StorageTable(StorageBody.next)->StorageTable(StorageBody.next)->StorageTable(StorageBody.next)->... 直到超过设定的分区范围
//!                         |                   |
//!                         |           StorageBody.start -> (强转)StorageData(StorageDataMsg.next) -> StorageData(StorageDataMsg.next) -> ...
//!                   StorageBody.start -> (强转)StorageData(StorageDataMsg.next) -> StorageData(StorageDataMsg.next) -> ... 直到超过设定的分区范围

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[repr(C)]
#[derive(Debug, Immutable, KnownLayout, FromBytes, IntoBytes)]
pub struct StorageTable {
    body: StorageBody,
    crc: u32,
    _pad: [u8; 4],
}
/// 采用真随机数填充id使其每个块id达到唯一性
#[repr(C)]
#[derive(Debug, Immutable, KnownLayout, FromBytes, IntoBytes)]
pub struct StorageBody {
    header: StorageHeader,
    seq: u64,
    next: u32,
    _pad: [u8; 4],
}
#[repr(C)]
#[derive(Debug, Immutable, KnownLayout, FromBytes, IntoBytes)]
pub struct StorageHeader {
    id: [u8; 32],
    start: u32,
    _pad: [u8; 4],
}

#[repr(C)]
#[derive(Debug, Immutable, KnownLayout, FromBytes, IntoBytes)]
pub struct StorageDataMsg {
    start: u32,
    current_length: u32,
    total_length: u32,
    next: u32,
}

#[repr(C)]
#[derive(Debug, Immutable, KnownLayout, FromBytes, IntoBytes)]
pub struct StorageData {
    msg: StorageDataMsg,
    crc: u32,
}
