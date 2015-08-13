extern crate num;
extern crate rand;
extern crate image;

use structures::Hit;
use consts::*;

use self::num::complex::Complex;
use self::rand::distributions::{Range, IndependentSample};

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::thread;
use std::sync::mpsc;
use std;

pub fn calc() {
    let file = OpenOptions::new().write(true).create(true).append(true).open("data.data").unwrap();
    let mut bw = BufWriter::new(file);

    let zero = Complex::new(0f64, 0f64);

    println!("start calculation");

    let (tx, rx) = mpsc::channel();
    for tnum in 0..THREADS-1 {
        let tx = tx.clone();
        let thread_num = tnum;
        
        thread::spawn(move || {
            println!("Start thread #{}", thread_num);
            let mut arr = Vec::<Hit>::with_capacity(BUFELEMS);
            unsafe { arr.set_len(BUFELEMS) };
            let mut start: usize = 0;
            let mut next;

            let mut rng = rand::thread_rng();
            let range_r = Range::new(RSTART, REND);
            let range_i = Range::new(ISTART, IEND);

            let mut progress = 0f64;

            for i in 0..PPT {
                let current_progress = (i as f64) / (PPT as f64);
                if current_progress  > progress + 0.01 {
                    progress = current_progress;
                    println!("Thread #{}: {}", thread_num, progress);
                }
                let re = range_r.ind_sample(&mut rng);
                let im = range_i.ind_sample(&mut rng);
                let c = Complex::<f64>::new(re, im);
                let mut z = zero;
                {
                    let ptr = unsafe { arr.get_unchecked_mut(start) };
                    *ptr = Hit::new(zero, c, 0);
                }
                next = start + 1;
                for i in 1..ITERATIONS {
                    z = z*z + c;
                    {
                        let ptr = unsafe { arr.get_unchecked_mut(next) };
                        *ptr = Hit::new(z, c, i);
                    }
                    next += 1;

                    if z.norm_sqr() > DIVERGCOMP {
                        if BUFELEMS - next >= ITERATIONS as usize {
                            start = next;
                        } else {
                            tx.send((arr.clone(), next)).unwrap();
                            start = 0;
                        }
                        break;
                    }
                }
            }
            tx.send((arr, start)).unwrap();
            println!("End thread #{}", thread_num);
        });
    }
    // shouldn't hold the last sender, otherwise the program will not stop
    drop(tx);

    while let Ok((arr, end)) = rx.recv() {
        println!("start writing {} hits", end);
        let slice = &arr[0..end];
        let ptr = slice.as_ptr() as *const u8;
        let size = slice.len() * std::mem::size_of::<Hit>();
        let uarr = unsafe { std::slice::from_raw_parts(ptr, size) };
        bw.write(uarr).unwrap();
        println!("end writing");
    }

    println!("end calculation");
}
