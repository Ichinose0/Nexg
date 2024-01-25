use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, ClearValue,
    Extent2D, Format, ImageLayout, Offset2D, Rect2D, RenderPassBeginInfo, RenderPassCreateInfo,
    SampleCountFlags, SubpassDescription,
};

use crate::{BindPoint, Device, DeviceConnecter, FrameBuffer};

#[derive(Clone, Copy)]
pub struct RenderPassBeginDescriptor<'a> {
    frame_buffer: Option<&'a FrameBuffer>,
    render_pass: Option<&'a RenderPass>,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl<'a> RenderPassBeginDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            frame_buffer: None,
            render_pass: None,
            width: 100,
            height: 100,
            x: 0,
            y: 0,
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

    pub fn frame_buffer(mut self, frame_buffer: &'a FrameBuffer) -> Self {
        self.frame_buffer = Some(frame_buffer);
        self
    }

    pub fn render_pass(mut self, render_pass: &'a RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }
}

impl Into<ash::vk::RenderPassBeginInfo> for RenderPassBeginDescriptor<'_> {
    fn into(self) -> ash::vk::RenderPassBeginInfo {
        let mut clear = ClearValue::default();
        unsafe {
            clear.color.float32[0] = 0.0;
            clear.color.float32[1] = 1.0;
            clear.color.float32[2] = 0.0;
            clear.color.float32[3] = 1.0;
        }
        RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.unwrap().render_pass)
            .framebuffer(self.frame_buffer.unwrap().frame_buffer)
            .render_area(
                Rect2D::builder()
                    .extent(
                        Extent2D::builder()
                            .width(self.width)
                            .height(self.height)
                            .build(),
                    )
                    .offset(
                        Offset2D::builder()
                            .x(self.x as i32)
                            .y(self.y as i32)
                            .build(),
                    )
                    .build(),
            )
            .clear_values(&[clear])
            .build()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

impl Into<ash::vk::AttachmentLoadOp> for LoadOp {
    fn into(self) -> ash::vk::AttachmentLoadOp {
        match self {
            LoadOp::Load => ash::vk::AttachmentLoadOp::LOAD,
            LoadOp::Clear => ash::vk::AttachmentLoadOp::CLEAR,
            LoadOp::DontCare => ash::vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreOp {
    Store,
    DontCare,
}

impl Into<ash::vk::AttachmentStoreOp> for StoreOp {
    fn into(self) -> ash::vk::AttachmentStoreOp {
        match self {
            StoreOp::Store => ash::vk::AttachmentStoreOp::STORE,
            StoreOp::DontCare => ash::vk::AttachmentStoreOp::DONT_CARE,
        }
    }
}

pub struct SubPassDescriptor {
    bind_point: BindPoint,
}

impl SubPassDescriptor {
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
    #[inline]
    pub fn new(connecter: DeviceConnecter, descriptor: &SubPassDescriptor) -> Self {
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

pub struct RenderPassDescriptor<'a> {
    load_op: LoadOp,
    store_op: StoreOp,
    subpasses: &'a [SubPass],
}

impl<'a> RenderPassDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            load_op: LoadOp::DontCare,
            store_op: StoreOp::Store,
            subpasses: &[],
        }
    }

    pub fn load_op(mut self, load_op: LoadOp) -> Self {
        self.load_op = load_op;
        self
    }

    pub fn store_op(mut self, store_op: StoreOp) -> Self {
        self.store_op = store_op;
        self
    }

    pub fn subpasses(mut self, subpasses: &'a [SubPass]) -> Self {
        self.subpasses = subpasses;
        self
    }
}

pub struct RenderPass {
    pub(crate) render_pass: ash::vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: &Device, descriptor: &RenderPassDescriptor) -> Self {
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
        let render_pass = unsafe { device.device.create_render_pass(&create_info, None) }.unwrap();
        Self { render_pass }
    }
}
