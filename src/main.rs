use image::{ImageBuffer, RgbImage};
use mandelbrot::{Options, parse_cli};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn generate(options: Options, out: &mut Vec<u32>) {
    println!("{}", options);
    let start = Instant::now();
    let current_line = Arc::new(Mutex::new(0));
    let (tx, rx) = mpsc::channel();

    for i in 0..options.threads {
        let mut local_options = options.clone();
        local_options.thread_id = Some(i);
        let local_tx = mpsc::Sender::clone(&tx);
        let current_line_ref = Arc::clone(&current_line);
        thread::spawn(move || mandelbrot::mandelbrot(local_options, local_tx, current_line_ref));
    }

    //Drop tx because we only need it for cloning and if we don't drop it the loop below will never end
    drop(tx);

    for (i, val) in rx {
        out[i as usize] = val;
    }

    //mandelbrot::mandelbrot(options, out);
    println!("time taken: {}ms", start.elapsed().as_millis());
}

fn main() {
    //Handle command line arguments
    let options = parse_cli();

    let mut buffer = vec![0; (options.width * options.height) as usize];

    generate(options.clone(), &mut buffer);

    //Create a blank image to write to
    let mut img: RgbImage = ImageBuffer::new(options.width, options.height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        //32 bit number but only storing rgb so split it into its 3 8 bit components
        let r = (buffer[y as usize * options.width as usize + x as usize] & 0x000000ff) as u8;
        let g = ((buffer[y as usize * options.width as usize + x as usize] & 0x0000ff00) >> 8) as u8;
        let b = ((buffer[y as usize * options.width as usize + x as usize] & 0x00ff0000) >> 16) as u8;
        *pixel = image::Rgb([r, g, b]);
    }

    img.save(&options.file_name).unwrap_or_else(|err| {
        eprintln!("Error: Could not write file. Error: {}", err);
    });
}
