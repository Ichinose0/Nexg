use fgl::{
    CommandPoolDescriptor, CommandRecorderDescriptor, Extent3d, FrameBuffer, FrameBufferDescriptor,
    Image, ImageDescriptor, InstanceBuilder, InstanceFeature, Pipeline, PipelineDescriptor,
    PipelineLayout, PipelineLayoutDescriptor, RenderPass, RenderPassBeginDescriptor,
    RenderPassDescriptor, Shader, ShaderKind, Spirv, SubPass, SubPassDescriptor, Surface,
    Swapchain,
};
use simple_logger::SimpleLogger;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    SimpleLogger::new().init().unwrap();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();
    let size = window.inner_size();

    let mut feature = InstanceFeature::empty();
    feature.use_surface(&window);
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

    let surface = Surface::new(&instance, &window);
    let swapchain = Swapchain::new(&surface, &instance, &device, connecter);

    let queue = device.get_queue(index);
    let desc = CommandPoolDescriptor::new().queue_family_index(index);
    let pool = device.create_command_pool(&desc);
    let desc = CommandRecorderDescriptor::new();
    let recorders = device.allocate_command_recorder(pool, &desc);
    let desc = ImageDescriptor::new().extent(Extent3d::new(size.width, size.height, 1));
    let image = Image::create(&device, connecter, &desc);
    let image_view = image.create_image_view();

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
    let shaders = &[vertex, fragment];
    let desc = SubPassDescriptor::empty();
    let subpass = SubPass::new(connecter, &desc);
    let subpasses = &[subpass];
    let desc = RenderPassDescriptor::empty().subpasses(subpasses);
    let render_pass = RenderPass::new(&device, &desc);
    let desc = PipelineLayoutDescriptor::empty().render_pass(&render_pass);
    let pipeline_layout = PipelineLayout::new(&device, &desc);
    let desc = PipelineDescriptor::empty()
        .shaders(shaders)
        .width(size.width)
        .height(size.height);
    let pipeline = Pipeline::new(&device, pipeline_layout, &render_pass, &desc);

    let desc = FrameBufferDescriptor::empty()
        .render_pass(&render_pass)
        .image_view(&image_view)
        .width(size.width)
        .height(size.height);
    let framebuffer = FrameBuffer::new(&device, &desc);

    let begin_desc = RenderPassBeginDescriptor::empty()
        .width(size.width)
        .height(size.height)
        .render_pass(&render_pass)
        .frame_buffer(&framebuffer);
    recorders[0].begin(begin_desc);
    recorders[0].end();
    recorders[0].draw(&pipeline[0], 3, 1, 0, 0);
    queue.submit(&recorders);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
