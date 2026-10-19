/* STM32F439ZI: 2 MiB Flash, 192 KiB SRAM, and 64 KiB CCM data RAM.
 * Standalone image: Flash starts at the reset vector, with no bootloader offset.
 */
MEMORY
{
    FLASH  : ORIGIN = 0x08000000, LENGTH = 2048K
    /* Contiguous SRAM1 (112 KiB) + SRAM2 (16 KiB) + SRAM3 (64 KiB). */
    RAM    : ORIGIN = 0x20000000, LENGTH = 192K
    /* CPU-only data RAM, not DMA-accessible. Declared for explicit placement;
     * custom sections and startup initialization are required before use.
     */
    CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}

