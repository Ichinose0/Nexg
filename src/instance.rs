use crate::{NxError, NxResult};
use ash::extensions::ext::DebugUtils;
use ash::vk::{
    self, DebugUtilsMessengerEXT, DeviceCreateInfo, PhysicalDevice, PhysicalDeviceMemoryProperties,
};
use ash::{vk::InstanceCreateInfo, Entry};
use std::ffi::c_char;

use crate::{vulkan_debug_callback, Device, DeviceConnecter, DeviceFeature};

/// Represents an additional feature of the instance.
pub struct InstanceFeature {
    #[doc(hidden)]
    extensions: Vec<*const c_char>,
    #[doc(hidden)]
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

    /// Allows surfaces to be created.
    /// If this option is not enabled when creating an instance,
    /// Vulkan will force a termination at its convenience when initializing the surface.
    /// **"window" feature is required.**
    #[cfg(feature = "window")]
    #[inline]
    pub fn use_surface(
        &mut self,
        handle: &impl raw_window_handle::HasRawDisplayHandle,
    ) -> NxResult<()> {
        let ext = match ash_window::enumerate_required_extensions(handle.raw_display_handle()) {
            Ok(x) => x,
            Err(e) => return Err(NxError::InternalError(e)),
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

/// Object that allows building windows.
pub struct InstanceBuilder {
    feature: InstanceFeature,
}

impl InstanceBuilder {
    /// Initializes a new builder with default values.
    pub fn new() -> Self {
        Self {
            feature: Default::default(),
        }
    }

    /// Specifies the functionality used by the instance.
    pub fn feature(mut self, feature: InstanceFeature) -> Self {
        self.feature = feature;
        self
    }

    /// Create an instance.
    /// This will fail if there is insufficient memory or if the device does not support **Vulkan 1.3** or **later**.
    pub fn build(mut self) -> NxResult<Instance> {
        self.feature
            .extensions
            .push(DebugUtils::name().as_ptr() as *const c_char);
        let entry = Entry::linked();
        let create_info = InstanceCreateInfo::builder()
            .enabled_extension_names(&self.feature.extensions)
            .build();
        let instance = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(x) => x,
            Err(e) => return Err(NxError::InternalError(e)),
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
                Err(e) => return Err(NxError::InternalError(e)),
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

impl Default for InstanceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestConnecterDescriptor {
    is_graphic_support: bool,
    is_compute_support: bool,
    is_transfer_support: bool,
}

impl RequestConnecterDescriptor {
    pub fn graphic_support(mut self, is_graphic_support: bool) -> Self {
        self.is_graphic_support = is_graphic_support;
        self
    }

    pub fn compute_support(mut self, is_compute_support: bool) -> Self {
        self.is_compute_support = is_compute_support;
        self
    }

    pub fn transfer_support(mut self, is_transfer_support: bool) -> Self {
        self.is_transfer_support = is_transfer_support;
        self
    }
}

impl RequestConnecterDescriptor {
    pub fn new() -> Self {
        Self {
            is_graphic_support: true,
            is_compute_support: true,
            is_transfer_support: true,
        }
    }
}

impl Default for RequestConnecterDescriptor {
    fn default() -> Self {
        Self::new()
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
    /// ```
    /// use nexg::{InstanceFeature,InstanceBuilder};
    ///
    ///  let feature = InstanceFeature::empty();
    ///  let instance = InstanceBuilder::new().feature(feature).build().unwrap();
    ///  let connecters = instance.enumerate_connecters().unwrap();
    ///  let mut index = 0;
    ///  let mut found_device = false;
    ///  for i in &connecters {
    ///     let properties = i.get_queue_family_properties(&instance).unwrap();
    ///     for i in properties {
    ///         if i.is_graphic_support() {
    ///             index = 0;
    ///             found_device = true;
    ///             break;
    ///         }
    ///     }
    ///  }
    ///  if !found_device {
    ///     panic!("No suitable device found.")
    ///  }
    ///
    ///  let connecter = connecters[index];
    ///
    ///  let device = connecter.create_device(&instance, index).unwrap();
    /// ```
    #[deprecated(since = "0.1.1", note = "Please use request_connecters")]
    pub fn enumerate_connecters(&self) -> NxResult<Vec<DeviceConnecter>> {
        let devices = match unsafe { self.instance.enumerate_physical_devices() } {
            Ok(x) => x,
            Err(e) => return Err(NxError::InternalError(e)),
        };
        let devices = devices
            .iter()
            .map(|x| DeviceConnecter(*x, 0))
            .collect::<Vec<DeviceConnecter>>();
        if !devices.is_empty() {
            Ok(devices)
        } else {
            Err(NxError::NoValue)
        }
    }

    pub fn request_connecters(
        &self,
        descriptors: &[RequestConnecterDescriptor],
    ) -> NxResult<Vec<DeviceConnecter>> {
        let mut connecter = vec![];
        for desc in descriptors {
            let mut connecters = self.enumerate_connecters()?;
            let mut index = 0;
            let mut count = 0;
            for i in &connecters {
                let properties = i.get_queue_family_properties(self).unwrap();
                for (n, i) in properties.iter().enumerate() {
                    count = 0;
                    if i.is_graphic_support() == desc.is_graphic_support {
                        count += 1;
                    }
                    if i.is_compute_support() == desc.is_compute_support {
                        count += 1;
                    }
                    if i.is_transfer_support() == desc.is_transfer_support {
                        count += 1;
                    }

                    if count == 3 {
                        index = n;
                        break;
                    }
                }
            }
            if count != 3 {
                return Err(NxError::NoValue);
            }

            let mut c = connecters[index];
            c.1 = index;
            connecter.push(c);
        }

        Ok(connecter)
    }

    /// Get the first connector.
    #[deprecated(
        since = "0.1.0",
        note = "Use enumerate_connecters() to manually get the appropriate one."
    )]
    pub fn default_connector(&self) -> DeviceConnecter {
        let devices = self.enumerate_connecters().unwrap();
        devices[0]
    }

    /// Get the version of Vulkan currently in use.
    /// This may not be possible to obtain.
    pub fn vulkan_version(&self) -> Option<String> {
        match self.entry.try_enumerate_instance_version() {
            Ok(v) => match v {
                Some(v) => {
                    let major = vk::api_version_major(v);
                    let minor = vk::api_version_minor(v);
                    let patch = vk::api_version_patch(v);
                    Some(format!("{}.{}.{}", major, minor, patch))
                }
                None => None,
            },
            Err(_) => None,
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
            Err(e) => return Err(NxError::InternalError(e)),
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
        if !props.is_empty() {
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
