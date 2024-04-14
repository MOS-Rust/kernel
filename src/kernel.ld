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
        *(.text .text.*)
    }
    __end_text = .;
    
    __start_data = .;
    .data : {
        *(.data .data.*)
    }
    __end_data = .;
    
    __start_bss = .;
    .bss : {
        *(.bss .bss.*)
    }
    __end_bss = .;

    __end_kernel = .;
    . = 0x80400000;
}