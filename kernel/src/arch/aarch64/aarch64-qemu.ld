ENTRY(__start)

PAGE_SIZE = 4096;
kernel_virtual_base = 0xFFFF800000000000;

PHDRS
{
   text PT_LOAD FLAGS(5);
   rodata PT_LOAD FLAGS(4);
   data PT_LOAD FLAGS(6);
}


SECTIONS
{
    . = kernel_virtual_base;
    start = .;

    .boot :
    {
        *(.text.boot)
    } :text

    .text ALIGN(PAGE_SIZE):
    {
        *(.text .text.*)
        . = ALIGN(PAGE_SIZE);
    } :text

    .rodata ALIGN(PAGE_SIZE):
    {
        *(.rodata .rodata.*)

        skerneltests = .;
        *(.kernel_tests)
        ekerneltests = .;

       . = ALIGN(4096);
    } :rodata

    .data ALIGN(PAGE_SIZE):
    {
        *(.data .data.*)

        . = ALIGN(8);
        sfixup = .;
        *(.fixup)
        efixup = .;

        . = ALIGN(4096);
    } :data

    .bss ALIGN(4096) (NOLOAD):
    {
        *(.bss .bss.*)
        . = ALIGN(PAGE_SIZE);
    } :data

    .data.percpu ALIGN(4096):
    {
        sdatapercpu = .;
        *(.percpu.data)
        . = ALIGN(PAGE_SIZE);
        edatapercpu = .;
    } :data

    .guard_page (NOLOAD):
    {
         SKIP(PAGE_SIZE)
    } :data

    .stack :
    {
         . = . + 0x50000;
         . = ALIGN(16);
         __STACK_START = .;
    } :data

    .guard_page (NOLOAD):
    {
         SKIP(PAGE_SIZE)
    } :data

    . = ALIGN(2 << 20);
    __end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
