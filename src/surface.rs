use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::Instance;

pub struct Surface {
    pub(crate) surface: ash::extensions::khr::Surface,
    pub(crate) khr: ash::vk::SurfaceKHR,
}

impl Surface {
    pub fn new(instance: &Instance,handle: &(impl HasRawWindowHandle+HasRawDisplayHandle)) -> Self {
        let surface = ash::extensions::khr::Surface::new(&instance.entry,&instance.instance);
        let khr = unsafe { ash_window::create_surface(&instance.entry, &instance.instance, handle.raw_display_handle(), handle.raw_window_handle(), None) }.unwrap();
        Self {
            surface,
            khr,
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface.destroy_surface(self.khr, None);
        }
    }
}