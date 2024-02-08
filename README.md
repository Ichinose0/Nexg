<div align="center">

# Nexg(Next GPU)

## **Low-level fast GPU Api**  

![GitHub License](https://img.shields.io/github/license/Ichinose0/Nexg)
![GitHub top language](https://img.shields.io/github/languages/top/Ichinose0/Gallium?logo=rust&logoColor=white&label=Rust&color=rgb(255%2C60%2C60))
[![dependency status](https://deps.rs/repo/github/linebender/vello/status.svg)](https://deps.rs/repo/github/Ichinose0/Nexg)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/nexg)](https://crates.io/crates/nexg)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Ichinose0/Nexg)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Ichinose0/Nexg/rust.yml)  
</div>

Nexg is a pure-rust library that makes coding Vulkan functionality easier and more Rust-like  

Nexg aims to support gaming applications as well as operation on GPUs  

# Examples
### Triangle
![triangle](media/img/triangle.png)
```rust
use std::ffi::c_void;
use std::mem::offset_of;
use std::{env, fs::File, io::BufWriter};

use nexg::{
    Buffer, BufferDescriptor, CommandPoolDescriptor, CommandRecorderDescriptor, DataFormat,
    Extent3d, FrameBuffer, FrameBufferDescriptor, Image, ImageDescriptor, ImageFormat,
    ImageViewDescriptor, InstanceBuilder, InstanceFeature, LoadOp, Pipeline, PipelineDescriptor,
    PipelineLayout, PipelineLayoutDescriptor, PipelineVertexInputDescriptor, QueueSubmitDescriptor,
    RenderPass, RenderPassBeginDescriptor, RenderPassDescriptor, Shader, ShaderStage,
    ShaderStageDescriptor, Spirv, StoreOp, SubPass, SubPassDescriptor,
    VertexInputAttributeDescriptor, VertexInputBindingDescriptor,
};
use png::text_metadata::ZTXtChunk;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vec4(f32, f32, f32, f32);
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pos: Vec4,
    color: Vec4,
}

const VERTEX: [Vertex; 3] = [
    Vertex {
        pos: Vec4(0.0, -0.5, 0.0, 0.0),
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
];

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
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
        &Spirv::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/shader/shader.vert.spv"
        ))
        .unwrap(),
    );

    let fragment = Shader::new(
        &device,
        &Spirv::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/shader/shader.frag.spv"
        ))
        .unwrap(),
    );

    let desc = BufferDescriptor::empty().size(std::mem::size_of::<Vertex>() * VERTEX.len());
    let vertex_buffer = Buffer::new(&instance, connecter, &device, &desc).unwrap();
    vertex_buffer.write(&device, VERTEX.as_ptr() as *const c_void);
    vertex_buffer.lock(&device);

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
        .clear(1.0, 1.0, 1.0, 1.0)
        .render_pass(&render_pass)
        .frame_buffer(&framebuffer);
    recorders[0].begin(&device, begin_desc);
    recorders[0].bind_pipeline(&device, &pipeline[0]);
    recorders[0].bind_vertex_buffer(&device, &vertex_buffer);
    recorders[0].draw(&device, 3, 1, 0, 0);
    recorders[0].end(&device);

    let desc = QueueSubmitDescriptor::empty();
    queue.submit(&device, &desc, &recorders);

    let file = File::create("triangle.png").unwrap();
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, WIDTH, HEIGHT);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    let slice = image.as_raw_data(&device, WIDTH, HEIGHT).unwrap();
    writer.write_image_data(&slice).unwrap(); // Save

    let tail_ztxt_chunk = ZTXtChunk::new(
        "Comment".to_string(),
        "A zTXt chunk after the image data.".to_string(),
    );
    writer.write_text_chunk(&tail_ztxt_chunk).unwrap();
}

```

# Notice
This API is not **an abstraction** to other graphics APIs (DirectX, Metal).  
It is an API that makes it easier to use and **optimize Vulkan's functionality**.

# Goals
 - Fast API with low overhead

# License
Nexg is licensed under MIT LICENSE
