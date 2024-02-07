use anyhow::Ok;
use anyhow::Result;
use ash::vk;
use ash::vk::ImageView;
use winapi::um::winuser::VK_NUMLOCK;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::ptr;
use std::ffi::CString;

use crate::vulkan::logical_device::Device;
use crate::vulkan::image::ImageDesc;
use crate::vulkan::buffer::Vertex;

use super::surface;







pub struct ShaderPipelineCommon {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub set_layout_info: Vec<HashMap<u32, vk::DescriptorType>>,
    pub descriptor_pool_sizes: Vec<vk::DescriptorPoolSize>,
    pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    pub pipeline_bind_point: vk::PipelineBindPoint,
}
pub struct ComputePipeline {
    pub common: ShaderPipelineCommon,
    pub group_size: [u32; 3],
}

impl std::ops::Deref for ComputePipeline {
    type Target = ShaderPipelineCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

pub struct RasterPipeline {
    pub common: ShaderPipelineCommon,
}

impl std::ops::Deref for RasterPipeline {
    type Target = ShaderPipelineCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ShaderPipelineStage {
    Vertex,
    Pixel,
    RayGen,
    RayMiss,
    RayClosestHit,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct PipelineShaderDesc {
    pub stage: ShaderPipelineStage,
    //pub descriptor_set_layout_flags: Option<Vec<(usize, vk::DescriptorSetLayoutCreateFlags)>>,
    pub push_constants_bytes: usize,
    pub entry: String,
    //pub source: ShaderSource,
}

impl PipelineShaderDesc {
    pub fn new(stage: ShaderPipelineStage) -> PipelineShaderDesc {
        PipelineShaderDesc {
            stage,
            push_constants_bytes: usize::default(),
            entry: String::default(),
        }
    }
}

#[derive(Clone)]
pub struct RasterPipelineDesc {
    //pub descriptor_set_opts: [Option<(u32, DescriptorSetLayoutOpts)>; MAX_DESCRIPTOR_SETS],
    pub render_pass: Arc<vk::RenderPass>,
    pub face_cull: bool,
    pub depth_write: bool,
    pub push_constants_bytes: usize,
}

impl RasterPipelineDesc {
    pub fn new() -> RasterPipelineDesc {
        RasterPipelineDesc {
            render_pass: Arc::default(),
            face_cull: true,
            depth_write: true,
            push_constants_bytes: usize::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderPassAttachmentDesc {
    pub format: vk::Format,
    pub load_op: vk::AttachmentLoadOp,
    pub store_op: vk::AttachmentStoreOp,
    pub samples: vk::SampleCountFlags,
}

impl RenderPassAttachmentDesc {
    pub fn create_color_attachment(format: vk::Format) -> Self {
        Self {
            format,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            samples: vk::SampleCountFlags::TYPE_1,
        }
    }

    pub fn create_depth_attachment(format: vk::Format) -> Self {
        Self {
            format,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            samples: vk::SampleCountFlags::TYPE_1,
        }
    }

    pub fn garbage_input(mut self) -> Self {
        self.load_op = vk::AttachmentLoadOp::DONT_CARE;
        self
    }

    pub fn clear_input(mut self) -> Self {
        self.load_op = vk::AttachmentLoadOp::CLEAR;
        self
    }

    pub fn discard_output(mut self) -> Self {
        self.store_op = vk::AttachmentStoreOp::DONT_CARE;
        self
    }

    fn to_vk(
        self,
        initial_layout: vk::ImageLayout,
        final_layout: vk::ImageLayout,
    ) -> vk::AttachmentDescription {
        vk::AttachmentDescription {
            format: self.format,
            samples: self.samples,
            load_op: self.load_op,
            store_op: self.store_op,
            initial_layout,
            final_layout,
            ..Default::default()
        }
    }
}

pub const MAX_COLOR_ATTACHMENTS: usize = 8;

#[derive(Eq, PartialEq, Hash)]
pub struct FramebufferCacheKey {
    pub dims: [u32; 2],
    pub attachments: Vec<(vk::ImageUsageFlags, vk::ImageCreateFlags)>,
}

impl FramebufferCacheKey {
    pub fn new<'a>(
        dims: [u32; 2],
        color_attachments: impl Iterator<Item = &'a ImageDesc>,
        depth_stencil_attachment: Option<&'a ImageDesc>,
    ) -> Self {
        let color_attachments : Vec<(vk::ImageUsageFlags, vk::ImageCreateFlags)> = color_attachments
            .chain(depth_stencil_attachment.into_iter())
            .copied()
            .map(|attachment| (attachment.usage, attachment.flags))
            .collect();

            //println!("color attachments len {}", color_attachments.len());

        Self {
            dims,
            attachments: color_attachments,
        }
    }

    pub fn new_simple<'a>(
        dims: [u32; 2],
        color_attachments: impl Iterator<Item = &'a ImageDesc>,
    ) -> Self {

        let color_attachments = color_attachments
        .copied()
        .map(|attachment| (attachment.usage, attachment.flags))
        .collect();

        Self {
            dims,
            attachments: color_attachments,
        }
    }

}

pub struct FramebufferCache {
    entries: Mutex<HashMap<FramebufferCacheKey, vk::Framebuffer>>,
    color_attachment_descs: Vec<RenderPassAttachmentDesc>,//[RenderPassAttachmentDesc; MAX_COLOR_ATTACHMENTS],
    color_attachment_count: usize,
    depth_attachment_descs: Vec<RenderPassAttachmentDesc>,//[RenderPassAttachmentDesc; MAX_COLOR_ATTACHMENTS],
    depth_attachment_count: usize,
    //render_pass: Arc<RenderPass>,
    //entry: vk::Framebuffer,
}

pub struct RenderPassDesc<'a> {
    pub color_attachments: &'a [RenderPassAttachmentDesc],
    pub depth_attachment: Option<RenderPassAttachmentDesc>,
}

pub struct RenderPass {
    pub raw: vk::RenderPass,
    pub framebuffer_cache: FramebufferCache,
    pub device: Arc<Device>,
}

impl RenderPass {
    // to do 
    pub fn create_render_pass(
        device: &Arc<Device>, 
        desc: RenderPassDesc<'_>,
        //format: vk::Format,
    ) -> Result<RenderPass> {
        
        let render_pass_attachments = desc
            .color_attachments
            .iter()
            .enumerate()
            .map(|(idx, renderpass_attachment_desc)| {
                vk::AttachmentDescription {
                    format: renderpass_attachment_desc.format,//desc.color_attachments[idx].format,
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    samples: renderpass_attachment_desc.samples,
                    load_op: renderpass_attachment_desc.load_op,
                    store_op: renderpass_attachment_desc.store_op,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,//COLOR_ATTACHMENT_OPTIMAL,UNDEFINED
                    final_layout: vk::ImageLayout::PRESENT_SRC_KHR,//COLOR_ATTACHMENT_OPTIMAL,
                    ..Default::default()
                }
            })
            .chain(desc.depth_attachment.as_ref().map(|renderpass_attachment_desc| {
                vk::AttachmentDescription {
                    format: renderpass_attachment_desc.format,
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    samples: renderpass_attachment_desc.samples,
                    load_op: renderpass_attachment_desc.load_op,
                    store_op: renderpass_attachment_desc.store_op,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,
                    final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    ..Default::default()
                }
            }))
            .collect::<Vec<_>>();

        let color_attachment_refs = (0..desc.color_attachments.len() as u32)
        .map(|attachment| vk::AttachmentReference {
            attachment,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        })
        .collect::<Vec<_>>();

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: desc.color_attachments.len() as u32,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        
        // let color_attachment = vk::AttachmentDescription {
        //     format,//: desc.color_attachments[0].format,
        //     flags: vk::AttachmentDescriptionFlags::empty(),
        //     samples: vk::SampleCountFlags::TYPE_1,
        //     load_op: vk::AttachmentLoadOp::CLEAR,
        //     store_op: vk::AttachmentStoreOp::STORE,
        //     stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        //     stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        //     initial_layout: vk::ImageLayout::UNDEFINED,
        //     final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        // };

        // let color_attachment_ref = vk::AttachmentReference {
        //     attachment: 0,
        //     layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        // };

        // let color_attachment = vk::AttachmentDescription {
        //     flags: vk::AttachmentDescriptionFlags::empty(),
        //     format: format,
        //     samples: vk::SampleCountFlags::TYPE_1,
        //     load_op: vk::AttachmentLoadOp::CLEAR,
        //     store_op: vk::AttachmentStoreOp::STORE,
        //     stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        //     stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        //     initial_layout: vk::ImageLayout::UNDEFINED,
        //     final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        // };

        // let depth_attachment = vk::AttachmentDescription {
        //     flags: vk::AttachmentDescriptionFlags::empty(),
        //     format: vk::Format::D32_SFLOAT,
        //     samples: vk::SampleCountFlags::TYPE_1,
        //     load_op: vk::AttachmentLoadOp::CLEAR,
        //     store_op: vk::AttachmentStoreOp::DONT_CARE,
        //     stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        //     stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        //     initial_layout: vk::ImageLayout::UNDEFINED,
        //     final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        // };

        // let color_attachments_ref = [vk::AttachmentReference {
        //     attachment: 0,
        //     layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        // }];

        // let depth_attachment_ref = vk::AttachmentReference {
        //     attachment: 1,
        //     layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        // };

        let subpass_description = [vk::SubpassDescription {
            color_attachment_count: color_attachment_refs.len() as u32,
            p_color_attachments: color_attachment_refs.as_ptr(),
            p_depth_stencil_attachment: &depth_attachment_ref,
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        }];
    
        //let render_pass_attachments = [color_attachment, depth_attachment];
    
        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ,
            dependency_flags: vk::DependencyFlags::empty(),
        }];
    
        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags: vk::RenderPassCreateFlags::empty(),
            p_next: ptr::null(),
            attachment_count: render_pass_attachments.len() as u32,
            p_attachments: render_pass_attachments.as_ptr(),
            subpass_count: subpass_description.len() as u32,
            p_subpasses: subpass_description.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
        };
    
        let render_pass = unsafe {
            device
                .raw
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        };

        // Arc::new(RenderPass {
        //     device: device.clone(),
        //     raw: render_pass,
        //     framebuffer_cache: FramebufferCache::new_1(
        //         //render_pass,
        //         //desc.color_attachments,
        //         //desc.depth_attachment,
        //     ),
        // })

        Ok(RenderPass {
            device: device.clone(),
            raw: render_pass,
            framebuffer_cache: FramebufferCache::new(
                //render_pass, 
                desc.color_attachments, 
                desc.depth_attachment.unwrap(),
            ),
        })
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.raw.destroy_render_pass(self.raw, None);
        }
    }
}

impl FramebufferCache {
    fn new(
        //render_pass: &RenderPass,
        color_attachments: &[RenderPassAttachmentDesc],
        depth_attachment: RenderPassAttachmentDesc,
    ) -> FramebufferCache {
        //let mut attachment_descs = Vec::new();
        let mut color_attachment_descs = Vec::new();
        let mut depth_attachment_descs = Vec::new();

        // for color_attachment in color_attachments.iter_mut() {
        //     attachment_desc[0] = (*color_attachment);
        // }

        // (0..color_attachments.len() as usize)
        // .map(|idx| {
        //     attachment_descs[idx] = color_attachments[idx];
        // });
        //.collect::<Vec<_>>();
        
        for color_attachment in color_attachments {
            color_attachment_descs.push(*color_attachment);
        }

        depth_attachment_descs.push(depth_attachment);

        //attachment_descs.push(depth_attachment);
        // color_attachments.iter().enumerate().map(|(idx, attachment_desc)| {
        //     attachment_descs[idx] = *attachment_desc;
        // });

        // attachment_descs[MAX_COLOR_ATTACHMENTS] = (depth_attachment);
        
        // let attachment_descs = color_attachments.iter()
        //     .map(|attachment_desc| {
        //         attachment_desc
        //     })
        //     .chain(&depth_attachment.map(|d| {
        //         d
        //     }))
        //     .collect::<Vec<_>>();

        
        FramebufferCache {
            color_attachment_count: color_attachment_descs.len(),
            color_attachment_descs,
            depth_attachment_descs,
            depth_attachment_count: 1,
            //render_pass: Arc::new(*render_pass),
            entries: Default::default(),
        }
    }

    // fn new_1() -> FramebufferCache {
    //     FramebufferCache{
    //         attachment_desc: Default::default(),
    //         render_pass: Default::default(),
    //         color_attachment_count: Default::default(),
    //         entry: Default::default(),
    //     }
    // }

    pub fn get_or_create(
        &self,
        device: &ash::Device,
        key: FramebufferCacheKey,
        image_view: ImageView,
        depth_image_view: ImageView,
        render_pass: vk::RenderPass,
    ) -> anyhow::Result<vk::Framebuffer> {
        let mut entries = self.entries.lock().unwrap();

        /* 
        if let Some((entry)) = entries.get(&key) {
            println!("get entry");
            Ok(*entry)
        }
        else {*/
            //println!("create entry");
            let entry = {
                /*
                let mut color_formats = Vec::new();// [; MAX_COLOR_ATTACHMENTS + 1];
                let [width, height] = key.dims;

                // for (idx, desc) in self.attachment_descs.iter().enumerate() {
                //     //color_formats[idx] = desc.format;
                //     color_formats.push(desc.format);
                // }

                for desc in self.color_attachment_descs.iter() {
                    color_formats.push(desc.format);
                }

                let attachments: Vec<vk::FramebufferAttachmentImageInfoKHR> = self
                    .color_attachment_descs
                    .iter()
                    .zip(key.attachments.iter())
                    .map(|(desc, (usage, flags))| {
                        vk::FramebufferAttachmentImageInfoKHR {
                            width,
                            height,
                            flags: *flags,
                            usage: *usage,
                            layer_count: 1,
                            view_format_count: color_formats.len() as u32,
                            p_view_formats: color_formats.as_ptr(),
                            ..Default::default()
                        }
                    }).collect::<Vec<_>>();
                    
                */
                //println!("FramebufferAttachmentImageInfoKHR {:#?}", attachments[0]);

                // let mut imageless_desc = vk::FramebufferAttachmentsCreateInfoKHR::builder()
                // .attachment_image_infos(&attachments);

                //
                /*
                let image_view = [image_view];
                let mut fbo_desc = vk::FramebufferCreateInfo::builder()
                        .flags(vk::FramebufferCreateFlags::empty())
                        .render_pass(self.render_pass)
                        .width(width as _)
                        .height(height as _)
                        .layers(1)
                        //.push_next(&mut imageless_desc);
                        .attachments(&image_view);

                fbo_desc.attachment_count = attachments.len() as _;
                fbo_desc.attachment_count = 1;

                unsafe { device.create_framebuffer(&fbo_desc, None)? }
            };
            */
                let attachments = [image_view, depth_image_view];
    
                let framebuffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: vk::FramebufferCreateFlags::empty(),
                    render_pass,
                    attachment_count: attachments.len() as u32,
                    p_attachments: attachments.as_ptr(),
                    width: key.dims[0],
                    height: key.dims[1],
                    layers: 1,
                };
    
                unsafe {
                    device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .expect("Failed to create Framebuffer!")
                }
            };

            //entries.insert(key, entry);
            Ok(entry)
        }
    
    //}
}

pub fn create_graphics_pipline(
    device: &Arc<Device>,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    //vert_shader_module: vk::ShaderModule,
    //frag_shader_module: vk::ShaderModule,
    ubo_set_layout: vk::DescriptorSetLayout,
) -> (vk::Pipeline, vk::PipelineLayout) {
    // let vert_shader_module = create_shader_module(
    //     device,
    //     include_bytes!("../../../../shaders/spv/09-shader-base.vert.spv").to_vec(),
    // );
    // let frag_shader_module = create_shader_module(
    //     device,
    //     include_bytes!("../../../../shaders/spv/09-shader-base.frag.spv").to_vec(),
    // );

    let vert_shader_module = create_shader_module(
        device,
        include_bytes!("../../../../shaders/spv/26-shader-depth.vert.spv").to_vec(),
    );
    let frag_shader_module = create_shader_module(
        device,
        include_bytes!("../../../../shaders/spv/26-shader-depth.frag.spv").to_vec(),
    );

    let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo {
            // Vertex Shader
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: vert_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
            stage: vk::ShaderStageFlags::VERTEX,
        },
        vk::PipelineShaderStageCreateInfo {
            // Fragment Shader
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: frag_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];

    //
    let binding_description = Vertex::get_binding_descriptions();
    let attribute_description = Vertex::get_attribute_descriptions();

    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_attribute_description_count: attribute_description.len() as u32,
        p_vertex_attribute_descriptions: attribute_description.as_ptr(),
        vertex_binding_description_count: binding_description.len() as u32,
        p_vertex_binding_descriptions: binding_description.as_ptr(),
    };
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        p_next: ptr::null(),
        primitive_restart_enable: vk::FALSE,
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
    };

    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];

    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_extent,
    }];

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        scissor_count: scissors.len() as u32,
        p_scissors: scissors.as_ptr(),
        viewport_count: viewports.len() as u32,
        p_viewports: viewports.as_ptr(),
    };

    let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: vk::FALSE,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        rasterizer_discard_enable: vk::FALSE,
        depth_bias_clamp: 0.0,
        depth_bias_constant_factor: 0.0,
        depth_bias_enable: vk::FALSE,
        depth_bias_slope_factor: 0.0,
    };
    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        p_next: ptr::null(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 0.0,
        p_sample_mask: ptr::null(),
        alpha_to_one_enable: vk::FALSE,
        alpha_to_coverage_enable: vk::FALSE,
    };

    let stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        compare_mask: 0,
        write_mask: 0,
        reference: 0,
    };

    let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: vk::TRUE,
        depth_write_enable: vk::TRUE,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: vk::FALSE,
        stencil_test_enable: vk::FALSE,
        front: stencil_state,
        back: stencil_state,
        max_depth_bounds: 1.0,
        min_depth_bounds: 0.0,
    };

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::FALSE,
        color_write_mask: vk::ColorComponentFlags::all(),
        src_color_blend_factor: vk::BlendFactor::ONE,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
    }];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: color_blend_attachment_states.len() as u32,
        p_attachments: color_blend_attachment_states.as_ptr(),
        blend_constants: [0.0, 0.0, 0.0, 0.0],
    };

    //descriptor set layouts
    let set_layouts = [ubo_set_layout];

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: set_layouts.len() as u32,
        p_set_layouts: set_layouts.as_ptr(),
        push_constant_range_count: 0,
        p_push_constant_ranges: ptr::null(),
    };

    let pipeline_layout = unsafe {
        device
            .raw
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout!")
    };

    let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: shader_stages.len() as u32,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_state_create_info,
        p_input_assembly_state: &vertex_input_assembly_state_info,
        p_tessellation_state: ptr::null(),
        p_viewport_state: &viewport_state_create_info,
        p_rasterization_state: &rasterization_statue_create_info,
        p_multisample_state: &multisample_state_create_info,
        p_depth_stencil_state: &depth_state_create_info,
        p_color_blend_state: &color_blend_state,
        p_dynamic_state: ptr::null(),
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: -1,
    }];

    let graphics_pipelines = unsafe {
        device
            .raw
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphic_pipeline_create_infos,
                None,
            )
            .expect("Failed to create Graphics Pipeline!.")
    };

    unsafe {
        device.raw.destroy_shader_module(vert_shader_module, None);
        device.raw.destroy_shader_module(frag_shader_module, None);
    }

    (graphics_pipelines[0], pipeline_layout)
}

pub fn create_shader_module(device: &Arc<Device>, code: Vec<u8>) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
    };

    unsafe {
        device
            .raw
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create Shader Module!")
    }
}

pub fn create_framebuffers(
    device: &Arc<Device>,
    render_pass: vk::RenderPass,
    image_views: &Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![];

    for &image_view in image_views.iter() {
        let attachments = [image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: swapchain_extent.width,
            height: swapchain_extent.height,
            layers: 1,
        };

        let framebuffer = unsafe {
            device
                .raw
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };

        framebuffers.push(framebuffer);
    }

    framebuffers
}