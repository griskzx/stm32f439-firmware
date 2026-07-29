#![no_std]
#![no_main]

use bootloader as _;

#[cortex_m_rt::entry]
fn main()->! {

    bootloader::exit();
}