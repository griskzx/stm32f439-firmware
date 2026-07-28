/*
 * This template initially uses the STM32F439's full memory map so it can be
 * built and flashed on its own. Before the bootloader jumps to this app,
 * reserve the bootloader region by changing FLASH ORIGIN and LENGTH here.
 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 2048K
  RAM   : ORIGIN = 0x20000000, LENGTH = 192K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}
