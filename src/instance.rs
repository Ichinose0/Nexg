use crate::{NxError,NxResult};
use ash::extensions::ext::DebugUtils;
use ash::vk::{
    self, DebugUtilsMessengerEXT, DeviceCreateInfo, PhysicalDevice, PhysicalDeviceMemoryProperties,
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
    pub const fn empty() -> Self {
        Self {
            extensions: vec![],
            device_exts: vec![],
        }
    }

    /// Enable Surface.
    /// "window" feature is required.
    #[cfg(feature = "window")]
    #[inline]
    pub fn use_surface(&mut self, handle: &impl raw_window_handle::HasRawDisplayHandle) -> NxResult<()> {
        let ext = match ash_window::enumerate_required_extensions(handle.raw_display_handle()) {
            Ok(x) => x,
            Err(e) => {
                return Err(NxError::InternalError(e))
            }
        };
        for i in ext {
            self.extensions.push(*i);
        }
        self.device_exts.push(DeviceFeature::Swapchain);
        Ok(())
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

    pub fn build(mut self) -> NxResult<Instance> {
        self.feature.extensions.push(DebugUtils::name().as_ptr());
        let entry = Entry::linked();
        let create_info = InstanceCreateInfo::builder()
            .enabled_extension_names(&self.feature.extensions)
            .build();
        let instance = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(x) => x,
            Err(e) => {
                return Err(NxError::InternalError(e))
            }
        };
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
            match unsafe { debug_utils.create_debug_utils_messenger(&debug_info, None) } {
                Ok(x) => x,
                Err(e) => {
                    return Err(NxError::InternalError(e))
                }
            };
        Ok(Instance {
            instance,
            entry,
            device_exts: self.feature.device_exts,
            debug_utils,
            debug_call_back,
        })
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
    pub fn enumerate_connecters(&self) -> NxResult<Vec<DeviceConnecter>> {
        let devices = match unsafe { self.instance.enumerate_physical_devices() } {
            Ok(x) => x,
            Err(e) => {
                return Err(NxError::InternalError(e))
            }
        };
        let devices = devices
            .iter()
            .map(|x| DeviceConnecter(*x))
            .collect::<Vec<DeviceConnecter>>();
        if devices.len() != 0 {
            Ok(devices)
        } else {
            Err(NxError::NoValue)
        }
    }

    /// Get the first connector.
    pub fn default_connector(&self) -> DeviceConnecter {
        let devices = self.enumerate_connecters().unwrap();
        devices[0]
    }

    /// Get the version of Vulkan currently in use.
    pub fn vulkan_version(&self) -> Option<String> {
        match self.entry.try_enumerate_instance_version() {
            Ok(v) => match v {
                Some(v) => {
                    let major = vk::api_version_major(v);
                    let minor = vk::api_version_minor(v);
                    let patch = vk::api_version_patch(v);
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
    ) -> NxResult<Device> {
        let device = match unsafe { self.instance.create_device(connecter.0, info, None) } {
            Ok(x) => x,
            Err(e) => {
                return Err(NxError::InternalError(e))
            }
        };
        Ok(Device::from(device))
    }

    #[doc(hidden)]
    pub(crate) fn get_queue_family_properties(
        &self,
        physical_device: PhysicalDevice,
    ) -> NxResult<Vec<crate::QueueFamilyProperties>> {
        let props = unsafe {
            self.instance
                .get_physical_device_queue_family_properties(physical_device)
        };
        let props = props
            .iter()
            .map(|x| crate::QueueFamilyProperties::from(*x))
            .collect::<Vec<crate::QueueFamilyProperties>>();
        if props.len() != 0 {
            Ok(props)
        } else {
            Err(NxError::NoValue)
        }
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
