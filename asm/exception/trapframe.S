.macro RESTORE_CONTEXT
.set noat
.set noreorder
lw   v0, 0x80($29)
mtc0 v0, $12
lw   v1, 0x88($29)
mtlo v1
lw   v0, 0x84($29)
mthi v0
lw   v0, 0x94($29)
mtc0 v0, $14
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
lw   $$29, 0x74($29)
.set at
.set reorder
.endmacro