use crate::Instance;

pub struct Device {
    device: ash::Device,
}

impl Device {
    pub(crate) fn from(device: ash::Device) -> Self {
        Self {
            device,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}