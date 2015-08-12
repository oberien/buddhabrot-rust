#![feature(box_syntax)]

extern crate num;
extern crate rand;
extern crate image;

mod structures;

use structures::Hit;

use num::complex::Complex;
use rand::distributions::{Range, IndependentSample};
use image::ImageBuffer;

use std::fs::{OpenOptions, File};
use std::io::{BufWriter, Write, Read};
use std::thread;
use std::sync::mpsc;
use std::cmp::Ordering;

const POINTS: i32 = 000000;
const ITERATIONS: i32 = 1000000;
const DIVERGNUM: f64 = 50f64;
const THREADS: i32 = 5;
const BUFSIZE: usize = 1024*1024*128;

const WIDTH: i32 = 1920;
const HEIGHT: i32 = 1080;

const RSTART: f64 = -2f64;
const REND: f64 = 2f64;

const EPS: f64 = 1e-10;

const PPT: i32 = POINTS/THREADS;

const RDIFF: f64 = REND - RSTART;
const IDIFF: f64 = RDIFF*9f64/16f64;
const IEND: f64 = IDIFF/2f64;
const ISTART: f64 = -IEND;

const DIVERGCOMP: f64 = DIVERGNUM*DIVERGNUM;
const BUFELEMS: usize = BUFSIZE / (8*4 + 4);

fn main() {
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
            let mut next: usize = 1;

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
        bw.write(uarr);
        println!("end writing");
    }

    println!("end calculation");
    render();
}


fn render() {
    // ci, cr, count
    
    let mut file = OpenOptions::new().read(true).open("data.data").unwrap();
    let mut buf = [0u8; 36];

    let mut iarr= box [[0u64; WIDTH as usize]; HEIGHT as usize];
    let mut carr = box [[Complex::new(0f64, 0f64); WIDTH as usize]; HEIGHT as usize];
    
    println!("start accumulating");
    while let Ok(size) = file.read(&mut buf) {
        // FIXME: read until buffer is really full
        if size == 0 {
            break;
        }
        let hit = Hit::from_bytes(&buf);
        if hit.i > 1 {
            // mirror on x-axis
            for sign in vec![-1f64, 1f64] {
                let (x, y) = to_pixel(hit.z.re, sign * hit.z.im);
                if 0 <= x && x < WIDTH && 0 <= y && y < HEIGHT {
                    let xu = x as usize;
                    let yu = y as usize;
                    iarr[yu][xu] += 1;
                    carr[yu][xu] = carr[yu][xu] + Complex::new(hit.c.re - RSTART, sign * hit.c.im - ISTART);
                }
            }
        }
    }
    println!("end accumulating");
    
    println!("start calcRGB");
    let mut rv = Vec::<f64>::with_capacity((WIDTH * HEIGHT) as usize);
    let mut gv = Vec::<f64>::with_capacity((WIDTH * HEIGHT) as usize);
    let mut bv = Vec::<f64>::with_capacity((WIDTH * HEIGHT) as usize);
    
    for (c, &i) in carr.iter().zip(iarr.iter()).flat_map(|(csub, isub)| csub.iter().zip(isub.iter())) {
        rv.push(c.re);
        gv.push(c.im);
        bv.push(i as f64);
    }
    let cmp_func = |a: &f64, b: &f64| {
        let delta = a - b;
        if delta.abs() < EPS {
            Ordering::Equal
        } else if delta < 0f64 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    };
    
    rv.sort_by(&cmp_func);
    gv.sort_by(&cmp_func);
    bv.sort_by(&cmp_func);
    
    println!("end calcRGB");

    println!("start render");
    let imgbuf = ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x,y| {
        let c = carr[y as usize][x as usize];
        let mut r = rv.binary_search_by(|a: &f64| { cmp_func(a, &c.re) }).unwrap() as f64;
        let mut g = gv.binary_search_by(|a: &f64| { cmp_func(a, &c.im) }).unwrap() as f64;
        let mut b = bv.binary_search_by(|a: &f64| { cmp_func(a, &(iarr[y as usize][x as usize] as f64)) }).unwrap() as f64;
        r /= (WIDTH*HEIGHT) as f64;
        g /= (WIDTH*HEIGHT) as f64;
        b /= (WIDTH*HEIGHT) as f64;
        let ru = (r.powi(10) * 255f64) as u8;
        let gu = (g.powi(10) * 255f64) as u8;
        let bu = (b.powi(10) * 255f64) as u8;
        
        image::Rgb([ru, gu, bu])
    });
    println!("end render");

    // Save the image as “fractal.png”
    let mut file = File::create("fractal.png").unwrap();

    println!("start save");
    // We must indicate the image’s color type and what format to save as
    image::ImageRgb8(imgbuf).save(&mut file, image::PNG);
    println!("end save");
}

fn to_pixel(re: f64, im: f64) -> (i32, i32) {
    let x = ((re - RSTART) * (WIDTH as f64) / RDIFF) as i32;
    let y = ((im - ISTART) * (HEIGHT as f64) / IDIFF) as i32;
    (x, y)
}

