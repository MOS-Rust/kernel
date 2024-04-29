OUTPUT_ARCH(mips)

ENTRY(_entry)

SECTIONS
{
    /DISCARD/ : {
	    *(.comment)
	    *(.eh_frame)
	    *(.eh_frame_hdr)
    }
    
    . = 0x80020000;
    __start_kernel = .;
    
    __start_text = .;
    .text : {
        . = ALIGN(4096);
        PROVIDE(__ebase = .);

        *(.text .text.*)
    }
    __end_text = .;
    
    __start_data = .;
    .data : {
        *(.data .data.*)
    }
    __end_data = .;
    
    . = ALIGN(4K);
    __start_bss = .;
    .bss : {
        *(.bss .bss.*)
    }
    
    . = 0x80300000;
    __end_bss = .;
    __stack_bottom = .;
    . = ALIGN(4096);
    .stack : {
        *(.bss.stack)
    }
    . = 0x80400000;
    __stack_top = .;
    __end_kernel = .;
}