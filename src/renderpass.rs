use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Format,
    ImageLayout, RenderPassCreateInfo, SampleCountFlags, SubpassDescription,
};

use crate::{
    BindPoint, Destroy, Device, DeviceConnecter, FrameBuffer, Instance, NxError, NxResult,
};

/// Stores information needed to start a render pass.
#[derive(Clone, Copy)]
pub struct RenderPassBeginDescriptor<'a> {
    pub(crate) frame_buffer: Option<&'a FrameBuffer>,
    pub(crate) render_pass: Option<&'a RenderPass>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
    pub(crate) a: f32,
}

impl<'a> RenderPassBeginDescriptor<'a> {
    /// Initializes a new descriptor with default values.
    pub fn empty() -> Self {
        Self {
            frame_buffer: None,
            render_pass: None,
            width: 100,
            height: 100,
            x: 0,
            y: 0,
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Specifies the width of the RenderPass.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Specifies the height of the RenderPass.
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Specify the color when clearing the screen.
    pub fn clear(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.r = r;
        self.g = g;
        self.b = b;
        self.a = a;
        self
    }

    #[must_use]
    pub fn frame_buffer(mut self, frame_buffer: &'a FrameBuffer) -> Self {
        self.frame_buffer = Some(frame_buffer);
        self
    }

    #[must_use]
    pub fn render_pass(mut self, render_pass: &'a RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

impl From<LoadOp> for AttachmentLoadOp {
    fn from(value: LoadOp) -> Self {
        match value {
            LoadOp::Load => AttachmentLoadOp::LOAD,
            LoadOp::Clear => AttachmentLoadOp::CLEAR,
            LoadOp::DontCare => AttachmentLoadOp::DONT_CARE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreOp {
    Store,
    DontCare,
}

impl From<StoreOp> for AttachmentStoreOp {
    fn from(value: StoreOp) -> Self {
        match value {
            StoreOp::Store => AttachmentStoreOp::STORE,
            StoreOp::DontCare => AttachmentStoreOp::DONT_CARE,
        }
    }
}

/// Stores information needed to create a FrameBuffer.
pub struct SubPassDescriptor {
    bind_point: BindPoint,
}

impl SubPassDescriptor {
    /// Initializes a new descriptor with default values.
    #[inline]
    pub fn empty() -> Self {
        Self {
            bind_point: BindPoint::Graphics,
        }
    }
}

pub struct SubPass {
    subpass_desc: SubpassDescription,
}

impl SubPass {
    /// Create a new Framebuffer.
    /// # Arguments
    ///
    /// * `connecter` - Appropriate DeviceConnecter.
    /// * `descriptor` - Appropriate SubPassDescriptor.
    #[inline]
    pub fn new(_connecter: DeviceConnecter, descriptor: &SubPassDescriptor) -> Self {
        let attachment_refs = vec![AttachmentReference::builder()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let subpass_desc = SubpassDescription::builder()
            .pipeline_bind_point(descriptor.bind_point.into())
            .color_attachments(&attachment_refs)
            .build();
        Self { subpass_desc }
    }
}

/// Stores information needed to create a RenderPass.
pub struct RenderPassDescriptor<'a> {
    load_op: LoadOp,
    store_op: StoreOp,
    subpasses: &'a [SubPass],
}

impl<'a> RenderPassDescriptor<'a> {
    /// Initializes a new descriptor with default values.
    #[inline]
    pub fn empty() -> Self {
        Self {
            load_op: LoadOp::DontCare,
            store_op: StoreOp::Store,
            subpasses: &[],
        }
    }

    #[inline]
    pub fn load_op(mut self, load_op: LoadOp) -> Self {
        self.load_op = load_op;
        self
    }

    #[inline]
    pub fn store_op(mut self, store_op: StoreOp) -> Self {
        self.store_op = store_op;
        self
    }

    #[inline]
    pub fn subpasses(mut self, subpasses: &'a [SubPass]) -> Self {
        self.subpasses = subpasses;
        self
    }
}

pub struct RenderPass {
    pub(crate) render_pass: ash::vk::RenderPass,
}

impl RenderPass {
    /// Create a new Framebuffer.
    /// # Arguments
    ///
    /// * `device` - Reference to the appropriate device.
    /// * `descriptor` - Appropriate RenderPassDescriptor.
    #[inline]
    pub fn new(device: &Device, descriptor: &RenderPassDescriptor) -> NxResult<Self> {
        let subpasses = descriptor
            .subpasses
            .iter()
            .map(|x| x.subpass_desc)
            .collect::<Vec<SubpassDescription>>();
        let attachments = vec![AttachmentDescription::builder()
            .format(Format::R8G8B8A8_UNORM)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(descriptor.store_op.into())
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::GENERAL)
            .build()];
        let create_info = RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&[])
            .build();
        let render_pass = match unsafe { device.device.create_render_pass(&create_info, None) } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                _ => Err(NxError::Unknown),
            }?,
        };
        Ok(Self { render_pass })
    }
}

impl Destroy for RenderPass {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
