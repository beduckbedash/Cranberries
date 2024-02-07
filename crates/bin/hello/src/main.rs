use cranberries_backend::vulkan::instance::{Instance};
use cranberries_backend::vulkan::physical_device::{PhysicalDevice};
use cranberries_backend::vulkan::logical_device::{Device, self};
use cranberries_backend::vulkan::surface::Surface;
use cranberries_backend::vulkan::swapchain::{Swapchain, SwapchainImage, SwapchainDesc};
use cranberries_backend::vulkan::shader::{RenderPass, FramebufferCacheKey};
use cranberries_backend::vulkan::shader::RenderPassDesc;
use cranberries_backend::vulkan::shader::RenderPassAttachmentDesc;
use cranberries_backend::vulkan::{buffer, shader};
use cranberries_backend::vulkan::buffer::{Buffer, UniformBufferObject};
// use cranberries_backend::vulkan::logical_device::CommandBuffer;
// use cranberries_backend::vulkan::swapchain::SwapchainDesc;
use cranberries_backend::vulkan::image::{ImageViewDesc, Image};

use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

use cranberries_backend::constants;
use cranberries_backend;
use cranberries_backend::window::{self, WindowApp};
use std::os::windows::raw;
use std::sync::Arc;
use cranberries_backend::constants::*;
use ash::vk::{self, CommandBuffer, DescriptorSet, Handle};
use std::{ptr, num, clone};
use ash::vk::ImageViewType;
// use ash::version::DeviceV1_0;
//use ash;

use std::path::Path;

/* 
fn init_window(event_loop: &EventLoop<()>) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(event_loop)
        .expect("Failed to create window.")
}
*/


struct TestApp{
    _window: winit::window::Window,
    _instance: Arc<Instance>,
    _graphics_pipeline: ash::vk::Pipeline,
    _pipeline_layout: ash::vk::PipelineLayout,
    //_command_buffers: Vec<CommandBuffer>,
    _physical_device: Arc<PhysicalDevice>,
    _logical_device: Arc<Device>,
    _swapchain: Swapchain,
    _framebuffers: Vec<ash::vk::Framebuffer>,
    _render_pass: Arc<RenderPass>,
    _surface: Arc<Surface>,
    // image_available_semaphores: Vec<ash::vk::Semaphore>,
    // render_finished_semaphores: Vec<ash::vk::Semaphore>,
    // in_flight_fences: Vec<ash::vk::Fence>,
    // current_frame: usize,

    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    _uniform_buffers: Vec<Buffer>,
    _descriptor_sets: Vec<vk::DescriptorSet>,
    _uniform_transform: UniformBufferObject,
    _texture_image: Image,
    _depth_image: Image,
}

impl TestApp{
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let _window = window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
        let _instance = 
            Arc::new(Instance::new().unwrap());
        let _surface =
            Arc::new(Surface::create(&_instance, &_window).unwrap());
        let _physical_device = 
            Arc::new(PhysicalDevice::create(&_instance, &_surface).unwrap());
        let _logical_device = 
            Arc::new(Device::create(&_instance, &_physical_device, &_surface,&constants::VALIDATION, &constants::DEVICE_EXTENSIONS).unwrap());
        // let _swapchain =
        //     Arc::new(Swapchain::create(&_instance, &_logical_device, &_physical_device, &_surface, &_physical_device.queue_family_index).unwrap());

        let mut swapchain_desc: SwapchainDesc = Default::default();
        swapchain_desc.dims = vk::Extent2D {
            width: constants::WINDOW_WIDTH,
            height: constants::WINDOW_HEIGHT,
        };

        let _swapchain = Swapchain::create(&_instance, &_logical_device, &_physical_device, &_surface, &_physical_device.queue_family_index, swapchain_desc).unwrap();

        // let _swapchain_imageviews = 
        //     Device::create_image_views(&_logical_device, _swapchain.swapchain_format, &_swapchain.swapchain_images, _swapchain.swapchain_dims);
        //let _render_pass_attachment_desc = RenderPassAttachmentDesc::new(_swapchain.swapchain_format);
        // let _render_pass = RenderPass::create_render_pass(&_logical_device, _swapchain.desc.format.format);

        // let (_graphics_pipeline, _pipeline_layout) = shader::create_graphics_pipline(
        //     &_logical_device, _render_pass.raw, _swapchain.desc.dims,
        // );

        // let _swapchain_framebuffers = shader::create_framebuffers(
        //     &_logical_device, _render_pass.raw, &_swapchain_imageviews, _swapchain.swapchain_dims);

        // let _swapchain_framebuffers: Vec<vk::Framebuffer> = Vec::new();
        // let num = _swapchain_framebuffers.len();

        //println!("_swapchain_framebuffers len {}", num);

        // let mut _command_buffers = Vec::new();
        // for number in 0..num {
        //     _command_buffers.push(CommandBuffer::create(&_logical_device, _physical_device.queue_family_index.graphics_family.unwrap()).unwrap());
        // }
        //Arc::new(CommandBuffer::create(&_logical_device, _physical_device.queue_family_index.graphics_family.unwrap()).unwrap());


        //let sync_objects = TestApp::create_sync_objects(&_logical_device.raw);

        //new version
        // let _render_pass = RenderPass::create_render_pass(&_logical_device, _swapchain.desc.format.format);

        // let (_graphics_pipeline, _pipeline_layout) = shader::create_graphics_pipline(
        //     &_logical_device, _render_pass.raw, _swapchain.desc.dims,
        // );

        // let image_view_desc = ImageViewDesc {
        //     view_type: Some(vk::ImageViewType::TYPE_2D),
        //     format: Some(_swapchain.desc.format.format),
        //     aspect_mask: vk::ImageAspectFlags::default(),
        //     base_mip_level: 0 as u32,
        //     level_count: Some(1 as u32),
        // };

        // let image_desc = cranberries_backend::vulkan::image::ImageDesc::new(_swapchain.desc.format.format, cranberries_backend::vulkan::image::ImageType::Tex2d, [_swapchain.desc.dims.width, _swapchain.desc.dims.height,0]);

        // let mut framebuffers = vec![];

        // for image in _swapchain.images.iter() {
        //     let image_view = _logical_device.create_image_view(image_view_desc, &image_desc, image.raw).unwrap();
        //     let attachments = [image_view];

        //     let framebuffer_create_info = vk::FramebufferCreateInfo {
        //         s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        //         p_next: ptr::null(),
        //         flags: vk::FramebufferCreateFlags::empty(),
        //         render_pass: _render_pass.raw,
        //         attachment_count: attachments.len() as u32,
        //         p_attachments: attachments.as_ptr(),
        //         width: _swapchain.desc.dims.width,
        //         height: _swapchain.desc.dims.height,
        //         layers: 1,
        //     };
    
        //     let framebuffer = unsafe {
        //         _logical_device
        //             .raw
        //             .create_framebuffer(&framebuffer_create_info, None)
        //             .expect("Failed to create Framebuffer!")
        //     };
    
        //     framebuffers.push(framebuffer);

        // }

        // new version 2
        let mut depth_format = vk::Format::D32_SFLOAT;
        let candidate_formats = [
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,];

        for &format in candidate_formats.iter() {
            let format_properties =
                unsafe { _instance.raw.get_physical_device_format_properties(_physical_device.raw, format) };
            if vk::ImageTiling::OPTIMAL == vk::ImageTiling::LINEAR
                && format_properties.linear_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            {
                depth_format = format.clone();
            } else if vk::ImageTiling::OPTIMAL == vk::ImageTiling::OPTIMAL
                && format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            {
                depth_format = format.clone();
            }
        }

        let _render_pass = Arc::new(RenderPass::create_render_pass(
            &_logical_device,
            RenderPassDesc {
                color_attachments: &[
                    // view-space geometry normal; * 2 - 1 to decode
                    RenderPassAttachmentDesc::create_color_attachment(_swapchain.desc.format.format)
                        .clear_input(),
                ],
                depth_attachment: Some(RenderPassAttachmentDesc::create_depth_attachment(depth_format)),
            },
        ).unwrap());


        // let image_view_desc = ImageViewDesc {
        //     view_type: Some(ImageViewType::TYPE_2D),
        //     format: Some(_swapchain.desc.format.format),
        //     aspect_mask: vk::ImageAspectFlags::default(),
        //     base_mip_level: 0 as u32,
        //     level_count: Some(1 as u32),
        // };

        // let image_desc = cranberries_backend::vulkan::image::ImageDesc::new(_swapchain.desc.format.format, cranberries_backend::vulkan::image::ImageType::Tex2d, [_swapchain.desc.dims.width, _swapchain.desc.dims.height,0]);

        
        // let key = FramebufferCacheKey::new(
        //     [_swapchain.desc.dims.width, _swapchain.desc.dims.height], 
        //     [image_desc].iter(), 
        //     Some(&image_desc));

        // _render_pass.framebuffer_cache.get_or_create(&_logical_device.raw, key);
        
        //descriptor set
        let _ubo_layout = _logical_device.create_descriptor_set_layout();

        //image 
        let images = _swapchain.images.iter().map(|image| {
            image.raw
        })
        .collect::<Vec<_>>();

        let image_views = _logical_device.create_image_views(_swapchain.desc.format.format, &images, _swapchain.desc.dims);

        let mut image_desc = cranberries_backend::vulkan::image::ImageDesc::create(_swapchain.desc.format.format, cranberries_backend::vulkan::image::ImageType::Tex2d, [_swapchain.desc.dims.width, _swapchain.desc.dims.height,0]);
        image_desc.flags = vk::ImageCreateFlags::default();
        image_desc.usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;

        //depth resources
        //let depth_format = vk::Format::D32_SFLOAT_S8_UINT;

        let mut depth_image_desc = cranberries_backend::vulkan::image::ImageDesc::create(depth_format, cranberries_backend::vulkan::image::ImageType::Tex2d, [_swapchain.desc.dims.width, _swapchain.desc.dims.height,0]);
        depth_image_desc.flags = vk::ImageCreateFlags::default();
        depth_image_desc.usage = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;

        let device_memory_properties = unsafe {
            _instance.raw.get_physical_device_memory_properties(_physical_device.raw)
        };

        let _depth_image = _logical_device.create_image(depth_image_desc, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL, 
            &device_memory_properties).unwrap();

        let depth_image_view_desc = ImageViewDesc {
            view_type: Some(vk::ImageViewType::TYPE_2D),
            format: Some(depth_format),
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0 as u32,
            level_count: Some(1 as u32),
        };

        //println!("depth_image_desc: {:#?}", depth_image_desc);
        //println!("depth_image_view_desc: {:#?}", depth_image_view_desc);


        let depth_image_view = _logical_device.create_image_view(depth_image_view_desc, &_depth_image.desc, _depth_image.raw).unwrap();

        let _framebuffers = image_views.iter().map(|image_view| {
            let _framebuffer_cachekey = FramebufferCacheKey::new(
                [_swapchain.desc.dims.width, _swapchain.desc.dims.height], 
                [image_desc].iter(),
                Some(&depth_image_desc), 
               );

            _render_pass.framebuffer_cache.get_or_create(&_logical_device.raw, _framebuffer_cachekey, *image_view, depth_image_view, _render_pass.raw).unwrap()
        }).collect::<Vec<_>>();

        let (_graphics_pipeline, _pipeline_layout) = shader::create_graphics_pipline(
            &_logical_device, _render_pass.raw, _swapchain.desc.dims,
            _ubo_layout,
        );

        let physical_device_memory_properties =
        unsafe { _instance.raw.get_physical_device_memory_properties(_physical_device.raw) };

        let _texture_image = _logical_device.create_texture_image(&physical_device_memory_properties, &Path::new(constants::TEXTURE_PATH)).unwrap();
        
        
        let mut _texture_image_view_desc = ImageViewDesc::default();
        _texture_image_view_desc.view_type = Some(vk::ImageViewType::TYPE_2D);
        _texture_image_view_desc.format = Some(vk::Format::R8G8B8A8_SRGB);
        _texture_image_view_desc.aspect_mask = vk::ImageAspectFlags::COLOR;

        let _texture_image_view = _logical_device.create_image_view(_texture_image_view_desc, &_texture_image.desc, _texture_image.raw).unwrap();
        
        let _texture_sampler = _logical_device.create_texture_sampler().unwrap();
        
        
        
        let _vertex_buffer = _logical_device.create_vertex_buffer(constants::RECT_TEX_COORD_VERTICES_DATA_2).unwrap();

        let _index_buffer = _logical_device.create_index_buffer(constants::RECT_TEX_COORD_INDICES_DATA_2).unwrap();

        let _uniform_buffers = _logical_device.create_uniform_buffer(_swapchain.images.len()).unwrap();
        
        let _descriptor_pool = _logical_device.create_descriptor_pool(_swapchain.images.len());
        let _descriptor_sets = _logical_device.create_descriptor_sets(
            _descriptor_pool, 
            _ubo_layout, 
            &_uniform_buffers,
            _texture_image_view,
            _texture_sampler, 
            _swapchain.images.len()).unwrap();

        TestApp {
            _window,
            _instance,
            _graphics_pipeline,
            _pipeline_layout,
            //_command_buffers,
            _physical_device,
            _logical_device,
            _swapchain,
            _framebuffers,
            _render_pass,
            _surface,
            // image_available_semaphores: sync_objects.image_available_semaphores,
            // render_finished_semaphores: sync_objects.render_finished_semaphores,
            // in_flight_fences: sync_objects.inflight_fences,
            // current_frame: 0,

            _vertex_buffer,
            _index_buffer,
            _uniform_buffers,
            _descriptor_sets,

            _uniform_transform: UniformBufferObject {
                // model: cgmath::Matrix4::<f32> {
                //     x: cgmath::Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
                //     y: cgmath::Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
                //     z: cgmath::Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
                //     w: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                // },
                model: cgmath::Matrix4::from_angle_z(cgmath::Deg(90.0)),
                view: cgmath::Matrix4::look_at(
                    cgmath::Point3::new(2.0, 2.0, 2.0),
                    cgmath::Point3::new(0.0, 0.0, 0.0),
                    cgmath::Vector3::new(0.0, 0.0, 1.0),
                ),
                proj: {
                    let mut proj = cgmath::perspective(
                        cgmath::Deg(45.0),
                        constants::WINDOW_WIDTH as f32
                            / constants::WINDOW_HEIGHT as f32,
                        0.1,
                        10.0,
                    );
                    proj[1][1] = proj[1][1] * -1.0;
                    proj
                },
            },
            _texture_image,
            _depth_image,
        }
    }
}

const MAX_FRAMES_IN_FLIGHT: usize = 2;

impl WindowApp for TestApp{
    // fn draw_frame(&mut self, delta_time: f32){
    //     //println!("draw_frame");

    //     let wait_fences = [self.in_flight_fences[self.current_frame]];

    //     let (image_index, _is_sub_optimal) = unsafe {
    //         self._logical_device
    //             .raw
    //             .wait_for_fences(&wait_fences, true, std::u64::MAX)
    //             .expect("Failed to wait for Fence!");

    //         self._swapchain
    //             .fns
    //             .acquire_next_image(
    //                 self._swapchain.raw,
    //                 std::u64::MAX,
    //                 self.image_available_semaphores[self.current_frame],
    //                 ash::vk::Fence::null(),
    //             )
    //             .expect("Failed to acquire next image.")
    //     };

    //     let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
    //     let wait_stages = [ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    //     let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

    //     //pervious
    //     // self.push_simple_commands(
    //     //     //self._command_buffers[image_index as usize].raw, 
    //     //     self._graphics_pipeline, 
    //     //     &self._swapchain_framebuffers[image_index as usize], 
    //     //     self._render_pass.raw, 
    //     //     self._swapchain.swapchain_dims);

        

    //     let submit_infos = [ash::vk::SubmitInfo {
    //         s_type: ash::vk::StructureType::SUBMIT_INFO,
    //         p_next: ptr::null(),
    //         wait_semaphore_count: wait_semaphores.len() as u32,
    //         p_wait_semaphores: wait_semaphores.as_ptr(),
    //         p_wait_dst_stage_mask: wait_stages.as_ptr(),
    //         command_buffer_count: 1,
    //         p_command_buffers: &self._command_buffers[image_index as usize].raw,
    //         signal_semaphore_count: signal_semaphores.len() as u32,
    //         p_signal_semaphores: signal_semaphores.as_ptr(),
    //     }];

    //     unsafe {
    //         self._logical_device
    //             .raw
    //             .reset_fences(&wait_fences)
    //             .expect("Failed to reset Fence!");

    //         self._logical_device
    //             .raw
    //             .queue_submit(
    //                 self._logical_device._graphics_queue,
    //                 &submit_infos,
    //                 self.in_flight_fences[self.current_frame],
    //             )
    //             .expect("Failed to execute queue submit.");
    //     }

    //     let swapchains = [self._swapchain.raw];

    //     let present_info = ash::vk::PresentInfoKHR {
    //         s_type: ash::vk::StructureType::PRESENT_INFO_KHR,
    //         p_next: ptr::null(),
    //         wait_semaphore_count: 1,
    //         p_wait_semaphores: signal_semaphores.as_ptr(),
    //         swapchain_count: 1,
    //         p_swapchains: swapchains.as_ptr(),
    //         p_image_indices: &image_index,
    //         p_results: ptr::null_mut(),
    //     };

    //     unsafe {
    //         self._swapchain
    //             .fns
    //             .queue_present(self._logical_device._present_queue, &present_info)
    //             .expect("Failed to execute queue present.");
    //     }

    //     self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

    // }

    fn draw_frame(&mut self, delta_time: f32){

        //to do fix
        //println!("draw frame");

        let current_frame = self._logical_device.begin_frame();
        
        //let mut a = self._swapchain;
        let result = self._swapchain.acquire_next_image();
        let mut swapchain_image: SwapchainImage = match result {
            std::result::Result::Ok(res) => {
                res
            },
            std::result::Result::Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR =>{ //} | vk::Result::SUBOPTIMAL_KHR=> {
                        //self.recreate_swapchain();
                        return;
                    },
                    _ => panic!("Failed to execute queue present."),
                    _ => panic!("Failed to acquire Swap Chain Image!"),
            },
        };

        //println!("swapchain index {}", swapchain_image.image_index);
        self._logical_device.test_update_uniform_buffer(delta_time, 
            self._swapchain.desc.dims, 
            &mut self._uniform_buffers[swapchain_image.image_index as usize],
            &mut self._uniform_transform);

        //let _device = self._logical_device.clone();

        // let callback = |command_buffer|{

        //     //let command_buffer = current_frame.main_command_buffer.raw;

        //     let clear_values = [ash::vk::ClearValue {
        //         color: ash::vk::ClearColorValue {
        //             float32: [0.0, 0.0, 0.0, 1.0],
        //         },
        //     }];
    
        //     // let image_view_desc = ImageViewDesc {
        //     //     view_type: Some(ImageViewType::TYPE_2D),
        //     //     format: Some(self._swapchain.desc.format.format),
        //     //     aspect_mask: vk::ImageAspectFlags::COLOR,
        //     //     base_mip_level: 0 as u32,
        //     //     level_count: Some(1 as u32),
        //     // };
    
        //     // let mut image_desc = cranberries_backend::vulkan::image::ImageDesc::new(self._swapchain.desc.format.format, cranberries_backend::vulkan::image::ImageType::Tex2d, [self._swapchain.desc.dims.width, self._swapchain.desc.dims.height,0]);
        //     // image_desc.flags = vk::ImageCreateFlags::default();
        //     // image_desc.usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
    
        //     // let image_view = self._logical_device.create_image_view(image_view_desc, &image_desc, swapchain_image.image.raw).unwrap();
            
        //     // //let image_views = self._logical_device.create_image_views(self._swapchain.desc.format.format, self._swapchain.images, self._swapchain.desc.dims);
            
        //     // let _framebuffer_cachekey = FramebufferCacheKey::new_simple(
        //     //     [self._swapchain.desc.dims.width, self._swapchain.desc.dims.height], 
        //     //     [image_desc].iter(), 
        //     //    );
    
        //     // let framebuffer = self._render_pass.framebuffer_cache.get_or_create(&self._logical_device.raw, _framebuffer_cachekey, image_view).unwrap();
    
        //     let render_pass_begin_info = ash::vk::RenderPassBeginInfo {
        //         s_type: ash::vk::StructureType::RENDER_PASS_BEGIN_INFO,
        //         p_next: ptr::null(),
        //         render_pass: self._render_pass.raw,
        //         framebuffer: self._framebuffers[self._swapchain.next_semaphore as usize],
        //         render_area: ash::vk::Rect2D {
        //             offset: ash::vk::Offset2D { x: 0, y: 0 },
        //             extent: self._swapchain.desc.dims,
        //         },
        //         clear_value_count: clear_values.len() as u32,
        //         p_clear_values: clear_values.as_ptr(),
        //     };
    
        //     unsafe {
        //         self._logical_device.raw.cmd_begin_render_pass(
        //             command_buffer,
        //             &render_pass_begin_info,
        //             vk::SubpassContents::INLINE,
        //         );
        //         self._logical_device.raw.cmd_bind_pipeline(
        //             command_buffer,
        //             vk::PipelineBindPoint::GRAPHICS,
        //             self._graphics_pipeline,
        //         );
        //         self._logical_device.raw.cmd_draw(command_buffer, 3, 1, 0, 0);
    
        //         self._logical_device.raw.cmd_end_render_pass(command_buffer);
        //     }
        // };        

        let command_buffer = &current_frame.main_command_buffer;
        //let cb = self._logical_device.setup_cb.lock().unwrap();

        unsafe {
            // self._logical_device.raw
            //     .reset_command_buffer(command_buffer.raw, vk::CommandBufferResetFlags::default())
            //     .unwrap();

            self._logical_device.raw
                .begin_command_buffer(command_buffer.raw, 
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE),//ONE_TIME_SUBMIT
                    )
                    .unwrap();
        }

        //let command_buffer = cb.raw; 

        let clear_values = [
            ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            ash::vk::ClearValue {
                // clear value for depth buffer
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
            ];

        // let image_view_desc = ImageViewDesc {
        //     view_type: Some(ImageViewType::TYPE_2D),
        //     format: Some(self._swapchain.desc.format.format),
        //     aspect_mask: vk::ImageAspectFlags::COLOR,
        //     base_mip_level: 0 as u32,
        //     level_count: Some(1 as u32),
        // };

        // let mut image_desc = cranberries_backend::vulkan::image::ImageDesc::new(self._swapchain.desc.format.format, cranberries_backend::vulkan::image::ImageType::Tex2d, [self._swapchain.desc.dims.width, self._swapchain.desc.dims.height,0]);
        // image_desc.flags = vk::ImageCreateFlags::default();
        // image_desc.usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;

        // let image_view = self._logical_device.create_image_view(image_view_desc, &image_desc, swapchain_image.image.raw).unwrap();
        
        // //let image_views = self._logical_device.create_image_views(self._swapchain.desc.format.format, self._swapchain.images, self._swapchain.desc.dims);
        
        // let _framebuffer_cachekey = FramebufferCacheKey::new_simple(
        //     [self._swapchain.desc.dims.width, self._swapchain.desc.dims.height], 
        //     [image_desc].iter(), 
        //    );

        // let framebuffer = self._render_pass.framebuffer_cache.get_or_create(&self._logical_device.raw, _framebuffer_cachekey, image_view).unwrap();

        let render_pass_begin_info = ash::vk::RenderPassBeginInfo {
            s_type: ash::vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: self._render_pass.raw,
            //framebuffer: self._framebuffers[self._swapchain.next_semaphore as usize],
            framebuffer: self._framebuffers[swapchain_image.image_index as usize],
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: self._swapchain.desc.dims,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            self._logical_device.raw.cmd_begin_render_pass(
                command_buffer.raw,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self._logical_device.raw.cmd_bind_pipeline(
                command_buffer.raw,
                vk::PipelineBindPoint::GRAPHICS,
                self._graphics_pipeline,
            );

            //vertex
            let vertex_buffers = [self._vertex_buffer.raw];
            let offsets = [0_u64];
            let descriptor_sets_to_bind = [self._descriptor_sets[swapchain_image.image_index as usize]];

            self._logical_device.raw.cmd_bind_vertex_buffers(command_buffer.raw, 0, &vertex_buffers, &offsets);
            //vertex

            //index
            self._logical_device.raw.cmd_bind_index_buffer(command_buffer.raw, self._index_buffer.raw, 0, vk::IndexType::UINT32);
            //index

            // descriptor set
            self._logical_device.raw.cmd_bind_descriptor_sets(
                command_buffer.raw,
                vk::PipelineBindPoint::GRAPHICS,
                self._pipeline_layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );
            //descriptor set

            //index draw
            self._logical_device.raw.cmd_draw_indexed(command_buffer.raw, constants::RECT_TEX_COORD_INDICES_DATA_2.len() as u32, 1, 0, 0, 0);

            //draw
           // self._logical_device.raw.cmd_draw(command_buffer.raw, constants::_VERTICES_DATA.len() as u32, 1, 0, 0);

            self._logical_device.raw.cmd_end_render_pass(command_buffer.raw);
        }

        unsafe {
            self._logical_device.raw.end_command_buffer(command_buffer.raw).unwrap();
        }

        // let submit_infos = [vk::SubmitInfo {
        //     s_type: vk::StructureType::SUBMIT_INFO,
        //     p_next: ptr::null(),
        //     wait_semaphore_count: wait_semaphores.len() as u32,
        //     p_wait_semaphores: wait_semaphores.as_ptr(),
        //     p_wait_dst_stage_mask: wait_stages.as_ptr(),
        //     command_buffer_count: 1,
        //     p_command_buffers: &command_buffer.raw,
        //     signal_semaphore_count: signal_semaphores.len() as u32,
        //     p_signal_semaphores: signal_semaphores.as_ptr(),
        // }];

        //two buffer
        // let submit_infos =[vk::SubmitInfo {
        //     s_type: vk::StructureType::SUBMIT_INFO,
        //     p_next: ptr::null(),
        //     wait_semaphore_count: u32::default(),
        //     p_wait_semaphores: std::ptr::null(),
        //     p_wait_dst_stage_mask: std::ptr::null(),
        //     command_buffer_count: 1,
        //     p_command_buffers: &command_buffer.raw,
        //     signal_semaphore_count: u32::default(),
        //     p_signal_semaphores: std::ptr::null(),
        // }];

        // unsafe {
        //     let submit_infos =[vk::SubmitInfo {
        //         s_type: vk::StructureType::SUBMIT_INFO,
        //         p_next: ptr::null(),
        //         wait_semaphore_count: u32::default(),
        //         p_wait_semaphores: std::ptr::null(),
        //         p_wait_dst_stage_mask: std::ptr::null(),
        //         command_buffer_count: 1,
        //         p_command_buffers: &command_buffer.raw,
        //         signal_semaphore_count: u32::default(),
        //         p_signal_semaphores: std::ptr::null(),
        //     }];

        //     self._logical_device.raw
        //         .reset_fences(std::slice::from_ref(&command_buffer.submit_done_fence))
        //         .expect("reset_fences");

        //     self._logical_device.raw
        //         .queue_submit(
        //             self._logical_device._graphics_queue,
        //             &submit_infos,
        //             command_buffer.submit_done_fence,
        //         )
        //         .expect("Failed to execute queue submit.");
        // }
        //two buffer
        
        //let presentation_cb = &current_frame.presentation_command_buffer;

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 1 as u32,
            p_wait_semaphores: std::slice::from_ref(&swapchain_image.acquire_semaphore).as_ptr(),
            p_wait_dst_stage_mask: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT].as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer.raw,
            signal_semaphore_count: 1 as u32,
            p_signal_semaphores: std::slice::from_ref(&swapchain_image.rendering_finished_semaphore).as_ptr(),
        }];
        
        unsafe {
            // self._logical_device.raw
            // .reset_command_buffer(presentation_cb.raw, vk::CommandBufferResetFlags::default())
            // .unwrap();

            // self._logical_device.raw
            //     .begin_command_buffer(presentation_cb.raw, 
            //         &vk::CommandBufferBeginInfo::builder()
            //             .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE),
            //         )
            //         .unwrap();

            // self._logical_device.raw.end_command_buffer(presentation_cb.raw).unwrap();

            self._logical_device.raw
                //.reset_fences(&[command_buffer.submit_done_fence])
                .reset_fences(std::slice::from_ref(&command_buffer.submit_done_fence))
                .expect("Failed to reset Fence!");

            self._logical_device.raw
                .queue_submit(
                    self._logical_device._graphics_queue,
                    &submit_infos,
                    command_buffer.submit_done_fence,
                )
                .expect("Failed to execute queue submit.");
        }

        //self._logical_device.with_setup_cb(callback).unwrap();

        self._swapchain.present_image(swapchain_image);

        self._logical_device.finish_frame(current_frame);

    }

    fn recreate_swapchain(&mut self){
        // println!("recreate_swapchain");
        // unsafe {
        //     self._logical_device.raw
        //         .device_wait_idle()
        //         .expect("Failed to wait device idle!")
        // };

        // let window_size = self._window.inner_size();
        // let mut swapchain_desc: SwapchainDesc = Default::default();
        // swapchain_desc.dims = vk::Extent2D {
        //     width: window_size.width,
        //     height: window_size.height,
        // };

        // let _swapchain = Swapchain::create(&self._instance, &self._logical_device, &self._physical_device, &self._surface, &self._physical_device.queue_family_index, swapchain_desc).unwrap();

        // let _render_pass = Arc::new(RenderPass::create_render_pass(
        //     &self._logical_device,
        //     RenderPassDesc {
        //         color_attachments: &[
        //             // view-space geometry normal; * 2 - 1 to decode
        //             RenderPassAttachmentDesc::new(_swapchain.desc.format.format)
        //                 .clear_input(),
        //         ],
        //         depth_attachment: Some(RenderPassAttachmentDesc::new(_swapchain.desc.format.format)),
        //     },
        // ).unwrap());

        // let images = _swapchain.images.iter().map(|image| {
        //     image.raw
        // })
        // .collect::<Vec<_>>();

        // let image_views = self._logical_device.create_image_views(_swapchain.desc.format.format, &images, _swapchain.desc.dims);

        // let mut image_desc = cranberries_backend::vulkan::image::ImageDesc::new(_swapchain.desc.format.format, cranberries_backend::vulkan::image::ImageType::Tex2d, [_swapchain.desc.dims.width, _swapchain.desc.dims.height,0]);
        // image_desc.flags = vk::ImageCreateFlags::default();
        // image_desc.usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;

        // let _framebuffers = image_views.iter().map(|image_view| {
        //     let _framebuffer_cachekey = FramebufferCacheKey::new_simple(
        //         [_swapchain.desc.dims.width, _swapchain.desc.dims.height], 
        //         [image_desc].iter(), 
        //        );

        //     _render_pass.framebuffer_cache.get_or_create(&self._logical_device.raw, _framebuffer_cachekey, *image_view).unwrap()
        // }).collect::<Vec<_>>();


        // let (_graphics_pipeline, _pipeline_layout) = shader::create_graphics_pipline(
        //     &self._logical_device, _render_pass.raw, _swapchain.desc.dims,
        // );

        // self._swapchain = _swapchain;
        // self._render_pass = _render_pass;
        // self._framebuffers = _framebuffers;
        // self._graphics_pipeline = _graphics_pipeline;
        // self._pipeline_layout = _pipeline_layout;

    }

    fn cleanup_swapchain(&self){
        //println!("cleanup_swapchain");
    }

    fn wait_device_idle(&self){
        //println!("wait_device_idle");
        unsafe {
            self._logical_device.raw.device_wait_idle().expect("Failed to wait device idle!")
        };
    }

    fn resize_framebuffer(&mut self){
        //println!("resize_framebuffer");
    }

    fn window_ref(&self) -> &winit::window::Window{
        &self._window
    }
}

struct SyncObjects {
    image_available_semaphores: Vec<ash::vk::Semaphore>,
    render_finished_semaphores: Vec<ash::vk::Semaphore>,
    inflight_fences: Vec<ash::vk::Fence>,
}
/*
impl TestApp {
    pub fn create_sync_objects(device: &ash::Device) -> SyncObjects {
        let mut sync_objects = SyncObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            inflight_fences: vec![],
        };

        let semaphore_create_info = ash::vk::SemaphoreCreateInfo {
            s_type: ash::vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ash::vk::SemaphoreCreateFlags::empty(),
        };

        let fence_create_info = ash::vk::FenceCreateInfo {
            s_type: ash::vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ash::vk::FenceCreateFlags::SIGNALED,
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.inflight_fences.push(inflight_fence);
            }
        }

        sync_objects
    }

    pub fn push_simple_commands(
        &self,
        //command_buffer: vk::CommandBuffer,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &vk::Framebuffer,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
    ) {
        
        // let command_buffer_begin_info = vk::CommandBufferBeginInfo {
        //     s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        //     p_next: ptr::null(),
        //     p_inheritance_info: ptr::null(),
        //     flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        // };

        // unsafe {
        //     self._logical_device
        //         .raw
        //         .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        //         .expect("Failed to begin recording Command Buffer at beginning!");
        // }

        let current_frame = self._logical_device.begin_frame();

        let command_buffer = current_frame.main_command_buffer.raw;

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass,
            framebuffer: *framebuffers,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            self._logical_device.raw.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self._logical_device.raw.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
            self._logical_device.raw.cmd_draw(command_buffer, 3, 1, 0, 0);

            self._logical_device.raw.cmd_end_render_pass(command_buffer);

            
            // self._logical_device.raw
            //     .end_command_buffer(command_buffer)
            //     .expect("Failed to record Command Buffer at Ending!");
        }


        self._logical_device.finish_frame(current_frame);
    }
}
*/
// fn test00() {
//     println!("Hello, cranberries!");

//     let event_loop = EventLoop::new();
//     let _window = init_window(&event_loop);

//     main_loop(event_loop);
// }

fn test01() {
    println!("Hello, cranberries!");

    /* 
    let event_loop = EventLoop::new();
    let _window = init_window(&event_loop);

    let _backend = cranberries_backend::vulkan::instance::Instance::new().unwrap();
    main_loop(event_loop);
    */

    let proc = window::ProgramProc::new();
    let app = TestApp::new(&proc.event_loop);
    proc.main_loop(app);
}

fn main() {
    test01();
}
