.text
.globl _entry
_entry:
.set at
.set reorder
    /* Disable interrupts */
    mtc0    $zero, $12

    /* Set up stack pointer */
    la      $sp, 0x80400000
    
    j       kernel_init