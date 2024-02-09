use std::os::raw::c_void;

use crate::{
    Destroy, Device, DeviceConnecter, DeviceMemory, Extent3d, Instance, NxError, NxResult,
};
use ash::vk::{
    ComponentMapping, ComponentSwizzle, Format, ImageAspectFlags, ImageCreateInfo, ImageLayout,
    ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageViewCreateInfo, ImageViewType,
    MemoryMapFlags, SampleCountFlags, SharingMode,
};

/// Indicates the format of the image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFormat {
    R8G8B8A8Unorm,
    R8G8B8A8Srgb,
    R8G8B8A8Sscaled,
    R8G8B8A8Sint,
    R8G8B8A8Snorm,
    R8G8B8A8Uint,
    B8G8R8Sscaled,
    B8G8R8Srgb,
    B8G8R8Snorm,
    B8G8R8Sint,
    A1R5G5B5UnormPack16,
    A2B10G10R10SintPack32,
    A2B10G10R10SnormPack32,
    A2B10G10R10SscaledPack32,
    A2B10G10R10UintPack32,
    Undefined,
    B8G8R8A8Unorm,
}

impl Into<ImageFormat> for Format {
    fn into(self) -> ImageFormat {
        match self {
            Format::R8G8B8A8_UNORM => ImageFormat::R8G8B8A8Unorm,
            Format::B8G8R8A8_UNORM => ImageFormat::B8G8R8A8Unorm,
            Format::A1R5G5B5_UNORM_PACK16 => ImageFormat::A1R5G5B5UnormPack16,
            Format::A2B10G10R10_SINT_PACK32 => ImageFormat::A2B10G10R10SintPack32,
            Format::A2B10G10R10_SNORM_PACK32 => ImageFormat::A2B10G10R10SnormPack32,
            Format::A2B10G10R10_SSCALED_PACK32 => ImageFormat::A2B10G10R10SscaledPack32,
            Format::A2B10G10R10_UINT_PACK32 => ImageFormat::A2B10G10R10UintPack32,
            Format::B8G8R8_SINT => ImageFormat::B8G8R8Sint,
            Format::B8G8R8_SNORM => ImageFormat::B8G8R8Snorm,
            Format::B8G8R8_SRGB => ImageFormat::B8G8R8Srgb,
            Format::B8G8R8_SSCALED => ImageFormat::B8G8R8Sscaled,
            Format::R8G8B8A8_SINT => ImageFormat::R8G8B8A8Sint,
            Format::R8G8B8A8_SRGB => ImageFormat::R8G8B8A8Srgb,
            Format::R8G8B8A8_SSCALED => ImageFormat::R8G8B8A8Sscaled,
            Format::R8G8B8A8_SNORM => ImageFormat::R8G8B8A8Snorm,
            Format::R8G8B8A8_UINT => ImageFormat::R8G8B8A8Uint,

            _ => ImageFormat::Undefined,
        }
    }
}

impl Into<Format> for ImageFormat {
    fn into(self) -> Format {
        match self {
            ImageFormat::R8G8B8A8Unorm => Format::R8G8B8A8_UNORM,
            ImageFormat::R8G8B8A8Srgb => Format::R8G8B8A8_SRGB,
            ImageFormat::R8G8B8A8Sscaled => Format::R8G8B8A8_SSCALED,
            ImageFormat::R8G8B8A8Sint => Format::R8G8B8A8_SINT,
            ImageFormat::R8G8B8A8Snorm => Format::R8G8B8A8_SNORM,
            ImageFormat::R8G8B8A8Uint => Format::R8G8B8A8_UINT,
            ImageFormat::B8G8R8A8Unorm => Format::B8G8R8A8_UNORM,
            ImageFormat::B8G8R8Sscaled => Format::B8G8R8_SSCALED,
            ImageFormat::B8G8R8Srgb => Format::B8G8R8_SRGB,
            ImageFormat::B8G8R8Snorm => Format::B8G8R8_SNORM,
            ImageFormat::B8G8R8Sint => Format::B8G8R8_SINT,
            ImageFormat::A1R5G5B5UnormPack16 => Format::A1R5G5B5_UNORM_PACK16,
            ImageFormat::A2B10G10R10SintPack32 => Format::A2B10G10R10_SINT_PACK32,
            ImageFormat::A2B10G10R10SnormPack32 => Format::A2B10G10R10_SNORM_PACK32,
            ImageFormat::A2B10G10R10SscaledPack32 => Format::A2B10G10R10_SSCALED_PACK32,
            ImageFormat::A2B10G10R10UintPack32 => Format::A2B10G10R10_UINT_PACK32,

            ImageFormat::Undefined => Format::UNDEFINED,
        }
    }
}

/// Represents the dimension of the image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageDimension {
    D2,
    D3,
}

impl Into<ash::vk::ImageType> for ImageDimension {
    fn into(self) -> ash::vk::ImageType {
        match self {
            ImageDimension::D2 => ash::vk::ImageType::TYPE_2D,
            ImageDimension::D3 => ash::vk::ImageType::TYPE_3D,
        }
    }
}

/// Stores information needed to create a Image.
pub struct ImageDescriptor {
    image_type: ImageDimension,
    extent: Extent3d,
    mip_levels: u32,
    array_layers: u32,
    format: ImageFormat,
}

impl ImageDescriptor {
    #[inline]
    /// Initializes a new descriptor with default values.
    pub const fn new() -> Self {
        Self {
            image_type: ImageDimension::D2,
            extent: Extent3d::new(100, 100, 1),
            mip_levels: 1,
            array_layers: 1,
            format: ImageFormat::R8G8B8A8Unorm,
        }
    }

    #[inline]
    /// Specifies the dimension of the Image.
    pub fn image_type(mut self, image_type: ImageDimension) -> Self {
        self.image_type = image_type;
        self
    }

    #[inline]
    /// Specifies the extent of the Image.
    pub fn extent(mut self, extent: Extent3d) -> Self {
        self.extent = extent;
        self
    }

    #[inline]
    /// Specifies the format of the Image.
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }
}

pub struct Image {
    image: ash::vk::Image,
    memory: Option<DeviceMemory>,

    size: Option<u64>,
}

impl Image {
    /// Create a new Image.
    /// # Arguments
    ///
    /// * `instance` - appropriate Instance.
    /// * `device` - Reference to the appropriate device.
    /// * `connecter` - Appropriate device connecter.
    /// * `descriptor` - Appropriate ImageDescriptor.
    pub fn create(
        instance: &Instance,
        device: &Device,
        connecter: DeviceConnecter,
        descriptor: &ImageDescriptor,
    ) -> NxResult<Self> {
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
        let mem_props = connecter.get_memory_properties(instance);
        let mem_req = unsafe { device.device.get_image_memory_requirements(image) };

        let memory = DeviceMemory::alloc_image_memory(&device.device, image, mem_props, mem_req)?;
        Ok(Self {
            image,
            size: Some(mem_req.size),
            memory: Some(memory),
        })
    }

    #[deprecated(since = "0.1.0", note = "Please use as_raw_data()")]
    /// Maps the memory of the image
    pub fn map_memory(&self, device: &Device) -> NxResult<*mut c_void> {
        match unsafe {
            device.device.map_memory(
                self.memory.as_ref().unwrap().memory,
                0,
                self.size.unwrap(),
                MemoryMapFlags::empty(),
            )
        } {
            Ok(x) => Ok(x),
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                ash::vk::Result::ERROR_MEMORY_MAP_FAILED => Err(NxError::MemoryMapFailed),
                _ => Err(NxError::Unknown),
            }?,
        }
    }

    pub fn as_raw_data(&self, device: &Device, width: u32, height: u32) -> NxResult<Vec<u8>> {
        match self.memory.as_ref() {
            None => Err(NxError::Unknown),
            Some(x) => {
                let data = x.map(device, self.size.unwrap())?;
                let slice: &[u8] = unsafe {
                    std::slice::from_raw_parts(data as *const u8, (width * height * 4) as usize)
                };
                let data = slice.to_vec();
                x.unmap(device);
                Ok(data)
            }
        }
    }

    /// Create a image view from self.
    pub fn create_image_view(
        &self,
        device: &Device,
        descriptor: &ImageViewDescriptor,
    ) -> ImageView {
        ImageView::new(device, self, &descriptor)
    }

    #[doc(hidden)]
    pub(crate) fn from_raw(image: ash::vk::Image) -> Self {
        Self {
            image,
            memory: None,
            size: None,
        }
    }
}

pub struct ImageViewDescriptor {
    format: ImageFormat,
}

impl ImageViewDescriptor {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            format: ImageFormat::R8G8B8A8Unorm,
        }
    }

    #[inline]
    pub const fn format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }
}

pub struct ImageView {
    pub(crate) image_view: ash::vk::ImageView,
}

impl ImageView {
    #[inline]
    #[doc(hidden)]
    pub(crate) fn new(device: &Device, image: &Image, descriptor: &ImageViewDescriptor) -> Self {
        let create_info = ImageViewCreateInfo::builder()
            .image(image.image)
            .view_type(ImageViewType::TYPE_2D)
            .format(descriptor.format.into())
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

impl Destroy for Image {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_image(self.image, None);
        }
        match self.memory.as_ref() {
            None => {}
            Some(x) => {
                device.destroy(x);
            }
        }
    }
}

impl Destroy for ImageView {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_image_view(self.image_view, None);
        }
    }
}
