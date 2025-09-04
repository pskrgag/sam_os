ENTRY(__start)

kernel_virtual_base = 0xffffffa000000000;

PHDRS
{
   text PT_LOAD FLAGS(5);
   rodata PT_LOAD FLAGS(6);
   data PT_LOAD FLAGS(4);
}

SECTIONS
{
    start = 0;

    .text :
    {
        *(.text .text.*)
    } :text

    .rela :
    {
        _rela_begin = .;
        *(.rela.dyn)
        _rela_end = .;
    } :rodata

    .rodata :
    {
        *(.rodata .rodata.*)
        *(.kernel_tests)
    } :rodata

    .data :
    {
        *(.data .data.*)
    } :data

    .bss :
    {
        *(.bss .bss.*)
    } :data


    . = . + 0x50000;
    . = ALIGN(16);
    __stack_start = .;

    __end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
