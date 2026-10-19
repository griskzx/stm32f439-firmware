/* STM32F439ZI Boot partition: sectors S0-S4, 0x08000000..0x0801FFFF.
 * App A starts at 0x08020000; Boot must fit entirely below that address.
 * RAM: 192 KiB SRAM and 64 KiB CCM data RAM.
 */
MEMORY
{
    FLASH  : ORIGIN = 0x08000000, LENGTH = 128K
    /* Contiguous SRAM1 (112 KiB) + SRAM2 (16 KiB) + SRAM3 (64 KiB). */
    RAM    : ORIGIN = 0x20000000, LENGTH = 192K
    /* CPU-only data RAM, not DMA-accessible. Declared for explicit placement;
     * custom sections and startup initialization are required before use.
     */
    CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}

