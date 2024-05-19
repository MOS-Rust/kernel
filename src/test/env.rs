#![allow(dead_code)]

// use log::debug;
// 
// use crate::{mm::{addr::VA, layout::UTOP}};

pub fn env_test() {
    // let e1 = env_alloc(0).unwrap();
    // let e2 = env_alloc(0).unwrap();
    // let e3 = env_alloc(0).unwrap();
// 
    // assert!(e1.id != 0);
    // assert!(e2.id != 0 && e2.id != e1.id);
    // assert!(e3.id != 0 && e3.id != e2.id && e3.id != e1.id);
// 
    // assert!(e1.id == 2048);
    // assert!(e2.id == 4097);
    // assert!(e3.id == 6146);
// 
    // // pages not mapped therefore not checked
// 
    // let base_pgdir = get_base_pgdir();
    // // for page_addr in (0..(NENV * size_of::<Env>())).step_by(PAGE_SIZE) {
    // //     unsafe {
    // //         assert!(base_pgdir.va2pa(VA(UENVS + page_addr)).unwrap() 
    // //             == VA(addr_of_mut!(ENVS) as usize).paddr() + page_addr);
    // //     }
    // // }
// 
    // assert!(e3.pgdir().pte_at(VA(UTOP).pdx()).0 == base_pgdir.pte_at(VA(UTOP).pdx()).0);
    // assert!(e3.pgdir().pte_at(VA(UTOP).pdx() - 1).0 == 0);
// 
    // env_free(e3);
    // env_free(e2);
    // env_free(e1);
// 
    // debug!("env_check succeeded!\n");
}

// 	for (page_addr = 0; page_addr < NENV * sizeof(struct Env); page_addr += PAGE_SIZE) {
// 		assert(va2pa(base_pgdir, UENVS + page_addr) == PADDR(envs) + page_addr);
// 	}
// 	/* check env_setup_vm() work well */
// 	printk("pe1->env_pgdir %x\n", pe1->env_pgdir);

// 	assert(pe2->env_pgdir[PDX(UTOP)] == base_pgdir[PDX(UTOP)]);
// 	assert(pe2->env_pgdir[PDX(UTOP) - 1] == 0);
// 	printk("env_setup_vm passed!\n");

// 	printk("pe2`s sp register %x\n", pe2->env_tf.regs[29]);

// 	/* free all env allocated in this function */
// 	TAILQ_INSERT_TAIL(&env_sched_list, pe0, env_sched_link);
// 	TAILQ_INSERT_TAIL(&env_sched_list, pe1, env_sched_link);
// 	TAILQ_INSERT_TAIL(&env_sched_list, pe2, env_sched_link);

// 	env_free(pe2);
// 	env_free(pe1);
// 	env_free(pe0);

// 	printk("env_check() succeeded!\n");