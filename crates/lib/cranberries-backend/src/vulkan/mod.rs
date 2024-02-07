pub mod instance;
pub mod logical_device;
pub mod physical_device;
pub mod surface;
pub mod swapchain;
pub mod image;
pub mod shader;
pub mod buffer;

use std::sync::Arc;

use anyhow;
use anyhow::Ok;

use crate::window;
use crate::constants;

pub struct RenderBackend {
    pub device: Arc<logical_device::Device>,
    pub surface: Arc<surface::Surface>,
    pub swapchain: swapchain::Swapchain,
}

#[derive(Clone, Copy)]
pub struct RenderBackendConfig {
    pub swapchain_extent: [u32; 2],
    pub vsync: bool,
    pub graphics_debugging: bool,
    pub device_index: Option<usize>,
}

impl RenderBackend {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        config: RenderBackendConfig,
    ) -> anyhow::Result<RenderBackend> {

        let window = window::init_window(&event_loop, constants::WINDOW_TITLE, constants::WINDOW_WIDTH, constants::WINDOW_HEIGHT);
        
        let instance = 
            Arc::new(instance::Instance::new().unwrap());
        
        let surface =
            Arc::new(surface::Surface::create(&instance, &window).unwrap());
        
        let physical_device = 
            Arc::new(physical_device::PhysicalDevice::create(&instance, &surface).unwrap());
        
        let device = 
            Arc::new(logical_device::Device::create(&instance, &physical_device, &surface,&constants::VALIDATION, &constants::DEVICE_EXTENSIONS).unwrap());

        let swapchain_desc: swapchain::SwapchainDesc = Default::default();
        let swapchain = swapchain::Swapchain::create(&instance, &device, &physical_device, &surface, &physical_device.queue_family_index, swapchain_desc).unwrap();

        anyhow::Ok(RenderBackend {
            device,
            surface,
            swapchain,
        })
    }
}