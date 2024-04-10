.text
.globl _entry
_entry:
.set at
.set reorder
    /* Disable interrupts */
    mtc0    ZERO, CP0_STATUS

    /* Set up stack pointer */
    la      SP, KSTACKTOP
    
    j       kernel_init