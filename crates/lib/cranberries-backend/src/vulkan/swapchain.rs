use anyhow::Ok;
use anyhow::bail;
use ash::vk;
use ash::extensions::khr;
use ash::vk::DeviceMemory;
use ash::vk::Queue;

use crate::vulkan::logical_device::Device;
use crate::vulkan::surface::Surface;
use crate::vulkan::instance::Instance;
use crate::vulkan::physical_device::PhysicalDevice;
use crate::vulkan::physical_device::QueueFamilyIndices;
use crate::constants;
use crate::vulkan::image::*;
use crate::vulkan::swapchain;

use std::sync::Arc;
use anyhow::Result;
use std::ptr;




#[derive(Clone, Copy, Default)]
pub struct SwapchainDesc {
    pub format: vk::SurfaceFormatKHR,
    pub dims: vk::Extent2D,
    pub vsync: bool,
}

pub struct Swapchain {
    pub fns: khr::Swapchain,
    pub raw: vk::SwapchainKHR,
    pub desc: SwapchainDesc,
    //pub swapchain_images: Vec<vk::Image>,
    //pub swapchain_format: vk::Format,
    //pub swapchain_dims: vk::Extent2D,
    pub images: Vec<Arc<Image>>,
    pub acquire_semaphores: Vec<vk::Semaphore>,

    // TODO: move out of swapchain, make a single semaphore
    pub rendering_finished_semaphores: Vec<vk::Semaphore>,
    pub next_semaphore: usize,

    // Keep a reference in order not to drop after the device
    #[allow(dead_code)]
    pub(crate) device: Arc<Device>,

    // Ditto
    #[allow(dead_code)]
    surface: Arc<Surface>,

    //pub swapchain_images: Vec<SwapchainImage>,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct SwapchainImage {
    pub image: Arc<Image>,
    pub image_index: u32,
    pub acquire_semaphore: vk::Semaphore,
    pub rendering_finished_semaphore: vk::Semaphore,
}

pub enum SwapchainAcquireImageErr {
    RecreateFramebuffer,
}

impl Swapchain {
    pub fn create(
        instance: &Arc<Instance>,
        device: &Arc<Device>,
        physical_device: &Arc<PhysicalDevice>,
        surface: &Arc<Surface>,
        queue_family: &QueueFamilyIndices,
        swapchain_desc: SwapchainDesc,
    ) -> Result<Swapchain> {
        let swapchain_support = Swapchain::query_swapchain_support(physical_device.raw, surface);
        
        let surface_format = Swapchain::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Swapchain::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Swapchain::choose_swapchain_extent(&swapchain_support.capabilities, swapchain_desc.dims);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.raw,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
        };

        println!("{:#?}", swapchain_create_info);

        let fns = ash::extensions::khr::Swapchain::new(&instance.raw, &device.raw);
        let raw = unsafe {
            fns
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        let vk_images = unsafe {
            fns
                .get_swapchain_images(raw)
                .expect("Failed to get Swapchain Images.")
        };
        
        let images: Vec<Arc<Image>> = vk_images
        .into_iter()
        .map(|vk_image| {
            Arc::new(Image {
                raw: vk_image,
                desc: ImageDesc {
                    image_type: ImageType::Tex2d,
                    usage: vk::ImageUsageFlags::STORAGE,
                    flags: vk::ImageCreateFlags::empty(),
                    format: vk::Format::B8G8R8A8_UNORM,
                    extent: [extent.width, extent.height, 0],
                    tiling: vk::ImageTiling::OPTIMAL,
                    mip_levels: 1,
                    array_elements: 1,
                },
                views: Default::default(),
                device: device.raw.clone(),
                device_memory: DeviceMemory::default(),
            })
        })
        .collect();
    
        assert_eq!(image_count, images.len() as u32);

        // let swapchain_images = images
        // .into_iter()
        // .map(|image| {
        //     unsafe {
        //         let acquire_semaphore = device
        //         .raw
        //         .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        //         .unwrap();

        //         let rendering_finished_semaphore = device
        //         .raw
        //         .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        //         .unwrap();

        //         SwapchainImage {
        //             image,
        //             image_index: 0,
        //             acquire_semaphore,
        //             rendering_finished_semaphore,
        //         }
        //     }
        // })
        // .collect();

        let acquire_semaphores = (0..images.len())
        .map(|_| {
            unsafe {
                device
                    .raw
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            }
            .unwrap()
        })
        .collect();

    let rendering_finished_semaphores = (0..images.len())
        .map(|_| {
            unsafe {
                device
                    .raw
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            }
            .unwrap()
        })
        .collect();

        Ok(Swapchain {
            fns,
            raw,
            //swapchain_images,
            device: device.clone(),
            surface: surface.clone(),
            //swapchain_format: surface_format.format,
            //swapchain_dims: extent,
            desc: SwapchainDesc {
                format: surface_format,
                dims: extent,
                vsync: false,
            },
            acquire_semaphores,
            rendering_finished_semaphores,
            next_semaphore: 0,
            images,
        })
    }

    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> SwapChainSupportDetail {
        unsafe {
            let capabilities = surface
                .fns
                .get_physical_device_surface_capabilities(physical_device, surface.raw)
                .expect("Failed to query for surface capabilities.");
            let formats = surface
                .fns
                .get_physical_device_surface_formats(physical_device, surface.raw)
                .expect("Failed to query for surface formats.");
            let present_modes = surface
                .fns
                .get_physical_device_surface_present_modes(physical_device, surface.raw)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>
    ) -> vk::SurfaceFormatKHR {
        // check if list contains most widely used R8G8B8A8 format with nonlinear color space
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        // return the first format from the list
        return available_formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR, desired_extent: vk::Extent2D) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D {
                width: clamp(
                    //constants:: WINDOW_WIDTH,
                    desired_extent.width,         
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    //constants::WINDOW_HEIGHT,
                    desired_extent.height,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub fn acquire_next_image(&mut self) -> std::result::Result<SwapchainImage, vk::Result> {
        let acquire_semaphore = self.acquire_semaphores[self.next_semaphore];
        let rendering_finished_semaphore = self.rendering_finished_semaphores[self.next_semaphore];

        let present_index = unsafe {
            self.fns.acquire_next_image(self.raw, 
                std::u64::MAX, 
                acquire_semaphore, 
                vk::Fence::null(),
            )
        }
        .map(|(val, _)| val as usize);

        match present_index {
            std::result::Result::Ok(present_index) => {
                assert_eq!(present_index, self.next_semaphore);

                self.next_semaphore = (self.next_semaphore + 1) % self.images.len();
                
                std::result::Result::Ok(SwapchainImage {
                    image: self.images[present_index].clone(),
                    image_index: present_index as u32,
                    acquire_semaphore,
                    rendering_finished_semaphore,
                })
            }
            Err(err)
                if err == vk::Result::ERROR_OUT_OF_DATE_KHR
                    || err == vk::Result::SUBOPTIMAL_KHR =>
            {
                // bail!("Could not recreate framebuffer")
                std::result::Result::Err(err)
            }
            err => {
                panic!("Could not acquire swapchain image: {:?}", err);
            }
        }
    }

    pub fn present_image(&self, image: SwapchainImage) {

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(std::slice::from_ref(&image.rendering_finished_semaphore))
            .swapchains(std::slice::from_ref(&self.raw))
            .image_indices(std::slice::from_ref(&image.image_index));

        unsafe {
            match self
                .fns
                .queue_present(self.device._present_queue, &present_info)
            {
                std::result::Result::Ok(_) => (),
                Err(err)
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || err == vk::Result::SUBOPTIMAL_KHR =>
                {
                    // Handled in the next frame
                }
                err => {
                    panic!("Could not present image: {:?}", err);
                }
            }
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for acquire_semaphore in self.acquire_semaphores.iter_mut() {
                self.device
                .raw
                .destroy_semaphore(*acquire_semaphore, None);
            }

            for rendering_finished_semaphore in self.rendering_finished_semaphores.iter_mut() {
                self.device
                .raw
                .destroy_semaphore(*rendering_finished_semaphore, None);
            }

            self.fns
            .destroy_swapchain(self.raw, None);
        }
    }
}