use crate::{
    CommandPool, CommandPoolDescriptor, CommandRecorder, CommandRecorderDescriptor, Destroy,
    NxResult, Queue,
};

pub(crate) enum DeviceFeature {
    Swapchain,
}

#[derive(Clone)]
pub struct Device {
    pub(crate) device: ash::Device,
}

impl Device {
    pub(crate) fn from(device: ash::Device) -> Self {
        Self { device }
    }

    /// Get the queue corresponding to queue_family_index.
    pub fn get_queue(&self, queue_family_index: usize) -> Queue {
        Queue(unsafe { self.device.get_device_queue(queue_family_index as u32, 0) })
    }

    /// Create a command pool.
    pub fn create_command_pool(&self, descriptor: &CommandPoolDescriptor) -> NxResult<CommandPool> {
        CommandPool::create(&self.device, descriptor)
    }

    /// Allocate command recorder.
    pub fn allocate_command_recorder(
        &self,
        pool: CommandPool,
        descriptor: &CommandRecorderDescriptor,
    ) -> NxResult<Vec<CommandRecorder>> {
        CommandRecorder::create(self, pool, descriptor)
    }

    pub fn destroy<D>(&self, object: &D)
    where
        D: Destroy,
    {
        object.device(&self);
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
