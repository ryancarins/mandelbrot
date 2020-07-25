use argparse::{ArgumentParser, Collect, Store};
use mandelbrot::Options;
use std::process;

fn generate(iterations: u32, centrex: f32, centrey: f32, scaley: f32, width: u32, height: u32, scale :u32, out: &mut u32){
    println!("generating at ({}, {}) with scale {} and {} iterations at size {}x{} {} samples per pixel", centrex, centrey, scaley, iterations, width, height, scale);
    //mandelbrot(iterations, centrex, centrey, scaley, width, height, scale, &out);
}

fn main() {
    let default_max_width = 4096;
    let default_max_height = 4096;
    let default_max_colours = 256;
    let default_width = 1024;
    let default_height = 1024;
    let default_max_iter = 256;
    let default_centrex:f32 = -0.75;
    let default_centrey:f32 = 0.0;
    let default_scaley: f32 = 2.5;
    let default_samples = 1;


    let mut options = Options::new(
        default_max_width,
        default_max_height,
        default_max_colours,
        default_max_iter,
        default_width,
        default_height,
        default_centrex,
        default_centrey,
        default_scaley,
        default_samples
    );

    //Handle command line arguments
    {
        //Using variables here because I wanted to format and parser takes a &str
        let height_text = format!("Set height (default {})", default_height);
        let width_text = format!("Set width (default {})", default_width);
        let centrex_text = format!("Set centrex (default {})", default_centrex);
        let centrey_text = format!("Set centrey (default {})", default_centrey);
        let max_iter_text = format!("Set maximum number of iterations (default {})", default_max_iter);
        let scaley_text = format!("Set scale(default {})", default_scaley);
        let samples_text = format!("Set samples for supersampling(default {})", default_samples);

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
