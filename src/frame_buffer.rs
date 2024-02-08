use ash::vk::FramebufferCreateInfo;

use crate::{Destroy, Device, ImageView, Instance, NxError, NxResult, RenderPass};

/// Stores information needed to create a FrameBuffer.
pub struct FrameBufferDescriptor<'a> {
    width: u32,
    height: u32,
    render_pass: Option<&'a RenderPass>,
    image_view: Option<&'a ImageView>,
}

impl<'a> FrameBufferDescriptor<'a> {
    #[inline]
    /// Initializes a new descriptor with default values.
    pub const fn empty() -> Self {
        Self {
            width: 100,
            height: 100,
            render_pass: None,
            image_view: None,
        }
    }

    #[inline]
    /// Specifies the width of the FrameBuffer.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    #[inline]
    /// Specifies the height of the FrameBuffer.
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    #[inline]
    #[must_use]
    /// ImageView used to create the FrameBuffer.
    pub fn image_view(mut self, image_view: &'a ImageView) -> Self {
        self.image_view = Some(image_view);
        self
    }

    #[inline]
    #[must_use]
    /// RenderPass used to create the FrameBuffer.
    pub fn render_pass(mut self, render_pass: &'a RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }
}

pub struct FrameBuffer {
    pub(crate) frame_buffer: ash::vk::Framebuffer,
}

impl FrameBuffer {
    #[inline]
    /// Create a new Framebuffer.
    /// # Arguments
    ///
    /// * `device` - Reference to the appropriate device.
    /// * `descriptor` - Appropriate FenceDescriptor.
    pub fn new(device: &Device, descriptor: &FrameBufferDescriptor) -> NxResult<Self> {
        let render_pass = descriptor.render_pass.unwrap();
        let image_view = descriptor.image_view.unwrap();
        let create_info = FramebufferCreateInfo::builder()
            .width(descriptor.width)
            .height(descriptor.height)
            .layers(1)
            .render_pass(render_pass.render_pass)
            .attachments(&[image_view.image_view])
            .build();
        let frame_buffer = match unsafe { device.device.create_framebuffer(&create_info, None) } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        };
        Ok(Self { frame_buffer })
    }
}

impl Destroy for FrameBuffer {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_framebuffer(self.frame_buffer, None);
        }
    }
}
