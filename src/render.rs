extern crate num;
extern crate image;

use structures::Hit;
use consts::*;

use self::num::complex::Complex;
use self::image::{ImageBuffer, DynamicImage, ImageRgb8};

use std::fs::{OpenOptions, File};
use std::io::Read;
use std::cmp::Ordering;
use std::thread;
use std;

pub fn render() {
    let (mut iarr, mut carr) = accumulate();

    let img = render_5d(&mut iarr, &mut carr);

    println!("start save");
    save_image(img, "fractal.png");
    println!("end save");
}

pub fn render_animation() {
    let mut file = OpenOptions::new().read(true).open("data.data").unwrap();
    let metadata = file.metadata().unwrap();
    let len = metadata.len() as i32;
    let hits = len/std::mem::size_of::<Hit>() as i32;
    let hits_per_frame = hits/FRAMES;

    let mut iarr = box [[0u64; WIDTH as usize]; HEIGHT as usize];
    let mut carr = box [[Complex::new(0f64, 0f64); WIDTH as usize]; HEIGHT as usize];
    
    for i in 0..FRAMES {
        println!("start accumulating frame #{}", i);
        accumulate_n(&mut file, &hits_per_frame, &mut iarr, &mut carr);
        println!("end accumulating frame #{}", i);

        println!("start rendering frame #{}", i);
        let img = render_5d(&mut iarr, &mut carr);
        println!("start rendering frame #{}", i);

        thread::spawn(move || {
           println!("start saving frame #{}", i);
           save_image(img, &format!("animation/{}.png", i));
           println!("end saving frame #{}", i);
        });
    }
}

fn render_5d(iarr: &mut Box<[[u64; WIDTH as usize]; HEIGHT as usize]>,
                carr: &mut Box<[[Complex<f64>; WIDTH as usize]; HEIGHT as usize]>) -> DynamicImage {
    println!("start calcRGB");
    let mut rv = Vec::<f64>::with_capacity((WIDTH * HEIGHT) as usize);
    let mut gv = Vec::<f64>::with_capacity((WIDTH * HEIGHT) as usize);
    let mut bv = Vec::<f64>::with_capacity((WIDTH * HEIGHT) as usize);
    
    for (c, &i) in carr.iter().zip(iarr.iter()).flat_map(|(csub, isub)| csub.iter().zip(isub.iter())) {
        rv.push(c.re);
        gv.push(c.im);
        bv.push(i as f64);
    }

    histogram(&mut rv, &mut gv, &mut bv);
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
    println!("start converting image");
    let img = ImageRgb8(imgbuf);
    println!("end converting image");
    img
}

fn accumulate() -> (Box<[[u64; WIDTH as usize]; HEIGHT as usize]>, Box<[[Complex<f64>; WIDTH as usize]; HEIGHT as usize]>) {
    let mut file = OpenOptions::new().read(true).open("data.data").unwrap();
    let metadata = file.metadata().unwrap();

    let mut iarr = box [[0u64; WIDTH as usize]; HEIGHT as usize];
    let mut carr = box [[Complex::new(0f64, 0f64); WIDTH as usize]; HEIGHT as usize];
    
    println!("start accumulating");
    accumulate_n(&mut file, &(metadata.len() as i32), &mut iarr, &mut carr);
    println!("end accumulating");
    (iarr, carr)
}

// TODO: get Read instead of File
fn accumulate_n(file: &mut File, num: &i32, iarr: &mut Box<[[u64; WIDTH as usize]; HEIGHT as usize]>,
                carr: &mut Box<[[Complex<f64>; WIDTH as usize]; HEIGHT as usize]>) {
    let mut buf = [0u8; 36];

    for _ in 0..*num {
        let size = match file.read(&mut buf) {
            Ok(size) => size,
            Err(_) => break,
        };
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
}

fn cmp_func(a: &f64, b: &f64) -> Ordering {
    a.partial_cmp(b).unwrap()
}

fn histogram(rv: &mut Vec<f64>, gv: &mut Vec<f64>, bv: &mut Vec<f64>) {
    
    rv.sort_by(&cmp_func);
    gv.sort_by(&cmp_func);
    bv.sort_by(&cmp_func);
}

fn save_image(img: DynamicImage, name: &str) {
    // Save the image as the file supplied with "name"
    let mut file = File::create(name).unwrap();

    // We must indicate the image’s color type and what format to save as
    img.save(&mut file, image::PNG).unwrap();
}

fn to_pixel(re: f64, im: f64) -> (i32, i32) {
    let x = ((re - RSTART) * (WIDTH as f64) / RDIFF) as i32;
    let y = ((im - ISTART) * (HEIGHT as f64) / IDIFF) as i32;
    (x, y)
}
