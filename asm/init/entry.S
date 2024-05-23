.text
.globl _entry
_entry:
.set at
.set reorder
/* Disable interrupts */
mtc0    $0, $12

/* Set up stack pointer */
la      $29, __stack_top;

j       kernel_init
nop