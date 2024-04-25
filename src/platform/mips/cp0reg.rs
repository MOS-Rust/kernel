//! MIPS CP0 registers and bit definitions
use crate::{const_export_str, const_export_usize};

const_export_str!(CP0_INDEX, "$0");
const_export_str!(CP0_RANDOM, "$1");
const_export_str!(CP0_ENTRYLO0, "$2");
const_export_str!(CP0_ENTRYLO1, "$3");
const_export_str!(CP0_CONTEXT, "$4");
const_export_str!(CP0_PAGEMASK, "$5");
const_export_str!(CP0_WIRED, "$6");
const_export_str!(CP0_BADVADDR, "$8");
const_export_str!(CP0_COUNT, "$9");
const_export_str!(CP0_ENTRYHI, "$10");
const_export_str!(CP0_COMPARE, "$11");
const_export_str!(CP0_STATUS, "$12");
const_export_str!(CP0_CAUSE, "$13");
const_export_str!(CP0_EPC, "$14");
const_export_str!(CP0_PRID, "$15");
const_export_str!(CP0_EBASE, "$15, 1");
const_export_str!(CP0_CONFIG, "$16");
const_export_str!(CP0_LLADDR, "$17");
const_export_str!(CP0_WATCHLO, "$18");
const_export_str!(CP0_WATCHHI, "$19");
const_export_str!(CP0_XCONTEXT, "$20");
const_export_str!(CP0_FRAMEMASK, "$21");
const_export_str!(CP0_DIAGNOSTIC, "$22");
const_export_str!(CP0_PERFORMANCE, "$25");
const_export_str!(CP0_ECC, "$26");
const_export_str!(CP0_CACHEERR, "$27");
const_export_str!(CP0_TAGLO, "$28");
const_export_str!(CP0_TAGHI, "$29");
const_export_str!(CP0_ERROREPC, "$30");

const_export_usize!(STATUS_CU3, 0x80000000);
const_export_usize!(STATUS_CU2, 0x40000000);
const_export_usize!(STATUS_CU1, 0x20000000);
const_export_usize!(STATUS_CU0, 0x10000000);
const_export_usize!(STATUS_BEV, 0x00400000);
const_export_usize!(STATUS_IM0, 0x0100);
const_export_usize!(STATUS_IM1, 0x0200);
const_export_usize!(STATUS_IM2, 0x0400);
const_export_usize!(STATUS_IM3, 0x0800);
const_export_usize!(STATUS_IM4, 0x1000);
const_export_usize!(STATUS_IM5, 0x2000);
const_export_usize!(STATUS_IM6, 0x4000);
const_export_usize!(STATUS_IM7, 0x8000);
const_export_usize!(STATUS_UM, 0x0010);
const_export_usize!(STATUS_R0, 0x0008);
const_export_usize!(STATUS_ERL, 0x0004);
const_export_usize!(STATUS_EXL, 0x0002);
const_export_usize!(STATUS_IE, 0x0001);
