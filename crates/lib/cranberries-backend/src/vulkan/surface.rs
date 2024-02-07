use anyhow::Result;
use ash::{extensions::khr, vk};
use std::sync::Arc;

use crate::{vulkan::instance::Instance, platforms};

pub struct Surface {
    pub(crate) raw: vk::SurfaceKHR,
    pub(crate) fns: khr::Surface,
}

impl Surface {
    pub fn create(
        instance: &Instance,
        window: &winit::window::Window,
    ) -> Result<Surface> {
        let surface =
        unsafe { ash_window::create_surface(&instance._entry, &instance.raw, window, None)? };

        let surface_loader = 
            khr::Surface::new(&instance._entry, &instance.raw);

        Ok(Surface {
            raw: surface,
            fns: surface_loader,
        })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.fns.destroy_surface(self.raw, None);
        }
    }
}