.text
.globl _entry
_entry:
.set at
.set reorder
    /* Disable interrupts */
    mtc0    ZERO, CP0_STATUS

    /* Set up stack pointer */
    la      SP, __stack_top;

    /* Set up ebase */
    la      K0, __ebase
    mtc0    K0, CP0_EBASE
    
    j       kernel_init
    nop