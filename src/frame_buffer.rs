use ash::vk::FramebufferCreateInfo;

use crate::{image, Device, ImageView, RenderPass};

pub struct FrameBufferDescriptor<'a> {
    width: u32,
    height: u32,
    render_pass: Option<&'a RenderPass>,
    image_view: Option<&'a ImageView>,
}

impl<'a> FrameBufferDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            width: 100,
            height: 100,
            render_pass: None,
            image_view: None,
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn image_view(mut self, image_view: &'a ImageView) -> Self {
        self.image_view = Some(image_view);
        self
    }

    pub fn render_pass(mut self, render_pass: &'a RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }
}

pub struct FrameBuffer {
    pub(crate) frame_buffer: ash::vk::Framebuffer,
}

impl FrameBuffer {
    pub fn new(device: &Device, descriptor: &FrameBufferDescriptor) -> Self {
        let render_pass = descriptor.render_pass.unwrap();
        let image_view = descriptor.image_view.unwrap();
        let create_info = FramebufferCreateInfo::builder()
            .width(descriptor.width)
            .height(descriptor.height)
            .layers(1)
            .render_pass(render_pass.render_pass)
            .attachments(&[image_view.image_view])
            .build();
        let frame_buffer = unsafe { device.device.create_framebuffer(&create_info, None) }.unwrap();
        Self { frame_buffer }
    }
}