use ash::vk::FenceCreateInfo;

use crate::{Destroy, Device, Instance, NxError, NxResult};

/// Stores information needed to create a Fence.
pub struct FenceDescriptor {
    signaled: bool,
}

impl FenceDescriptor {
    /// Initializes a new descriptor with default values.
    pub const fn empty() -> Self {
        Self { signaled: false }
    }

    /// Put the fence in signal state.
    pub const fn signaled(mut self, signaled: bool) -> Self {
        self.signaled = signaled;
        self
    }
}

/// An object to wait for work.
/// This is used by the CPU to wait for the GPU to finish its work.
pub struct Fence {
    pub(crate) fence: ash::vk::Fence,
}

impl Fence {
    /// Create a new fence.
    /// # Arguments
    ///
    /// * `device` - Reference to the appropriate device.
    /// * `descriptor` - Appropriate FenceDescriptor.
    pub fn new(device: &Device, descriptor: &FenceDescriptor) -> NxResult<Self> {
        let flag = match descriptor.signaled {
            true => ash::vk::FenceCreateFlags::SIGNALED,
            false => ash::vk::FenceCreateFlags::empty(),
        };
        let create_info = FenceCreateInfo::builder().flags(flag).build();
        let fence = match unsafe { device.device.create_fence(&create_info, None) } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        };
        Ok(Self { fence })
    }

    /// Wait until the GPU finishes processing.
    /// The time until timeout must be specified.
    pub fn wait(&self, device: &Device, timeout: u64) -> NxResult<()> {
        match unsafe { device.device.wait_for_fences(&[self.fence], true, timeout) } {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        }
    }

    /// Reset fence status.
    pub fn reset(&self, device: &Device) -> NxResult<()> {
        match unsafe { device.device.reset_fences(&[self.fence]) } {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        }
    }
}

impl Destroy for Fence {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_fence(self.fence, None);
        }
    }
}
