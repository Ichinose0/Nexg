use std::{env, fs::File, io::BufWriter};
use std::ffi::c_void;

use fgl::{Buffer, BufferDescriptor, CommandPoolDescriptor, CommandRecorderDescriptor, Extent3d, FrameBuffer, FrameBufferDescriptor, Image, ImageDescriptor, ImageFormat, ImageViewDescriptor, InstanceBuilder, InstanceFeature, LoadOp, Pipeline, PipelineDescriptor, PipelineLayout, PipelineLayoutDescriptor, QueueSubmitDescriptor, RenderPass, RenderPassBeginDescriptor, RenderPassDescriptor, Shader, ShaderKind, Spirv, StoreOp, SubPass, SubPassDescriptor, Surface, Swapchain};
use png::text_metadata::ZTXtChunk;
use simple_logger::SimpleLogger;
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const VERTEX: [Vertex; 3] = [
    Vertex {
        x: 0.0,
        y: -0.5,
        z: 0.0,
        w: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.0,
        w: 0.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.0,
        w: 0.0,
    },
];

fn main() {
    SimpleLogger::new().init().unwrap();
    let mut feature = InstanceFeature::empty();
    let instance = InstanceBuilder::new().feature(feature).build();
    let connecters = instance.enumerate_connecters();
    let mut index = 0;
    let mut found_device = false;
    for i in &connecters {
        let properties = i.get_queue_family_properties();
        for i in properties {
            if i.is_graphic_support() {
                index = 0;
                found_device = true;
                break;
            }
        }
    }
    if !found_device {
        panic!("No suitable device found.")
    }

    let connecter = connecters[index];

    let device = connecter.create_device(index);

    let queue = device.get_queue(index);
    let desc = CommandPoolDescriptor::empty().queue_family_index(index);
    let pool = device.create_command_pool(&desc);
    let desc = CommandRecorderDescriptor::empty();
    let recorders = device.allocate_command_recorder(pool, &desc);
    let desc = ImageDescriptor::new().extent(Extent3d::new(WIDTH, HEIGHT, 1));
    let image = Image::create(&device, connecter, &desc);
    let desc = ImageViewDescriptor::empty().format(ImageFormat::R8G8B8A8Unorm);
    let image_view = image.create_image_view(&device, &desc);

    let vertex = Shader::new(
        &device,
        Spirv::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/shader/shader.vert.spv"
        )),
        ShaderKind::Vertex,
    );

    let fragment = Shader::new(
        &device,
        Spirv::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/shader/shader.frag.spv"
        )),
        ShaderKind::Fragment,
    );

    let desc = BufferDescriptor::empty().size(std::mem::size_of::<Vertex>()*VERTEX.len());
    let vertex_buffer = Buffer::new(&instance,connecter,&device,&desc);
    vertex_buffer.write(&device,VERTEX.as_ptr() as *const c_void);
    vertex_buffer.lock(&device);

    let shaders = &[vertex, fragment];
    let desc = SubPassDescriptor::empty();
    let subpass = SubPass::new(connecter, &desc);
    let subpasses = &[subpass];
    let desc = RenderPassDescriptor::empty()
        .subpasses(subpasses)
        .load_op(LoadOp::Clear)
        .store_op(StoreOp::Store);
    let render_pass = RenderPass::new(&device, &desc);
    let desc = PipelineLayoutDescriptor::empty().render_pass(&render_pass);
    let pipeline_layout = PipelineLayout::new(&device, &desc);
    let desc = PipelineDescriptor::empty()
        .shaders(shaders)
        .width(WIDTH)
        .height(HEIGHT);
    let pipeline = Pipeline::new(&device, pipeline_layout, &render_pass, &desc);

    let desc = FrameBufferDescriptor::empty()
        .render_pass(&render_pass)
        .image_view(&image_view)
        .width(WIDTH)
        .height(HEIGHT);
    let framebuffer = FrameBuffer::new(&device, &desc);

    let begin_desc = RenderPassBeginDescriptor::empty()
        .width(WIDTH)
        .height(HEIGHT)
        .clear(1.0, 1.0, 1.0, 1.0)
        .render_pass(&render_pass)
        .frame_buffer(&framebuffer);
    recorders[0].begin(&device, begin_desc);
    recorders[0].draw(&pipeline[0], &device, 3, 1, 0, 0);
    recorders[0].end(&device);

    let desc = QueueSubmitDescriptor::empty();
    queue.submit(&device, &desc, &recorders);

    let file = File::create("triangle.png").unwrap();
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, WIDTH, HEIGHT); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    // Adding text chunks to the header
    encoder
        .add_text_chunk(
            "Testing tEXt".to_string(),
            "This is a tEXt chunk that will appear before the IDAT chunks.".to_string(),
        )
        .unwrap();
    encoder
        .add_ztxt_chunk(
            "Testing zTXt".to_string(),
            "This is a zTXt chunk that is compressed in the png file.".to_string(),
        )
        .unwrap();
    encoder
        .add_itxt_chunk(
            "Testing iTXt".to_string(),
            "iTXt chunks support all of UTF8. Example: हिंदी.".to_string(),
        )
        .unwrap();

    let mut writer = encoder.write_header().unwrap();
    let data = image.map_memory(&device);
    let slice: &[u8] =
        unsafe { std::slice::from_raw_parts(data as *const u8, (WIDTH * HEIGHT * 4) as usize) };
    writer.write_image_data(&slice).unwrap(); // Save

    // We can add a tEXt/zTXt/iTXt at any point before the encoder is dropped from scope. These chunks will be at the end of the png file.
    let tail_ztxt_chunk = ZTXtChunk::new(
        "Comment".to_string(),
        "A zTXt chunk after the image data.".to_string(),
    );
    writer.write_text_chunk(&tail_ztxt_chunk).unwrap();
}
