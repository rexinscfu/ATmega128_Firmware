MEMORY
{
  text   (rx)   : ORIGIN = 0x1E000, LENGTH = 0x2000
  data   (rw!x) : ORIGIN = 0x800100, LENGTH = 0x1000
}

SECTIONS
{
  .text :
  {
    *(.vectors)
    *(.text*)
    *(.rodata*)
    . = ALIGN(2);
  } > text

  .data :
  {
    *(.data*)
    . = ALIGN(2);
  } > data AT > text

  .bss :
  {
    *(.bss*)
    *(COMMON)
    . = ALIGN(2);
  } > data

  .noinit (NOLOAD) :
  {
    *(.noinit*)
  } > data
}
