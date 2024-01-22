use crate::{Instance, Queue};

pub struct Device {
    device: ash::Device,
}

impl Device {
    pub(crate) fn from(device: ash::Device) -> Self {
        Self {
            device,
        }
    }

    pub fn get_queue(&self,queue_family_index: usize) -> Queue {
        Queue(unsafe { self.device.get_device_queue(queue_family_index as u32, 0)})
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}