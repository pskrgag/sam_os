ENTRY(_start)

PAGE_SIZE = 4096;

SECTIONS
{
    . = 0x40000000; /* AArch64 boot address is 0x80000, 4K-aligned */
    start = .;
    .text :
    {
        *(.text.boot)
        //. = ALIGN(PAGE_SIZE);
        *(.data.boot)
        . = ALIGN(PAGE_SIZE); /* Here boot code ends */
        __BOOT_END = .; // __BOOT_END must be 4KiB aligned
        __RO_START = .;
        *(.text .text.*)
    }

    .vectors ALIGN(2048):
    {
        KEEP(*(.vectors))
    }

    .rodata ALIGN(16):
    {
        *(.rodata .rodata.*)
        FILL(0x00)
    }
    . = ALIGN(PAGE_SIZE); /* Fill up to page size */
    __RO_END = .; /* __RO_END must be 4KiB aligned */
    __DATA_START = .; /* __DATA_START must be 4KiB aligned */

    .data : /* @todo align data to 4K -- it's already aligned up to __RO_END marker now */
    {
        *(.data .data.*)
        FILL(0x00)
    }

    .bss ALIGN(4096) (NOLOAD):
    {
        __BSS_START = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(PAGE_SIZE); /* Align up to page size */
        __BSS_END = .;
    }

    . = . + 0x100000;
    . = ALIGN(16);
    __STACK_START = .; /* Stack grows from here towards 0x0. */

    end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
