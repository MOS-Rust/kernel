.macro SAVE_CONTEXT
.set noat
.set noreorder
mfc0 $26, $12
andi $26, 0x0010
beqz $26, 1f
move $26, $29
li   $29, 0x80400000

1:
subu $29, $29, 0x98
sw   $26, 0x74($29)
mfc0 $26, $12
sw   $26, 0x80($29)
mfc0 $26, $13
sw   $26, 0x90($29)
mfc0 $26, $14
sw   $26, 0x94($29)
mfc0 $26, $8
sw   $26, 0x8C($29)
mfhi $26
sw   $26, 0x84($29)
mflo $26
sw   $26, 0x88($29)
sw   $0, 0x0($29)
sw   $1, 0x4($29)
sw   $2, 0x8($29)
sw   $3, 0xC($29)
sw   $4, 0x10($29)
sw   $5, 0x14($29)
sw   $6, 0x18($29)
sw   $7, 0x1C($29)
sw   $8, 0x20($29)
sw   $9, 0x24($29)
sw   $10, 0x28($29)
sw   $11, 0x2C($29)
sw   $12, 0x30($29)
sw   $13, 0x34($29)
sw   $14, 0x38($29)
sw   $15, 0x3C($29)
sw   $16, 0x40($29)
sw   $17, 0x44($29)
sw   $18, 0x48($29)
sw   $19, 0x4C($29)
sw   $20, 0x50($29)
sw   $21, 0x54($29)
sw   $22, 0x58($29)
sw   $23, 0x5C($29)
sw   $24, 0x60($29)
sw   $25, 0x64($29)
sw   $26, 0x68($29)
sw   $27, 0x6C($29)
sw   $28, 0x70($29)
sw   $30, 0x78($29)
sw   $31, 0x7C($29)
.set at
.set reorder
.endm

.macro RESTORE_CONTEXT
.set noat
.set noreorder
lw   $2, 0x80($29)
mtc0 $2, $12
lw   $3, 0x88($29)
mtlo $3
lw   $2, 0x84($29)
mthi $2
lw   $2, 0x94($29)
mtc0 $2, $14
lw   $31, 0x7C($29)
lw   $30, 0x78($29)
lw   $28, 0x70($29)
lw   $25, 0x64($29)
lw   $24, 0x60($29)
lw   $23, 0x5C($29)
lw   $22, 0x58($29)
lw   $21, 0x54($29)
lw   $20, 0x50($29)
lw   $19, 0x4C($29)
lw   $18, 0x48($29)
lw   $17, 0x44($29)
lw   $16, 0x40($29)
lw   $15, 0x3C($29)
lw   $14, 0x38($29)
lw   $13, 0x34($29)
lw   $12, 0x30($29)
lw   $11, 0x2C($29)
lw   $10, 0x28($29)
lw   $9, 0x24($29)
lw   $8, 0x20($29)
lw   $7, 0x1C($29)
lw   $6, 0x18($29)
lw   $5, 0x14($29)
lw   $4, 0x10($29)
lw   $3, 0xC($29)
lw   $2, 0x8($29)
lw   $1, 0x4($29)
lw   $29, 0x74($29)
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
mfc0    $8, $12
and     $8, $8, ~(0x0010 | 0x0002 | 0x0001)
mtc0    $8, $12
mfc0    $8, $13
andi    $8, 0x7c
move    $5, $8
lw      $8, exception_handlers($8)
jr      $8

.globl _ret_from_exception
.type _ret_from_exception, @function
_ret_from_exception:
RESTORE_CONTEXT
eret