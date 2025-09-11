ENTRY(__start)

PHDRS
{
   text PT_LOAD FLAGS(5);
   rodata PT_LOAD FLAGS(4);
   data PT_LOAD FLAGS(6);
}

SECTIONS
{
    .text_boot :
    {
        *(.text.boot)
    } :text

    .rela :
    {
        _rela_begin = .;
        *(.rela.*)
        _rela_end = .;
    } :rodata

    .text ALIGN(4096):
    {
        _text_begin = .;
        *(.text .text.*)
        . = ALIGN(4096);
        _text_end = .;
    } :text

    .rodata :
    {
        *(.rodata .rodata.*)

        . = ALIGN(16);
        __start_linkme_UARTS = .;
        KEEP(*(linkme_UARTS))
        __stop_linkme_UARTS = .;

        __start_linkm2_UARTS = .;
        __stop_linkm2_UARTS = .;
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
    . = ALIGN(4096);
    __end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
