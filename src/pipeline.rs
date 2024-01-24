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
