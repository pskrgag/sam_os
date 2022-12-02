ENTRY(__start)

PAGE_SIZE = 4096;

load_addr = 0x40000000;
mmio_base = 0x08000000;
kernel_virtual_base = 0xffffffffc0000000;

SECTIONS
{
    . = kernel_virtual_base;
    start = .;
    .text : AT(load_addr)
    {
        stext = .;
        *(.text.boot)
        //. = ALIGN(PAGE_SIZE);
        *(.data.boot)
        . = ALIGN(PAGE_SIZE); /* Here boot code ends */
        __BOOT_END = .; // __BOOT_END must be 4KiB aligned
        __RO_START = .;
        *(.text .text.*)
	. = ALIGN(4096);
        etext = .;
    }

    .vectors ALIGN(2048):
    {
        KEEP(*(.vectors))
    }

    .rodata ALIGN(16):
    {
        srodata = .;
        *(.rodata .rodata.*)
        FILL(0x00)
	. = ALIGN(4096);
	erodata = .;
    }
    . = ALIGN(PAGE_SIZE); /* Fill up to page size */
    __RO_END = .; /* __RO_END must be 4KiB aligned */
    __DATA_START = .; /* __DATA_START must be 4KiB aligned */

    .data : /* @todo align data to 4K -- it's already aligned up to __RO_END marker now */
    {
        sdata = .;
        *(.data .data.*)
        FILL(0x00)
	. = ALIGN(4096);
        edata = .;
    }

    .bss ALIGN(4096) (NOLOAD):
    {
        sbss = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(PAGE_SIZE); /* Align up to page size */
        ebss = .;
    }

    . = . + 0x100000;
    . = ALIGN(16);
    __STACK_START = .; /* Stack grows from here towards 0x0. */

    end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }

    . = ALIGN(PAGE_SIZE);
    mmio_start = .;
    . += 0x00040000;
    mmio_end = .;
}
