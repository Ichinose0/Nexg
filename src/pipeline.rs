use std::ffi::CString;

use ash::vk::{
    ColorComponentFlags, CullModeFlags, Extent2D, Format, FrontFace, GraphicsPipelineCreateInfo,
    Offset2D, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, SampleCountFlags,
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, Viewport,
};

use crate::{Destroy, Device, Instance, NxError, NxResult, RenderPass, ShaderStageDescriptor};

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

pub struct VertexInputBindingDescriptor {
    binding: u32,
    stride: usize,
}

impl VertexInputBindingDescriptor {
    pub fn empty() -> Self {
        Self {
            binding: 0,
            stride: 0,
        }
    }

    pub fn binding(mut self, binding: u32) -> Self {
        self.binding = binding;
        self
    }

    pub fn stride(mut self, stride: usize) -> Self {
        self.stride = stride;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataFormat {
    R32G32SFloat,
    R32G32B32SFloat,
    R32G32B32A32SFloat,
}

impl Into<Format> for DataFormat {
    fn into(self) -> Format {
        match self {
            DataFormat::R32G32SFloat => Format::R32G32_SFLOAT,
            DataFormat::R32G32B32SFloat => Format::R32G32B32_SFLOAT,
            DataFormat::R32G32B32A32SFloat => Format::R32G32B32A32_SFLOAT,
        }
    }
}

pub struct VertexInputAttributeDescriptor {
    binding: u32,
    location: u32,
    offset: usize,
    format: DataFormat,
}

impl VertexInputAttributeDescriptor {
    pub fn empty() -> Self {
        Self {
            binding: 0,
            location: 0,
            offset: 0,
            format: DataFormat::R32G32SFloat,
        }
    }

    pub fn binding(mut self, binding: u32) -> Self {
        self.binding = binding;
        self
    }

    pub fn location(mut self, location: u32) -> Self {
        self.location = location;
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn format(mut self, format: DataFormat) -> Self {
        self.format = format;
        self
    }
}

pub struct PipelineVertexInputDescriptor<'a> {
    binding_desc: &'a [VertexInputBindingDescriptor],
    attribute_desc: &'a [VertexInputAttributeDescriptor],
}

impl<'a> PipelineVertexInputDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            binding_desc: &[],
            attribute_desc: &[],
        }
    }

    pub fn binding_desc(mut self, binding_desc: &'a [VertexInputBindingDescriptor]) -> Self {
        self.binding_desc = binding_desc;
        self
    }

    pub fn attribute_desc(mut self, attribute_desc: &'a [VertexInputAttributeDescriptor]) -> Self {
        self.attribute_desc = attribute_desc;
        self
    }
}

pub struct PipelineDescriptor<'a> {
    width: u32,
    height: u32,
    max_depth: f32,
    min_depth: f32,
    shader_stages: &'a [ShaderStageDescriptor<'a>],
    input_descriptor: Option<&'a PipelineVertexInputDescriptor<'a>>,
}

impl<'a> PipelineDescriptor<'a> {
    pub const fn empty() -> Self {
        Self {
            width: 100,
            height: 100,
            max_depth: 1.0,
            min_depth: 0.0,
            shader_stages: &[],
            input_descriptor: None,
        }
    }

    #[inline]
    pub const fn input_descriptor(
        mut self,
        input_descriptor: &'a PipelineVertexInputDescriptor,
    ) -> Self {
        self.input_descriptor = Some(input_descriptor);
        self
    }

    #[inline]
    pub const fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    #[inline]
    pub const fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    #[inline]
    pub const fn shader_stages(mut self, shader_stages: &'a [ShaderStageDescriptor]) -> Self {
        self.shader_stages = shader_stages;
        self
    }
}

pub struct PipelineLayout<'a> {
    layout: ash::vk::PipelineLayout,
    device: &'a Device,
}

impl<'a> PipelineLayout<'a> {
    #[inline]
    pub fn new(device: &'a Device, _descriptor: &PipelineLayoutDescriptor) -> NxResult<Self> {
        let layout_info = PipelineLayoutCreateInfo::builder().set_layouts(&[]).build();
        let layout = match unsafe { device.device.create_pipeline_layout(&layout_info, None) } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        };

        Ok(Self { layout, device })
    }
}

impl Destroy for PipelineLayout<'_> {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_pipeline_layout(self.layout, None);
        }
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
    ) -> NxResult<Vec<Self>> {
        let mut stages = vec![];
        let name = CString::new("main").unwrap();
        for i in descriptor.shader_stages {
            let create_info = PipelineShaderStageCreateInfo::builder()
                .stage(i.stage.into())
                .module(i.shaders.unwrap().inner)
                .name(name.as_c_str())
                .build();
            stages.push(create_info);
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

        let layout = pipeline_layout.layout;

        let vertex_input_state = match descriptor.input_descriptor {
            None => PipelineVertexInputStateCreateInfo::builder().build(),
            Some(desc) => {
                let mut binding_desc = vec![];
                for i in desc.binding_desc {
                    binding_desc.push(
                        VertexInputBindingDescription::builder()
                            .binding(i.binding)
                            .stride(i.stride as u32)
                            .input_rate(VertexInputRate::VERTEX)
                            .build(),
                    );
                }
                let mut attribute_desc = vec![];
                for i in desc.attribute_desc {
                    attribute_desc.push(
                        VertexInputAttributeDescription::builder()
                            .binding(i.binding)
                            .offset(i.offset as u32)
                            .location(i.location)
                            .format(i.format.into())
                            .build(),
                    );
                }

                PipelineVertexInputStateCreateInfo::builder()
                    .vertex_attribute_descriptions(&attribute_desc)
                    .vertex_binding_descriptions(&binding_desc)
                    .build()
            }
        };

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
            .vertex_input_state(&vertex_input_state)
            .build();

        let pipelines = unsafe {
            device
                .device
                .create_graphics_pipelines(PipelineCache::null(), &[create_info], None)
        }
        .unwrap();
        Ok(pipelines
            .iter()
            .map(|x| Self { pipeline: *x })
            .collect::<Vec<Pipeline>>())
    }
}

impl Destroy for Pipeline {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
