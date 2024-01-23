use ash::vk::{MemoryAllocateInfo, MemoryRequirements, PhysicalDeviceMemoryProperties};

pub struct DeviceMemory {
    memory: ash::vk::DeviceMemory
}

impl DeviceMemory {
    fn alloc(device: &ash::Device,mem_props: PhysicalDeviceMemoryProperties,mem_req: MemoryRequirements) -> ash::vk::DeviceMemory {
        let mut info = MemoryAllocateInfo::builder().allocation_size(mem_req.size);
        let mut mem_found = false;

        for i in 0..mem_props.memory_type_count {
            if (mem_req.memory_type_bits & (1 << i)) != 0 {
                info.memory_type_index = i;
                mem_found = true;
            }
        }

        if !mem_found {
            panic!("No suitable memory found");
        } 

        unsafe { device.allocate_memory(&info.build(), None) }.unwrap()
    }

    fn alloc_image_memory(device: &ash::Device,image: ash::vk::Image,mem_props: PhysicalDeviceMemoryProperties,mem_req: MemoryRequirements) -> Self {
        let memory = Self::alloc(device,mem_props,mem_req);
        unsafe {
            device.bind_image_memory(image, memory, 0).unwrap();
        }
        Self {
            memory
        }
    }
}