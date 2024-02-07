use anyhow::{Result, Ok};
use crate::vulkan::instance::Instance;
use crate::tools;
use crate::vulkan::surface::Surface;
use crate::constants;
use crate::vulkan::swapchain::Swapchain;
use crate::debug;

use ash::vk::{self, Queue};
//use ash::{vk_version_major, vk_version_minor, vk_version_patch};

use std::sync::Arc;
use std::collections::HashSet;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,

}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}
//to do : finish it
pub struct PhysicalDevice {
    pub instance: Arc<Instance>,
    pub raw: vk::PhysicalDevice,
    pub queue_family_index: QueueFamilyIndices,
    //pub(crate) queue_families: Vec<QueueFamily>,
    /* 
    pub(crate) presentation_requested: bool,
    pub properties: PhysicalDeviceProperties,
    pub memory_properties: PhysicalDeviceMemoryProperties,
    */
}

impl PhysicalDevice {
    pub fn create(
        instance: &Arc<Instance>,
        surface: &Surface,
    ) -> Result<PhysicalDevice> {
        let physical_device = PhysicalDevice::pick_physical_device(&instance.raw, &surface, &constants::DEVICE_EXTENSIONS).unwrap();
        let queue_family_index = PhysicalDevice::find_queue_family(&instance.raw, physical_device, surface);
        
        Ok(PhysicalDevice {
            instance: instance.clone(),
            raw: physical_device,
            queue_family_index,
        })
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface: &Surface,
        required_device_extensions: &debug::DeviceExtension,
    ) -> Result<vk::PhysicalDevice> {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        println!(
            "{} devices (GPU) found with vulkan support.",
            physical_devices.len()
        );

        let result = physical_devices.iter().find(|physical_device| {
            let is_suitable = PhysicalDevice::is_physical_device_suitable(
                instance,
                **physical_device,
                surface,
                required_device_extensions,
            );
    
            // if is_suitable {
            //     let device_properties = instance.get_physical_device_properties(**physical_device);
            //     let device_name = super::tools::vk_to_string(&device_properties.device_name);
            //     println!("Using GPU: {}", device_name);
            // }
    
            is_suitable
        });

        match result {
            Some(p_physical_device) => Ok(*p_physical_device),
            None => panic!("Failed to find a suitable GPU!"),
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
        required_device_extensions: &debug::DeviceExtension,
    ) -> bool {
        let device_properties = 
            unsafe { instance.get_physical_device_properties(physical_device) };
        let device_features = 
            unsafe { instance.get_physical_device_features(physical_device) };
        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        
        // let device_type = match device_properties.device_type {
        //     vk::PhysicalDeviceType::CPU => "Cpu",
        //     vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
        //     vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
        //     vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
        //     vk::PhysicalDeviceType::OTHER => "Unknown",
        //     _ => panic!(),
        // };

        // let device_name = tools::vk_to_string(&device_properties.device_name);
        // println!(
        //     "\tDevice Name: {}, id: {}, type: {}",
        //     device_name, device_properties.device_id, device_type
        // );

        // let major_version = vk_version_major!(device_properties.api_version);
        // let minor_version = vk_version_minor!(device_properties.api_version);
        // let patch_version = vk_version_patch!(device_properties.api_version);

        // println!(
        //     "\tAPI Version: {}.{}.{}",
        //     major_version, minor_version, patch_version
        // );

        // println!("\tSupport Queue Family: {}", device_queue_families.len());
        // println!("\t\tQueue Count | Graphics, Compute, Transfer, Sparse Binding");
        // for queue_family in device_queue_families.iter() {
        //     let is_graphics_support = if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        //     {
        //         "support"
        //     } else {
        //         "unsupport"
        //     };
        //     let is_compute_support = if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
        //         "support"
        //     } else {
        //         "unsupport"
        //     };
        //     let is_transfer_support = if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
        //     {
        //         "support"
        //     } else {
        //         "unsupport"
        //     };
        //     let is_sparse_support = if queue_family
        //         .queue_flags
        //         .contains(vk::QueueFlags::SPARSE_BINDING)
        //     {
        //         "support"
        //     } else {
        //         "unsupport"
        //     };

        //     println!(
        //         "\t\t{}\t    | {},  {},  {},  {}",
        //         queue_family.queue_count,
        //         is_graphics_support,
        //         is_compute_support,
        //         is_transfer_support,
        //         is_sparse_support
        //     );
        // }

        // // there are plenty of features
        // println!(
        //     "\tGeometry Shader support: {}",
        //     if device_features.geometry_shader == 1 {
        //         "Support"
        //     } else {
        //         "Unsupport"
        //     }
        // );

        let indices = PhysicalDevice::find_queue_family(instance, physical_device, surface);

        let is_queue_family_supported = indices.is_complete();
        let is_device_extension_supported = PhysicalDevice::check_device_extension_support(instance, physical_device);

        let is_swapchain_supported = 
            if is_device_extension_supported {
                let swapchain_support = Swapchain::query_swapchain_support(physical_device, surface);
                !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
            }
            else {
                println!("Current device doesn't support swapchain!");
                false
            };

        let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;

        return is_queue_family_supported 
            && is_device_extension_supported 
            && is_swapchain_supported
            && is_support_sampler_anisotropy;
    }

    pub fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface
                    .fns
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface.raw,
                    )
            };

            if queue_family.queue_count > 0 && is_present_support.unwrap() {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        //to do
    //     queue_families.into_iter()
    //     .map()
    //     .filter(|queue_index, queue_family| {
    //         if queue_family.queue_count > 0
    //         && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
    //             Ok(queue_index)
    //         }
    //         else {
    //             None
    //         };
    //     }
    // );
        
        queue_family_indices
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties.")
        };

        let mut available_extension_names = vec![];

        println!("\tAvailable Device Extensions: ");
        for extension in available_extensions.iter() {
            let extension_name = tools::vk_to_string(&extension.extension_name);
            println!(
                "\t\tName: {}, Version: {}",
                extension_name, extension.spec_version
            );

            available_extension_names.push(extension_name);
        }

        let mut required_extensions = HashSet::new();
        for extension in constants::DEVICE_EXTENSIONS.names.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        return required_extensions.is_empty();
    }

}

// fn with_presentation_support(self, surface: &Surface) -> Self {
//     self.into_iter()
//         .filter_map(|mut pdevice| {
//             pdevice.presentation_requested = true;

//             let supports_presentation =
//                 pdevice
//                     .queue_families
//                     .iter()
//                     .enumerate()
//                     .any(|(queue_index, info)| unsafe {
//                         info.properties
//                             .queue_flags
//                             .contains(vk::QueueFlags::GRAPHICS)
//                             && surface
//                                 .fns
//                                 .get_physical_device_surface_support(
//                                     pdevice.raw,
//                                     queue_index as u32,
//                                     surface.raw,
//                                 )
//                                 .unwrap()
//                     });

//             if supports_presentation {
//                 Some(pdevice)
//             } else {
//                 None
//             }
//         })
//         .collect()
// }