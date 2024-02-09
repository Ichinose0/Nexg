use ash::vk::ShaderStageFlags;
use ash::{
    util::read_spv,
    vk::{ShaderModule, ShaderModuleCreateInfo},
};
use std::io::{Cursor, Read};

use crate::{Destroy, Device, Instance, NxError, NxResult};

/// Represents a Spir-V intermediate representation
///
/// This structure contains binary data that has been processed so that Vulkan can read it
///
/// # Example
/// ```no_run
/// let fragment_shader = device
/// .create_shader_module(
///     Spirv::new("examples/shader/shader.frag.spv"),
///     ShaderKind::Fragment,
/// )
/// .unwrap();
/// ```
pub struct Spirv {
    pub(crate) data: Vec<u32>,
}

impl Spirv {
    /// Process the spv file so that Vulkan can read it
    /// # Arguments
    ///
    /// * `file` - Spv file path.
    pub fn new(file: &str) -> NxResult<Self> {
        let mut file = std::fs::File::open(file).expect("file open failed");
        let mut buf = Vec::new();
        match file.read_to_end(&mut buf) {
            Ok(_) => {}
            Err(e) => return Err(NxError::IoError(e.to_string())),
        }
        let mut spirv_file = Cursor::new(&buf);
        let spirv = match read_spv(&mut spirv_file) {
            Ok(x) => x,
            Err(e) => return Err(NxError::IoError(e.to_string())),
        };

        Ok(Self { data: spirv })
    }

    /// Process the spv file so that Vulkan can read it
    /// # Arguments
    ///
    /// * `data` - Raw binary data.
    pub fn from_raw(data: &[u8]) -> NxResult<Self> {
        let mut spirv_file = Cursor::new(data);
        let spirv = match read_spv(&mut spirv_file) {
            Ok(x) => x,
            Err(e) => return Err(NxError::IoError(e.to_string())),
        };

        Ok(Self { data: spirv })
    }
}

/// Represents a shader
///
/// It can be created with create_shader_module from Device
#[derive(Clone, Copy, Debug)]
pub struct Shader {
    pub(crate) inner: ShaderModule,
}

impl Shader {
    pub fn new(device: &Device, spirv: &Spirv) -> Shader {
        let shader_create_info = ShaderModuleCreateInfo::builder().code(&spirv.data).build();
        let shader = unsafe {
            device
                .device
                .create_shader_module(&shader_create_info, None)
        }
        .unwrap();
        Shader { inner: shader }
    }
}

impl Destroy for Shader {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_shader_module(self.inner, None);
        }
    }
}

/// Indicates shader type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShaderStage {
    /// Vertex shader
    Vertex,
    /// Fragment shader
    Fragment,
}

impl From<ShaderStage> for ShaderStageFlags {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => ShaderStageFlags::FRAGMENT,
        }
    }
}

/// Register shader information.
/// Required for pipeline creation.
pub struct ShaderStageDescriptor<'a> {
    #[doc(hidden)]
    pub(crate) shaders: Option<&'a Shader>,
    #[doc(hidden)]
    pub(crate) entry_point: &'a str,
    #[doc(hidden)]
    pub(crate) stage: ShaderStage,
}

impl<'a> ShaderStageDescriptor<'a> {
    /// Create an empty descriptor.
    pub fn empty() -> Self {
        Self {
            shaders: None,
            entry_point: "main",
            stage: ShaderStage::Vertex,
        }
    }

    /// Shader to be registered.
    pub fn shaders(mut self, shaders: &'a Shader) -> Self {
        self.shaders = Some(shaders);
        self
    }

    /// Entry point name of shader.
    pub fn entry_point(mut self, entry_point: &'a str) -> Self {
        self.entry_point = entry_point;
        self
    }

    /// Shader stage.
    pub fn stage(mut self, stage: ShaderStage) -> Self {
        self.stage = stage;
        self
    }
}
