.macro BUILD_HANDLER exception handler
.globl _handle_\exception
.align 2
.type _handle_\exception, @function
.ent _handle_\exception
_handle_\exception:
.frame $29, 0 ,$31
move    $4, $29
addiu   $29, $29, -8
jal     \handler
addiu   $29, $29, 8
j       _ret_from_exception
.end _handle_\exception
.size _handle_\exception, .-_handle_\exception
.endm

BUILD_HANDLER tlb _do_tlb_refill
BUILD_HANDLER mod do_tlb_mod
BUILD_HANDLER ade do_address_error
BUILD_HANDLER syscall do_syscall
BUILD_HANDLER unhandled do_unhandled

.globl _handle_int;
.align 2;
.type _handle_int, @function;
.ent _handle_int;
_handle_int:
.frame $29, 0x98, $0
li     $4, 0
j      schedule
.end _handle_int
.size _handle_int, .-_handle_int