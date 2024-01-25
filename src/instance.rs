use ash::extensions::ext::DebugUtils;
use ash::vk::{
    self, DebugUtilsMessengerEXT, DeviceCreateInfo, MemoryRequirements, PhysicalDevice,
    PhysicalDeviceMemoryProperties,
};
use ash::{vk::InstanceCreateInfo, Entry};

use crate::{vulkan_debug_callback, Device, DeviceConnecter, DeviceFeature};

/// Represents an additional feature of the instance.
pub struct InstanceFeature {
    extensions: Vec<*const i8>,
    device_exts: Vec<DeviceFeature>,
}

impl InstanceFeature {
    /// Empty InstanceFeature, no additional functionality.
    #[inline]
    pub fn empty() -> Self {
        Self {
            extensions: vec![],
            device_exts: vec![],
        }
    }

    /// Enable Surface.
    /// "window" feature is required.
    #[cfg(feature = "window")]
    #[inline]
    pub fn use_surface(&mut self, handle: &impl raw_window_handle::HasRawDisplayHandle) {
        let ext = ash_window::enumerate_required_extensions(handle.raw_display_handle()).unwrap();
        for i in ext {
            self.extensions.push(*i);
        }
        self.device_exts.push(DeviceFeature::Swapchain);
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
            feature: Default::default(),
        }
    }

    pub fn feature(mut self, feature: InstanceFeature) -> Self {
        self.feature = feature;
        self
    }

    pub fn build(mut self) -> Instance {
        self.feature.extensions.push(DebugUtils::name().as_ptr());
        let entry = ash::Entry::linked();
        let create_info = InstanceCreateInfo::builder()
            .enabled_extension_names(&self.feature.extensions)
            .build();
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
        Instance {
            instance,
            entry,
            device_exts: self.feature.device_exts,
            debug_utils,
            debug_call_back,
        }
    }
}

pub struct Instance {
    pub(crate) instance: ash::Instance,
    pub(crate) entry: Entry,

    pub(crate) device_exts: Vec<DeviceFeature>,

    debug_utils: DebugUtils,
    debug_call_back: DebugUtilsMessengerEXT,
}

impl Instance {
    /// Enumerate available connectors.
    /// You can get the appropriate connector by getting the QueueFamilyProperties from the connector.
    /// # Example
    ///```
    /// use gear::{InstanceBuilder,CommandPoolDescriptor,CommandRecorderDescriptor};
    /// fn main() {
    ///     let instance = InstanceBuilder::new().build();
    ///     let connecters = instance.enumerate_connecters();
    ///     let mut index = 0;
    ///     let mut found_device = false;
    ///     for i in &connecters {
    ///         let properties = i.get_queue_family_properties();
    ///         for i in properties {
    ///             if i.is_compute_support() {
    ///                 index = 0;
    ///                 found_device = true;
    ///                 break;
    ///             }
    ///         }
    ///     }
    ///     if !found_device {
    ///         panic!("No suitable device found.")
    ///     }
    ///
    ///     let connecter = connecters[index];
    /// }
    ///```
    pub fn enumerate_connecters(&self) -> Vec<DeviceConnecter> {
        let devices = unsafe { self.instance.enumerate_physical_devices() }.unwrap();
        let devices = devices
            .iter()
            .map(|x| DeviceConnecter(*x, &self))
            .collect::<Vec<DeviceConnecter>>();
        devices
    }

    /// Get the first connector.
    pub fn default_connector(&self) -> DeviceConnecter {
        let devices = self.enumerate_connecters();
        devices[0]
    }

    /// Get the version of Vulkan currently in use.
    pub fn vulkan_version(&self) -> Option<String> {
        match self.entry.try_enumerate_instance_version() {
            Ok(v) => match v {
                Some(v) => {
                    let major = ash::vk::api_version_major(v);
                    let minor = ash::vk::api_version_minor(v);
                    let patch = ash::vk::api_version_patch(v);
                    Some(format!("{}.{}.{}", major, minor, patch))
                }
                None => return None,
            },
            Err(_) => return None,
        }
    }

    #[doc(hidden)]
    pub(crate) fn create_device(
        &self,
        connecter: DeviceConnecter,
        info: &DeviceCreateInfo,
    ) -> Device {
        let device = unsafe { self.instance.create_device(connecter.0, info, None) }.unwrap();
        Device::from(device)
    }

    #[doc(hidden)]
    pub(crate) fn get_queue_family_properties(
        &self,
        physical_device: PhysicalDevice,
    ) -> Vec<crate::QueueFamilyProperties> {
        let props = unsafe {
            self.instance
                .get_physical_device_queue_family_properties(physical_device)
        };
        props
            .iter()
            .map(|x| crate::QueueFamilyProperties::from(*x))
            .collect::<Vec<crate::QueueFamilyProperties>>()
    }

    #[doc(hidden)]
    pub(crate) fn get_memory_properties(
        &self,
        physical_device: PhysicalDevice,
    ) -> PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(physical_device)
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
    }
}
