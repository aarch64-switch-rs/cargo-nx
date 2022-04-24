OUTPUT_FORMAT(elf32-littlearm)
OUTPUT_ARCH(arm)
ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
    dynamic PT_DYNAMIC;
}

SECTIONS {
    . = 0;

    /* Text sections */

    . = ALIGN(8);

    .text : {
        PROVIDE_HIDDEN(__text_start = .);
        KEEP(*(.text.rrt0))
        . = 0x80;
        KEEP(*(.text.mod0))
        *(.text .text.*)
        *(.plt .plt.*)
    }

    /* Read-only sections */

    . = ALIGN(0x1000);

    .module_name : {
        KEEP(*(.module_name))
    } :rodata
    
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .dynsym : {
        *(.dynsym .dynsym.*)
    }

    .dynstr : {
        *(.dynstr .dynstr.*)
    }

    .rela.dyn : {
        *(.rela.dyn)
    }

    .hash : {
        *(.hash)
    }

    .eh_frame_hdr : {
        PROVIDE_HIDDEN(__eh_frame_hdr_start = .);
        *(.eh_frame_hdr .eh_frame_hdr.*)
        __eh_frame_hdr_end = .;
    }
    
    .eh_frame : {
        PROVIDE_HIDDEN(__eh_frame_start = .);
        *(.eh_frame .eh_frame.*)
        PROVIDE_HIDDEN(__eh_frame_end = .);
    }

    /* Read-write sections */

    . = ALIGN(0x1000);

    .data : {
        *(.data .data.*)
        *(.got .got.*)
        *(.got.plt .got.plt.*)
    } :data

    .dynamic : {
        PROVIDE_HIDDEN(__dynamic_start = .);
        *(.dynamic)
    }

    /* BSS section */

    . = ALIGN(8);

    .bss : {
        PROVIDE_HIDDEN(__bss_start = .);
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        PROVIDE_HIDDEN(__bss_end = .);
    } :data
}