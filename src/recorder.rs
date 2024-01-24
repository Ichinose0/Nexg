use crate::Device;
use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel,
    CommandPoolCreateInfo,
};

pub struct CommandPoolDescriptor {
    queue_family_index: Option<usize>,
}

impl CommandPoolDescriptor {
    pub fn new() -> Self {
        Self {
            queue_family_index: None,
        }
    }

    pub fn queue_family_index(mut self, queue_family_index: usize) -> Self {
        self.queue_family_index = Some(queue_family_index);
        self
    }
}

pub struct CommandPool(pub(crate) ash::vk::CommandPool);

impl CommandPool {
    #[doc(hidden)]
    pub(crate) fn create(device: &ash::Device, descriptor: &CommandPoolDescriptor) -> Self {
        let queue_family_index = descriptor.queue_family_index.unwrap();
        let create_info = CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index as u32)
            .build();
        let pool = unsafe { device.create_command_pool(&create_info, None) }.unwrap();
        Self(pool)
    }
}

pub struct CommandRecorderDescriptor {
    recorder_count: u32,
}

impl CommandRecorderDescriptor {
    pub fn new() -> Self {
        Self { recorder_count: 1 }
    }
}

pub struct CommandRecorder<'a> {
    pub(crate) buffer: CommandBuffer,
    device: &'a Device,
}

impl<'a> CommandRecorder<'a> {
    #[doc(hidden)]
    pub(crate) fn create(
        device: &'a Device,
        pool: CommandPool,
        descriptor: &CommandRecorderDescriptor,
    ) -> Vec<Self> {
        let create_info = CommandBufferAllocateInfo::builder()
            .command_pool(pool.0)
            .command_buffer_count(descriptor.recorder_count)
            .level(CommandBufferLevel::PRIMARY)
            .build();
        let buffers = unsafe { device.device.allocate_command_buffers(&create_info) }.unwrap();
        assert_eq!(descriptor.recorder_count, buffers.len() as u32);
        buffers
            .iter()
            .map(|x| Self { buffer: *x, device })
            .collect::<Vec<Self>>()
    }

    pub fn begin(&self) {
        let create_info = CommandBufferBeginInfo::builder().build();
        unsafe {
            self.device
                .device
                .begin_command_buffer(self.buffer, &create_info)
                .unwrap()
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.device.end_command_buffer(self.buffer).unwrap();
        }
    }
}
