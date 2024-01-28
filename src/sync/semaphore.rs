use crate::{Destroy, Device, Fence, Instance};
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
    pub fn new(device: &Device, descriptor: &SemaphoreDescriptor) -> Self {
        let create_info = SemaphoreCreateInfo::builder().build();
        let semaphore = unsafe { device.device.create_semaphore(&create_info, None) }.unwrap();
        Self { semaphore }
    }
}

impl Destroy for Semaphore {
    fn instance(&self, instance: &Instance) {

    }

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_semaphore(self.semaphore,None);
        }
    }
}