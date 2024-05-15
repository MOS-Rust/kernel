.text
.globl _env_pop_trapframe;           
.align 2;                            
.type _env_pop_trapframe, @function; 
.ent _env_pop_trapframe;             
_env_pop_trapframe:                  
.frame SP, 0, RA
.set reorder
.set at
	mtc0    A1, CP0_ENTRYHI
	move    SP, A0
	mtc0    ZERO, CP0_COUNT
	mtc0	T0, CP0_COMPARE
	j       _ret_from_exception
.end _env_pop_trapframe
.size _env_pop_trapframe, .-_env_pop_trapframe
