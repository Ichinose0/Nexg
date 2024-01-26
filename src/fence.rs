use ash::vk::FenceCreateInfo;

use crate::Device;

pub struct Fence {
    pub(crate) fence: ash::vk::Fence,
}

impl Fence {
    pub fn new(device: &Device) -> Self {
        let create_info = FenceCreateInfo::builder().build();
        let fence = unsafe { device.device.create_fence(&create_info, None) }.unwrap();
        Self { fence }
    }
}
