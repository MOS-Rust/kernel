.macro SAVE_CONTEXT
.set noat
.set noreorder
    mfc0 K0, CP0_STATUS
    andi K0, STATUS_UM
    beqz K0, 1f
    move K0, SP
    li   SP, KSTACKTOP

1:
    subu SP, SP, TF_SIZE
    sw   K0, TF_REG29(SP)
    mfc0 K0, CP0_STATUS
    sw   K0, TF_STATUS(SP)
    mfc0 K0, CP0_CAUSE
    sw   K0, TF_CAUSE(SP)
    mfc0 K0, CP0_EPC
    sw   K0, TF_EPC(SP)
    mfc0 K0, CP0_BADVADDR
    sw   K0, TF_BADVADDR(SP)
    mfhi K0
    sw   K0, TF_HI(SP)
    mflo K0
    sw   K0, TF_LO(SP)
    sw   $0, TF_REG0(SP)
    sw   $1, TF_REG1(SP)
    sw   $2, TF_REG2(SP)
    sw   $3, TF_REG3(SP)
    sw   $4, TF_REG4(SP)
    sw   $5, TF_REG5(SP)
    sw   $6, TF_REG6(SP)
    sw   $7, TF_REG7(SP)
    sw   $8, TF_REG8(SP)
    sw   $9, TF_REG9(SP)
    sw   $10, TF_REG10(SP)
    sw   $11, TF_REG11(SP)
    sw   $12, TF_REG12(SP)
    sw   $13, TF_REG13(SP)
    sw   $14, TF_REG14(SP)
    sw   $15, TF_REG15(SP)
    sw   $16, TF_REG16(SP)
    sw   $17, TF_REG17(SP)
    sw   $18, TF_REG18(SP)
    sw   $19, TF_REG19(SP)
    sw   $20, TF_REG20(SP)
    sw   $21, TF_REG21(SP)
    sw   $22, TF_REG22(SP)
    sw   $23, TF_REG23(SP)
    sw   $24, TF_REG24(SP)
    sw   $25, TF_REG25(SP)
    sw   $26, TF_REG26(SP)
    sw   $27, TF_REG27(SP)
    sw   $28, TF_REG28(SP)
    sw   $30, TF_REG30(SP)
    sw   $31, TF_REG31(SP)
.set at
.set reorder
.endm

.macro RESTORE_CONTEXT
.set noat
.set noreorder
    lw   V0, TF_STATUS(SP)
    mtc0 V0, CP0_STATUS
    lw   V1, TF_LO(SP)
    mtlo V1
    lw   V0, TF_HI(SP)
    mthi V0
    lw   V0, TF_EPC(SP)
    mtc0 V0, CP0_EPC
    lw   $31, TF_REG31(SP)
    lw   $30, TF_REG30(SP)
    lw   $28, TF_REG28(SP)
    lw   $25, TF_REG25(SP)
    lw   $24, TF_REG24(SP)
    lw   $23, TF_REG23(SP)
    lw   $22, TF_REG22(SP)
    lw   $21, TF_REG21(SP)
    lw   $20, TF_REG20(SP)
    lw   $19, TF_REG19(SP)
    lw   $18, TF_REG18(SP)
    lw   $17, TF_REG17(SP)
    lw   $16, TF_REG16(SP)
    lw   $15, TF_REG15(SP)
    lw   $14, TF_REG14(SP)
    lw   $13, TF_REG13(SP)
    lw   $12, TF_REG12(SP)
    lw   $11, TF_REG11(SP)
    lw   $10, TF_REG10(SP)
    lw   $9, TF_REG9(SP)
    lw   $8, TF_REG8(SP)
    lw   $7, TF_REG7(SP)
    lw   $6, TF_REG6(SP)
    lw   $5, TF_REG5(SP)
    lw   $4, TF_REG4(SP)
    lw   $3, TF_REG3(SP)
    lw   $2, TF_REG2(SP)
    lw   $1, TF_REG1(SP)
    lw   SP, TF_REG29(SP)
.set at
.set reorder
.endmacro

.section .text.exception_entry
.globl _tlb_refill_entry
_tlb_refill_entry:
    j _exception_entry


.org 0x180
.globl _exception_entry
_exception_entry:
    SAVE_CONTEXT
    mfc0    T0, CP0_STATUS
    and     T0, T0, ~(STATUS_UM | STATUS_EXL | STATUS_IE)
    mtc0    T0, CP0_STATUS
    mfc0    T0, CP0_CAUSE
    andi    T0, 0x7c
    srl     T0, 2
    move    A1, T0
	# lw      T0, exception_handlers(T0)
	# jr      T0 
    move   A0, SP
    j exception_handler

.globl _ret_from_exception
.type _ret_from_exception, @function
_ret_from_exception:
    RESTORE_CONTEXT
    eret