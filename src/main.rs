#![feature(box_syntax)]

extern crate nix;

mod structures;
mod calc;
mod render;
mod consts;

fn main() {
    calc::calc();
    //render::render();
    //render::render_animation();
}

