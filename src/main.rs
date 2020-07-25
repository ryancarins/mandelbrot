use argparse::{ArgumentParser, Store};
use mandelbrot::Options;
use std::process;

const DEFAULT_MAX_WIDTH: usize = 1024;
const DEFAULT_MAX_HEIGHT: usize = 1024;
const DEFAULT_MAX_COLOURS: u32 = 256;
const DEFAULT_WIDTH: u32 = 1024;
const DEFAULT_HEIGHT: u32 = 1024;
const DEFAULT_MAX_ITER: u32 = 256;
const DEFAULT_CENTREX:f32 = -0.75;
const DEFAULT_CENTREY:f32 = 0.0;
const DEFAULT_SCALEY: f32 = 2.5;
const DEFAULT_SAMPLES: u32 = 1;

fn generate(iterations: u32, centrex: f32, centrey: f32, scaley: f32, width: u32, height: u32, scale :u32, out: &mut u32){
    println!("generating at ({}, {}) with scale {} and {} iterations at size {}x{} {} samples per pixel", centrex, centrey, scaley, iterations, width, height, scale);
    //mandelbrot(iterations, centrex, centrey, scaley, width, height, scale, &out);
}

fn main() {
    let buffer: [u32;DEFAULT_MAX_WIDTH*DEFAULT_MAX_HEIGHT];

    let mut options = Options::new(
        DEFAULT_MAX_WIDTH,
        DEFAULT_MAX_HEIGHT,
        DEFAULT_MAX_COLOURS,
        DEFAULT_MAX_ITER,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        DEFAULT_CENTREX,
        DEFAULT_CENTREY,
        DEFAULT_SCALEY,
        DEFAULT_SAMPLES
    );

    //Handle command line arguments
    {
        //Using variables here because I wanted to format and parser takes a &str
        let height_text = format!("Set height (default {})", DEFAULT_HEIGHT);
        let width_text = format!("Set width (default {})", DEFAULT_WIDTH);
        let centrex_text = format!("Set centrex (default {})", DEFAULT_CENTREX);
        let centrey_text = format!("Set centrey (default {})", DEFAULT_CENTREY);
        let max_iter_text = format!("Set maximum number of iterations (default {})", DEFAULT_MAX_ITER);
        let scaley_text = format!("Set scale(default {})", DEFAULT_SCALEY);
        let samples_text = format!("Set samples for supersampling(default {})", DEFAULT_SAMPLES);


        let mut parser = ArgumentParser::new();
        parser.set_description("Mandelbrot generator");
        parser
            .refer(&mut options.width)
            .add_option(&["-w", "--width"], Store, &width_text);

        parser
            .refer(&mut options.height)
            .add_option(&["-h", "--height"], Store, &height_text);

        parser
            .refer(&mut options.centrex)
            .add_option(&["--centrex"], Store, &centrex_text);
        parser
            .refer(&mut options.centrey)
            .add_option(&["--centrey"], Store, &centrey_text);
        parser
            .refer(&mut options.max_iter)
            .add_option(&["--iterations"], Store, &max_iter_text);
        parser
            .refer(&mut options.scaley)
            .add_option(&["--scale"], Store, &scaley_text);
        parser
            .refer(&mut options.samples)
            .add_option(&["--samples"], Store, &samples_text);
        parser.parse_args_or_exit();
    }
}
