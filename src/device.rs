use crate::{
    CommandPool, CommandPoolDescriptor, CommandRecorder, CommandRecorderDescriptor, Destroy,
    NxResult, Queue, Resource, ResourceUpdateDescriptor,
};
use ash::vk::{DescriptorBufferInfo, WriteDescriptorSet};

#[doc(hidden)]
pub(crate) enum DeviceFeature {
    Swapchain,
}

#[derive(Clone)]
pub struct Device {
    #[doc(hidden)]
    pub(crate) device: ash::Device,
}

impl Device {
    #[doc(hidden)]
    pub(crate) fn from(device: ash::Device) -> Self {
        Self { device }
    }

    /// Get the queue corresponding to queue_family_index.
    /// # Example
    /// ```
    /// /// Appropriate instance.
    /// let instance = ..;
    /// // Index of the appropriate queue family. This must be obtained manually.
    /// let index = 0;
    /// // Get DeviceConnecter in some way.
    /// let connecter = ..;
    /// let device = connecter.create_device(&instance, index).unwrap();
    /// let queue = device.get_queue(index);
    ///```
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

    pub fn update_resource(&self, descriptor: &ResourceUpdateDescriptor) {
        let buffer_info = descriptor
            .buffer_desc
            .iter()
            .map(|x| {
                DescriptorBufferInfo::builder()
                    .buffer(x.buffer.buffer)
                    .offset(x.offset)
                    .range(x.range as u64)
                    .build()
            })
            .collect::<Vec<DescriptorBufferInfo>>();
        let desc = WriteDescriptorSet::builder()
            .dst_set(descriptor.resource.descriptor_set)
            .dst_binding(descriptor.binding)
            .dst_array_element(descriptor.array_element)
            .descriptor_type(descriptor.resource_type.into())
            .buffer_info(&buffer_info)
            .build();
        unsafe {
            self.device.update_descriptor_sets(&[desc], &[]);
        }
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
