use crate::{Instance, Queue,CommandPool,CommandRecorder,CommandRecorderDescriptor};

pub struct Device {
    pub(crate) device: ash::Device,
}

impl Device {
    pub(crate) fn from(device: ash::Device) -> Self {
        Self {
            device,
        }
    }

    pub fn get_queue(&self,queue_family_index: usize) -> Queue {
        Queue(unsafe { self.device.get_device_queue(queue_family_index as u32, 0)},&self)
    }

    pub fn create_command_pool(&self,descriptor: &CommandPoolDescriptor) -> CommandPool {
        CommandPool::create(&self.device,descriptor)
    }

    pub fn allocate_command_recorder(&self,pool: CommandPool,descriptor: &CommandRecorderDescriptor) -> Vec<CommandRecorder> {
        CommandRecorder::create(&self.device,descriptor)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}