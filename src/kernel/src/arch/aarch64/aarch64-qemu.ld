ENTRY(__start)

PAGE_SIZE = 4096;

#if defined(CONFIG_BOARD_QEMU)
mmio_base = 0x08000000;
#elif defined(CONFIG_BOARD_ORPIPC2)
mmio_base = 0x01C00000;
#else
# error "Misconfiguration"
#endif

kernel_virtual_base = 0xffffffa000000000;

SECTIONS
{
    . = kernel_virtual_base;
    start = .;

    .boot :
    {
        *(.text.boot)
    }

    .rela :
    {
        _rela_begin = .;
        *(.rela.dyn)
        _rela_end = .;
    }

    .text ALIGN(4096):
    {
        stext = .;
        *(.text .text.*)
        . = ALIGN(4096);
        etext = .;
    }

    .rodata ALIGN(16):
    {
        srodata = .;
        *(.rodata .rodata.*)

        skerneltests = .;
        *(.kernel_tests)
        ekerneltests = .;

       . = ALIGN(4096);
       erodata = .;
    }

    .data ALIGN(4096):
    {
        sdata = .;
        *(.data .data.*)

        . = ALIGN(8);
        sfixup = .;
        *(.fixup)
        efixup = .;

        . = ALIGN(4096);
        edata = .;
    }

    .bss ALIGN(4096) (NOLOAD):
    {
        sbss = .;
        *(.bss .bss.*)
        FILL(0x00)
        . = ALIGN(PAGE_SIZE);
        ebss = .;
    }

    .data.percpu ALIGN(4096):
    {
        sdatapercpu = .;
        *(.percpu.data)
        . = ALIGN(PAGE_SIZE);
        edatapercpu = .;
    }

    .guard_page (NOLOAD):
    {
         SKIP(4096)
    } :data

    . = . + 0x50000;
    . = ALIGN(16);
    __STACK_START = .;

    .guard_page (NOLOAD):
    {
         SKIP(4096)
    } :data

    . = ALIGN(2 << 20);
    __end = .;
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
