use crate::{CommandRecorder, Device};
use ash::vk::{CommandBuffer, Fence, SubmitInfo};

#[derive(Clone, Copy)]
pub struct Queue<'a>(pub(crate) ash::vk::Queue, pub(crate) &'a Device);

impl<'a> Queue<'a> {
    #[inline]
    pub fn submit(&self, recorders: &[CommandRecorder]) {
        let buffers = recorders
            .iter()
            .map(|x| x.buffer)
            .collect::<Vec<CommandBuffer>>();
        let submit_info = SubmitInfo::builder().command_buffers(&buffers).build();
        unsafe {
            self.1
                .device
                .queue_submit(self.0, &[submit_info], Fence::null())
                .unwrap();
            self.1.device.queue_wait_idle(self.0).unwrap();
        }
    }
}
