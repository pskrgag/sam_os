ENTRY(__start)

PAGE_SIZE = 4096;
kernel_virtual_base = 0xffffffa000000000;

PHDRS
{
   text PT_LOAD FLAGS(5);
   rodata PT_LOAD FLAGS(6);
   data PT_LOAD FLAGS(4);
}


SECTIONS
{
    . = kernel_virtual_base;
    start = .;

    .boot :
    {
        *(.text.boot)
    } :text

    .rela :
    {
        _rela_begin = .;
        *(.rela.dyn)
        _rela_end = .;
    } :text

    .text ALIGN(PAGE_SIZE):
    {
        stext = .;
        *(.text .text.*)
        . = ALIGN(PAGE_SIZE);
        etext = .;
    } :text

    .rodata ALIGN(PAGE_SIZE):
    {
        srodata = .;
        *(.rodata .rodata.*)

        skerneltests = .;
        *(.kernel_tests)
        ekerneltests = .;

       . = ALIGN(4096);
       erodata = .;
    } :rodata

    .data ALIGN(PAGE_SIZE):
    {
        sdata = .;
        *(.data .data.*)

        . = ALIGN(8);
        sfixup = .;
        *(.fixup)
        efixup = .;

        . = ALIGN(4096);
        edata = .;
    } :data

    .bss ALIGN(4096) (NOLOAD):
    {
        sbss = .;
        *(.bss .bss.*)
        FILL(0x00)
        . = ALIGN(PAGE_SIZE);
        ebss = .;
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

    . = . + 0x50000;
    . = ALIGN(16);
    __STACK_START = .;

    .guard_page (NOLOAD):
    {
         SKIP(PAGE_SIZE)
    } :data

    . = ALIGN(2 << 20);
    __end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
