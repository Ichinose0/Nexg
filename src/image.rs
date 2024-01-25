use std::os::raw::c_void;

use crate::{Device, DeviceConnecter, DeviceMemory, Extent3d};
use ash::vk::{
    ComponentMapping, ComponentSwizzle, Format, ImageAspectFlags, ImageCreateInfo, ImageLayout,
    ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageViewCreateInfo, ImageViewType,
    MemoryMapFlags, MemoryPropertyFlags, SampleCountFlags, SharingMode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageType {
    e2D,
    e3D,
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
    array_layers: u32,
}

impl ImageDescriptor {
    #[inline]
    pub fn new() -> Self {
        Self {
            image_type: ImageType::e2D,
            extent: Extent3d::new(100, 100, 1),
            mip_levels: 1,
            array_layers: 1,
        }
    }

    #[inline]
    pub fn image_type(mut self, image_type: ImageType) -> Self {
        self.image_type = image_type;
        self
    }

    #[inline]
    pub fn extent(mut self, extent: Extent3d) -> Self {
        self.extent = extent;
        self
    }
}

pub struct Image<'a> {
    image: ash::vk::Image,
    memory: Option<DeviceMemory>,
    size: Option<u64>,
    device: &'a Device,
}

impl<'a> Image<'a> {
    pub fn create(
        device: &'a Device,
        connecter: DeviceConnecter,
        descriptor: &ImageDescriptor,
    ) -> Self {
        let create_info = ImageCreateInfo::builder()
            .image_type(descriptor.image_type.into())
            .extent(descriptor.extent.into())
            .mip_levels(descriptor.mip_levels)
            .array_layers(descriptor.array_layers)
            .format(Format::R8G8B8A8_UNORM)
            .tiling(ImageTiling::LINEAR)
            .initial_layout(ImageLayout::UNDEFINED)
            .usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .sharing_mode(SharingMode::EXCLUSIVE)
            .samples(SampleCountFlags::TYPE_1)
            .build();
        let image = unsafe { device.device.create_image(&create_info, None) }.unwrap();
        let mem_props = connecter.get_memory_properties();
        let mem_req = unsafe { device.device.get_image_memory_requirements(image) };

        let memory = DeviceMemory::alloc_image_memory(&device.device, image, mem_props, mem_req);
        Self {
            image,
            device,
            size: Some(mem_req.size),
            memory: Some(memory),
        }
    }

    pub fn map_memory(&self) -> *mut c_void {
        unsafe {
            self.device.device.map_memory(
                self.memory.as_ref().unwrap().memory,
                0,
                self.size.unwrap(),
                MemoryMapFlags::empty(),
            )
        }
        .unwrap()
    }

    pub fn create_image_view(&self) -> ImageView {
        ImageView::new(&self.device, &self)
    }

    pub(crate) fn from_raw(image: ash::vk::Image, device: &'a Device) -> Self {
        Self {
            image,
            device,
            memory: None,
            size: None,
        }
    }
}

pub struct ImageView {
    pub(crate) image_view: ash::vk::ImageView,
}

impl ImageView {
    pub(crate) fn new(device: &Device, image: &Image) -> Self {
        let create_info = ImageViewCreateInfo::builder()
            .image(image.image)
            .view_type(ImageViewType::TYPE_2D)
            .format(Format::R8G8B8A8_UNORM)
            .components(
                ComponentMapping::builder()
                    .r(ComponentSwizzle::IDENTITY)
                    .g(ComponentSwizzle::IDENTITY)
                    .b(ComponentSwizzle::IDENTITY)
                    .a(ComponentSwizzle::IDENTITY)
                    .build(),
            )
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();
        let image_view = unsafe { device.device.create_image_view(&create_info, None) }.unwrap();
        Self { image_view }
    }
}
