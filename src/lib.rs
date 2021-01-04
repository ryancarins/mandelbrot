use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

//Struct for storing arguments
#[derive(Copy, Clone, Debug)]
pub struct Options {
    pub max_colours: u32,
    pub max_iter: u32,

    pub width: u32,
    pub height: u32,
    pub centrex: f32,
    pub centrey: f32,
    pub scaley: f32,

    pub samples: u32,
    pub colour: u32,
    pub colourise: bool,
    pub threads: u32,
    pub thread_id: Option<u32>,
    pub progress: bool,
}

impl Options {
    pub fn new(
        max_colours: u32,
        max_iter: u32,
        width: u32,
        height: u32,
        centrex: f32,
        centrey: f32,
        scaley: f32,
        samples: u32,
        colour: u32,
        colourise: bool,
        threads: u32,
        progress: bool,
    ) -> Options {
        Options {
            max_colours,
            max_iter,
            width,
            height,
            centrex,
            centrey,
            scaley,
            samples,
            colour,
            colourise,
            threads,
            thread_id: None,
            progress,
        }
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Position ({}, {}) with scale {} and {} iterations at size {}x{} {} samples per pixel {} threads and colour code {}",
            self.centrex,
            self.centrey,
            self.scaley,
            self.max_iter,
            self.width,
            self.height,
            self.samples * self.samples,
            self.threads,
            self.colour
        )
    }
}

#[inline]
fn iterations2colour(options: &Options, iter: u32, max_iter: u32, flags: u32) -> u32 {
    let iter = (iter * options.max_colours / max_iter) & (options.max_colours - 1);
    return (((flags & 4) << 14) | ((flags & 2) << 7) | (flags & 1)) * iter;
}

fn interlocked_increment(shared: Arc<Mutex<u32>>) -> u32 {
    let mut current = shared.lock().unwrap();
    let temp = *current;
    *current += 1;
    temp
}

pub fn mandelbrot(options: Options, sender: Sender<(u32, u32)>, current_line: Arc<Mutex<u32>>) {
    let scalex: f32 = options.scaley * options.width as f32 / options.height as f32;
    let colour: u32;
    if options.colourise {
        colour = options.thread_id.unwrap() % 7 + 1;
    } else {
        colour = options.colour;
    }

    let dx: f32 = scalex / options.width as f32 / options.samples as f32;
    let dy: f32 = options.scaley / options.height as f32 / options.samples as f32;

    let startx = options.centrex - scalex * 0.5;
    let starty = options.centrey - options.scaley * 0.5;
    let mut iy = interlocked_increment(current_line.clone());

    while iy < options.height {
        for ix in 0..options.width {
            let mut totaliter: u32 = 0;

            for itery in 0..options.samples {
                for iterx in 0..options.samples {
                    let mut iter: u32 = 0;

                    let x0: f32 = startx + (ix as f32 * options.samples as f32 + iterx as f32) * dx;
                    let y0: f32 = starty + (iy as f32 * options.samples as f32 + itery as f32) * dy;
                    let mut x: f32 = x0;
                    let mut y: f32 = y0;
                    let mut xtemp: f32;
                    while x * x + y * y < (2 * 2) as f32 && iter <= options.max_iter {
                        xtemp = x * x - y * y + x0 as f32;

                        y = 2.0 * x * y + y0 as f32;
                        x = xtemp;
                        iter += 1;
                    }

                    if iter <= options.max_iter {
                        totaliter += iter;
                    }
                }
            }

            sender
                .send((
                    iy * options.width + ix,
                    iterations2colour(
                        &options,
                        totaliter / (options.samples * options.samples),
                        options.max_iter,
                        colour,
                    ),
                ))
                .unwrap();
        }
        iy = interlocked_increment(current_line.clone());
    }
}
