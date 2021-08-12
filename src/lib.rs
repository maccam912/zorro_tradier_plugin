#![feature(once_cell)]

use libc::{c_double, c_float};

type Date = c_double;
type Var = c_double;

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
#[repr(align(1))]
#[repr(C)]
pub struct T6 {
    pub time: Date,
    pub fHigh: c_float,
    pub fLow: c_float,
    pub fOpen: c_float,
    pub fClose: c_float,
    pub fVal: c_float,
    pub fVol: c_float,
}

pub mod dll;
pub mod util;
