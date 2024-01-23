use crate::{Device,CommandRecorder};
use ash::vk::{CommandBuffer,Fence,SubmitInfo};

#[derive(Clone,Copy)]
pub struct Queue<'a>(pub(crate) ash::vk::Queue,pub(crate) &'a Device);

impl<'a> Queue<'a> {
    pub fn submit(&self,recorders: Vec<CommandRecorder>) {
        let buffers = recorders.iter().map(|x| {
            x.buffeer
        }).collect::<Vec<CommandBuffer>>();
        let submit_info = SubmitInfo::builder().command_buffers(&buffers).build();
        unsafe {
            self.1.queue_submit(self.0,&[submit_info],Fence::null()).unwrap();
        }
    }
}