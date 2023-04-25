use bytemuck::{Pod, Zeroable};
use ocl::ProQue;
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tracing::{span, Level};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::GenericMemoryAllocator;
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, MemoryAllocatePreference, MemoryUsage,
};
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::sync::GpuFuture;
use vulkano::VulkanLibrary;

//Struct for storing arguments
#[derive(Copy, Clone, Debug)]
pub struct Options {
    pub max_colours: u32,
    pub max_iter: u32,

    pub width: u32,
    pub height: u32,
    pub centrex: f64,
    pub centrey: f64,
    pub scaley: f64,

    pub samples: u32,
    pub colour: u32,
    pub colourise: bool,
    pub threads: u32,
    pub thread_id: Option<u32>,
    pub progress: bool,
    pub ocl: bool,
    pub vulkan: bool,
    pub service: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub struct VulkanOpts {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub iterations: u32,
    pub scaley: f64,
    pub centrex: f64,
    pub centrey: f64,
}

impl Options {
    pub fn new(
        max_colours: u32,
        max_iter: u32,
        width: u32,
        height: u32,
        centrex: f64,
        centrey: f64,
        scaley: f64,
        samples: u32,
        colour: u32,
        colourise: bool,
        threads: u32,
        progress: bool,
        ocl: bool,
        vulkan: bool,
        service: bool,
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
            ocl,
            vulkan,
            service,
        }
    }

    pub fn as_vulkan_opts(&self) -> VulkanOpts {
        VulkanOpts {
            width: self.width,
            height: self.height,
            samples: self.samples,
            iterations: self.max_iter,
            scaley: self.scaley,
            centrex: self.centrex,
            centrey: self.centrey,
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

#[inline(always)]
fn iterations2colour(options: &Options, iter: u32, max_iter: u32, flags: u32) -> u32 {
    let span = span!(Level::TRACE, "iterations2colour");
    let _enter = span.enter();
    let iter = (iter * options.max_colours / max_iter) & (options.max_colours - 1);
    (((flags & 4) << 14) | ((flags & 2) << 7) | (flags & 1)) * iter
}

fn interlocked_increment(shared: Arc<Mutex<u32>>) -> u32 {
    let mut current = shared.lock().unwrap();
    let temp = *current;
    *current += 1;
    temp
}

pub fn mandelbrot(options: Options, sender: Sender<(u32, u32)>, current_line: Arc<Mutex<u32>>) {
    let span = span!(Level::TRACE, "mandelbrot");
    let _enter = span.enter();
    let scalex: f64 = options.scaley * options.width as f64 / options.height as f64;
    let colour = if options.colourise {
        options.thread_id.unwrap() % 7 + 1
    } else {
        options.colour
    };

    let dx: f64 = scalex / options.width as f64 / options.samples as f64;
    let dy: f64 = options.scaley / options.height as f64 / options.samples as f64;

    let startx = options.centrex - scalex * 0.5;
    let starty = options.centrey - options.scaley * 0.5;
    let mut iy = interlocked_increment(current_line.clone());

    while iy < options.height {
        for ix in 0..options.width {
            let mut totaliter: u32 = 0;

            for itery in 0..options.samples {
                for iterx in 0..options.samples {
                    let mut iter: u32 = 0;

                    let x0: f64 = startx + (ix as f64 * options.samples as f64 + iterx as f64) * dx;
                    let y0: f64 = starty + (iy as f64 * options.samples as f64 + itery as f64) * dy;
                    let mut x: f64 = x0;
                    let mut y: f64 = y0;
                    let mut xtemp: f64;
                    while x * x + y * y < (2 * 2) as f64 && iter <= options.max_iter {
                        xtemp = x * x - y * y + x0;

                        y = 2.0 * x * y + y0;
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

pub fn opencl_mandelbrot(options: Options, vec: &mut Vec<u32>) -> ocl::Result<()> {
    let span = span!(Level::TRACE, "opencl");
    let _enter = span.enter();
    let src = r#"#define MAX_COLOURS 256

inline unsigned int iterations2colour(unsigned int iter, unsigned int max_iter, unsigned int flags)
{
    iter = (iter * MAX_COLOURS / max_iter) & (MAX_COLOURS - 1);

    return (((flags & 4) << 14) | ((flags & 2) << 7) | (flags & 1)) * iter;
}

__kernel void mandelbrot(unsigned int iterations, double centrex, double centrey, double scaley, unsigned int samples, __global unsigned int* out)
{
    unsigned int width = get_global_size(1);
    unsigned int height = get_global_size(0);

    double scalex = scaley * width / height;

    double dx = scalex / width / samples;
    double dy = scaley / height / samples;

    double startx = centrex - scalex * 0.5f;
    double starty = centrey - scaley * 0.5f;

    unsigned int ix = get_global_id(1);
    unsigned int iy = get_global_id(0);
    int totalCalc = 0;

    for (unsigned int aay = 0; aay < samples; aay++)
    {
        for (unsigned int aax = 0; aax < samples; aax++)
        {
            unsigned int iter = 0;

            double x0 = startx + (ix * samples + aax) * dx;
            double y0 = starty + (iy * samples + aay) * dy;

            double x = x0;
            double y = y0;

            while (x * x + y * y < (2 * 2) && iter <= iterations)
            {
                double xtemp = x * x - y * y + x0;

                y = 2 * x * y + y0;
                x = xtemp;
                iter += 1;
            }

            if (iter <= iterations) totalCalc += iter;
        }
    }

    out[iy * width + ix] = iterations2colour(totalCalc / (samples * samples), iterations, 7);
}"#;

    let pro_que = ProQue::builder()
        .src(src)
        .dims((options.width, options.height))
        .build()?;

    let buffer = pro_que.create_buffer::<u32>()?;

    let kernel = pro_que
        .kernel_builder("mandelbrot")
        .arg(options.max_iter)
        .arg(options.centrex)
        .arg(options.centrey)
        .arg(options.scaley)
        .arg(options.samples)
        .arg(&buffer)
        .build()?;

    unsafe {
        kernel.enq()?;
    }

    buffer.read(vec).enq()?;

    //println!("The value at index [{}] is now '{}'!", 200007, vec[200007]);
    Ok(())
}

mod vulkan_mandelbrot {
    vulkano_shaders::shader! {
            ty: "compute",
            src: "

#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

uint MAX_COLOURS = 256;
uint iterations2colour(uint iter, uint max_iter, uint flags)
{
    iter = (iter * MAX_COLOURS / max_iter) & (MAX_COLOURS - 1);

    return (((flags & 4) << 14) | ((flags & 2) << 7) | (flags & 1)) * iter;
}

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

layout(set = 1, binding = 0) buffer Opts {
    uint width;
    uint height;
    uint samples;
    uint iterations;
    double scaley;
    double centrex;
    double centrey;
} opts;

void main() {
    uint ix = gl_GlobalInvocationID.x;
    uint iy = gl_GlobalInvocationID.y;
    double scalex = opts.scaley * opts.width / opts.height;

    double dx = scalex / opts.width / opts.samples;
    double dy = opts.scaley / opts.height / opts.samples;

    double startx = opts.centrex - scalex * 0.5f;
    double starty = opts.centrey - opts.scaley * 0.5f;
    int totalCalc = 0;
 
    for (uint aay = 0; aay < opts.samples; aay++)
    {
        for (uint aax = 0; aax < opts.samples; aax++)
        {
            uint iter = 0;

            double x0 = startx + (ix * opts.samples + aax) * dx;
            double y0 = starty + (iy * opts.samples + aay) * dy;

            double x = x0;
            double y = y0;

            while (x * x + y * y < (2 * 2) && iter <= opts.iterations)
            {
                double xtemp = x * x - y * y + x0;

                y = 2 * x * y + y0;
                x = xtemp;
                iter += 1;
            }

            if (iter <= opts.iterations) totalCalc += int(iter);
        }
    }

    buf.data[iy * opts.width + ix] = iterations2colour(totalCalc / (opts.samples * opts.samples), opts.iterations, 7);
}
"
    }
}

pub fn vulkan_mandelbrot(options: Options, vec: &mut [u32]) {
    let span = span!(Level::TRACE, "vulkan");
    let _enter = span.enter();
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance =
        Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");
    let physical = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");

    let valid_queues: Vec<(usize, &vulkano::device::QueueFamilyProperties)> = physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .filter(|x| {
            x.1.queue_flags.contains(QueueFlags::COMPUTE)
                && x.1.queue_flags.contains(QueueFlags::TRANSFER)
        })
        .collect();

    let mut max_queue_count = 0;
    let mut best_index = None;

    for (idx, properties) in valid_queues {
        if properties.queue_count > max_queue_count {
            max_queue_count = properties.queue_count;
            best_index = Some(idx);
        }
    }

    if best_index.is_none() {
        println!("GPU has no valid queues, exiting..");
        return;
    }
    let queue_family_index = best_index.unwrap() as u32;

    println!(
        "{:?}",
        physical.queue_family_properties()[best_index.unwrap()]
    );

    let f64_features = Features {
        shader_float64: true,
        ..Features::empty()
    };

    if physical.supported_features().contains(&f64_features) {
        println!("Supports f64");
    } else {
        println!("GPU doesn't support f64 exiting");
        return;
    }

    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            enabled_features: f64_features,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    let data = 0..(options.width * options.height);

    let allocator = GenericMemoryAllocator::<Arc<FreeListAllocator>>::new_default(device.clone());
    let data_buffer = Buffer::from_iter(
        &allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Download,
            allocate_preference: MemoryAllocatePreference::AlwaysAllocate,
            ..Default::default()
        },
        data,
    )
    .unwrap();

    let opts = options.as_vulkan_opts();
    let opts_buffer = Buffer::from_data(
        &allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            allocate_preference: MemoryAllocatePreference::AlwaysAllocate,
            ..Default::default()
        },
        opts,
    )
    .unwrap();

    let shader = vulkan_mandelbrot::load(device.clone()).expect("failed to create shader module");

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    )
    .expect("failed to create compute pipeline");

    let descriptor_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
    let data_set = PersistentDescriptorSet::new(
        &descriptor_allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())],
    )
    .unwrap();

    let layout = compute_pipeline.layout().set_layouts().get(1).unwrap();
    let opts_set = PersistentDescriptorSet::new(
        &descriptor_allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, opts_buffer)],
    )
    .unwrap();

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0,
            data_set,
        )
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            1,
            opts_set,
        )
        .dispatch([options.width / 8, options.height / 8, 1])
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = vulkano::sync::now(device)
        .then_execute(queue, command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let content = data_buffer.read().unwrap();
    for (i, val) in content.iter().enumerate() {
        vec[i] = *val;
    }
}
