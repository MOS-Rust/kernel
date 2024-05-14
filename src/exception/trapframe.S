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


.globl _ret_from_exception
.type _ret_from_exception, @function
_ret_from_exception:
    RESTORE_CONTEXT
    eret