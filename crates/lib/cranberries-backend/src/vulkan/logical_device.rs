use ash::vk::{self, Buffer};
use ash::vk::Handle;

use std::fmt::Error;
use std::sync::Arc;
use anyhow::{Result, Ok};

use crate::vulkan::instance::Instance;
use crate::vulkan::physical_device::PhysicalDevice;
use crate::vulkan::surface::Surface;
use crate::vulkan::buffer::UniformBufferObject;
use crate::debug;
use crate::vulkan::buffer;

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use super::image::ImageViewDesc;
use super::physical_device;
use super::physical_device::QueueFamilyIndices;








pub struct Device {
    pub raw: ash::Device,
    pub(crate) pdevice: Arc<PhysicalDevice>,
    pub(crate) instance: Arc<Instance>,
    //pub universal_queue: vk::Queue,
    pub _graphics_queue: vk::Queue,
    pub _present_queue: vk::Queue,
    /*
    pub(crate) global_allocator: Arc<Mutex<VulkanAllocator>>,
    pub(crate) immutable_samplers: HashMap<SamplerDesc, vk::Sampler>,
    pub(crate) setup_cb: Mutex<CommandBuffer>,

    pub(crate) crash_tracking_buffer: Buffer,
    pub(crate) crash_marker_names: Mutex<CrashMarkerNames>,

    pub acceleration_structure_ext: khr::AccelerationStructure,
    pub ray_tracing_pipeline_ext: khr::RayTracingPipeline,
    // pub ray_query_ext: khr::RayQuery,
    pub ray_tracing_pipeline_properties: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,

    frames: [Mutex<Arc<DeviceFrame>>; 2],

    ray_tracing_enabled: bool,
    */
    pub setup_cb: Mutex<CommandBuffer>,
    pub frames: [Mutex<Arc<DeviceFrame>>; 2],
}

impl Device {
    pub fn create(
        instance: &Arc<Instance>,
        physical_device: &Arc<PhysicalDevice>,
        surface: &Surface,
        validation: &debug::ValidationInfo,
        device_extensions: &debug::DeviceExtension,
    ) -> Result<Device> {
        let indices = PhysicalDevice::find_queue_family(
            &instance.raw,
            physical_device.raw,
            surface,
        );

        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());


        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for &queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE,
            ..Default::default() // default just enable no feature.
        };

        let requred_validation_layer_raw_names: Vec<CString> = validation
        .required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    
        let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

        let enable_extension_names = device_extensions.get_extensions_raw_names();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if validation.is_enable {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if validation.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            p_enabled_features: &physical_device_features,
        };

        //println!("{:#?}", device_create_info);

        let device: ash::Device = unsafe {
            instance
                .raw
                .create_device(physical_device.raw, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        let graphics_queue = 
            unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };

        let frame0 = DeviceFrame::new(physical_device, &device, &physical_device.queue_family_index);

        let frame1 = DeviceFrame::new(physical_device, &device, &physical_device.queue_family_index);

        let setup_cb = CommandBuffer::create(&device, physical_device.queue_family_index.graphics_family.unwrap()).unwrap();
        
        Ok(Device {
            raw: device,
            pdevice: physical_device.clone(),
            instance: instance.clone(),
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            setup_cb: Mutex::new(setup_cb),
            frames: [
                Mutex::new(Arc::new(frame0)),
                Mutex::new(Arc::new(frame1)),
            ],
        })
    }

    pub fn begin_frame(&self) -> Arc<DeviceFrame> {
        let mut frame0 = self.frames[0].lock().unwrap();
        {
            let frame0: &mut DeviceFrame = Arc::get_mut(&mut frame0).unwrap_or_else(|| {
                panic!("Unable to begin frame: frame data is being held by user code")
            });

            unsafe {
                self.raw
                    .wait_for_fences(
                        &[
                            frame0.main_command_buffer.submit_done_fence,
                            frame0.presentation_command_buffer.submit_done_fence,
                        ], 
                        true, 
                        std::u64::MAX,
                    )
                    .expect("Wait for fence failed.");
            }
        }

        frame0.clone()
    }

    pub fn with_setup_cb(
        &self,
        callback: impl FnOnce(vk::CommandBuffer),
    ) -> Result<()> {
        let cb = self.setup_cb.lock().unwrap();

        unsafe {
            self.raw
                .begin_command_buffer(cb.raw, 
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                    )
                    .unwrap();
        }

        callback(cb.raw);

        unsafe {
            self.raw.end_command_buffer(cb.raw).unwrap();

            let submit_info = 
                vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&cb.raw));

            self.raw
                .queue_submit(
                    self._graphics_queue, 
                    &[submit_info.build()], 
                    vk::Fence::null(),
                )
                .expect("queue submit failed.");

            //neccessary?
            self.raw
                .queue_wait_idle(self._graphics_queue)
                .expect("Failed to wait Queue idle.");

            Ok(self.raw.device_wait_idle()?)
        }
    }

    pub fn finish_frame(&self, frame: Arc<DeviceFrame>) {
        drop(frame);
        
        let mut frame0 = self.frames[0].lock().unwrap();
        let frame0: &mut DeviceFrame = Arc::get_mut(&mut frame0).unwrap_or_else(|| {
            panic!("Unable to finish frame: frame data is being held by user code")
        });

        {
            let mut frame1 = self.frames[1].lock().unwrap();
            let frame1: &mut DeviceFrame = Arc::get_mut(&mut frame1).unwrap();

            //let mut frame2 = self.frames[2].lock();
            //let frame2: &mut DeviceFrame = Arc::get_mut(&mut frame2).unwrap();

            std::mem::swap(frame0, frame1);
            //std::mem::swap(frame1, frame2);
        }
    }

    pub fn create_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        let ubo_layout_bindings = [
        vk::DescriptorSetLayoutBinding {
            // transform uniform
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            // sampler uniform
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        },
        ];

        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: ubo_layout_bindings.len() as u32,
            p_bindings: ubo_layout_bindings.as_ptr(),
        };

        unsafe {
            self.raw
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout!")
        }
    }

    pub fn create_descriptor_pool(
        &self,
        swapchain_images_size: usize,
    ) -> vk::DescriptorPool {
        let pool_sizes = [
            // transform descriptor pool
            vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain_images_size as u32,
        },
        vk::DescriptorPoolSize {
            // sampler descriptor pool
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: swapchain_images_size as u32,
        },
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: swapchain_images_size as u32,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        unsafe {
            self.raw
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        }
    }

    pub fn create_descriptor_sets(
        &self,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<buffer::Buffer>,
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
        swapchain_images_size: usize,
    ) -> Result<Vec<vk::DescriptorSet>> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..swapchain_images_size {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool,
            descriptor_set_count: swapchain_images_size as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            self.raw
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo {
                buffer: uniform_buffers[i].raw,
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as u64,
            }];

            let descriptor_image_infos = [vk::DescriptorImageInfo {
                sampler: texture_sampler,
                image_view: texture_image_view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            }];

            let descriptor_write_sets = [
                vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descriptor_set,
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_image_info: ptr::null(),
                p_buffer_info: descriptor_buffer_info.as_ptr(),
                p_texel_buffer_view: ptr::null(),
            },
                vk::WriteDescriptorSet {
                // sampler uniform
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descriptor_set,
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: descriptor_image_infos.as_ptr(),
                p_buffer_info: ptr::null(),
                p_texel_buffer_view: ptr::null(),
            },];

            unsafe {
                self.raw.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        Ok(descriptor_sets)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
        }
    }
}

pub struct CommandBuffer {
    pub raw: vk::CommandBuffer,
    pub submit_done_fence: vk::Fence,
    //pub device: Arc<Device>,
    pub logical_device: ash::Device,
    pub pool: vk::CommandPool,
}

impl CommandBuffer {
    pub fn create(
        device: &ash::Device,
        queue_family_index: u32
    ) -> Result<CommandBuffer> {
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);

        let pool = unsafe { device.create_command_pool(&pool_create_info, None).unwrap() };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let cb = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap()
        }[0];

        let submit_done_fence = unsafe {
            device.create_fence(
                &vk::FenceCreateInfo::builder()
                    .flags(vk::FenceCreateFlags::SIGNALED)
                    .build(),
                None,
            )
        }?;

        Ok(CommandBuffer {
            raw: cb,
            pool,
            submit_done_fence,
            //device: device.clone(),
            logical_device: device.clone(),
        })
    }
}

// there wrong
impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_command_pool(self.pool, None);

            self.logical_device.destroy_fence(self.submit_done_fence, None);
        }
    }
}

pub struct DeviceFrame {
    pub swapchain_acquired_semaphore: Option<vk::Semaphore>,
    pub rendering_complete_semaphore: Option<vk::Semaphore>,
    pub main_command_buffer: CommandBuffer,
    pub presentation_command_buffer: CommandBuffer,
    //pub pending_resource_releases: Mutex<PendingResourceReleases>,
    //pub profiler_data: VkProfilerData,
}

impl DeviceFrame {
    pub fn new(
        physical_device: &Arc<PhysicalDevice>,
        logical_device: &ash::Device,
        //global_allocator: &mut VulkanAllocator,
        queue_family: &QueueFamilyIndices,
    ) -> DeviceFrame {

        DeviceFrame {
            swapchain_acquired_semaphore: None,
            rendering_complete_semaphore: None,
            main_command_buffer: CommandBuffer::create(logical_device, queue_family.graphics_family.unwrap()).unwrap(),
            presentation_command_buffer: CommandBuffer::create(logical_device, queue_family.graphics_family.unwrap()).unwrap(),
        }
    }
}