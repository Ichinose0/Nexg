use crate::{Destroy, Device, Instance, NxError, NxResult};
use ash::vk::SemaphoreCreateInfo;

/// Stores information needed to create a Semaphore.
pub struct SemaphoreDescriptor {}

impl SemaphoreDescriptor {
    /// Initializes a new descriptor with default values.
    #[inline]
    pub fn empty() -> Self {
        Self {}
    }
}

/// An object to wait for work.
/// Used to wait for a specific operation.
#[derive(Clone, Copy)]
pub struct Semaphore {
    pub(crate) semaphore: ash::vk::Semaphore,
}

impl Semaphore {
    /// Create a new semaphore.
    /// # Arguments
    ///
    /// * `device` - Reference to the appropriate device.
    /// * `descriptor` - Appropriate SemaphoreDescriptor.
    pub fn new(device: &Device, _descriptor: &SemaphoreDescriptor) -> NxResult<Self> {
        let create_info = SemaphoreCreateInfo::builder().build();
        let semaphore = match unsafe { device.device.create_semaphore(&create_info, None) } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        };
        Ok(Self { semaphore })
    }
}

impl Destroy for Semaphore {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_semaphore(self.semaphore, None);
        }
    }
}
