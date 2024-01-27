use fgl::{
    CommandPoolDescriptor, CommandRecorderDescriptor, Extent3d, Fence, FenceDescriptor,
    FrameBuffer, FrameBufferDescriptor, Image, ImageDescriptor, ImageViewDescriptor,
    InstanceBuilder, InstanceFeature, Pipeline, PipelineDescriptor, PipelineLayout,
    PipelineLayoutDescriptor, QueuePresentDescriptor, QueueSubmitDescriptor, RenderPass,
    RenderPassBeginDescriptor, RenderPassDescriptor, Semaphore, SemaphoreDescriptor, Shader,
    ShaderKind, Spirv, SubPass, SubPassDescriptor, Surface, Swapchain,
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
    let desc = CommandPoolDescriptor::empty().queue_family_index(index);
    let pool = device.create_command_pool(&desc);
    let desc = CommandRecorderDescriptor::empty();
    let recorders = device.allocate_command_recorder(pool, &desc);
    let images = swapchain.images();
    let mut swapchain_images = vec![];
    let desc = ImageViewDescriptor::empty().format(swapchain.format());
    for i in images {
        swapchain_images.push(i.create_image_view(&device, &desc));
    }

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

    let mut frame_buffers = vec![];
    for i in &swapchain_images {
        let desc = FrameBufferDescriptor::empty()
            .render_pass(&render_pass)
            .width(size.width)
            .image_view(i)
            .height(size.height);
        frame_buffers.push(FrameBuffer::new(&device, &desc));
    }

    let desc = FenceDescriptor::empty().signaled(true);
    let image_rendered_fence = Fence::new(&device, &desc);

    let semaphore_desc = SemaphoreDescriptor::empty();

    let swapchain_image_semaphore = Semaphore::new(&device, &semaphore_desc);
    let image_rendered_semaphore = Semaphore::new(&device, &semaphore_desc);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::RedrawRequested(id) => {
                let (img, state) = swapchain.acquire_next_image(Some(&swapchain_image_semaphore));
                println!("{:?}", state);
                image_rendered_fence.wait(&device, u64::MAX);
                image_rendered_fence.reset(&device);
                let begin_desc = RenderPassBeginDescriptor::empty()
                    .width(size.width)
                    .height(size.height)
                    .clear(0.95, 0.95, 0.95, 1.0)
                    .render_pass(&render_pass)
                    .frame_buffer(&frame_buffers[img]);
                recorders[0].reset(&device);
                recorders[0].begin(&device, begin_desc);
                recorders[0].draw(&pipeline[0], &device, 3, 1, 0, 0);
                recorders[0].end(&device);

                let w_semaphores = &[swapchain_image_semaphore];
                let s_semaphores = &[image_rendered_semaphore];
                let desc = QueueSubmitDescriptor::empty()
                    .wait_semaphores(w_semaphores)
                    .signal_semaphores(s_semaphores)
                    .fence(&image_rendered_fence);

                queue.submit(&device, &desc, &recorders);

                let w_semaphores = &[image_rendered_semaphore];
                let desc = QueuePresentDescriptor::empty()
                    .wait_semaphores(w_semaphores)
                    .queue(&queue);

                swapchain.present(&desc, img as u32);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { .. },
                window_id,
            } if window_id == window.id() => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
