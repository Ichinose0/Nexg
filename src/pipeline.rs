use ash::vk::{CullModeFlags, Extent2D, FrontFace, Offset2D, PipelineInputAssemblyStateCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, SampleCountFlags, Viewport};

use crate::Device;

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

pub struct PipelineDescriptor {
    width: u32,
    height: u32,
    min_depth: f32,
    max_depth: f32
}

impl PipelineDescriptor {
    #[inline]
    pub fn empty() -> Self {
        Self {
            width: 100,
            height: 100,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

pub struct Pipeline {

}

impl Pipeline {
    #[inline]
    pub fn new(device: &Device,descriptor: &PipelineDescriptor) -> Self {
        let viewports = vec![Viewport::builder().width(descriptor.width as f32).height(descriptor.height as f32).min_depth(descriptor.min_depth).max_depth(descriptor.max_depth).x(0.0).y(0.0).build()];
        let scissors = vec![Rect2D::builder().offset(Offset2D::builder().x(0).y(0).build()).extent(Extent2D::builder().width(descriptor.width).height(descriptor.height).build()).build()];
        let viewport_state = PipelineViewportStateCreateInfo::builder().viewports(&viewports).scissors(&scissors).scissor_count(1).build();
        let vertex_input_info = PipelineVertexInputStateCreateInfo::builder().vertex_attribute_descriptions(&[]).vertex_binding_descriptions(&[]).build();
        let input_assembly = PipelineInputAssemblyStateCreateInfo::builder().topology(PrimitiveTopology::TRIANGLE_LIST).primitive_restart_enable(false).build();
        let rasterizer = PipelineRasterizationStateCreateInfo::builder().depth_clamp_enable(false).rasterizer_discard_enable(false).polygon_mode(PolygonMode::FILL).line_width(1.0).cull_mode(CullModeFlags::BACK).front_face(FrontFace::CLOCKWISE).depth_bias_enable(false).build();
        let multi_sample = PipelineMultisampleStateCreateInfo::builder().sample_shading_enable(false).rasterization_samples(SampleCountFlags::TYPE_1).build();
        
        Self {

        }
    }
}