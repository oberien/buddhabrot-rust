extern crate num;
extern crate image;

use structures::Hit;
use consts::*;

use self::num::complex::Complex;
use self::image::ImageBuffer;

use std::fs::{OpenOptions, File};
use std::io::Read;

pub fn render() {
    let mut file = OpenOptions::new().read(true).open("data.data").unwrap();
    let mut buf = [0u8; 36];

    let mut iarr= box [[0u64; WIDTH as usize]; HEIGHT as usize];
    let mut carr = box [[Complex::new(0f64, 0f64); WIDTH as usize]; HEIGHT as usize];
    
    println!("start accumulating");
    let mut hit_in = 0u64;
    while let Ok(size) = file.read(&mut buf) {
        // FIXME: read until buffer is really full
        if size == 0 {
            break;
        }
        let hit = Hit::from_bytes(&buf);
        if hit.i > 1 {
            // mirror on x-axis
            for sign in vec![-1f64, 1f64] {
                hit_in += 1;
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
    println!("hits in area: {}", hit_in);
    
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
        a.partial_cmp(b).unwrap()
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
    image::ImageRgb8(imgbuf).save(&mut file, image::PNG).unwrap();
    println!("end save");
}

fn to_pixel(re: f64, im: f64) -> (i32, i32) {
    let x = ((re - RSTART) * (WIDTH as f64) / RDIFF) as i32;
    let y = ((im - ISTART) * (HEIGHT as f64) / IDIFF) as i32;
    (x, y)
}
