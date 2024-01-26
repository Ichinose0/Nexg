use crate::{CommandRecorder, Device, Swapchain};
use ash::vk::{CommandBuffer, Fence, SubmitInfo};

#[derive(Clone)]
pub struct Queue(pub(crate) ash::vk::Queue);

impl Queue {
    #[inline]
    pub fn submit(&self, device: &Device, recorders: &[CommandRecorder]) {
        let buffers = recorders
            .iter()
            .map(|x| x.buffer)
            .collect::<Vec<CommandBuffer>>();
        let submit_info = SubmitInfo::builder().command_buffers(&buffers).build();
        unsafe {
            device
                .device
                .queue_submit(self.0, &[submit_info], Fence::null())
                .unwrap();
            device.device.queue_wait_idle(self.0).unwrap();
        }
    }
}
