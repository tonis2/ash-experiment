use std::path::Path;
use vulkan::{prelude::*, Buffer, Image, VkInstance};

use gltf::mesh::util::{colors, tex_coords, ReadNormals};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [u16; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct Node {
    pub parent: Option<usize>, //Parent Index
    pub children: Vec<usize>,  //Children Indices
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct GltfResult {
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub skeleton_index: Option<usize>,
}

pub struct Importer {
    doc: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

impl Importer {
    pub fn load<P: AsRef<Path>>(path: P) -> Importer {
        let (doc, buffers, images) = gltf::import(path).expect("Failed to load gltf file");
        Importer {
            doc,
            buffers,
            images,
        }
    }

    pub fn build(&self, vulkan: &VkInstance) -> GltfResult {
        let mut meshes: Vec<Mesh> = Vec::new();
        let mut nodes = Vec::new();

        //Build image textures
        let images: Vec<Image> = self
            .images
            .iter()
            .map(|info| Self::create_texture_image(info, &vulkan))
            .collect();

        //Store Nodes
        for node in self.doc.nodes() {
            let children_indices = node
                .children()
                .map(|child| child.index())
                .collect::<Vec<usize>>();

            let (translation, rotation, scale) = node.transform().decomposed();

            let mut parent = None;

            //If we encounter ourselves (node) when searching children, we've found our parent
            for potential_parent in self.doc.nodes() {
                if potential_parent
                    .children()
                    .find(|child| child.index() == node.index())
                    .is_some()
                {
                    parent = Some(potential_parent.index());
                }
            }

            nodes.push(Node {
                parent: parent,
                children: children_indices,
                translation: translation,
                rotation: rotation,
                scale: scale,
            });

            // let parent_index = match nodes[nodes.len() - 1].parent {
            //     Some(index) => index.to_string(),
            //     None => "N/A".to_string(),
            // };
        }

        for mesh in self.doc.meshes() {
            for primitive in mesh.primitives() {
                let mut vertices: Vec<Vertex> = Vec::new();
                let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));

                //Read mesh data
                let indices: Option<Vec<u32>> = reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect());
                let mut colors: Option<colors::CastingIter<colors::RgbU16>> =
                    reader.read_colors(0).map(|color| color.into_rgb_u16());
                let mut normals: Option<ReadNormals> = reader.read_normals();
                let mut uvs: Option<tex_coords::CastingIter<tex_coords::F32>> =
                    reader.read_tex_coords(0).map(|uvs| uvs.into_f32());

                //Build mesh vertices
                for (index, position) in reader.read_positions().unwrap().enumerate() {
                    vertices.push(Vertex {
                        position,
                        color: [1, 1, 1],
                        normal: [0.0, 0.0, 0.0],
                        uv: [0.0, 0.0],
                    });

                    if let Some(color_iter) = &mut colors {
                        vertices[index].color = color_iter.next().unwrap();
                    }

                    if let Some(normal_iter) = &mut normals {
                        vertices[index].normal = normal_iter.next().unwrap();
                    }

                    if let Some(uv) = &mut uvs {
                        vertices[index].uv = uv.next().unwrap();
                    }
                }
                meshes.push(Mesh {
                    vertices,
                    indices,
                    skeleton_index: None,
                });
            }
        }

        GltfResult { meshes, nodes }
    }

    fn to_vk_texture_format(format: gltf::image::Format) -> vk::Format {
        match format {
            gltf::image::Format::R8 => vk::Format::R8_UNORM,
            gltf::image::Format::R8G8 => vk::Format::R8G8_UNORM,
            gltf::image::Format::R8G8B8A8 => vk::Format::R8G8B8A8_UNORM,
            gltf::image::Format::B8G8R8A8 => vk::Format::B8G8R8A8_UNORM,
            gltf::image::Format::R8G8B8 => vk::Format::R8G8B8_UNORM,
            gltf::image::Format::B8G8R8 => vk::Format::B8G8R8_UNORM,
            gltf::image::Format::R16 => vk::Format::R16_UNORM,
            gltf::image::Format::R16G16 => vk::Format::B8G8R8_UNORM,
            gltf::image::Format::R16G16B16 => vk::Format::R16G16B16_UNORM,
            gltf::image::Format::R16G16B16A16 => vk::Format::R16G16B16A16_UNORM,
        }
    }

    fn create_texture_image(properties: &gltf::image::Data, vulkan: &VkInstance) -> Image {
        let format = vk::Format::R8G8B8A8_UNORM;
        let image_size =
            (std::mem::size_of::<u8>() as u32 * properties.width * properties.height * 4)
                as vk::DeviceSize;

        let buffer = Buffer::new_mapped_basic(
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuOnly,
            vulkan.context(),
        );

        buffer.upload_to_buffer::<u8>(&properties.pixels, 0);
        let mut image = Image::create_image(
            vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                image_type: vk::ImageType::TYPE_2D,
                format,
                extent: vk::Extent3D {
                    width: properties.width,
                    height: properties.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            },
            vk_mem::MemoryUsage::GpuOnly,
            vulkan.context(),
        );

        vulkan.transition_image_layout(
            image.image,
            format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            1,
        );

        vulkan.copy_buffer_to_image(
            buffer.buffer,
            image.image,
            vec![vk::BufferImageCopy {
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image_extent: vk::Extent3D {
                    width: properties.width,
                    height: properties.height,
                    depth: 1,
                },
                buffer_offset: 0,
                buffer_image_height: 0,
                buffer_row_length: 0,
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            }],
        );

        vulkan.transition_image_layout(
            image.image,
            format,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            1,
        );

        image.attach_view(vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            image: image.image,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        });

        image.attach_sampler(vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            max_lod: 1.0,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            ..Default::default()
        });

        image
    }
}