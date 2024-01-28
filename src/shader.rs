use ash::{
    util::read_spv,
    vk::{ShaderModule, ShaderModuleCreateInfo},
};
use std::io::{Cursor, Read};
use ash::vk::ShaderStageFlags;

use crate::{Destroy, Device, Fence, Instance};

///Indicates shader type
///
/// # Value Meaning
/// * `Vertex` - Vertex shader.
/// * `Fragment` - Fragment shader.
#[derive(Clone, Copy, Debug)]
pub enum ShaderKind {
    Vertex,
    Fragment,
}

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
    pub fn new(file: &str) -> Self {
        let mut file = std::fs::File::open(file).expect("file open failed");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("file read failed");
        let mut spirv_file = Cursor::new(&buf);
        let spirv = read_spv(&mut spirv_file).unwrap();

        Self { data: spirv }
    }
}

/// Represents a shader
///
/// It can be created with create_shader_module from Device
#[derive(Clone, Copy, Debug)]
pub struct Shader {
    pub(crate) inner: ShaderModule,
    pub(crate) kind: ShaderKind,
}

impl Shader {
    pub fn new(device: &Device, spirv: Spirv, kind: ShaderKind) -> Shader {
        let shader_create_info = ShaderModuleCreateInfo::builder().code(&spirv.data).build();
        let shader = unsafe {
            device
                .device
                .create_shader_module(&shader_create_info, None)
        }
        .unwrap();
        Shader {
            inner: shader,
            kind,
        }
    }
}

impl Destroy for Shader {
    fn instance(&self, instance: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.destroy_shader_module(self.inner, None);
        }
    }
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ShaderStage{
    Vertex,
    Fragment
}

impl Into<ash::vk::ShaderStageFlags> for ShaderStage {
    fn into(self) -> ShaderStageFlags {
        match self {
            ShaderStage::Vertex => ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => ShaderStageFlags::FRAGMENT
        }
    }
}

pub struct ShaderStageDescriptor<'a> {
    pub(crate) shaders: Option<&'a Shader>,
    pub(crate) entry_point: &'a str,
    pub(crate) stage: ShaderStage
}

impl<'a> ShaderStageDescriptor<'a> {
    pub fn empty() -> Self {
        Self {
            shaders: None,
            entry_point: "main",
            stage: ShaderStage::Vertex
        }
    }

    pub fn shaders(mut self,shaders: &'a Shader) -> Self {
        self.shaders = Some(shaders);
        self
    }

    pub fn entry_point(mut self,entry_point: &'a str) -> Self {
        self.entry_point = entry_point;
        self
    }

    pub fn stage(mut self,stage: ShaderStage) -> Self {
        self.stage = stage;
        self
    }
}
