use crate::{Destroy, Device, Instance, NxError, NxResult};
use ash::vk::SemaphoreCreateInfo;

pub struct SemaphoreDescriptor {}

impl SemaphoreDescriptor {
    #[inline]
    pub fn empty() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct Semaphore {
    pub(crate) semaphore: ash::vk::Semaphore,
}

impl Semaphore {
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
