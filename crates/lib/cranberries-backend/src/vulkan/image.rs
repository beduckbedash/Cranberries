use anyhow::Ok;
use ash::vk;
use ash::vk::Extent2D;

use crate::vulkan::logical_device::Device;
use crate::vulkan::swapchain::SwapchainImage;
use crate::vulkan::buffer::{Buffer, BufferDesc};

use std::default;
use std::sync::Arc;
use std::sync::Mutex;
use anyhow::Result;
use anyhow;
use std::ptr;
use std::collections::HashMap;
use std::path::Path;
use image::GenericImageView;


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageType {
    Tex1d = 0,
    Tex1dArray = 1,
    Tex2d = 2,
    Tex2dArray = 3,
    Tex3d = 4,
    Cube = 5,
    CubeArray = 6,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ImageDesc {
    pub image_type: ImageType,
    pub usage: vk::ImageUsageFlags,
    pub flags: vk::ImageCreateFlags,
    pub format: vk::Format,
    pub extent: [u32; 3],
    pub tiling: vk::ImageTiling,
    pub mip_levels: u16,
    pub array_elements: u32,
}

pub struct Image {
    pub raw: vk::Image,
    pub desc: ImageDesc,
    pub views: Mutex<HashMap<ImageViewDesc, vk::ImageView>>,
    //allocation: gpu_allocator::SubAllocation,
    pub device: ash::Device,
    pub device_memory: vk::DeviceMemory,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct ImageViewDesc {
    pub view_type: Option<vk::ImageViewType>,
    pub format: Option<vk::Format>,
    pub aspect_mask: vk::ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: Option<u32>,
}

impl ImageViewDesc {
    pub fn default() -> ImageViewDesc{

        ImageViewDesc {
            view_type: Some(vk::ImageViewType::default()),
            format: Some(vk::Format::default()),
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0 as u32,
            level_count: Some(1),
        }
    }
}

// pub struct ImageViewDesc {
//     pub format: Option<vk::Format>,
// }

// pub struct SwapchainImageView {
//     image_views: Vec<vk::ImageView>,
//     device: ash::Device,
// }

impl Image {
    pub fn view(
        &self,
        device: &Arc<Device>,
        desc: &ImageViewDesc,
    ) -> Result<vk::ImageView> {
        let mut views = self.views.lock().unwrap();

        if let Some(entry) = views.get(desc) {
            Ok(*entry)
        } else {
            let view = device.create_image_view(*desc, &self.desc, self.raw)?;
            Ok(*views.entry(*desc).or_insert(view))
        }
    }

    pub fn view_desc(&self, desc: &ImageViewDesc) -> vk::ImageViewCreateInfo {
        Self::view_desc_impl(*desc, &self.desc)
    }

    fn view_desc_impl(
        desc: ImageViewDesc,
        image_desc: &ImageDesc,
    ) -> vk::ImageViewCreateInfo {
        vk::ImageViewCreateInfo {
            format: desc.format.unwrap_or(image_desc.format),
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            view_type: desc.view_type.unwrap(),//
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: desc.aspect_mask,
                base_mip_level: desc.base_mip_level,
                level_count: desc.level_count.unwrap_or(image_desc.mip_levels as u32),
                base_array_layer: 0,
                layer_count: match image_desc.image_type {
                    ImageType::Cube | ImageType::CubeArray => 6,
                    _ => 1,
                }
            },
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::default(),
            image: vk::Image::default(),
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image(self.raw, None);
            self.device.free_memory(self.device_memory, None);
        }
    }
}

impl Device {
    pub fn create_image_view(
        &self,
        desc: ImageViewDesc,
        image_desc: &ImageDesc,
        image_raw: vk::Image,
    ) -> Result<vk::ImageView> {
        if image_desc.format == vk::Format::D32_SFLOAT
            && !desc.aspect_mask.contains(vk::ImageAspectFlags::DEPTH) {
                anyhow::bail!("Depth-only resource used without the vk::ImageAspectFlags::DEPTH flag");
            }
        
        let create_info = vk::ImageViewCreateInfo {
            image: image_raw,
            ..Image::view_desc_impl(desc, image_desc)
        };

        //println!("image view {:#?}", create_info);

        Ok(
            unsafe {
                self.raw.create_image_view(&create_info, None)
                .expect("Failded to create Image View!")
            }
        )
    }

    pub fn create_image_views(
        &self,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
        //images: &Vec<SwapchainImage>,
        extent: vk::Extent2D,
    ) -> Vec<vk::ImageView> {
        let image_view_desc = ImageViewDesc {
            view_type: Some(vk::ImageViewType::TYPE_2D),
            format: Some(surface_format),
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0 as u32,
            level_count: Some(1 as u32),
        };

        let image_desc = ImageDesc::create(surface_format, ImageType::Tex2d, [extent.width, extent.height,0]);

        let image_views :Vec<vk::ImageView> = images
            .iter()
            .map(|image| {
                self.create_image_view(
                    image_view_desc,
                    &image_desc,
                    *image,
                ).unwrap()
            })
            .collect();

        image_views
    }
}

impl ImageDesc {
    pub fn create(
        format: vk::Format,
        image_type: ImageType,
        extent: [u32; 3]
    ) -> ImageDesc {

        ImageDesc {
            image_type,
            usage: vk::ImageUsageFlags::default(),
            flags: vk::ImageCreateFlags::empty(),
            format,
            extent,
            tiling: vk::ImageTiling::OPTIMAL,
            mip_levels: 1,
            array_elements: 1,
        }
    }
}

// pub struct ImageSubResourceData<'a> {
//     pub data: &'a [u8],
//     pub row_pitch: usize,
//     pub slice_pitch: usize,
// }

impl Device {
    pub fn create_image(
        &self,
        desc: ImageDesc,
        //initial_data: Vec<ImageSubResourceData>,
        required_memory_properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> Result<Image> {
        let create_info = get_image_create_info(&desc, false);

        //println!("image create info {:#?}",  create_info);

        let image = unsafe {
            self.raw
                .create_image(&create_info, None)
                .expect("create_image")
        };

        let requirements = unsafe { self.raw.get_image_memory_requirements(image) };

        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: find_memory_type(
                requirements.memory_type_bits,
                required_memory_properties,
                device_memory_properties,
            ),
        };

        let image_memory = unsafe {
            self.raw
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };

        unsafe {
            self.raw
                .bind_image_memory(image, image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        }

        Ok(Image {
            raw: image,
            //allocation,
            desc,
            views: Default::default(),
            device: self.raw.clone(),
            device_memory: image_memory,
        })
    }

    pub fn transition_image_layout(
        &self,
        image: &Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: image.raw,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        self.with_setup_cb(|cb|
            unsafe {
                self.raw
                    .cmd_pipeline_barrier(
                        cb,
                        source_stage,
                        destination_stage, 
                        vk::DependencyFlags::empty(),
                        &[],
                        &[], 
                        &image_barriers);
        }).unwrap();
    }

    pub fn create_texture_image(
        &self,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        image_path: &Path,
    ) -> Result<Image> {
        //println!("path: {}", image_path);
        let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
        image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let image_data = match &image_object {
            image::DynamicImage::ImageLuma8(_)
            | image::DynamicImage::ImageBgr8(_)
            | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba().into_raw(),
            image::DynamicImage::ImageLumaA8(_)
            | image::DynamicImage::ImageBgra8(_)
            | image::DynamicImage::ImageRgba8(_) => image_object.raw_pixels(),
        };

        if image_size <= 0 {
            panic!("Failed to load texture image!")
        }

        let staging_buffer_desc = BufferDesc {
            size: image_size as usize,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            required_memory_flags: vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            alignment: None,
        };

        let mut staging_buffer = self.create_buffer(staging_buffer_desc, None).unwrap();
        
        unsafe {
            let data_ptr = self.raw
                .map_memory(
                    staging_buffer.device_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

            self.raw.unmap_memory(staging_buffer.device_memory);
        }

        let texture_image_desc = ImageDesc {
            image_type: ImageType::Tex2d,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            format: vk::Format::R8G8B8A8_SRGB,
            extent: [image_width, image_height, 1],
            tiling: vk::ImageTiling::OPTIMAL,
            mip_levels: 1,
            array_elements: 1,
        };

        let mut texture_image = self.create_image(texture_image_desc, vk::MemoryPropertyFlags::DEVICE_LOCAL, device_memory_properties).unwrap();

        self.transition_image_layout(&texture_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        self.with_setup_cb(|cb| {
            let buffer_image_regions = [vk::BufferImageCopy {
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image_extent: vk::Extent3D {
                    width: image_width,
                    height: image_height,
                    depth: 1,
                },
                buffer_offset: 0,
                buffer_image_height: 0,
                buffer_row_length: 0,
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            }];

            unsafe {
                self.raw.cmd_copy_buffer_to_image(
                    cb,
                    staging_buffer.raw,
                    texture_image.raw,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &buffer_image_regions,
                );
            }
        }).unwrap();

        self.transition_image_layout(&texture_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        
        self.immediate_destroy_buffer(&mut staging_buffer);

        Ok(texture_image)
    }

    pub fn create_texture_sampler(&self) -> Result<vk::Sampler> {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };

        let sampler = unsafe {
            self.raw
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        };
        
        Ok(sampler)
    }
}

pub fn get_image_create_info(desc: &ImageDesc, initial_data: bool) -> vk::ImageCreateInfo {
    let (image_type, image_extent, image_layers) = match desc.image_type {
        ImageType::Tex1d => (
            vk::ImageType::TYPE_1D,
            vk::Extent3D {
                width: desc.extent[0],
                height: 1,
                depth: 1,
            },
            1,
        ),
        ImageType::Tex1dArray => (
            vk::ImageType::TYPE_1D,
            vk::Extent3D {
                width: desc.extent[0],
                height: 1,
                depth: 1,
            },
            desc.array_elements,
        ),
        ImageType::Tex2d => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent[0],
                height: desc.extent[1],
                depth: 1,
            },
            1,
        ),
        ImageType::Tex2dArray => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent[0],
                height: desc.extent[1],
                depth: 1,
            },
            desc.array_elements,
        ),
        ImageType::Tex3d => (
            vk::ImageType::TYPE_3D,
            vk::Extent3D {
                width: desc.extent[0],
                height: desc.extent[1],
                depth: desc.extent[2],
            },
            1,
        ),
        ImageType::Cube => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent[0],
                height: desc.extent[1],
                depth: 1,
            },
            6,
        ),
        ImageType::CubeArray => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent[0],
                height: desc.extent[1],
                depth: 1,
            },
            6 * desc.array_elements,
        ),
    };

    let mut image_usage = desc.usage;

    if initial_data {
        image_usage |= vk::ImageUsageFlags::TRANSFER_DST;
    }

    vk::ImageCreateInfo {
        flags: desc.flags,
        image_type,
        format: desc.format,
        extent: image_extent,
        mip_levels: desc.mip_levels as u32,
        array_layers: image_layers,
        samples: vk::SampleCountFlags::TYPE_1, // TODO: desc.sample_count
        tiling: desc.tiling,
        usage: image_usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        initial_layout: match initial_data {
            true => vk::ImageLayout::PREINITIALIZED,
            false => vk::ImageLayout::UNDEFINED,
        },
        ..Default::default()
    }
}

pub fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: &vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
        {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type!")
}