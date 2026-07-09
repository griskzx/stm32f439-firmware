SECTIONS
{
  .ccmram (NOLOAD) : ALIGN(4)
  {
    . = ALIGN(4);
    __sccmram = .;
    *(.ccmram .ccmram.*);
    . = ALIGN(4);
    __eccmram = .;
  } > CCMRAM
} INSERT AFTER .uninit;
