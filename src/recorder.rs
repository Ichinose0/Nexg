use crate::{Destroy, Device, Instance, Pipeline, RenderPassBeginDescriptor};
use ash::vk::{
    ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandPoolCreateFlags, CommandPoolCreateInfo,
    Extent2D, Offset2D, PipelineBindPoint, Rect2D, RenderPassBeginInfo, SubpassContents,
};

pub struct CommandPoolDescriptor {
    queue_family_index: Option<usize>,
}

impl CommandPoolDescriptor {
    #[inline]
    pub fn empty() -> Self {
        Self {
            queue_family_index: None,
        }
    }

    #[inline]
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
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let pool = unsafe { device.create_command_pool(&create_info, None) }.unwrap();
        Self(pool)
    }
}

impl Destroy for CommandPool {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_command_pool(self.0, None);
        }
    }
}

pub struct CommandRecorderDescriptor {
    recorder_count: u32,
}

impl CommandRecorderDescriptor {
    #[inline]
    pub fn empty() -> Self {
        Self { recorder_count: 1 }
    }
}

pub struct CommandRecorder {
    pub(crate) buffer: CommandBuffer,
}

impl CommandRecorder {
    #[doc(hidden)]
    pub(crate) fn create(
        device: &Device,
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
            .map(|x| Self { buffer: *x })
            .collect::<Vec<Self>>()
    }

    #[inline]
    pub fn begin(&self, device: &Device, descriptor: RenderPassBeginDescriptor) {
        let create_info = CommandBufferBeginInfo::builder().build();
        let mut clear = ClearValue::default();
        unsafe {
            clear.color.float32[0] = descriptor.r;
            clear.color.float32[1] = descriptor.g;
            clear.color.float32[2] = descriptor.b;
            clear.color.float32[3] = descriptor.a;
        }
        let begin_info = RenderPassBeginInfo::builder()
            .render_pass(descriptor.render_pass.unwrap().render_pass)
            .framebuffer(descriptor.frame_buffer.unwrap().frame_buffer)
            .render_area(
                Rect2D::builder()
                    .extent(
                        Extent2D::builder()
                            .width(descriptor.width)
                            .height(descriptor.height)
                            .build(),
                    )
                    .offset(Offset2D::builder().x(0).y(0).build())
                    .build(),
            )
            .clear_values(&[clear])
            .build();
        unsafe {
            device
                .device
                .begin_command_buffer(self.buffer, &create_info)
                .unwrap();
            device
                .device
                .cmd_begin_render_pass(self.buffer, &begin_info, SubpassContents::INLINE);
        }
    }

    #[inline]
    pub fn end(&self, device: &Device) {
        unsafe {
            device.device.cmd_end_render_pass(self.buffer);
            device.device.end_command_buffer(self.buffer).unwrap();
        }
    }

    #[inline]
    pub fn draw(
        &self,
        pipeline: &Pipeline,
        device: &Device,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            device.device.cmd_bind_pipeline(
                self.buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );
            device.device.cmd_draw(
                self.buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    #[inline]
    pub fn reset(&self, device: &Device) {
        unsafe {
            device
                .device
                .reset_command_buffer(self.buffer, CommandBufferResetFlags::empty());
        }
    }
}
