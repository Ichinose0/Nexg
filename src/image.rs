use crate::{Device,Extent3d};
use ash::vk::{Format,ImageCreateInfo,ImageTiling,ImageLayout,ImageUsageFlags,SharingMode,SampleCountFlags};

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ImageType {
    e2D,
    e3D
}

impl Into<ash::vk::ImageType> for ImageType {
    fn into(self) -> ash::vk::ImageType {
        match self {
            ImageType::e2D => ash::vk::ImageType::TYPE_2D,
            ImageType::e3D => ash::vk::ImageType::TYPE_3D,
        }
    }
}

pub struct ImageDescriptor {
    image_type: ImageType,
    extent: Extent3d,
    mip_levels: u32,
    array_layers: u32
}

pub struct Image<'a> {
    image: ash::vk::Image,
    device: &'a Device
}

impl<'a> Image<'a> {
    pub fn create(device: &'a Device,descriptor: &ImageDescriptor) -> Self {
        let create_info = ImageCreateInfo::builder().image_type(descriptor.image_type.into()).extent(descriptor.extent.into()).mip_levels(descriptor.mip_levels).array_layers(descriptor.array_layers).format(Format::R8G8B8A8_UNORM).tiling(ImageTiling::LINEAR).initial_layout(ImageLayout::UNDEFINED).usage(ImageUsageFlags::COLOR_ATTACHMENT).sharing_mode(SharingMode::EXCLUSIVE).samples(SampleCountFlags::TYPE_1).build();
        let image = unsafe { device.create_image(&create_info,None) }.unwrap();
        Self {
            image,
            device
        }
    }
}