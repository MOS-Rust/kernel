//! Cause register (CP0 register 13, select 0)

#[derive(Clone, Copy, Debug)]
pub struct Cause {
    pub bits: u32,
}

register_rw!(13, 0);
register_struct_rw!(Cause);

#[derive(Debug, Clone, Copy)]
pub enum Exception {
    Int      ,
    Mod      ,
    TLBL     ,
    TLBS     ,
    AdEL     ,
    AdES     ,
    IBE      ,
    DBE      ,
    Sys      ,
    Bp       ,
    RI       ,
    CpU      ,
    Ov       ,
    Tr       ,
    FPE      ,
    IS1      ,
    CEU      ,
    C2E      ,
    WATCH    ,
    MCheck   ,
    CacheErr ,
    Reserved ,
    Unknown  ,
}

impl Exception {
    pub fn from(exccode: u32) -> Self {
        match exccode {
            0  => Exception::Int     ,
            1  => Exception::Mod     ,
            2  => Exception::TLBL    ,
            3  => Exception::TLBS    ,
            4  => Exception::AdEL    ,
            5  => Exception::AdES    ,
            6  => Exception::IBE     ,
            7  => Exception::DBE     ,
            8  => Exception::Sys     ,
            9  => Exception::Bp      ,
            10 => Exception::RI      ,
            11 => Exception::CpU     ,
            12 => Exception::Ov      ,
            13 => Exception::Tr      ,
            15 => Exception::FPE     ,
            16 => Exception::IS1     ,
            17 => Exception::CEU     ,
            18 => Exception::C2E     ,
            23 => Exception::WATCH   ,
            24 => Exception::MCheck  ,
            30 => Exception::CacheErr,
            14 => Exception::Reserved,
            19..=22 => Exception::Reserved,
            25..=29 => Exception::Reserved,
            _  => Exception::Unknown ,
        }
    }
}

impl Cause {
    #[inline]
    pub fn exception(&self) -> Exception {
        Exception::from((self.bits >> 2) & 0x1F)
    }
}