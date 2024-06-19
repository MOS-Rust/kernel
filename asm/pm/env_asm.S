.text
.globl _env_pop_trapframe;
.align 2;
.type _env_pop_trapframe, @function;
.ent _env_pop_trapframe;
_env_pop_trapframe:
.frame $29, 0, $31
.set reorder
.set at
mtc0    $5, $10
move    $29, $4
mtc0    $0, $9
mtc0	$8, $11
j       _ret_from_exception
.end _env_pop_trapframe
.size _env_pop_trapframe, .-_env_pop_trapframe