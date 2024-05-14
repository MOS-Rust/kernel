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

.section .text.exception_entry
.globl _tlb_refill_entry
_tlb_refill_entry:
    j _exception_entry


.org 0x180
.globl _exception_entry
_exception_entry:
    SAVE_CONTEXT
    # mfc0    T0, CP0_STATUS
	# and     T0, T0, ~(STATUS_UM | STATUS_EXL | STATUS_IE)
	# mtc0    T0, CP0_STATUS
    j exception_handler