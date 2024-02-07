//use ash::vk_make_version;
use crate::debug::ValidationInfo;
use crate::debug::DeviceExtension;

// pub const APPLICATION_VERSION: u32 = vk_make_version!(1, 0, 0);
// pub const ENGINE_VERSION: u32 = vk_make_version!(1, 0, 0);
// pub const API_VERSION: u32 = vk_make_version!(1, 0, 92);

// ash::vk::definitions::make_api_version(0, 1, 2, 0);

// vk::ApplicationInfo::builder().api_version(vk::make_api_version(0, 1, 2, 0));

pub const WINDOW_TITLE: &'static str = "cranberries";
pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;


pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

//VK_LAYER_KHRONOS_validation
//VK_LAYER_RENDERDOC_Capture
pub const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: ["VK_KHR_swapchain"],
};



use crate::vulkan::buffer::Vertex;
/*
pub const _VERTICES_DATA: [Vertex; 3] = [
    Vertex {
        pos: [0.0, -0.5],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub const _VERTICES_DATA_2: [Vertex; 4] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [1.0, 1.0, 1.0],
    },
];
pub const _INDICES_DATA_2: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub const RECT_VERTICES_DATA: [Vertex; 4] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [1.0, 1.0, 1.0],
    },
];*/
// pub const RECT_INDICES_DATA: [u32; 6] = [0, 1, 2, 2, 3, 0];


// pub const RECT_TEX_COORD_VERTICES_DATA: [Vertex; 4] = [
//     Vertex {
//         pos: [-0.75, -0.75],
//         color: [1.0, 0.0, 0.0],
//         tex_coord: [1.0, 0.0],
//     },
//     Vertex {
//         pos: [0.75, -0.75],
//         color: [0.0, 1.0, 0.0],
//         tex_coord: [0.0, 0.0],
//     },
//     Vertex {
//         pos: [0.75, 0.75],
//         color: [0.0, 0.0, 1.0],
//         tex_coord: [0.0, 1.0],
//     },
//     Vertex {
//         pos: [-0.75, 0.75],
//         color: [1.0, 1.0, 1.0],
//         tex_coord: [1.0, 1.0],
//     },
// ];


pub const TEXTURE_PATH: &'static str = "crates/assets/texture.jpg";

pub const RECT_TEX_COORD_VERTICES_DATA_2: [Vertex; 8] = [
    Vertex {
        pos: [-0.75, -0.75, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [0.75, -0.75, 0.0],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [0.75, 0.75, 0.0],
        color: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [-0.75, 0.75, 0.0],
        color: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    Vertex {
        pos: [-0.75, -0.75, -0.75],
        color: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [0.75, -0.75, -0.75],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [0.75, 0.75, -0.75],
        color: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [-0.75, 0.75, -0.75],
        color: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
];

pub const RECT_TEX_COORD_INDICES_DATA_2: [u32; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];