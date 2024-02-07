use crate::platforms;
use crate::constants::*;
use crate::tools;
use crate::debug::*;

use anyhow::Ok;
use ash::extensions::ext;
use ash::vk;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr;
use anyhow::Result;

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}

pub struct Instance {
    pub(crate) _entry: ash::Entry,
    pub raw: ash::Instance,
    /* 
    #[allow(dead_code)]
    pub(crate) debug_callback: Option<vk::DebugReportCallbackEXT>,
    #[allow(dead_code)]
    #[allow(deprecated)]
    pub(crate) debug_loader: Option<ext::DebugReport>,
    pub(crate) debug_utils: Option<ash::extensions::ext::DebugUtils>,
    */
    pub(crate) debug_utils: ash::extensions::ext::DebugUtils,
    pub(crate) debug_merssager: vk::DebugUtilsMessengerEXT,
}

impl Instance {
    pub fn new() -> Result<Instance> {
        let entry =  unsafe { ash::Entry::new()? };
        let instance = Instance::create_instance(&entry).unwrap();
        let (debug_utils, debug_merssager) = setup_debug_utils(VALIDATION.is_enable, &entry, &instance);

        Ok(Instance {
            _entry: entry,
            raw: instance,
            debug_utils,
            debug_merssager,
        })
    }

    fn create_instance(entry: &ash::Entry) -> Result<ash::Instance> {
        if VALIDATION.is_enable && Instance::check_validation_layer_support(entry) == false {
            panic!("Validation layers requested, but not available!");
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();

        let app_desc = vk::ApplicationInfo::builder().api_version(vk::make_api_version(0, 1, 2, 0));

        // let app_info = vk::ApplicationInfo {
        //     s_type: vk::StructureType::APPLICATION_INFO,
        //     p_next: ptr::null(),
        //     p_application_name: app_name.as_ptr(),
        //     application_version: APPLICATION_VERSION,
        //     p_engine_name: engine_name.as_ptr(),
        //     engine_version: ENGINE_VERSION,
        //     api_version: API_VERSION,
        // };

        let debug_utils_create_info = populate_debug_messenger_create_info();

        let extension_names = platforms::required_extension_names();

        let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        // let create_info = vk::InstanceCreateInfo {
        //     s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        //     p_next: if VALIDATION.is_enable {
        //         &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
        //             as *const c_void
        //     } else {
        //         ptr::null()
        //     },
        //     flags: vk::InstanceCreateFlags::empty(),
        //     p_application_info: &app_desc,
        //     pp_enabled_layer_names: if VALIDATION.is_enable {
        //         enable_layer_names.as_ptr()
        //     } else {
        //         ptr::null()
        //     },
        //     enabled_layer_count: if VALIDATION.is_enable {
        //         enable_layer_names.len()
        //     } else {
        //         0
        //     } as u32,
        //     pp_enabled_extension_names: extension_names.as_ptr(),
        //     enabled_extension_count: extension_names.len() as u32,
        // };

        let instance_desc = vk::InstanceCreateInfo::builder()
        .application_info(&app_desc)
        .enabled_layer_names(&enable_layer_names)
        .enabled_extension_names(&extension_names);
    
        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&instance_desc, None)
                .expect("Failed to create instance!")
        };

        Ok(instance)
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        // if support validation layer, then return true

        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layers Properties!");

        if layer_properties.len() <= 0 {
            eprintln!("No available layers.");
            return false;
        } else {
            println!("Instance Available Layers: ");
            for layer in layer_properties.iter() {
                let layer_name = tools::vk_to_string(&layer.layer_name);
                println!("\t{}", layer_name);
            }
        }

        for required_layer_name in VALIDATION.required_validation_layers.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name = tools::vk_to_string(&layer_property.layer_name);
                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;
                    break;
                }
            }

            if is_layer_found == false {
                return false;
            }
        }

        true
    }
}

fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if VALIDATION.is_enable {
                self.debug_utils
                    .destroy_debug_utils_messenger(self.debug_merssager, None);
            }
            self.raw.destroy_instance(None);
        }
    }
}