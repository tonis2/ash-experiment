use gltf::{
    material::{AlphaMode, Material as GltfMaterial, NormalTexture, OcclusionTexture},
    scene::Transform,
    texture,
};
use std::{path::Path, sync::Arc};
use vulkan::{prelude::*, Buffer, Image, VkThread};

pub struct Importer {
    doc: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
    pub textures: Vec<Image>,
    pub materials: Vec<Material>,
    pub vertices: Arc<Buffer>,
    pub indices: Arc<Buffer>,
    pub indices_len: u32,
}

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub material_id: isize,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub index: usize,
    pub mesh_index: Option<usize>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub translation: Transform,
    pub transform_matrix: cgmath::Matrix4<f32>,
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub vertex_offset: usize,
    pub indice_offset: u64,
    pub material_id: Option<isize>,
    pub vertice_len: usize,
    pub primitive_topology: vk::PrimitiveTopology,
}
#[allow(dead_code)]
pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
    pub index: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureInfo {
    index: isize,
    channel: u32,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub name: Option<String>,
    pub index: Option<usize>,
    pub base_color: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_color: [f32; 3],
    pub color: [f32; 4],
    pub emissive: [f32; 3],
    pub occlusion: f32,
    pub color_texture: TextureInfo,
    pub emissive_texture: TextureInfo,
    pub normals_texture: TextureInfo,
    pub occlusion_texture: TextureInfo,
    pub workflow: Workflow,
    pub alpha_mode: u32,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub is_unlit: bool,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct MaterialRaw {
    pub base_color: [f32; 4],
    pub color: [f32; 4],
    pub emissive: [f32; 4],
    pub emissive_color: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub occlusion: f32,
    pub color_texture: TextureInfo,
    pub emissive_texture: TextureInfo,
    pub normals_texture: TextureInfo,
    pub occlusion_texture: TextureInfo,
}

#[derive(Clone, Copy, Debug)]
pub enum Workflow {
    MetallicRoughness(MetallicRoughnessWorkflow),
    SpecularGlossiness(SpecularGlossinessWorkflow),
}

#[derive(Clone, Copy, Debug)]
pub struct MetallicRoughnessWorkflow {
    metallic: f32,
    roughness: f32,
    metallic_roughness_texture: TextureInfo,
}

#[derive(Clone, Copy, Debug)]
pub struct SpecularGlossinessWorkflow {
    specular: [f32; 3],
    glossiness: f32,
    specular_glossiness_texture: TextureInfo,
}

impl Scene {
    pub fn get_mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
    }

    pub fn get_raw_materials(&self) -> Vec<MaterialRaw> {
        self.materials
            .iter()
            .map(|material| material.raw())
            .collect()
    }
}

impl Material {
    pub fn raw(&self) -> MaterialRaw {
        MaterialRaw {
            base_color: self.base_color,
            color: self.color,
            emissive: [self.emissive[0], self.emissive[1], self.emissive[2], 1.0],
            emissive_color: [
                self.emissive_color[0],
                self.emissive_color[1],
                self.emissive_color[2],
                1.0,
            ],
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            occlusion: self.occlusion,
            color_texture: self.color_texture,
            emissive_texture: self.emissive_texture,
            normals_texture: self.normals_texture,
            occlusion_texture: self.occlusion_texture,
        }
    }
}
impl Default for TextureInfo {
    fn default() -> Self {
        TextureInfo {
            index: -1,
            channel: 0,
        }
    }
}

impl Default for MaterialRaw {
    fn default() -> Self {
        MaterialRaw {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.0,
            emissive_color: [1.0, 1.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: [1.0, 1.0, 1.0, 1.0],
            occlusion: 1.0,
            color_texture: TextureInfo::default(),
            emissive_texture: TextureInfo::default(),
            normals_texture: TextureInfo::default(),
            occlusion_texture: TextureInfo::default(),
        }
    }
}

impl<'a> From<GltfMaterial<'a>> for Material {
    fn from(material: GltfMaterial) -> Material {
        let color = match material.pbr_specular_glossiness() {
            Some(pbr) => pbr.diffuse_factor(),
            _ => material.pbr_metallic_roughness().base_color_factor(),
        };

        fn get_texture(texture_info: Option<texture::Info>) -> TextureInfo {
            texture_info
                .map(|tex_info| TextureInfo {
                    index: tex_info.texture().index() as isize,
                    channel: tex_info.tex_coord(),
                })
                .unwrap_or(TextureInfo {
                    index: -1,
                    channel: 0,
                })
        }

        fn get_normals_texture(texture_info: Option<NormalTexture>) -> TextureInfo {
            texture_info
                .map(|tex_info| TextureInfo {
                    index: tex_info.texture().index() as isize,
                    channel: tex_info.tex_coord(),
                })
                .unwrap_or(TextureInfo {
                    index: -1,
                    channel: 0,
                })
        }

        fn get_occlusion(texture_info: Option<OcclusionTexture>) -> (f32, TextureInfo) {
            let strength = texture_info
                .as_ref()
                .map_or(0.0, |tex_info| tex_info.strength());

            let texture = texture_info
                .map(|tex_info| TextureInfo {
                    index: tex_info.texture().index() as isize,
                    channel: tex_info.tex_coord(),
                })
                .unwrap_or(TextureInfo {
                    index: -1,
                    channel: 0,
                });

            (strength, texture)
        }

        let emissive = material.emissive_factor();

        let color_texture = match material.pbr_specular_glossiness() {
            Some(pbr) => pbr.diffuse_texture(),
            _ => material.pbr_metallic_roughness().base_color_texture(),
        };
        let color_texture = get_texture(color_texture);
        let emissive_texture = get_texture(material.emissive_texture());
        let normals_texture = get_normals_texture(material.normal_texture());
        let (occlusion, occlusion_texture) = get_occlusion(material.occlusion_texture());

        let workflow = match material.pbr_specular_glossiness() {
            Some(pbr) => Workflow::SpecularGlossiness(SpecularGlossinessWorkflow {
                specular: pbr.specular_factor(),
                glossiness: pbr.glossiness_factor(),
                specular_glossiness_texture: get_texture(pbr.specular_glossiness_texture()),
            }),
            _ => {
                let pbr = material.pbr_metallic_roughness();
                Workflow::MetallicRoughness(MetallicRoughnessWorkflow {
                    metallic: pbr.metallic_factor(),
                    roughness: pbr.roughness_factor(),
                    metallic_roughness_texture: get_texture(pbr.metallic_roughness_texture()),
                })
            }
        };

        let alpha_mode = match material.alpha_mode() {
            AlphaMode::Opaque => 1,
            AlphaMode::Mask => 2,
            AlphaMode::Blend => 3,
        };

        Material {
            index: material.index(),
            name: material.name().map(String::from),

            base_color: material.pbr_metallic_roughness().base_color_factor(),
            metallic_factor: material.pbr_metallic_roughness().metallic_factor(),
            roughness_factor: material.pbr_metallic_roughness().roughness_factor(),
            emissive_color: material.emissive_factor(),

            color,
            emissive,
            occlusion,
            color_texture,
            emissive_texture,
            normals_texture,
            occlusion_texture,
            workflow,
            alpha_mode,
            alpha_cutoff: material.alpha_cutoff(),
            double_sided: material.double_sided(),
            is_unlit: material.unlit(),
        }
    }
}

impl Importer {
    //Load gltf data from file
    pub fn load<P: AsRef<Path>>(path: P) -> Importer {
        let (doc, buffers, images) = gltf::import(path).expect("Failed to load gltf file");
        Importer {
            doc,
            buffers,
            images,
        }
    }
    //Parse and build gltf content
    pub fn build(&self, vulkan: &VkThread) -> Scene {
        let mut meshes: Vec<Mesh> = Vec::new();
        let mut nodes = Vec::new();

        let mut textures: Vec<Image> = self
            .images
            .iter()
            .map(|image| Self::create_texture_image(image, &vulkan))
            .collect();

        let samplers: Vec<vk::SamplerCreateInfo> = self
            .doc
            .samplers()
            .map(|sampler| Self::build_sampler(&sampler))
            .collect();

        let materials: Vec<Material> = self
            .doc
            .materials()
            .map(|material| Material::from(material))
            .collect();

        for texture in self.doc.textures() {
            let image_index = texture.source().index();
            let sampler_index = texture.sampler().index();

            if sampler_index.is_some() {
                textures[image_index].attach_sampler(samplers[sampler_index.unwrap()])
            } else {
                textures[image_index].attach_sampler(vk::SamplerCreateInfo {
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
                })
            }
        }

        let mut vertices_data: Vec<Vertex> = Vec::new();
        let mut indices_data: Vec<u32> = Vec::new();

        //Store Nodes
        for node in self.doc.nodes() {
            let children_indices = node
                .children()
                .map(|child| child.index())
                .collect::<Vec<usize>>();

            let local_transform = node.transform();
            let transform_matrix = compute_transform_matrix(&local_transform);
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
                index: node.index(),
                mesh_index: node.mesh().map(|mesh| mesh.index()),
                parent: parent,
                children: children_indices,
                translation: local_transform,
                transform_matrix,
            });
        }

        for mesh in self.doc.meshes() {
            let primitives: Vec<Primitive> = mesh
                .primitives()
                .map(|primitive| {
                    let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));
                    //Read mesh data
                    use gltf::mesh::Mode;
                    let primitive_topology = match primitive.mode() {
                        Mode::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
                        Mode::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
                        Mode::Lines => vk::PrimitiveTopology::LINE_LIST,
                        Mode::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
                        Mode::Points => vk::PrimitiveTopology::POINT_LIST,
                        mode @ _ => panic!("unsupported primitive mode: {:?}", mode),
                    };

                    let indices: Option<Vec<u32>> = reader
                        .read_indices()
                        .map(|read_indices| read_indices.into_u32().collect());
                    let colors: Vec<[f32; 4]> = reader
                        .read_colors(0)
                        .map_or(vec![], |color| color.into_rgba_f32().collect());
                    let normals: Vec<[f32; 3]> = reader
                        .read_normals()
                        .map_or(vec![], |normals| normals.collect());
                    let uvs: Vec<[f32; 2]> = reader
                        .read_tex_coords(0)
                        .map_or(vec![], |uvs| uvs.into_f32().collect());
                    let material_id: Option<isize> =
                        primitive.material().index().map(|num| num as isize);
                    let vertices: Vec<Vertex> = reader
                        .read_positions()
                        .unwrap()
                        .enumerate()
                        .map(|(index, position)| Vertex {
                            position,
                            color: *colors.get(index).unwrap_or(&[1.0, 1.0, 1.0, 1.0]),
                            uv: *uvs.get(index).unwrap_or(&[0.0, 0.0]),
                            normal: *normals.get(index).unwrap_or(&[1.0, 1.0, 1.0]),
                            material_id: material_id.unwrap_or(-1),
                        })
                        .collect();

                    vertices_data.extend_from_slice(&vertices);

                    let mut indice_offset: u64 = 0;
                    if indices.is_some() {
                        let indice_data = &indices.unwrap();

                        indice_offset = (std::mem::size_of_val(&indice_data)
                            - std::mem::size_of::<u64>())
                            as u64;
                        indices_data.extend_from_slice(indice_data);
                    }

                    Primitive {
                        vertex_offset: (vertices_data.len() - vertices.len())
                            * std::mem::size_of::<Vertex>(),
                        indice_offset,
                        material_id,
                        vertice_len: vertices.len(),
                        primitive_topology,
                    }
                })
                .collect();

            meshes.push(Mesh {
                index: mesh.index(),
                name: mesh.name().map(String::from),
                primitives,
            });
        }

        Scene {
            meshes,
            nodes,
            textures,
            materials,
            indices_len: indices_data.len() as u32,
            vertices: Arc::new(
                vulkan.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vertices_data),
            ),
            indices: Arc::new(
                vulkan.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &indices_data),
            ),
        }
    }

    fn build_sampler(sampler: &gltf::texture::Sampler) -> vk::SamplerCreateInfo {
        use gltf::texture::MagFilter;
        use gltf::texture::MinFilter;
        use gltf::texture::WrappingMode;

        fn address_mode(wrap_mode: WrappingMode) -> vk::SamplerAddressMode {
            match wrap_mode {
                WrappingMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
                WrappingMode::Repeat => vk::SamplerAddressMode::REPEAT,
                WrappingMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            }
        };

        fn min_filter_mimap_filter(min_filter: MinFilter) -> (vk::Filter, vk::SamplerMipmapMode) {
            match min_filter {
                MinFilter::Linear => (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR),
                MinFilter::Nearest => (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST),
                MinFilter::LinearMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
                MinFilter::LinearMipmapNearest => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                MinFilter::NearestMipmapNearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                MinFilter::NearestMipmapLinear => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::LINEAR)
                }
            }
        }

        let (min_filter, mipmap_filter) = min_filter_mimap_filter(
            sampler
                .min_filter()
                .unwrap_or(gltf::texture::MinFilter::Nearest),
        );

        let mag_filter = match sampler
            .mag_filter()
            .unwrap_or(gltf::texture::MagFilter::Nearest)
        {
            MagFilter::Nearest => vk::Filter::NEAREST,
            MagFilter::Linear => vk::Filter::LINEAR,
        };

        vk::SamplerCreateInfo {
            mag_filter,
            min_filter,
            mipmap_mode: mipmap_filter,
            address_mode_u: address_mode(sampler.wrap_s()),
            address_mode_v: address_mode(sampler.wrap_t()),
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            max_lod: 1.0,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            ..Default::default()
        }
    }

    fn create_texture_image(properties: &gltf::image::Data, vulkan: &VkThread) -> Image {
        use gltf::image::Format;
        use image::{Bgr, Bgra, ConvertBuffer, ImageBuffer, Rgb, Rgba};
        let format = vk::Format::R8G8B8A8_UNORM;
        type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;
        type BgraImage = ImageBuffer<Bgra<u8>, Vec<u8>>;
        type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;
        type BgrImage = ImageBuffer<Bgr<u8>, Vec<u8>>;

        //Convert image format to R8G8B8A8_UNORM
        let data = match properties.format {
            Format::R8 | Format::R8G8 | Format::R8G8B8 => {
                let rgba: RgbaImage = RgbImage::from_raw(
                    properties.width,
                    properties.height,
                    properties.pixels.clone(),
                )
                .unwrap()
                .convert();

                rgba.into_raw()
            }
            Format::B8G8R8 => {
                let bgra: RgbaImage = BgrImage::from_raw(
                    properties.width,
                    properties.height,
                    properties.pixels.clone(),
                )
                .unwrap()
                .convert();

                bgra.into_raw()
            }
            Format::B8G8R8A8 => {
                let bgra: RgbaImage = BgraImage::from_raw(
                    properties.width,
                    properties.height,
                    properties.pixels.clone(),
                )
                .unwrap()
                .convert();

                bgra.into_raw()
            }
            Format::R8G8B8A8 => properties.pixels.clone(),
            _ => {
                panic!("Unsupported texture format: {:?}", properties.format);
            }
        };

        let image_size =
            (std::mem::size_of::<u8>() as u32 * properties.width * properties.height * 4)
                as vk::DeviceSize;

        let buffer = Buffer::new_mapped_basic(
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuOnly,
            vulkan.context(),
        );

        buffer.upload_to_buffer::<u8>(&data, 0);
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
                usage: vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
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

        image
    }
}

fn compute_transform_matrix(transform: &Transform) -> cgmath::Matrix4<f32> {
    match transform {
        Transform::Matrix { matrix } => cgmath::Matrix4::from(*matrix),
        Transform::Decomposed {
            translation,
            rotation: [xr, yr, zr, wr],
            scale: [xs, ys, zs],
        } => {
            let translation =
                cgmath::Matrix4::from_translation(cgmath::Vector3::from(*translation));
            let rotation = cgmath::Matrix4::from(cgmath::Quaternion::new(*wr, *xr, *yr, *zr));
            let scale = cgmath::Matrix4::from_nonuniform_scale(*xs, *ys, *zs);
            translation * rotation * scale
        }
    }
}
