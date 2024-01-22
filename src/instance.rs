use ash::extensions::ext::DebugUtils;
use ash::vk::{self, DebugUtilsMessengerEXT};
use ash::{vk::InstanceCreateInfo, Entry};

use crate::{vulkan_debug_callback, DeviceConnecter};

pub struct InstanceFeature {
    extensions: Vec<*const i8>
}

impl InstanceFeature {
    pub fn empty() -> Self {
        Self {
            extensions: vec![]
        }
    }

    #[cfg(feature = "window")]
    pub fn use_surface(&mut self,handle: &impl raw_window_handle::HasRawDisplayHandle) {
        let ext = ash_window::enumerate_required_extensions(handle.raw_display_handle()).unwrap();
        for i in ext {
            self.extensions.push(*i);
        }
    }
}

impl Default for InstanceFeature {
    fn default() -> Self {
        Self::empty()
    }
}

pub struct InstanceBuilder {
    feature: InstanceFeature,
}

impl InstanceBuilder {
    pub fn new() -> Self {
        Self {
            feature: Default::default()
        }
    }

    pub fn build(mut self) -> Instance {
        self.feature.extensions.push(DebugUtils::name().as_ptr());
        let entry = ash::Entry::linked();
        let create_info = InstanceCreateInfo::builder().enabled_extension_names(&self.feature.extensions).build();
        let instance = unsafe { entry.create_instance(&create_info, None) }.unwrap();
        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default();

        debug_info.message_severity = vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO;
        debug_info.message_type = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;

        debug_info.pfn_user_callback = Some(vulkan_debug_callback);

        let debug_utils = DebugUtils::new(&entry, &instance);
        let debug_call_back =
            unsafe { debug_utils.create_debug_utils_messenger(&debug_info, None) }.unwrap();
        Instance { instance, entry, debug_utils, debug_call_back }
    }
}

pub struct Instance {
    instance: ash::Instance,
    entry: Entry,

    debug_utils: DebugUtils,
    debug_call_back: DebugUtilsMessengerEXT
}

impl Instance {
    pub fn enumerate_connecters(&self) -> Vec<DeviceConnecter> {
        let devices = unsafe { self.instance.enumerate_physical_devices() }.unwrap();
        let devices = devices.iter().map(|x| {
            DeviceConnecter(*x)
        }).collect::<Vec<DeviceConnecter>>();
        devices
    }

    pub fn default_connector(&self) -> DeviceConnecter {
        let devices = self.enumerate_connecters();
        devices[0]
    }

    pub fn vulkan_version(&self) -> Option<String> {
        match self.entry.try_enumerate_instance_version() {
            Ok(v) => {
                match v {
                    Some(v) => {
                        let major = ash::vk::api_version_major(v);
                        let minor = ash::vk::api_version_minor(v);
                        let patch = ash::vk::api_version_patch(v);
                        Some(format!("{}.{}.{}",major,minor,patch))
                    }
                    None => return None
                }
            },
            Err(_) => return None
        }
    }
}
