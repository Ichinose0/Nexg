use std::{borrow::Cow, ffi::CStr};

use ash::vk::{self, DeviceCreateInfo};

mod device;
mod instance;

pub use device::*;
pub use instance::*;

pub struct QueueFamilyProperties {
    
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
        value.
        QueueFamilyProperties {

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
