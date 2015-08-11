#![feature(box_syntax)]

extern crate num;
extern crate rand;
extern crate image;

use num::complex::Complex;
use rand::distributions::{Range, IndependentSample};
use image::ImageBuffer;

use std::fs::{OpenOptions, File};
use std::io::{BufWriter, Write, Read};
use std::thread;
use std::sync::mpsc;

const POINTS: i32 = 1000000;
const ITERATIONS: i32 = 1000000;
const DIVERGNUM: f64 = 50f64;
const THREADS: i32 = 7;
const BUFSIZE: usize = 1024*1024*128;

const RSTART: f64 = -2f64;
const REND: f64 = 2f64;
const RDIFF: f64 = REND - RSTART;

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

fn main() {

    let file = OpenOptions::new().write(true).create(true).append(true).open("data.data").unwrap();
    let mut bw = BufWriter::new(file);

    let zero = Complex::new(0f64, 0f64);

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
                let mut z = zero;
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
            tx.send((arr, start));
        });
    }
    drop(tx);

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

    println!("render start");
    render();
}

fn to_bytes_f64(f: f64) -> [u8; 8] {
    let raw_bytes: [u8; 8] = unsafe {std::mem::transmute(f)};
    raw_bytes
}

fn to_bytes_i32(i: i32) -> [u8; 4] {
    let raw_bytes: [u8; 4] = unsafe {std::mem::transmute(i)};
    raw_bytes
}

fn render() {
    // ci, cr, count
    
    let mut file = OpenOptions::new().read(true).open("data.data").unwrap();
    let mut buf = [0u8; 36];

    let mut arr = box [[0u8; WIDTH as usize]; HEIGHT as usize];
    
    let mut working = true;
    let mut max = 0;
    println!("start accumulating");
    while working {
        match file.read(&mut buf) {
            Ok(size) => {
                if size == 0 {
                    working = false;
                }
                let (z, c, i) = convert(&buf);
                if i > 1 && RSTART <= c.re && c.re <= REND && ISTART <= c.im && c.im <= IEND {
                    let x: usize = ((c.re - RSTART) * (WIDTH as f64) / RDIFF) as usize;
                    let y: usize = ((c.im - ISTART) * (HEIGHT as f64) / IDIFF) as usize;
                    let mut tmp = arr[y][x];
                    tmp += 1;
                    if max < tmp {
                        max = tmp;
                    }
                    arr[y][x] = tmp;
                } 
            },
            Err(_) => { working = false; },
        }
    }
    println!("end accumulating");

    println!("start render");
    let imgbuf = ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x,y| {
        let i = arr[y as usize][x as usize] as f64;
        let id = i / max as f64;
        let ir = id.powf(0.1);
        let ic = ir * 255f64;
        image::Luma([ic as u8]) 
    });
    println!("end render");

    // Save the image as “fractal.png”
    let mut file = File::create("fractal.png").unwrap();

    // We must indicate the image’s color type and what format to save as
    image::ImageLuma8(imgbuf).save(&mut file, image::PNG);
}

fn convert(u: &[u8; 36]) -> (Complex<f64>, Complex<f64>, i32) {
    let z = Complex::new(to_f64(&u[0..8]), to_f64(&u[8..16]));
    let c = Complex::new(to_f64(&u[16..24]), to_f64(&u[24..32]));
    let i = to_i32(&u[32..36]);
    (z, c, i)
}

fn to_f64(u: &[u8]) -> f64 {
    let a = [u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7]];
    let f: f64 = unsafe { std::mem::transmute(a) };
    f
}

fn to_i32(u: &[u8]) -> i32 {
    let a = [u[0], u[1], u[2], u[3]];
    let i: i32 = unsafe { std::mem::transmute(a) };
    i
}
