extern crate num;
extern crate rand;

use num::complex::Complex;
use rand::distributions::{Range, IndependentSample};

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::thread;
use std::sync::mpsc;

fn main() {
    const POINTS: i32 = 1000000;
    const ITERATIONS: i32 = 10000;
    const DIVERGNUM: f64 = 50f64;
    const THREADS: i32 = 7;
    const BUFSIZE: usize = 1024*1024*128;

    const RSTART: f64 = -2f64;
    const REND: f64 = 2f64;
    const RDIFF: f64 = REND - RSTART;
    //const ISTART: f64 = -1;
    //const IEND: f64 = 1;

    const PPT: i32 = POINTS/THREADS;
    
    const IDIFF: f64 = RDIFF*9f64/16f64;
    const IEND: f64 = IDIFF/2f64;
    const ISTART: f64 = -IEND;

    const WIDTH: i32 = 1920;
    const HEIGHT: i32 = 1080;

    const RD: f64 = RDIFF / WIDTH as f64;
    const ID: f64 = IDIFF / HEIGHT as f64;

    const DIVERGCOMP: f64 = DIVERGNUM*DIVERGNUM;
    const BUFELEMS: usize = BUFSIZE / (8*4 + 4);

    let file = OpenOptions::new().write(true).create(true).append(true).open("data.data").unwrap();
    let mut bw = BufWriter::new(file);

    let (tx, rx) = mpsc::channel();
    for _ in 0..THREADS-1 {
        let tx = tx.clone();
        
        thread::spawn(move || {
            let mut arr = Vec::<(Complex<f64>, Complex<f64>, i32)>::with_capacity(BUFELEMS);
            arr.reserve(0);
            let mut start: usize = 0;
            let mut next: usize = 1;

            let mut rng = rand::thread_rng();
            let range_r = Range::new(RSTART, REND);
            let range_i = Range::new(ISTART, IEND);

            for _ in 0..PPT {
                let re = range_r.ind_sample(&mut rng);
                let im = range_i.ind_sample(&mut rng);
                let c = Complex::<f64>::new(re, im);
                let mut z = c;
                {
                    let ptr = unsafe { arr.get_unchecked_mut(start) };
                    *ptr = (Complex::new(0f64, 0f64), c, 0);
                }
                next = start + 1;
                for i in 1..ITERATIONS {
                    z = z*z + c;
                    {
                        let ptr = unsafe { arr.get_unchecked_mut(next) };
                        *ptr = (z, c, i);
                    }
                    next += 1;

                    if z.norm_sqr() > DIVERGCOMP {
                        if BUFELEMS - next >= ITERATIONS as usize {
                            start = next;
                            next += 1;
                        } else {
                            tx.send((arr.clone(), next));
                            start = 0;
                            next = 1;
                        }
                        break;
                    }
                }
            }
        });
    }

    let mut working = true;
    while working {
        match rx.recv() {
            Ok(tuple) => {
                //let arr: Vec<(Complex<f64>, Complex<f64>, i32)> = tuple.0;
                //let end: i32 = tuple.1;
                let (arr, end) = tuple;
                for i in 0..end {
                    let (z, c, i) = *unsafe { arr.get_unchecked(i) };
                    bw.write(&to_bytes_f64(z.re));
                    bw.write(&to_bytes_f64(z.im));
                    bw.write(&to_bytes_f64(c.re));
                    bw.write(&to_bytes_f64(c.im));
                    bw.write(&to_bytes_i32(i));
                }
            },
            Err(_) => {working = false;},
        }
    }



    //let imgbuf = ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x,y| {
    //    let i = arr[y as usize][x as usize] as f64;
    //    let id = i / ITERATIONS as f64;
    //    let ir = id.powf(0.1);
    //    let ic = ir * 255f64;
    //    image::Luma([ic as u8]) 
    //});

    //// Save the image as “fractal.png”
    //let mut file = File::create("fractal.png").unwrap();

    //// We must indicate the image’s color type and what format to save as
    //image::ImageLuma8(imgbuf).save(&mut file, image::PNG);
}

fn to_bytes_f64(f: f64) -> [u8; 8] {
    let raw_bytes: [u8; 8] = unsafe {std::mem::transmute(f)};
    raw_bytes
}

fn to_bytes_i32(i: i32) -> [u8; 4] {
    let raw_bytes: [u8; 4] = unsafe {std::mem::transmute(i)};
    raw_bytes
}
