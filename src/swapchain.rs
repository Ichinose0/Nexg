use ash::vk::{
    ImageUsageFlags, PresentInfoKHR, Semaphore, SharingMode, SwapchainCreateInfoKHR, SwapchainKHR,
};

use crate::{
    Device, DeviceConnecter, Fence, Image, ImageFormat, Instance, Queue, QueuePresentDescriptor,
    Surface,
};

pub struct Swapchain {
    swapchain: ash::extensions::khr::Swapchain,
    khr: SwapchainKHR,
    format: ImageFormat,
}

impl Swapchain {
    pub fn new(
        surface: &Surface,
        instance: &Instance,
        device: &Device,
        connecter: DeviceConnecter,
    ) -> Self {
        if !connecter.is_support_swapchain() {
            panic!("This DeviceConnecter does not support Swapchain");
        }

        let surface_capabilities = connecter.get_surface_capabilities(surface);
        let surface_formats = connecter.get_surface_formats(surface);
        let surface_present_modes = connecter.get_surface_present_modes(surface);

        let format = surface_formats[0];
        let present_mode = surface_present_modes[0];

        let create_info = SwapchainCreateInfoKHR::builder()
            .surface(surface.khr)
            .min_image_count(surface_capabilities.min_image_count + 1)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .present_mode(present_mode)
            .clipped(true)
            .build();
        let swapchain = ash::extensions::khr::Swapchain::new(&instance.instance, &device.device);
        let khr = unsafe { swapchain.create_swapchain(&create_info, None) }.unwrap();
        let format = format.format.into();
        Self {
            swapchain,
            khr,
            format,
        }
    }

    pub fn acquire_next_image(&self, semaphore: Option<&crate::Semaphore>) -> usize {
        let semaphore = match semaphore {
            None => Semaphore::null(),
            Some(x) => x.semaphore,
        };
        let next = unsafe {
            self.swapchain.acquire_next_image(
                self.khr,
                1000000000,
                semaphore,
                ash::vk::Fence::null(),
            )
        }
        .unwrap();
        next.0 as usize
    }

    pub fn format(&self) -> ImageFormat {
        self.format
    }

    pub fn present(&self, descriptor: &QueuePresentDescriptor, image: u32) {
        let w_semaphores: Vec<ash::vk::Semaphore> = descriptor
            .wait_semaphores
            .iter()
            .map(|x| x.semaphore)
            .collect();
        let mut present_info = PresentInfoKHR::builder()
            .swapchains(&[self.khr])
            .image_indices(&[image])
            .wait_semaphores(&w_semaphores)
            .build();

        unsafe {
            self.swapchain
                .queue_present(descriptor.queue.unwrap().0, &present_info);
        }
    }

    pub fn images(&self) -> Vec<Image> {
        let images = unsafe { self.swapchain.get_swapchain_images(self.khr).unwrap() };
        images
            .iter()
            .map(|x| Image::from_raw(*x))
            .collect::<Vec<Image>>()
    }
}
