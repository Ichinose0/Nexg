use std::ffi::CString;

use ash::vk::{
    ColorComponentFlags, CullModeFlags, DescriptorPool, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayoutCreateInfo,
    DescriptorType, Extent2D, Format, FrontFace, GraphicsPipelineCreateInfo, Offset2D,
    PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, Rect2D, SampleCountFlags,
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, Viewport,
};

use crate::{
    Buffer, Destroy, Device, Instance, NxError, NxResult, RenderPass, ShaderStage,
    ShaderStageDescriptor,
};

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum PrimitiveTopology {
    TriangleList,
    TriangleFan,
    TriangleStrip,
    LineStrip
}

impl From<crate::PrimitiveTopology> for ash::vk::PrimitiveTopology {
    fn from(value: crate::PrimitiveTopology) -> Self {
        match value {
            crate::PrimitiveTopology::TriangleList => ash::vk::PrimitiveTopology::TRIANGLE_LIST,
            crate::PrimitiveTopology::TriangleFan => ash::vk::PrimitiveTopology::TRIANGLE_FAN,
            crate::PrimitiveTopology::TriangleStrip => ash::vk::PrimitiveTopology::TRIANGLE_STRIP,
            crate::PrimitiveTopology::LineStrip => ash::vk::PrimitiveTopology::LINE_STRIP,
        }
    }
}

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
    renderpass: Option<&'a RenderPass>,
    set_layout_descriptor: Option<&'a ResourceLayout>,
}

impl<'a> PipelineLayoutDescriptor<'a> {
    #[inline]
    pub fn empty() -> Self {
        Self {
            renderpass: None,
            set_layout_descriptor: None,
        }
    }

    #[inline]
    pub fn resource(mut self, resource: &'a ResourceLayout) -> Self {
        self.set_layout_descriptor = Some(resource);
        self
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

#[derive(Clone, Copy)]
pub enum ResourceType {
    UniformBuffer,
}

impl From<ResourceType> for DescriptorType {
    fn from(value: ResourceType) -> Self {
        match value {
            ResourceType::UniformBuffer => DescriptorType::UNIFORM_BUFFER,
        }
    }
}

pub struct ResourcePoolSize {
    resource_type: ResourceType,
    count: u32,
}

impl ResourcePoolSize {
    pub fn empty() -> Self {
        Self {
            resource_type: ResourceType::UniformBuffer,
            count: 1,
        }
    }
}

pub struct ResourcePoolDescriptor<'a> {
    pool_sizes: &'a [ResourcePoolSize],
    max_sets: u32,
}

impl<'a> ResourcePoolDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            pool_sizes: &[],
            max_sets: 1,
        }
    }

    pub fn pool_sizes(mut self, pool_sizes: &'a [ResourcePoolSize]) -> Self {
        self.pool_sizes = pool_sizes;
        self
    }

    pub fn max_sets(mut self, max_sets: u32) -> Self {
        self.max_sets = max_sets;
        self
    }
}

pub struct ResourcePool {
    pool: DescriptorPool,
}

impl ResourcePool {
    pub fn new(device: &Device, descriptor: &ResourcePoolDescriptor) -> Self {
        let pool_sizes = descriptor
            .pool_sizes
            .iter()
            .map(|x| {
                DescriptorPoolSize::builder()
                    .descriptor_count(x.count)
                    .ty(x.resource_type.into())
                    .build()
            })
            .collect::<Vec<DescriptorPoolSize>>();
        let create_info = DescriptorPoolCreateInfo::builder()
            .max_sets(descriptor.max_sets)
            .pool_sizes(&pool_sizes)
            .build();
        let pool = unsafe { device.device.create_descriptor_pool(&create_info, None) }.unwrap();
        Self { pool }
    }
}

impl Destroy for ResourcePool {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResourceBufferDescriptor<'a> {
    pub(crate) buffer: &'a Buffer,
    pub(crate) offset: u64,
    pub(crate) range: usize,
}

impl<'a> ResourceBufferDescriptor<'a> {
    pub fn new<T>(buffer: &'a Buffer) -> Self {
        let range = std::mem::size_of::<T>();
        Self {
            buffer,
            offset: 0,
            range,
        }
    }
}

pub struct ResourceUpdateDescriptor<'a> {
    pub(crate) resource: &'a Resource,
    pub(crate) binding: u32,
    pub(crate) array_element: u32,
    pub(crate) resource_type: ResourceType,
    pub(crate) buffer_desc: &'a [ResourceBufferDescriptor<'a>],
}

impl<'a> ResourceUpdateDescriptor<'a> {
    pub fn new(resource: &'a Resource) -> Self {
        Self {
            resource,
            binding: 0,
            array_element: 0,
            resource_type: ResourceType::UniformBuffer,
            buffer_desc: &[],
        }
    }

    pub fn buffer_desc(mut self, buffer_desc: &'a [ResourceBufferDescriptor]) -> Self {
        self.buffer_desc = buffer_desc;
        self
    }
}

pub struct Resource {
    pub(crate) descriptor_set: DescriptorSet,
    pool: DescriptorPool,
}

impl Resource {
    pub fn allocate(device: &Device, pool: &ResourcePool, layout: &ResourceLayout) -> Vec<Self> {
        let alloc_info = DescriptorSetAllocateInfo::builder()
            .set_layouts(&[layout.inner])
            .descriptor_pool(pool.pool)
            .build();
        let descriptor_set =
            unsafe { device.device.allocate_descriptor_sets(&alloc_info) }.unwrap();
        descriptor_set
            .iter()
            .map(|x| Self {
                descriptor_set: *x,
                pool: pool.pool,
            })
            .collect()
    }
}

impl Destroy for Resource {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device
                .device
                .free_descriptor_sets(self.pool, &[self.descriptor_set])
                .unwrap();
        }
    }
}

pub struct ResourceLayoutBinding {
    binding: u32,
    desc_type: ResourceType,
    count: u32,
    flags: ShaderStage,
}

impl ResourceLayoutBinding {
    pub fn empty() -> Self {
        Self {
            binding: 0,
            desc_type: ResourceType::UniformBuffer,
            count: 0,
            flags: ShaderStage::Vertex,
        }
    }

    pub fn binding(mut self, binding: u32) -> Self {
        self.binding = binding;
        self
    }

    pub fn resource_type(mut self, desc_type: ResourceType) -> Self {
        self.desc_type = desc_type;
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    pub fn shader_stage(mut self, stage: ShaderStage) -> Self {
        self.flags = stage;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLayout {
    inner: ash::vk::DescriptorSetLayout,
}

impl ResourceLayout {
    pub fn new(device: &Device, descriptor: &[ResourceLayoutBinding]) -> Self {
        let mut bindings = vec![];
        for descriptor in descriptor {
            bindings.push(
                ash::vk::DescriptorSetLayoutBinding::builder()
                    .binding(descriptor.binding)
                    .descriptor_type(descriptor.desc_type.into())
                    .descriptor_count(descriptor.count)
                    .stage_flags(descriptor.flags.into())
                    .build(),
            );
        }
        let create_info = DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .build();
        let inner = unsafe {
            device
                .device
                .create_descriptor_set_layout(&create_info, None)
        }
        .unwrap();
        Self { inner }
    }
}

impl Destroy for ResourceLayout {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device
                .device
                .destroy_descriptor_set_layout(self.inner, None);
        }
    }
}

pub struct PipelineDescriptor<'a> {
    width: u32,
    height: u32,
    max_depth: f32,
    min_depth: f32,
    topology: PrimitiveTopology,
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
            topology: PrimitiveTopology::TriangleList,
            shader_stages: &[],
            input_descriptor: None,
        }
    }

    #[inline]
    pub const fn topology(mut self,topology: PrimitiveTopology) -> Self {
        self.topology = topology;
        self
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

#[derive(Clone, Copy)]
pub struct PipelineLayout {
    pub(crate) layout: ash::vk::PipelineLayout,
}

impl PipelineLayout {
    #[inline]
    pub fn new(device: &Device, descriptor: &PipelineLayoutDescriptor) -> NxResult<Self> {
        let layout_info = PipelineLayoutCreateInfo::builder().set_layouts(&[]);
        let mut layouts = vec![];
        match descriptor.set_layout_descriptor {
            None => {}
            Some(x) => {
                layouts.push(x.inner);
            }
        }
        let layout_info = layout_info.set_layouts(&layouts).build();
        let layout = match unsafe { device.device.create_pipeline_layout(&layout_info, None) } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        };

        Ok(Self { layout })
    }
}

impl Destroy for PipelineLayout {
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
            .topology(descriptor.topology.into())
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
