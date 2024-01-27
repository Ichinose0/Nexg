use ash::vk::FenceCreateInfo;

use crate::Device;

pub struct FenceDescriptor {
    signaled: bool,
}

impl FenceDescriptor {
    pub fn empty() -> Self {
        Self { signaled: false }
    }

    pub fn signaled(mut self, signaled: bool) -> Self {
        self.signaled = signaled;
        self
    }
}

pub struct Fence {
    pub(crate) fence: ash::vk::Fence,
}

impl Fence {
    pub fn new(device: &Device, descriptor: &FenceDescriptor) -> Self {
        let flag = match descriptor.signaled {
            true => ash::vk::FenceCreateFlags::SIGNALED,
            false => ash::vk::FenceCreateFlags::empty(),
        };
        let create_info = FenceCreateInfo::builder().flags(flag).build();
        let fence = unsafe { device.device.create_fence(&create_info, None) }.unwrap();
        Self { fence }
    }

    pub fn wait(&self, device: &Device, timeout: u64) {
        unsafe {
            device
                .device
                .wait_for_fences(&[self.fence], true, timeout)
                .unwrap();
        }
    }

    pub fn reset(&self, device: &Device) {
        unsafe {
            device.device.reset_fences(&[self.fence]).unwrap();
        }
    }
}
