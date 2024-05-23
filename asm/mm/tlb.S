/* Leaf function _tlb_out */
.globl _tlb_out
.align 2
.type _tlb_out, @function
.ent  _tlb_out
_tlb_out:
mfc0    $8, $10
mtc0    $4, $10
nop
tlbp
nop
mfc0    $9, $0
.set reorder
bltz    $9, NO_TLB_ENTRY
.set noreorder
mtc0    $0, $10
mtc0    $0, $2
mtc0    $0, $3
nop
tlbwi
.set reorder

NO_TLB_ENTRY:
mtc0    $8, $10
jr      $31
.end _tlb_out
.size _tlb_out, .-_tlb_out


/* Nested function _do_tlb_refill */
.globl _do_tlb_refill
.align 2
.type _do_tlb_refill, @function
.ent _do_tlb_refill
_do_tlb_refill:
.frame $29, 24, $0
mfc0    $5, $8
mfc0    $6, $10
andi    $6, $6, 0xff
addi    $29, $29, -24
sw      $31, 20($29)
addi    $4, $29, 12
jal     do_tlb_refill
lw      $4, 12($29)
lw      $5, 16($29)
lw      $31, 20($29)
addi    $29, $29, 24
mtc0    $4, $2
mtc0    $5, $3
nop
tlbwr
jr      $31
.end _do_tlb_refill
.size _do_tlb_refill, .-_do_tlb_refill