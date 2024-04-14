/* Leaf function _tlb_out */
.globl _tlb_out
.align 2
.type _tlb_out, @function
.ent  _tlb_out
_tlb_out:
    mfc0    T0, CP0_ENTRYHI
    mtc0    A0, CP0_ENTRYHI
    nop
    tlbp
    nop
    mfc0    T1, CP0_INDEX
.set reorder
    bltz    T1, NO_TLB_ENTRY
.set noreorder
    mtc0    ZERO, CP0_ENTRYHI
    mtc0    ZERO, CP0_ENTRYLO0
    mtc0    ZERO, CP0_ENTRYLO1
    nop
    tlbwi
.set reorder

NO_TLB_ENTRY:
    mtc0    T0, CP0_ENTRYHI
    jr      RA
.end _tlb_out
.size _tlb_out, .-_tlb_out


/* Nested function _do_tlb_refill */
.globl _do_tlb_refill
.align 2
.type _do_tlb_refill, @function
.ent _do_tlb_refill
_do_tlb_refill:
.frame SP, 24, ZERO
    mfc0    A1, CP0_BADVADDR
    mfc0    A2, CP0_ENTRYHI
    andi    A2, A2, 0xff
.globl _do_tlb_refill_call:
    andi    SP, SP, -24
    sw      RA, 20(SP)
    addi    A0, SP, 12
    jal     _do_tlb_refill
    lw      A0, 12(SP)
    lw      A1, 16(SP)
    lw      RA, 20(SP)
    addi    SP, SP, 24
    mtc0    A0, CP0_ENTRYLO0
    mtc0    A1, CP0_ENTRYLO1
    nop
    tlbwr
    jr      RA
.end _do_tlb_refill
.size _do_tlb_refill, .-_do_tlb_refill
