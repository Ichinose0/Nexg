use std::ffi::CString;

use ash::vk::{
    ColorComponentFlags, CullModeFlags, Extent2D, FrontFace, GraphicsPipelineCreateInfo, Offset2D,
    PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, SampleCountFlags,
    ShaderStageFlags, Viewport,
};

use crate::{Device, RenderPass, Shader};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindPoint {
    Graphics,
}

impl Into<ash::vk::PipelineBindPoint> for BindPoint {
    fn into(self) -> ash::vk::PipelineBindPoint {
        match self {
            BindPoint::Graphics => ash::vk::PipelineBindPoint::GRAPHICS,
        }
    }
}

pub struct PipelineLayoutDescriptor<'a> {
    width: u32,
    height: u32,
    min_depth: f32,
    max_depth: f32,
    renderpass: Option<&'a RenderPass>,
}

impl<'a> PipelineLayoutDescriptor<'a> {
    #[inline]
    pub fn empty() -> Self {
        Self {
            width: 100,
            height: 100,
            min_depth: 0.0,
            max_depth: 1.0,
            renderpass: None,
        }
    }

    #[inline]
    pub fn render_pass(mut self, render_pass: &'a RenderPass) -> Self {
        self.renderpass = Some(render_pass);
        self
    }
}

pub struct PipelineDescriptor<'a> {
    width: u32,
    height: u32,
    max_depth: f32,
    min_depth: f32,
    shaders: &'a [Shader],
}

impl<'a> PipelineDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            width: 100,
            height: 100,
            max_depth: 1.0,
            min_depth: 0.0,
            shaders: &[],
        }
    }

    #[inline]
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    #[inline]
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    #[inline]
    pub fn shaders(mut self, shaders: &'a [Shader]) -> Self {
        self.shaders = shaders;
        self
    }
}

pub struct PipelineLayout<'a> {
    layout: ash::vk::PipelineLayout,
    device: &'a Device,
}

impl<'a> PipelineLayout<'a> {
    #[inline]
    pub fn new(device: &'a Device, descriptor: &PipelineLayoutDescriptor) -> Self {
        let layout_info = PipelineLayoutCreateInfo::builder().set_layouts(&[]).build();
        let layout = unsafe { device.device.create_pipeline_layout(&layout_info, None) }.unwrap();

        Self { layout, device }
    }
}

pub struct Pipeline {
    pub(crate) pipeline: ash::vk::Pipeline,
}

impl Pipeline {
    #[inline]
    pub fn new(
        device: &Device,
        pipeline_layout: PipelineLayout,
        renderpass: &RenderPass,
        descriptor: &PipelineDescriptor,
    ) -> Vec<Self> {
        let name = CString::new("main").unwrap();
        let mut stages = vec![];
        for i in descriptor.shaders {
            match i.kind {
                crate::ShaderKind::Vertex => stages.push(
                    PipelineShaderStageCreateInfo::builder()
                        .stage(ShaderStageFlags::VERTEX)
                        .module(i.inner)
                        .name(name.as_c_str())
                        .build(),
                ),
                crate::ShaderKind::Fragment => stages.push(
                    PipelineShaderStageCreateInfo::builder()
                        .stage(ShaderStageFlags::FRAGMENT)
                        .module(i.inner)
                        .name(name.as_c_str())
                        .build(),
                ),
            }
        }
        let viewports = vec![Viewport::builder()
            .width(descriptor.width as f32)
            .height(descriptor.height as f32)
            .min_depth(descriptor.min_depth)
            .max_depth(descriptor.max_depth)
            .x(0.0)
            .y(0.0)
            .build()];
        let scissors = vec![Rect2D::builder()
            .offset(Offset2D::builder().x(0).y(0).build())
            .extent(
                Extent2D::builder()
                    .width(descriptor.width)
                    .height(descriptor.height)
                    .build(),
            )
            .build()];
        let viewport_state = PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors)
            .build();
        let vertex_input_info = PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&[])
            .vertex_binding_descriptions(&[])
            .build();
        let input_assembly = PipelineInputAssemblyStateCreateInfo::builder()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();
        let rasterizer = PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();
        let multi_sample = PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .build();
        let blend_attachments = vec![PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                ColorComponentFlags::A
                    | ColorComponentFlags::R
                    | ColorComponentFlags::G
                    | ColorComponentFlags::B,
            )
            .blend_enable(false)
            .build()];
        let render_pass = renderpass;
        let blend = PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&blend_attachments)
            .build();
        let layout_info = PipelineLayoutCreateInfo::builder().set_layouts(&[]).build();
        let layout = unsafe { device.device.create_pipeline_layout(&layout_info, None) }.unwrap();
        let create_info = GraphicsPipelineCreateInfo::builder()
            .viewport_state(&viewport_state)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .rasterization_state(&rasterizer)
            .multisample_state(&multi_sample)
            .color_blend_state(&blend)
            .layout(layout)
            .stages(&stages)
            .render_pass(render_pass.render_pass)
            .subpass(0)
            .build();
        let pipelines = unsafe {
            device
                .device
                .create_graphics_pipelines(PipelineCache::null(), &[create_info], None)
        }
        .unwrap();
        pipelines
            .iter()
            .map(|x| Self { pipeline: *x })
            .collect::<Vec<Pipeline>>()
    }
}
