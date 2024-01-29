#[macro_use]
extern crate log;

use std::{borrow::Cow, ffi::CStr};

use ash::vk::{
    self, DebugUtilsMessageSeverityFlagsEXT, DeviceCreateInfo, DeviceQueueCreateInfo, QueueFlags,
};

mod buffer;
mod device;
mod fence;
mod frame_buffer;
mod image;
mod instance;
mod mem;
mod pipeline;
mod queue;
mod recorder;
mod renderpass;
mod shader;
#[cfg(feature = "window")]
mod surface;
#[cfg(feature = "window")]
mod swapchain;
mod sync;

pub use buffer::*;
pub use device::*;
pub use fence::*;
pub use frame_buffer::*;
pub use image::*;
pub use instance::*;
pub(crate) use mem::*;
pub use pipeline::*;
pub use queue::*;
pub use recorder::*;
pub use renderpass::*;
pub use shader::*;
#[cfg(feature = "window")]
pub use surface::*;
#[cfg(feature = "window")]
pub use swapchain::*;
pub use sync::*;

pub struct QueueFamilyProperties {
    graphic_support: bool,
    compute_support: bool,
    transfer_support: bool,
    queue_count: u32,
}

impl QueueFamilyProperties {
    pub fn count(&self) -> u32 {
        self.queue_count
    }

    pub fn is_graphic_support(&self) -> bool {
        self.graphic_support
    }

    pub fn is_compute_support(&self) -> bool {
        self.compute_support
    }

    pub fn is_transfer_support(&self) -> bool {
        self.transfer_support
    }
}

#[derive(Clone, Copy)]
pub struct DeviceConnecter<'a>(pub(crate) vk::PhysicalDevice, &'a Instance);

impl<'a> DeviceConnecter<'a> {
    pub fn create_device(self, queue_family_index: usize) -> Device {
        let extensions = &self
            .1
            .device_exts
            .iter()
            .map(|x| match x {
                DeviceFeature::Swapchain => ash::extensions::khr::Swapchain::name().as_ptr(),
            })
            .collect::<Vec<*const i8>>();
        let queue_infos = vec![DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index as u32)
            .queue_priorities(&[1.0])
            .build()];
        let create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&extensions)
            .build();
        let device = self.1.create_device(self, &create_info);
        device
    }

    pub fn get_queue_family_properties(&self) -> Vec<QueueFamilyProperties> {
        self.1.get_queue_family_properties(self.0)
    }

    #[doc(hidden)]
    pub(crate) fn get_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        self.1.get_memory_properties(self.0)
    }

    #[doc(hidden)]
    #[cfg(feature = "window")]
    pub(crate) fn is_support_swapchain(&self) -> bool {
        let features = unsafe {
            self.1
                .instance
                .enumerate_device_extension_properties(self.0)
        }
        .unwrap();
        let mut support = false;
        for i in features {
            let name = unsafe { CStr::from_ptr(i.extension_name.as_ptr()) };
            if name == ash::extensions::khr::Swapchain::name() {
                support = true;
            }
        }
        support
    }

    #[doc(hidden)]
    #[cfg(feature = "window")]
    pub(crate) fn get_surface_capabilities(&self, surface: &Surface) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            surface
                .surface
                .get_physical_device_surface_capabilities(self.0, surface.khr)
                .unwrap()
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "window")]
    pub(crate) fn get_surface_formats(&self, surface: &Surface) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            surface
                .surface
                .get_physical_device_surface_formats(self.0, surface.khr)
                .unwrap()
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "window")]
    pub(crate) fn get_surface_present_modes(&self, surface: &Surface) -> Vec<vk::PresentModeKHR> {
        unsafe {
            surface
                .surface
                .get_physical_device_surface_present_modes(self.0, surface.khr)
                .unwrap()
        }
    }
}

impl From<vk::QueueFamilyProperties> for QueueFamilyProperties {
    fn from(value: vk::QueueFamilyProperties) -> Self {
        let graphic_support = value.queue_flags.contains(QueueFlags::GRAPHICS);
        let compute_support = value.queue_flags.contains(QueueFlags::COMPUTE);
        let transfer_support = value.queue_flags.contains(QueueFlags::TRANSFER);
        QueueFamilyProperties {
            graphic_support,
            compute_support,
            transfer_support,
            queue_count: value.queue_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Extent3d {
    width: u32,
    height: u32,
    depth: u32,
}

impl Extent3d {
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }
}

impl Into<vk::Extent3D> for Extent3d {
    fn into(self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }
}

#[doc(hidden)]
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let _message_id_number = callback_data.message_id_number;

    let _message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::INFO => info!("[Vulkan] {}", message),
        DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("[Vulkan] {}", message),
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => trace!("[Vulkan] {}", message),
        DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("[Vulkan] {}", message),
        _ => todo!(),
    }

    vk::FALSE
}

pub trait Destroy {
    fn instance(&self, instance: &Instance);
    fn device(&self, device: &Device);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
