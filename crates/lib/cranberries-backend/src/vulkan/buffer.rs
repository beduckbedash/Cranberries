use ash::vk;
use ash;

use anyhow::{Ok, Result};
use memoffset::offset_of;
use std::ptr;
use std::sync::Arc;
use cgmath::{Deg, Matrix4, Point3, Vector3};

use crate::{constants, vulkan::logical_device::Device};


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, tex_coord) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl Device {
    pub fn create_vertex_buffer(
        &self,
        vertex_data: [Vertex; 8],
    ) ->Result<Buffer> {
        // let vertex_buffer_create_info = vk::BufferCreateInfo {
        //     s_type: vk::StructureType::BUFFER_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: vk::BufferCreateFlags::empty(),
        //     size: std::mem::size_of_val(&vertex_data) as u64,
        //     usage: vk::BufferUsageFlags::VERTEX_BUFFER,
        //     sharing_mode: vk::SharingMode::EXCLUSIVE,
        //     queue_family_index_count: 0,
        //     p_queue_family_indices: ptr::null(),
        // };

        // let vertex_buffer = unsafe {
        //     self.raw
        //         .create_buffer(&vertex_buffer_create_info, None)
        //         .expect("Failed to create Vertex Buffer")
        // };

        // let mem_requirements = unsafe { self.raw.get_buffer_memory_requirements(vertex_buffer) };
        // let mem_properties =
        //     unsafe { self.instance.raw.get_physical_device_memory_properties(self.pdevice.raw) };
        // let required_memory_flags: vk::MemoryPropertyFlags =
        //     vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        // let memory_type = Device::find_memory_type(
        //     mem_requirements.memory_type_bits,
        //     required_memory_flags,
        //     mem_properties,
        // );

        // let allocate_info = vk::MemoryAllocateInfo {
        //     s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        //     p_next: ptr::null(),
        //     allocation_size: mem_requirements.size,
        //     memory_type_index: memory_type,
        // };

        // let vertex_buffer_memory = unsafe {
        //     self.raw
        //         .allocate_memory(&allocate_info, None)
        //         .expect("Failed to allocate vertex buffer memory!")
        // };
        
        // unsafe {
        //     self.raw
        //         .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
        //         .expect("Failed to bind Buffer");

        //     let data_ptr = self.raw
        //         .map_memory(
        //             vertex_buffer_memory,
        //             0,
        //             vertex_buffer_create_info.size,
        //             vk::MemoryMapFlags::empty(),
        //         )
        //         .expect("Failed to Map Memory") as *mut Vertex;

        //     data_ptr.copy_from_nonoverlapping(vertex_data.as_ptr(), vertex_data.len());

        //     self.raw.unmap_memory(vertex_buffer_memory);
        // }
        
        let buffer_size = std::mem::size_of_val(&vertex_data);

        let staging_buffer_desc = BufferDesc {
            size: buffer_size,
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
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to Map Memory") as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(vertex_data.as_ptr(), vertex_data.len());

            self.raw.unmap_memory(staging_buffer.device_memory);
        }

        let vertex_buffer_desc = BufferDesc {
            size: std::mem::size_of_val(&vertex_data),
            usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            required_memory_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            alignment: None,
        };

        let vertex_buffer = self.create_buffer(vertex_buffer_desc, None).unwrap();

        self.copy_buffer(&staging_buffer, &vertex_buffer);

        self.immediate_destroy_buffer(&mut staging_buffer);

        Ok(vertex_buffer)
    }

    pub fn create_index_buffer(
        &self,
        index_data: [u32; 12],
    ) ->Result<Buffer> {
        let buffer_size = std::mem::size_of_val(&index_data);

        let staging_buffer_desc = BufferDesc {
            size: buffer_size,
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
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(index_data.as_ptr(), index_data.len());

            self.raw.unmap_memory(staging_buffer.device_memory);
        }

        let index_buffer_desc = BufferDesc {
            size: buffer_size,
            usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            required_memory_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            alignment: None,
        };

        let index_buffer = self.create_buffer(index_buffer_desc, None).unwrap();

        self.copy_buffer(&staging_buffer, &index_buffer);

        self.immediate_destroy_buffer(&mut staging_buffer);

        Ok(index_buffer)
    }

    pub fn create_uniform_buffer(
        &self,
        swapchain_image_count: usize,
    ) ->Result<Vec<Buffer>> {
        let buffer_size = std::mem::size_of::<UniformBufferObject>();

        let mut uniform_buffers = vec![];

        let uniform_buffer_desc = BufferDesc {
            size: buffer_size,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            required_memory_flags: vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            alignment: None,
        };

        for _ in 0..swapchain_image_count {
            let uniform_buffer = self.create_buffer(uniform_buffer_desc, None).unwrap();
            uniform_buffers.push(uniform_buffer);
        }

        Ok(uniform_buffers)
    }

    pub fn test_update_uniform_buffer (
        &self,
        delta_time: f32,
        extent: vk::Extent2D,
        uniform_buffer: &mut Buffer,
        uniform_transform: &mut UniformBufferObject,
    ) {
        // let uniform_transform = UniformBufferObject {
        //     model: Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time) * uniform_transform.model,
        //     proj: uniform_transform.proj,
        //     view: uniform_transform.view,
        // };

        //uniform_transform.model = Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time) * uniform_transform.model;

        // let ubos = [UniformBufferObject {
        //     model: Matrix4::from_angle_z(Deg(90.0 * delta_time * 50.0 as f32)),
        //     view: Matrix4::look_at(
        //         Point3::new(2.0, 2.0, 2.0),
        //         Point3::new(0.0, 0.0, 0.0),
        //         Vector3::new(0.0, 0.0, 1.0),
        //     ),
        //     proj: cgmath::perspective(
        //         Deg(45.0),
        //         extent.width as f32 / extent.height as f32,
        //         0.1,
        //         10.0,
        //     ),
        // }];

        let ubos = [uniform_transform.clone()];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self.raw
            .map_memory(
                uniform_buffer.device_memory,
                0,
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.raw.unmap_memory(uniform_buffer.device_memory);
        }
    }

    fn find_memory_type(
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            //if (type_filter & (1 << i)) > 0 && (memory_type.property_flags & required_properties) == required_properties {
            //    return i as u32
            // }

            // same implementation
            if (type_filter & (1 << i)) > 0
                && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }

        panic!("Failed to find suitable memory type!")
    }
}


pub struct Buffer {
    pub raw: vk::Buffer,
    pub desc: BufferDesc,
    pub device_memory: vk::DeviceMemory,
    //pub allocation: gpu_allocator::SubAllocation,
    pub device: ash::Device,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct BufferDesc {
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    //pub memory_location: MemoryLocation,
    pub required_memory_flags: vk::MemoryPropertyFlags,
    pub alignment: Option<u64>,
}

impl Device {
    pub fn create_buffer_impl(
        &self,
        desc: BufferDesc,
        //name: &str,
    ) -> Result<Buffer> {
        let buffer_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage: desc.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            self.raw.create_buffer(&buffer_info, None)
                .expect("Failed to create buffer")
        };

        let mut requirements = unsafe { self.raw.get_buffer_memory_requirements(buffer) };

        if let Some(alignment) = desc.alignment {
            requirements.alignment = requirements.alignment.max(alignment);
        }

        if desc.usage.contains(vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR)
        {
            // TODO: query device props
            requirements.alignment = requirements.alignment.max(64);
        }

        let mem_properties =
        unsafe { self.instance.raw.get_physical_device_memory_properties(self.pdevice.raw) };
        
        //to do
        // let required_memory_flags: vk::MemoryPropertyFlags =
        //     vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        
        let memory_type = Device::find_memory_type(
            requirements.memory_type_bits,
            desc.required_memory_flags,
            mem_properties,
        );

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: memory_type,
        };

        let buffer_memory = unsafe {
            self.raw
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate vertex buffer memory!")
        };

        unsafe {
            self.raw.bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("bind_buffer_memory")
        };

        Ok(Buffer {
            raw: buffer,
            desc,
            device_memory: buffer_memory,
            device: self.raw.clone(),
        })
    }

    pub fn create_buffer(
        &self,
        mut desc: BufferDesc,
        initial_data: Option<&[u8]>,
    ) -> Result<Buffer> {
        if initial_data.is_some() {
            desc.usage |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let buffer = self.create_buffer_impl(desc)?;

        Ok(buffer)
    }

    pub fn immediate_destroy_buffer(&self, buffer: &mut Buffer) {
        drop(buffer);
        // unsafe {
        //     self.raw.destroy_buffer(buffer.raw, None);
        // }
    }

    pub fn copy_buffer(
        &self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
    ) {
        self.with_setup_cb(|cb| unsafe {
            let copy_regions = [vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: src_buffer.desc.size as u64,
            }];
            
            self.raw.cmd_copy_buffer(cb, 
                src_buffer.raw, 
                dst_buffer.raw, 
                &copy_regions);
        }).unwrap();
    }
    
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.raw, None);

            self.device.free_memory(self.device_memory, None);
        }
    }
}