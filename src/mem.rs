use crate::{Destroy, Device, Instance, NxError, NxResult};
use ash::vk::{
    MemoryAllocateInfo, MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties,
};

pub struct DeviceMemory {
    pub(crate) memory: ash::vk::DeviceMemory,
}

impl DeviceMemory {
    fn alloc(
        device: &ash::Device,
        mem_props: PhysicalDeviceMemoryProperties,
        mem_req: MemoryRequirements,
    ) -> NxResult<ash::vk::DeviceMemory> {
        let mut info = MemoryAllocateInfo::builder().allocation_size(mem_req.size);
        let mut mem_found = false;

        for i in 0..mem_props.memory_type_count {
            if (mem_req.memory_type_bits & (1 << i)) != 0
                && (mem_props.memory_types[i as usize].property_flags
                    & MemoryPropertyFlags::HOST_VISIBLE)
                    .as_raw()
                    != 0
            {
                info.memory_type_index = i;
                mem_found = true;
            }
        }

        if !mem_found {
            panic!("No suitable memory found");
        }

        match unsafe { device.allocate_memory(&info.build(), None) } {
            Ok(x) => Ok(x),
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        }
    }

    pub fn alloc_image_memory(
        device: &ash::Device,
        image: ash::vk::Image,
        mem_props: PhysicalDeviceMemoryProperties,
        mem_req: MemoryRequirements,
    ) -> NxResult<Self> {
        let memory = match Self::alloc(device, mem_props, mem_req) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        match unsafe { device.bind_image_memory(image, memory, 0) } {
            Ok(_) => {}
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        }
        Ok(Self { memory })
    }

    pub fn alloc_buffer_memory(
        device: &ash::Device,
        buffer: ash::vk::Buffer,
        mem_props: PhysicalDeviceMemoryProperties,
        mem_req: MemoryRequirements,
    ) -> NxResult<Self> {
        let memory = match Self::alloc(device, mem_props, mem_req) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        match unsafe { device.bind_buffer_memory(buffer, memory, 0) } {
            Ok(_) => {}
            Err(e) => match e {
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(NxError::OutOfDeviceMemory),
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(NxError::OutOfHostMemory),
                _ => Err(NxError::Unknown),
            }?,
        }
        Ok(Self { memory })
    }

    pub fn size(&self, device: &Device) -> u64 {
        unsafe { device.device.get_device_memory_commitment(self.memory) }
    }
}

impl Destroy for DeviceMemory {
    fn instance(&self, _: &Instance) {}

    fn device(&self, device: &Device) {
        unsafe {
            device.device.free_memory(self.memory, None);
        }
    }
}
