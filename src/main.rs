#[macro_use]
extern crate rocket;
use argparse::{ArgumentParser, Store, StoreTrue};
use image::{ImageBuffer, RgbImage};
use mandelbrot::Options;
use pbr::ProgressBar;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::thread;
use std::time::Instant;
use rocket::fs::{FileServer, relative};
use image::EncodableLayout;

const DEFAULT_MAX_COLOURS: u32 = 256;
const DEFAULT_WIDTH: u32 = 1024;
const DEFAULT_HEIGHT: u32 = 1024;
const DEFAULT_MAX_ITER: u32 = 256;
const DEFAULT_CENTREX: f32 = -0.75;
const DEFAULT_CENTREY: f32 = 0.0;
const DEFAULT_SCALEY: f32 = 2.5;
const DEFAULT_SAMPLES: u32 = 1;
const DEFAULT_THREADS: u32 = 1;
const DEFAULT_FILENAME: &str = "output.bmp";
const DEFAULT_COLOUR_CODE: u32 = 7;
const DEFAULT_COLOURISE: bool = false;
const DEFAULT_PROGRESS: bool = false;
const DEFAULT_OCL: bool = false;
const DEFAULT_SERVICE: bool = false;

fn generate(options: Options, out: &mut Vec<u32>) {
    println!("{}", options);
    let start = Instant::now();

    //Run the opencl version and return
    if options.ocl {
        println!(
            "Running opencl version threads flag will be ignored and no progress bar can be shown"
        );
        mandelbrot::opencl_mandelbrot(options, out).expect("Failed to generate image with opencl");
        println!("time taken: {}ms", start.elapsed().as_millis());
        return;
    }

    let current_line = Arc::new(Mutex::new(0));
    let (tx, rx) = mpsc::channel();

    for i in 0..options.threads {
        let mut local_options = options;
        local_options.thread_id = Some(i);
        let local_tx = mpsc::Sender::clone(&tx);
        let current_line_ref = Arc::clone(&current_line);
        thread::spawn(move || mandelbrot::mandelbrot(local_options, local_tx, current_line_ref));
    }

    //Drop tx because we only need it for cloning and if we don't drop it the loop below will never end
    drop(tx);

    let mut pb = ProgressBar::new(100);
    pb.show_bar = options.progress;
    pb.show_counter = options.progress;
    pb.show_message = options.progress;
    pb.show_percent = options.progress;
    pb.show_speed = false;
    pb.show_time_left = false;
    pb.show_tick = false;
    let mut pos = 0;
    for (i, val) in rx {
        pos += 1;
        if pos % (options.width * options.height / 100) == 0 {
            pb.inc();
        }
        out[i as usize] = val;
    }
    pb.finish_print("done");

    //mandelbrot::mandelbrot(options, out);
    println!("time taken: {}ms", start.elapsed().as_millis());
}

#[get("/?<max_colours>&<max_iter>&<width>&<height>&<threads>&<ocl>&<samples>&<scale>")]
fn mandelbrot_rest(
    max_colours: Option<u32>,
    max_iter: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    threads: Option<u32>,
    ocl: Option<bool>,
    samples: Option<u32>,
    scale: Option<f32>
) -> String {
    let options = Options::new(
        max_colours.unwrap_or(DEFAULT_MAX_COLOURS),
        max_iter.unwrap_or(DEFAULT_MAX_ITER),
        width.unwrap_or(DEFAULT_WIDTH),
        height.unwrap_or(DEFAULT_HEIGHT),
        DEFAULT_CENTREX,
        DEFAULT_CENTREY,
        scale.unwrap_or(DEFAULT_SCALEY),
        samples.unwrap_or(DEFAULT_SAMPLES),
        DEFAULT_COLOUR_CODE,
        DEFAULT_COLOURISE,
        threads.unwrap_or(DEFAULT_THREADS),
        DEFAULT_PROGRESS,
        ocl.unwrap_or(DEFAULT_OCL),
        true,
    );


    let filename = format!("images/{}-{}-{}-{}-{}-{}-{}-{}-{}-{}.png",options.width,options.height,options.max_iter,options.max_colours, options.centrex, options.centrey,options.scaley,options.samples,options.colour,options.colourise);

    if Path::new(&filename).exists() {
        return filename;
    }

    let mut buffer = vec![0; (options.width * options.height) as usize];
    generate(options, &mut buffer);
    let mut img: RgbImage = ImageBuffer::new(options.width, options.height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        //32 bit number but only storing rgb so split it into its 3 8 bit components
        let b = ((buffer[y as usize * options.width as usize + x as usize] & 0x00ff0000) >> 16)
            as u8;
        let g = ((buffer[y as usize * options.width as usize + x as usize] & 0x0000ff00) >> 8)
            as u8;
        let r = (buffer[y as usize * options.width as usize + x as usize] & 0x000000ff) as u8;
        *pixel = image::Rgb([r, g, b]);
    }

    img.save(&filename).unwrap_or_else(|_| {
        eprintln!("Error: Could not write file");
    });

    return filename;
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
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
        DEFAULT_COLOUR_CODE,
        DEFAULT_COLOURISE,
        DEFAULT_THREADS,
        DEFAULT_PROGRESS,
        DEFAULT_OCL,
        DEFAULT_SERVICE,
    );

    //Handle command line arguments
    {
        //Using variables here because I wanted to format and parser takes a &str
        let height_text = format!("Set height (default {})", DEFAULT_HEIGHT);
        let width_text = format!("Set width (default {})", DEFAULT_WIDTH);
        let centrex_text = format!("Set centrex (default {})", DEFAULT_CENTREX);
        let centrey_text = format!("Set centrey (default {})", DEFAULT_CENTREY);
        let colourise_text = format!(
            "Use a different colour for each thread (default {})",
            DEFAULT_COLOURISE
        );
        let max_iter_text = format!(
            "Set maximum number of iterations (default {})",
            DEFAULT_MAX_ITER
        );
        let scaley_text = format!("Set scale(default {})", DEFAULT_SCALEY);
        let samples_text = format!("Set samples for supersampling(default {})", DEFAULT_SAMPLES);
        let colour_text = format!("Set colour for image(default {})", DEFAULT_COLOUR_CODE);
        let progress_text = format!("Display progress bar (default {})", DEFAULT_PROGRESS);
        let ocl_text = format!("Use opencl instead of cpu (default {})", DEFAULT_OCL);
        let service_text = format!("Run as a REST service (default {})", DEFAULT_OCL);
        let threads_text = format!(
            "Set number of threads to use for processing(default {})",
            DEFAULT_THREADS
        );
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
            .refer(&mut options.colour)
            .add_option(&["--colour"], Store, &colour_text);
        parser
            .refer(&mut options.threads)
            .add_option(&["--threads", "-j"], Store, &threads_text);
        parser
            .refer(&mut filename)
            .add_option(&["--name"], Store, &filename_text);
        parser.refer(&mut options.colourise).add_option(
            &["--colourise"],
            StoreTrue,
            &colourise_text,
        );
        parser
            .refer(&mut options.progress)
            .add_option(&["--progress"], StoreTrue, &progress_text);
        parser
            .refer(&mut options.ocl)
            .add_option(&["--ocl"], StoreTrue, &ocl_text);

        parser
            .refer(&mut options.service)
            .add_option(&["--service"], StoreTrue, &service_text);

        parser.parse_args_or_exit();
    }

    if options.service {
        let _rocket = rocket::build()
            .mount("/", routes![mandelbrot_rest])
            .mount("/images", FileServer::from(relative!("images")))
            .launch().await?;
    } else {
        let mut buffer = vec![0; (options.width * options.height) as usize];

        generate(options, &mut buffer);
        //Create a blank image to write to
        let mut img: RgbImage = ImageBuffer::new(options.width, options.height);

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            //32 bit number but only storing rgb so split it into its 3 8 bit components
            let b = ((buffer[y as usize * options.width as usize + x as usize] & 0x00ff0000) >> 16)
                as u8;
            let g = ((buffer[y as usize * options.width as usize + x as usize] & 0x0000ff00) >> 8)
                as u8;
            let r = (buffer[y as usize * options.width as usize + x as usize] & 0x000000ff) as u8;
            *pixel = image::Rgb([r, g, b]);
        }

        img.save(&filename).unwrap_or_else(|_| {
            eprintln!("Error: Could not write file");
        });
    }
    return Ok(());
}
