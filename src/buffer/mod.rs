use crate::mem::DeviceMemory;
use crate::{Device, DeviceConnecter, Instance, NxError, NxResult};
use ash::vk::{BufferCreateInfo, BufferUsageFlags, MappedMemoryRange, MemoryMapFlags, SharingMode};
use std::ffi::c_void;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BufferUsage {
    Vertex,
    Index,
}

impl Into<BufferUsageFlags> for BufferUsage {
    fn into(self) -> BufferUsageFlags {
        match self {
            BufferUsage::Vertex => BufferUsageFlags::VERTEX_BUFFER,
            BufferUsage::Index => BufferUsageFlags::INDEX_BUFFER,
        }
    }
}

pub struct BufferDescriptor {
    size: usize,
    usage: BufferUsage,
}

impl BufferDescriptor {
    pub fn empty() -> Self {
        Self {
            size: 0,
            usage: BufferUsage::Vertex,
        }
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn usage(mut self, usage: BufferUsage) -> Self {
        self.usage = usage;
        self
    }
}

pub struct Buffer {
    pub(crate) buffer: ash::vk::Buffer,
    memory: DeviceMemory,
    size: usize,
}

impl Buffer {
    pub fn new(
        instance: &Instance,
        connecter: DeviceConnecter,
        device: &Device,
        descriptor: &BufferDescriptor,
    ) -> NxResult<Self> {
        let create_info = BufferCreateInfo::builder()
            .size(descriptor.size as u64)
            .usage(descriptor.usage.into())
            .sharing_mode(SharingMode::EXCLUSIVE)
            .build();
        let buffer = unsafe { device.device.create_buffer(&create_info, None) }.unwrap();
        let mem_props = connecter.get_memory_properties(instance);
        let mem_req = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let memory =
            match DeviceMemory::alloc_buffer_memory(&device.device, buffer, mem_props, mem_req) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };

        Ok(Self {
            buffer,
            memory,
            size: descriptor.size,
        })
    }

    pub fn size(&self, device: &Device) -> u64 {
        self.memory.size(device)
    }

    pub fn write(&self, device: &Device, data: *const c_void) -> NxResult<()> {
        let mapped_memory = match unsafe {
            device.device.map_memory(
                self.memory.memory,
                0,
                self.size as u64,
                MemoryMapFlags::empty(),
            )
        } {
            Ok(x) => x,
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                ash::vk::Result::ERROR_MEMORY_MAP_FAILED => Err(NxError::MemoryMapFailed),
                _ => Err(NxError::Unknown),
            }?,
        };

        mem_copy(mapped_memory, data, self.size);
        let flush_memory_range = MappedMemoryRange::builder()
            .memory(self.memory.memory)
            .offset(0)
            .size(self.size as u64)
            .build();
        unsafe {
            device
                .device
                .flush_mapped_memory_ranges(&[flush_memory_range])
                .unwrap();
        }

        Ok(())
    }

    pub fn lock(&self, device: &Device) {
        unsafe {
            device.device.unmap_memory(self.memory.memory);
        }
    }
}

pub(crate) fn mem_copy<T>(dst: *mut T, src: *const T, count: usize) {
    unsafe {
        std::ptr::copy_nonoverlapping(src, dst, count);
    }
}
