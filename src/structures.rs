extern crate num;

use self::num::complex::Complex;
use std;

#[repr(packed)]
#[derive(Clone)]
pub struct Hit/*Markt*/ {
    pub z: Complex<f64>,
    pub c: Complex<f64>,
    pub i: i32,
}

impl Hit {
    pub fn new(z: Complex<f64>, c: Complex<f64>, i: i32) -> Hit {
        Hit { z: z, c: c, i: i }
    }

    pub fn from_bytes(u: &[u8; 36]) -> Hit {
        unsafe {
            //let z = Complex::new(from_bytes(&u[0..8]), from_bytes(&u[8..16]));
            //let c = Complex::new(from_bytes(&u[16..24]), from_bytes(&u[24..32]));
            //let i = from_bytes(&u[32..36]);
            //Hit { z: z, c: c, i: i }
            std::mem::transmute(*u)
        }
    }
}
