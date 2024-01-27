use crate::{CommandRecorder, Device, Semaphore, Swapchain};
use ash::vk::{CommandBuffer, Fence, SubmitInfo};

pub struct QueuePresentDescriptor<'a> {
    pub(crate) wait_semaphores: &'a [Semaphore],
    pub(crate) signal_semaphores: &'a [Semaphore],
    pub(crate) queue: Option<&'a Queue>,
}

impl<'a> QueuePresentDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            wait_semaphores: &[],
            signal_semaphores: &[],
            queue: None,
        }
    }

    pub fn wait_semaphores(mut self, semaphore: &'a [Semaphore]) -> Self {
        self.wait_semaphores = semaphore;
        self
    }

    pub fn signal_semaphores(mut self, semaphore: &'a [Semaphore]) -> Self {
        self.signal_semaphores = semaphore;
        self
    }

    pub fn queue(mut self, queue: &'a Queue) -> Self {
        self.queue = Some(queue);
        self
    }
}

pub struct QueueSubmitDescriptor<'a> {
    wait_semaphores: &'a [Semaphore],
    signal_semaphores: &'a [Semaphore],
    fence: Option<&'a crate::Fence>,
}

impl<'a> QueueSubmitDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            wait_semaphores: &[],
            signal_semaphores: &[],
            fence: None,
        }
    }

    pub fn fence(mut self, fence: &'a crate::Fence) -> Self {
        self.fence = Some(fence);
        self
    }

    pub fn wait_semaphores(mut self, semaphores: &'a [Semaphore]) -> Self {
        self.wait_semaphores = semaphores;
        self
    }

    pub fn signal_semaphores(mut self, semaphores: &'a [Semaphore]) -> Self {
        self.signal_semaphores = semaphores;
        self
    }
}

#[derive(Clone)]
pub struct Queue(pub(crate) ash::vk::Queue);

impl Queue {
    #[inline]
    pub fn submit(
        &self,
        device: &Device,
        descriptor: &QueueSubmitDescriptor,
        recorders: &[CommandRecorder],
    ) {
        let buffers = recorders
            .iter()
            .map(|x| x.buffer)
            .collect::<Vec<CommandBuffer>>();
        let w_semaphores = descriptor
            .wait_semaphores
            .iter()
            .map(|x| x.semaphore)
            .collect::<Vec<ash::vk::Semaphore>>();
        let s_semaphores = descriptor
            .signal_semaphores
            .iter()
            .map(|x| x.semaphore)
            .collect::<Vec<ash::vk::Semaphore>>();
        let fence = match descriptor.fence {
            Some(x) => x.fence,
            None => Fence::null(),
        };
        let submit_info = SubmitInfo::builder()
            .wait_semaphores(&w_semaphores)
            .signal_semaphores(&s_semaphores)
            .command_buffers(&buffers)
            .build();
        unsafe {
            device
                .device
                .queue_submit(self.0, &[submit_info], fence)
                .unwrap();
        }
    }
}
