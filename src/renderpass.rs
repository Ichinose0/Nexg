use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Format,
    ImageLayout, RenderPassCreateInfo, SampleCountFlags, SubpassDescription,
};

use crate::{BindPoint, Device, DeviceConnecter};

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

    pub fn subpasses(mut self, subpasses: &'a [SubPass]) -> Self {
        self.subpasses = subpasses;
        self
    }
}

pub struct RenderPass {
    render_pass: ash::vk::RenderPass,
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
            .load_op(descriptor.load_op.into())
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
