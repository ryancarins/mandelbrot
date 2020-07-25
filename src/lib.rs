//Struct for storing arguments
pub struct Options {
    pub max_width: u32,
    pub max_height: u32,
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
    pub fn new(max_width: u32, max_height: u32, max_colours: u32, max_iter: u32, width: u32, height: u32, centrex: f32, centrey: f32, scaley: f32, samples: u32) -> Options {
        Options {
            max_width,
            max_height,
            max_colours,
            max_iter,
            width,
            height,
            centrex,
            centrey,
            scaley,
            samples
        }
    }
}
