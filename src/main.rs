use argparse::{ArgumentParser, Store};
use image::{ImageBuffer, RgbImage};
use mandelbrot::Options;

const DEFAULT_MAX_COLOURS: u32 = 256;
const DEFAULT_WIDTH: u32 = 1024;
const DEFAULT_HEIGHT: u32 = 1024;
const DEFAULT_MAX_ITER: u32 = 256;
const DEFAULT_CENTREX: f32 = -0.75;
const DEFAULT_CENTREY: f32 = 0.0;
const DEFAULT_SCALEY: f32 = 2.5;
const DEFAULT_SAMPLES: u32 = 1;
const DEFAULT_FILENAME: &str = "output.bmp";

fn generate(options: &Options, out: &mut Vec<u32>) {
    println!("{}", options);
    mandelbrot::mandelbrot(options, out);
}

fn main() {
    let mut buffer: Vec<u32> = vec![];
    let mut filename = std::string::String::from(DEFAULT_FILENAME);

    let mut options = Options::new(
        DEFAULT_MAX_COLOURS,
        DEFAULT_MAX_ITER,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        DEFAULT_CENTREX,
        DEFAULT_CENTREY,
        DEFAULT_SCALEY,
        DEFAULT_SAMPLES,
    );

    //Handle command line arguments
    {
        //Using variables here because I wanted to format and parser takes a &str
        let height_text = format!("Set height (default {})", DEFAULT_HEIGHT);
        let width_text = format!("Set width (default {})", DEFAULT_WIDTH);
        let centrex_text = format!("Set centrex (default {})", DEFAULT_CENTREX);
        let centrey_text = format!("Set centrey (default {})", DEFAULT_CENTREY);
        let max_iter_text = format!(
            "Set maximum number of iterations (default {})",
            DEFAULT_MAX_ITER
        );
        let scaley_text = format!("Set scale(default {})", DEFAULT_SCALEY);
        let samples_text = format!("Set samples for supersampling(default {})", DEFAULT_SAMPLES);
        let filename_text = format!(
            "Set filename(default {}) supported formats are PNG, JPEG, BMP, and TIFF",
            DEFAULT_FILENAME
        );

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
        parser
            .refer(&mut filename)
            .add_option(&["--name"], Store, &filename_text);

        parser.parse_args_or_exit();
    }

    generate(&options, &mut buffer);

    //Create a blank image to write to
    let mut img: RgbImage = ImageBuffer::new(options.width, options.height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        //32 bit number but only storing rgb so split it into its 3 8 bit components
        let r =
            ((buffer[y as usize * options.width as usize + x as usize] & 0x00ff0000) >> 16) as u8;
        let g =
            ((buffer[y as usize * options.width as usize + x as usize] & 0x0000ff00) >> 8) as u8;
        let b = (buffer[y as usize * options.width as usize + x as usize] & 0x000000ff) as u8;
        *pixel = image::Rgb([r, g, b]);
    }
    img.save(&filename).unwrap_or_else(|_| {
        eprintln!("Error: Could not write file");
    });
}
