use std::fmt;

//Struct for storing arguments
pub struct Options {
    pub max_colours: u32,
    pub max_iter: u32,

    pub width: u32,
    pub height: u32,
    pub centrex: f32,
    pub centrey: f32,
    pub scaley: f32,

    pub samples: u32,
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
        }
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Position ({}, {}) with scale {} and {} iterations at size {}x{} {} samples per pixel",
            self.centrex,
            self.centrey,
            self.scaley,
            self.max_iter,
            self.width,
            self.height,
            self.samples
        )
    }
}

fn iterations2colour(options: &Options, iter: u32, max_iter: u32, flags: u32) -> u32 {
    let iter = (iter * options.max_colours / max_iter) & (options.max_colours - 1);
    return (((flags & 4) << 14) | ((flags & 2) << 7) | (flags & 1)) * iter;
}

pub fn mandelbrot(options: &Options, out: &mut Vec<u32>) {
    let scalex: f32 = options.scaley * options.width as f32 / options.height as f32;

    let dx: f32 = scalex / options.width as f32 / options.samples as f32;
    let dy: f32 = options.scaley / options.height as f32 / options.samples as f32;

    let startx = options.centrex - scalex * 0.5;
    let starty = options.centrey - options.scaley * 0.5;

    for iy in 0..options.height {
        for ix in 0..options.width {
            let mut totaliter: u32 = 0;

            for itery in 0..options.samples {
                for iterx in 0..options.samples {
                    let mut iter: u32 = 0;

                    let x0: f32 =
                        startx + ix as f32 * dx * options.samples as f32 + iterx as f32 * dx;
                    let y0: f32 =
                        starty + iy as f32 * dy * options.samples as f32 + itery as f32 * dy;
                    let mut x: f32 = x0;
                    let mut y: f32 = y0;

                    while x * x + y * y < (2 * 2) as f32 && iter <= options.max_iter {
                        let xtemp: f32 = x * x - y * y + x0 as f32;

                        y = 2 as f32 * x * y + y0 as f32;
                        x = xtemp;
                        iter += 1;
                    }

                    if iter <= options.max_iter {
                        totaliter += iter;
                    }
                }
            }

            out.push(iterations2colour(
                options,
                totaliter / (options.samples * options.samples),
                options.max_iter,
                7,
            ));
        }
    }
}
