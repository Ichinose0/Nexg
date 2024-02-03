extern crate nalgebra_glm as glm;

use std::ffi::c_void;
use std::mem::offset_of;
use std::{env, fs::File, io::BufWriter};

use nexg::{Buffer, BufferDescriptor, BufferUsage, CommandPoolDescriptor, CommandRecorderDescriptor, DataFormat, Extent3d, FrameBuffer, FrameBufferDescriptor, Image, ImageDescriptor, ImageFormat, ImageViewDescriptor, InstanceBuilder, InstanceFeature, LoadOp, Pipeline, PipelineDescriptor, PipelineLayout, PipelineLayoutDescriptor, PipelineVertexInputDescriptor, QueueSubmitDescriptor, RenderPass, RenderPassBeginDescriptor, RenderPassDescriptor, Shader, ShaderKind, ShaderStage, ShaderStageDescriptor, Spirv, StoreOp, SubPass, SubPassDescriptor, VertexInputAttributeDescriptor, VertexInputBindingDescriptor};
use png::text_metadata::ZTXtChunk;
use simple_logger::SimpleLogger;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vec4(f32, f32, f32, f32);
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pos: Vec4,
    color: Vec4,
}

pub struct Ubo {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4
}

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const VERTEX: [Vertex; 4] = [
    Vertex {
        pos: Vec4(-0.5, -0.5, 0.0, 0.0),
        color: Vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec4(0.5, 0.5, 0.0, 0.0),
        color: Vec4(0.0, 1.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec4(-0.5, 0.5, 0.0, 0.0),
        color: Vec4(0.0, 0.0, 1.0, 1.0),
    },
    Vertex {
        pos: Vec4(0.5, -0.5, 0.0, 0.0),
        color: Vec4(1.0, 1.0, 1.0, 1.0),
    },
];

const INDICES: [u16;6] = [0,1,2,1,0,3];

fn main() {
    SimpleLogger::new().init().unwrap();
    let feature = InstanceFeature::empty();
    let instance = InstanceBuilder::new().feature(feature).build().unwrap();
    let connecters = instance.enumerate_connecters().unwrap();
    let mut index = 0;
    let mut found_device = false;
    for i in &connecters {
        let properties = i.get_queue_family_properties(&instance).unwrap();
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

    let device = connecter.create_device(&instance, index).unwrap();

    let queue = device.get_queue(index);
    let desc = CommandPoolDescriptor::empty().queue_family_index(index);
    let pool = device.create_command_pool(&desc).unwrap();
    let desc = CommandRecorderDescriptor::empty();
    let recorders = device.allocate_command_recorder(pool, &desc).unwrap();
    let desc = ImageDescriptor::new().extent(Extent3d::new(WIDTH, HEIGHT, 1));
    let image = Image::create(&instance, &device, connecter, &desc).unwrap();
    let desc = ImageViewDescriptor::empty().format(ImageFormat::R8G8B8A8Unorm);
    let image_view = image.create_image_view(&device, &desc);

    let vertex = Shader::new(
        &device,
        Spirv::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/shader/shader2.vert.spv"
        ))
            .unwrap(),
    );

    let fragment = Shader::new(
        &device,
        Spirv::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/shader/shader.frag.spv"
        ))
            .unwrap(),
    );

    let desc = BufferDescriptor::empty().size(std::mem::size_of::<Vertex>() * VERTEX.len());
    let vertex_buffer = Buffer::new(&instance, connecter, &device, &desc).unwrap();
    vertex_buffer.write(&device, VERTEX.as_ptr() as *const c_void);
    vertex_buffer.lock(&device);
    let desc = BufferDescriptor::empty().size(std::mem::size_of::<u16>() * INDICES.len()).usage(BufferUsage::Index);
    let index_buffer = Buffer::new(&instance, connecter, &device, &desc).unwrap();
    index_buffer.write(&device, INDICES.as_ptr() as *const c_void);
    index_buffer.lock(&device);

    let desc = SubPassDescriptor::empty();
    let subpass = SubPass::new(connecter, &desc);
    let subpasses = &[subpass];
    let desc = RenderPassDescriptor::empty()
        .subpasses(subpasses)
        .load_op(LoadOp::Clear)
        .store_op(StoreOp::Store);
    let render_pass = RenderPass::new(&device, &desc).unwrap();
    let desc = PipelineLayoutDescriptor::empty().render_pass(&render_pass);
    let pipeline_layout = PipelineLayout::new(&device, &desc).unwrap();
    let shader_stages = vec![
        ShaderStageDescriptor::empty()
            .entry_point("main")
            .stage(ShaderStage::Vertex)
            .shaders(&vertex),
        ShaderStageDescriptor::empty()
            .entry_point("main")
            .stage(ShaderStage::Fragment)
            .shaders(&fragment),
    ];
    let binding_desc = vec![VertexInputBindingDescriptor::empty()
        .binding(0)
        .stride(std::mem::size_of::<Vertex>())];
    let attribute_desc = vec![
        VertexInputAttributeDescriptor::empty()
            .binding(0)
            .location(0)
            .format(DataFormat::R32G32SFloat)
            .offset(offset_of!(Vertex, pos)),
        VertexInputAttributeDescriptor::empty()
            .binding(0)
            .location(1)
            .format(DataFormat::R32G32B32SFloat)
            .offset(offset_of!(Vertex, color)),
    ];
    let vertex_input_desc = PipelineVertexInputDescriptor::empty()
        .attribute_desc(&attribute_desc)
        .binding_desc(&binding_desc);
    let desc = PipelineDescriptor::empty()
        .shader_stages(&shader_stages)
        .input_descriptor(&vertex_input_desc)
        .width(WIDTH)
        .height(HEIGHT);
    let pipeline = Pipeline::new(&device, pipeline_layout, &render_pass, &desc).unwrap();

    let desc = FrameBufferDescriptor::empty()
        .render_pass(&render_pass)
        .image_view(&image_view)
        .width(WIDTH)
        .height(HEIGHT);
    let framebuffer = FrameBuffer::new(&device, &desc).unwrap();

    let begin_desc = RenderPassBeginDescriptor::empty()
        .width(WIDTH)
        .height(HEIGHT)
        .clear(0.88, 0.88, 0.88, 1.0)
        .render_pass(&render_pass)
        .frame_buffer(&framebuffer);
    recorders[0].begin(&device, begin_desc);
    recorders[0].bind_pipeline(&device, &pipeline[0]);
    recorders[0].bind_vertex_buffer(&device, &vertex_buffer);
    recorders[0].bind_index_buffer(&device,&index_buffer);
    recorders[0].draw_indexed(&device,INDICES.len() as u32, 1, 0, 0, 0);
    recorders[0].end(&device);

    let desc = QueueSubmitDescriptor::empty();
    queue.submit(&device, &desc, &recorders);

    let file = File::create("descriptor.png").unwrap();
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
    let data = image.map_memory(&device).unwrap();
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
