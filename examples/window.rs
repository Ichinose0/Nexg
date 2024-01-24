use simple_logger::SimpleLogger;
use fgl::{CommandPoolDescriptor, CommandRecorderDescriptor, Extent3d, Image, ImageDescriptor, InstanceBuilder, InstanceFeature, Surface, Swapchain};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().init().unwrap();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();

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

    let surface = Surface::new(&instance,&window);
    let swapchain = Swapchain::new(&surface,&instance,&device,connecter);

    let queue = device.get_queue(index);
    let desc = CommandPoolDescriptor::new().queue_family_index(index);
    let pool = device.create_command_pool(&desc);
    let desc = CommandRecorderDescriptor::new();
    let recorders = device.allocate_command_recorder(pool,&desc);
    let desc = ImageDescriptor::new().extent(Extent3d::new(640, 480, 1));
    let image = Image::create(&device,connecter,&desc);

    recorders[0].begin();
    recorders[0].end();
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
