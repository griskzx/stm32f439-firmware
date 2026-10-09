/* Standalone STM32F439ZI layout; no boot/app partition is defined yet. */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 2M
  RAM : ORIGIN = 0x20000000, LENGTH = 192K
  /* CPU-only CCM RAM; not accessible by DMA. */
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}

/* NOLOAD: initialize each CCM allocation explicitly before reading it. */
SECTIONS
{
  .ccm (NOLOAD) : ALIGN(4)
  {
    *(.ccm .ccm.*);
    . = ALIGN(4);
  } > CCMRAM
}
INSERT AFTER .bss;
