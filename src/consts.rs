pub const POINTS: i32 = 1000000;
pub const ITERATIONS: i32 = 1000000;
pub const DIVERGNUM: f64 = 50f64;
pub const THREADS: i32 = 7;

pub const BUFSIZE: usize = 1024*1024*128;

pub const RSTART: f64 = -2f64;
pub const REND: f64 = 2f64;

pub const FPS: i32 = 30;
pub const SEC: i32 = 15;


pub const PPT: i32 = POINTS/THREADS;

pub const RDIFF: f64 = REND - RSTART;
pub const IDIFF: f64 = RDIFF*9f64/16f64;
pub const IEND: f64 = IDIFF/2f64;
pub const ISTART: f64 = -IEND;

pub const DIVERGCOMP: f64 = DIVERGNUM*DIVERGNUM;
pub const BUFELEMS: usize = BUFSIZE / (8*4 + 4);

pub const WIDTH: i32 = 1920;
pub const HEIGHT: i32 = 1080;

pub const FRAMES: i32 = SEC*FPS;
