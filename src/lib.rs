use std::{borrow::Cow, ffi::CStr};

use ash::vk::{self, DeviceCreateInfo, QueueFlags};

mod device;
mod instance;
mod queue;

pub use device::*;
pub use instance::*;
pub use queue::*;

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
pub struct DeviceConnecter<'a>(pub(crate) ash::vk::PhysicalDevice,&'a Instance);

impl<'a> DeviceConnecter<'a> {
    pub fn create_device(&self) -> Device {
        let create_info = DeviceCreateInfo::builder().build();
        let device = self.1.create_device(self.0, &create_info);
        device
    }

    pub fn get_queue_family_properties(&self) -> Vec<QueueFamilyProperties> {
        self.1.get_queue_family_properties(self.0)
    }
}

impl From<ash::vk::QueueFamilyProperties> for QueueFamilyProperties {
    fn from(value: ash::vk::QueueFamilyProperties) -> Self {
        let graphic_support = value.queue_flags.contains(QueueFlags::GRAPHICS);
        let compute_support = value.queue_flags.contains(QueueFlags::COMPUTE);
        let transfer_support =  value.queue_flags.contains(QueueFlags::TRANSFER);
        QueueFamilyProperties {
            graphic_support,
            compute_support,
            transfer_support,
            queue_count: value.queue_count
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
