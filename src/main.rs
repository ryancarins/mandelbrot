#[macro_use]
extern crate rocket;
use argparse::{ArgumentParser, Store, StoreTrue};
use image::{ImageBuffer, RgbImage};
use mandelbrot::Options;
use pbr::ProgressBar;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::{relative, FileServer};
use rocket::http::Header;
use rocket::{Request, Response};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const DEFAULT_FILENAME: &str = "output.bmp";

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

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
    } else if options.vulkan {
        println!(
            "Running vulkan version threads flag will be ignored and no progress bar can be shown"
        );
        mandelbrot::vulkan_mandelbrot(options, out);
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

#[get(
    "/?<max_iter>&<width>&<height>&<threads>&<ocl>&<samples>&<scale>&<x>&<y>&<colourise>&<vulkan>"
)]
fn mandelbrot_rest(
    max_iter: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    threads: Option<u32>,
    ocl: Option<bool>,
    vulkan: Option<bool>,
    colourise: Option<bool>,
    samples: Option<u32>,
    scale: Option<f64>,
    x: Option<f64>,
    y: Option<f64>,
) -> String {
    let mut options = Options::default();
    options.service = true;
    options.max_iter = max_iter.unwrap_or(options.max_iter);
    options.width = width.unwrap_or(options.width);
    options.height = height.unwrap_or(options.height);
    options.centrex = x.unwrap_or(options.centrex);
    options.centrey = y.unwrap_or(options.centrey);
    options.samples = samples.unwrap_or(options.samples);
    options.colourise = colourise.unwrap_or(options.colourise);
    options.threads = threads.unwrap_or(options.threads);
    options.ocl = ocl.unwrap_or(options.ocl);
    options.vulkan = vulkan.unwrap_or(options.vulkan);
    options.scaley = scale.unwrap_or(options.scaley);

    let filename = format!(
        "images/{}-{}-{}-{}-{}-{}-{}-{}-{}-{}.png",
        options.width,
        options.height,
        options.max_iter,
        options.max_colours,
        options.centrex,
        options.centrey,
        options.scaley,
        options.samples,
        options.colour,
        options.colourise
    );

    if Path::new(&filename).exists() {
        return filename;
    }

    let mut buffer = vec![0; (options.width * options.height) as usize];
    generate(options, &mut buffer);
    let mut img: RgbImage = ImageBuffer::new(options.width, options.height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        //32 bit number but only storing rgb so split it into its 3 8 bit components
        let b =
            ((buffer[y as usize * options.width as usize + x as usize] & 0x00ff0000) >> 16) as u8;
        let g =
            ((buffer[y as usize * options.width as usize + x as usize] & 0x0000ff00) >> 8) as u8;
        let r = (buffer[y as usize * options.width as usize + x as usize] & 0x000000ff) as u8;
        *pixel = image::Rgb([r, g, b]);
    }

    img.save(&filename).unwrap_or_else(|_| {
        eprintln!("Error: Could not write file");
    });

    filename
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let mut filename = std::string::String::from(DEFAULT_FILENAME);

    let mut options = Options::default();

    //Handle command line arguments
    {
        //Using variables here because I wanted to format and parser takes a &str
        let height_text = format!("Set height (default {})", options.height);
        let width_text = format!("Set width (default {})", options.width);
        let centrex_text = format!("Set centrex (default {})", options.centrex);
        let centrey_text = format!("Set centrey (default {})", options.centrey);
        let colourise_text = format!(
            "Use a different colour for each thread (default {})",
            options.colourise
        );
        let max_iter_text = format!(
            "Set maximum number of iterations (default {})",
            options.max_iter
        );
        let scaley_text = format!("Set scale(default {})", options.scaley);
        let samples_text = format!("Set samples for supersampling(default {})", options.samples);
        let colour_text = format!("Set colour for image(default {})", options.colour);
        let progress_text = format!("Display progress bar (default {})", options.progress);
        let ocl_text = format!("Use opencl instead of cpu (default {})", options.ocl);
        let vulkan_text = format!("Use vulkan instead of cpu (default {})", options.vulkan);
        let service_text = format!("Run as a REST service (default {})", options.service);
        let threads_text = format!(
            "Set number of threads to use for processing(default {})",
            options.threads
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
            .refer(&mut options.vulkan)
            .add_option(&["--vulkan"], StoreTrue, &vulkan_text);
        parser
            .refer(&mut options.service)
            .add_option(&["--service"], StoreTrue, &service_text);

        parser.parse_args_or_exit();
    }

    if options.service {
        let file_options = rocket::fs::Options::Index;
        let _rocket = rocket::build()
            .attach(CORS)
            .mount("/", routes![mandelbrot_rest])
            .mount(
                "/images",
                FileServer::new(relative!("images"), file_options),
            )
            .launch()
            .await?;
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
    Ok(())
}
