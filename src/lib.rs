//! <div align="center">
//!
//! # Nexg
//!
//! ## **Low-level fast GPU Api**
//!
//! ![GitHub License](https://img.shields.io/github/license/Ichinose0/Nexg)
//! ![GitHub top language](https://img.shields.io/github/languages/top/Ichinose0/Gallium?logo=rust&logoColor=white&label=Rust&color=rgb(255%2C60%2C60))
//! [![dependency status](https://deps.rs/repo/github/linebender/vello/status.svg)](https://deps.rs/repo/github/Ichinose0/Nexg)
//! ![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Ichinose0/Nexg)
//! ![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Ichinose0/Nexg/rust.yml)
//! </div>
//!
//! # Set up Device
//!
//! ```
//! use nexg::{InstanceFeature,InstanceBuilder};
//!
//!  let feature = InstanceFeature::empty();
//!  let instance = InstanceBuilder::new().feature(feature).build().unwrap();
//!  let connecters = instance.enumerate_connecters().unwrap();
//!  let mut index = 0;
//!  let mut found_device = false;
//!  for i in &connecters {
//!     let properties = i.get_queue_family_properties(&instance).unwrap();
//!     for i in properties {
//!         if i.is_graphic_support() {
//!             index = 0;
//!             found_device = true;
//!             break;
//!         }
//!     }
//!  }
//!  if !found_device {
//!     panic!("No suitable device found.")
//!  }
//!
//!  let connecter = connecters[index];
//!
//!  let device = connecter.create_device(&instance, index).unwrap();
//! ```
//!
//! ## Examples
//!
//! ### Triangle
//! **[Code](https://github.com/Ichinose0/Gallium/blob/main/examples/triangle.rs)**
//!
//! ![triangle](https://github.com/Ichinose0/Nexg/blob/main/media/img/triangle.png?raw=true)

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
#[doc(hidden)]
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

use thiserror::Error;

pub type NxResult<T> = std::result::Result<T, NxError>;

#[derive(Debug, Error)]
pub enum NxError {
    /// Unknown error. Usually does not occur.
    #[error("Unknown error")]
    Unknown,
    #[error("Nothing of value could be obtained.")]
    NoValue,
    #[error("Device does not support this operation.")]
    HardwareError,
    #[error("Out of host memory")]
    OutOfHostMemory,
    #[error("Out of device memory")]
    OutOfDeviceMemory,
    #[error("Failed to map memory.")]
    MemoryMapFailed,
    #[error("`{0}`")]
    InternalError(#[from] ash::vk::Result),
    #[error("`{0}`")]
    IoError(String),
}

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

    /// Check to see if the graphic is supported
    pub fn is_graphic_support(&self) -> bool {
        self.graphic_support
    }

    /// Check to see if the compute is supported
    pub fn is_compute_support(&self) -> bool {
        self.compute_support
    }

    /// Check to see if the transfer is supported
    pub fn is_transfer_support(&self) -> bool {
        self.transfer_support
    }
}

/// Represents a handle to a physical device.
#[derive(Clone, Copy)]
pub struct DeviceConnecter(pub(crate) vk::PhysicalDevice);

impl DeviceConnecter {
    /// Create a device.
    pub fn create_device(self, instance: &Instance, queue_family_index: usize) -> NxResult<Device> {
        let extensions = &instance
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
            .enabled_extension_names(extensions)
            .build();
        instance.create_device(self, &create_info)
    }

    pub fn get_queue_family_properties(
        &self,
        instance: &Instance,
    ) -> NxResult<Vec<QueueFamilyProperties>> {
        instance.get_queue_family_properties(self.0)
    }

    #[doc(hidden)]
    pub(crate) fn get_memory_properties(
        &self,
        instance: &Instance,
    ) -> vk::PhysicalDeviceMemoryProperties {
        instance.get_memory_properties(self.0)
    }

    #[doc(hidden)]
    #[cfg(feature = "window")]
    pub(crate) fn is_support_swapchain(&self, instance: &Instance) -> bool {
        let features = unsafe {
            instance
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

/// Implement on objects that need to be destroyed.
/// They are called from the instance or from the destroy method of the device.
pub trait Destroy {
    /// Destroy objects using instance.
    fn instance(&self, instance: &Instance);
    /// Destroy objects using device.
    fn device(&self, device: &Device);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
