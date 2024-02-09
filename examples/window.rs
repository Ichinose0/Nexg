use nexg::{Buffer, BufferDescriptor, BufferUsage, CommandPoolDescriptor, CommandRecorderDescriptor, DataFormat, Fence, FenceDescriptor, FrameBuffer, FrameBufferDescriptor, ImageViewDescriptor, InstanceBuilder, InstanceFeature, LoadOp, Pipeline, PipelineDescriptor, PipelineLayout, PipelineLayoutDescriptor, PipelineVertexInputDescriptor, QueuePresentDescriptor, QueueSubmitDescriptor, RenderPass, RenderPassBeginDescriptor, RenderPassDescriptor, RequestConnecterDescriptor, Resource, ResourceBufferDescriptor, ResourceLayout, ResourceLayoutBinding, ResourcePool, ResourcePoolDescriptor, ResourcePoolSize, ResourceType, ResourceUpdateDescriptor, Semaphore, SemaphoreDescriptor, Shader, ShaderStage, ShaderStageDescriptor, Spirv, StoreOp, SubPass, SubPassDescriptor, Surface, Swapchain, VertexInputAttributeDescriptor, VertexInputBindingDescriptor};
use simple_logger::SimpleLogger;
use std::ffi::c_void;
use std::mem::offset_of;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use winit::event::StartCause;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vec4(f32, f32, f32, f32);
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pos: Vec4,
    color: Vec4,
}

struct SceneData {
    rect_center: Vec4,
}

const SCENE_DATA: SceneData = SceneData {
    rect_center: Vec4(0.3, -0.2, 0.0, 0.0),
};

const VERTEX_S: &'static [u8] = include_bytes!("shader/shader2.vert.spv");
const FRAGMENT_S: &'static [u8] = include_bytes!("shader/shader.frag.spv");

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const VERTEX: [Vertex; 4] = [
    Vertex {
        pos: Vec4(-0.5, -0.5, 0.0, 0.0),
        color: Vec4(0.0, 0.0, 1.0, 1.0),
    },
    Vertex {
        pos: Vec4(0.5, 0.5, 0.0, 0.0),
        color: Vec4(0.0, 1.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec4(-0.5, 0.5, 0.0, 0.0),
        color: Vec4(1.0, 0.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec4(0.5, -0.5, 0.0, 0.0),
        color: Vec4(1.0, 1.0, 1.0, 1.0),
    },
];

const INDICES: [u16; 6] = [0, 1, 2, 1, 0, 3];

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut scene_data = SceneData {
        rect_center: Vec4(0.3, -0.2, 0.0, 0.0),
    };

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(640, 480))
        .build(&event_loop)
        .unwrap();
    let size = window.inner_size();

    let mut feature = InstanceFeature::empty();
    feature.use_surface(&window).unwrap();
    let instance = InstanceBuilder::new().feature(feature).build().unwrap();
    let desc = RequestConnecterDescriptor::new()
        .graphic_support(true)
        .compute_support(true)
        .transfer_support(true);
    let connecters = instance.request_connecters(&[desc]).unwrap();
    let connecter = connecters[0];
    let index = connecter.get_queue_family_index();

    let device = connecter.create_device(&instance, index).unwrap();

    let surface = Surface::new(&instance, &window).unwrap();
    let swapchain = Swapchain::new(&surface, &instance, &device, connecter).unwrap();

    let queue = device.get_queue(index);
    let desc = CommandPoolDescriptor::empty().queue_family_index(index);
    let pool = device.create_command_pool(&desc).unwrap();
    let desc = CommandRecorderDescriptor::empty();
    let recorders = device.allocate_command_recorder(pool, &desc).unwrap();
    let images = swapchain.images().unwrap();
    let mut swapchain_images = vec![];
    let desc = ImageViewDescriptor::empty().format(swapchain.format());
    for i in &images {
        swapchain_images.push(i.create_image_view(&device, &desc));
    }

    let vertex = Shader::new(&device, &Spirv::from_raw(VERTEX_S).unwrap());

    let fragment = Shader::new(&device, &Spirv::from_raw(FRAGMENT_S).unwrap());


    let desc = BufferDescriptor::empty().size(std::mem::size_of::<Vertex>() * VERTEX.len());
    let vertex_buffer = Buffer::new(&instance, connecter, &device, &desc).unwrap();
    vertex_buffer.write(&device, VERTEX.as_ptr() as *const c_void);
    vertex_buffer.lock(&device);
    let desc = BufferDescriptor::empty()
        .size(std::mem::size_of::<u16>() * INDICES.len())
        .usage(BufferUsage::Index);
    let index_buffer = Buffer::new(&instance, connecter, &device, &desc).unwrap();
    index_buffer
        .write(&device, INDICES.as_ptr() as *const c_void)
        .unwrap();
    index_buffer.lock(&device);
    let desc = BufferDescriptor::empty()
        .size(std::mem::size_of::<SceneData>())
        .usage(BufferUsage::Uniform);
    let uniform_buffer = Buffer::new(&instance, connecter, &device, &desc).unwrap();
    uniform_buffer
        .write(&device, &SCENE_DATA as *const SceneData as *const c_void)
        .unwrap();
    uniform_buffer.lock(&device);

    let resource_layout_bindings = vec![ResourceLayoutBinding::empty()
        .binding(0)
        .resource_type(ResourceType::UniformBuffer)
        .count(1)
        .shader_stage(ShaderStage::Vertex)];
    let resource_layout = ResourceLayout::new(&device, &resource_layout_bindings);
    let pool_sizes = vec![ResourcePoolSize::empty()];
    let pool_desc = ResourcePoolDescriptor::empty()
        .pool_sizes(&pool_sizes)
        .max_sets(1);
    let resource_pool = ResourcePool::new(&device, &pool_desc);
    let resource = Resource::allocate(&device, &resource_pool, &resource_layout);

    let buffer_desc = ResourceBufferDescriptor::new::<SceneData>(&uniform_buffer);
    let desc = vec![buffer_desc];
    let update_desc = ResourceUpdateDescriptor::new(&resource[0]).buffer_desc(&desc);
    device.update_resource(&update_desc);

    let desc = SubPassDescriptor::empty();
    let subpass = SubPass::new(connecter, &desc);
    let subpasses = &[subpass];
    let desc = RenderPassDescriptor::empty()
        .subpasses(subpasses)
        .load_op(LoadOp::Clear)
        .store_op(StoreOp::Store);
    let render_pass = RenderPass::new(&device, &desc).unwrap();
    let desc = PipelineLayoutDescriptor::empty()
        .render_pass(&render_pass)
        .resource(&resource_layout);
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
        .width(size.width)
        .height(size.height);
    let pipeline = Pipeline::new(&device, pipeline_layout, &render_pass, &desc).unwrap();

    let mut frame_buffers = vec![];
    for i in &swapchain_images {
        let desc = FrameBufferDescriptor::empty()
            .render_pass(&render_pass)
            .width(size.width)
            .image_view(i)
            .height(size.height);
        frame_buffers.push(FrameBuffer::new(&device, &desc).unwrap());
    }

    let desc = FenceDescriptor::empty().signaled(true);
    let image_rendered_fence = Fence::new(&device, &desc).unwrap();

    let semaphore_desc = SemaphoreDescriptor::empty();

    let swapchain_image_semaphore = Semaphore::new(&device, &semaphore_desc).unwrap();
    let image_rendered_semaphore = Semaphore::new(&device, &semaphore_desc).unwrap();

    let mut time = 0.0;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::RedrawRequested(id) => {
                let (img, state) = swapchain
                    .acquire_next_image(Some(&swapchain_image_semaphore))
                    .unwrap();

                scene_data.rect_center = Vec4((0.3 * f64::cos(time)) as f32, (0.3 * f64::sin(time)) as f32, 0.0, 0.0);
                time += 0.001;

                uniform_buffer.write(&device,&scene_data as *const SceneData as *const c_void).unwrap();


                image_rendered_fence.wait(&device, u64::MAX);
                image_rendered_fence.reset(&device);




                let begin_desc = RenderPassBeginDescriptor::empty()
                    .width(size.width)
                    .height(size.height)
                    .clear(0.0,0.0,0.0, 1.0)
                    .render_pass(&render_pass)
                    .frame_buffer(&frame_buffers[img]);
                recorders[0].reset(&device);
                recorders[0].begin(&device, begin_desc);
                recorders[0].bind_pipeline(&device, &pipeline[0]);
                recorders[0].bind_vertex_buffer(&device, &vertex_buffer);
                recorders[0].bind_index_buffer(&device, &index_buffer);
                recorders[0].bind_resource(&device, &resource[0], &pipeline_layout);
                recorders[0].draw_indexed(&device, INDICES.len() as u32, 1, 0, 0, 0);
                recorders[0].end(&device).unwrap();

                let w_semaphores = &[swapchain_image_semaphore];
                let s_semaphores = &[image_rendered_semaphore];
                let desc = QueueSubmitDescriptor::empty()
                    .wait_semaphores(w_semaphores)
                    .signal_semaphores(s_semaphores)
                    .fence(&image_rendered_fence);

                queue.submit(&device, &desc, &recorders).unwrap();

                let w_semaphores = &[image_rendered_semaphore];
                let desc = QueuePresentDescriptor::empty()
                    .wait_semaphores(w_semaphores)
                    .queue(&queue);

                swapchain.present(&desc, img as u32);
            }
            _ => (),
        }
    });
}
