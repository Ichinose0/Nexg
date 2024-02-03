use ash::vk::FenceCreateInfo;

use crate::{Destroy, Device, Instance, NxError, NxResult};

pub struct FenceDescriptor {
    signaled: bool,
}

impl FenceDescriptor {
    pub const fn empty() -> Self {
        Self { signaled: false }
    }

    pub const fn signaled(mut self, signaled: bool) -> Self {
        self.signaled = signaled;
        self
    }
}

pub struct Fence {
    pub(crate) fence: ash::vk::Fence,
}

impl Fence {
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
