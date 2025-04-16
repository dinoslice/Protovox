use crate::components::Transform;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::math::Vertex;
use crate::rendering::sized_buffer::SizedBuffer;
use gltf::buffer::Data;
use shipyard::{Component, IntoIter, UniqueView, UniqueViewMut, ViewMut};
use std::ops::Deref;
use std::path::Path;
use std::slice::Iter;
use glm::Quat;
use gltf::image::Format;
use image::{ColorType, DynamicImage, GenericImage, Rgba};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::BufferUsages;
use crate::entity::ModelView;
use crate::rendering::model_render::ModelMap;
use crate::rendering::texture::Texture;

pub struct Model {
    parts: Vec<ModelPart>,
    gltf: gltf::Document,
    buffers: Vec<Data>,
    images: Vec<gltf::image::Data>,
    pub texture: Vec<DynamicImage>,
}

impl Model {
    pub fn open(path: impl AsRef<Path>) -> Model {
        let mut parts = Vec::new();

        let (gltf, buffers, images) = gltf::import(path).expect("Failed to read GLTF");

        let mut dyn_images = Vec::new();
        for (i, image) in images.iter().enumerate() {
            let format = match image.format {
                Format::R8 => ColorType::L8,
                Format::R8G8 => ColorType::La8,
                Format::R8G8B8 => ColorType::Rgb8,
                Format::R8G8B8A8 => ColorType::Rgba8,
                Format::R16 => ColorType::L16,
                Format::R16G16 => ColorType::La16,
                Format::R16G16B16 => ColorType::Rgb16,
                Format::R16G16B16A16 => ColorType::Rgba16,
                Format::R32G32B32FLOAT => ColorType::Rgb32F,
                Format::R32G32B32A32FLOAT => ColorType::Rgba32F,
            };

            let mut dyn_image = DynamicImage::new(image.width, image.height, format);

            let bytes_per_channel = format.bytes_per_pixel() / format.channel_count();
            let mut x = 0;
            let mut y = 0;
            for pixel in image.pixels.chunks_exact(format.bytes_per_pixel() as _) {
                let mut rgba = [0u32, 0, 0, 0];
                for x in 0..format.channel_count() as usize {
                    for b in 0..bytes_per_channel as usize {
                        rgba[x] |= (pixel[x + b] << (b * 8)) as u32;
                    }
                }

                dyn_image.put_pixel(x, y, Rgba([rgba[0] as _, rgba[1] as _, rgba[2] as _, rgba[3] as _]));
                x += 1;
                if x >= image.width {
                    x = 0;
                    y += 1;
                }
            }

            dyn_images.push(dyn_image);
        }

        for model in gltf.nodes() {
            let mesh = model.mesh();
            if let Some(mesh) = mesh {
                parts.push(ModelPart::from(mesh, &buffers, model.transform()));
            }
        }

        Model { parts, gltf, buffers, images, texture: dyn_images }
    }

    pub fn update(&mut self, g_ctx: &GraphicsContext) {
        for part in self.parts.iter_mut() {
            part.update(&g_ctx);
        }
    }

    pub fn iter(&self) -> Iter<'_, ModelPart> {
        self.parts.iter()
    }
}

pub struct ModelPart {
    offset: Transform,
    rot: Quat,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    pub buffer: Option<SizedBuffer>,
    dirty: bool,
}

impl ModelPart {
    pub fn from(mesh: gltf::Mesh, buffers: &Vec<Data>, transform: gltf::scene::Transform) -> ModelPart {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| {
                Some(&buffers[buffer.index()])
            });

            let Some(positions) = reader.read_positions() else {
                continue;
            };

            let Some(textures) = reader.read_tex_coords(0) else {
                continue;
            };

            let Some(index) = reader.read_indices() else {
                continue;
            };

            // TODO: make it so that position's length is preserved
            for (position, tex_coords) in positions.zip(textures.into_f32().into_iter()) {
                vertices.push(Vertex { position, tex_coords });
            }

            indices.append(&mut index.into_u32().into_iter().collect::<Vec<_>>());
        }

        Self {
            rot: Quat::from(transform.clone().decomposed().1),
            offset: Transform::from(transform),
            vertices,
            indices,
            buffer: None,
            dirty: true,
        }
    }

    pub fn apply_transform(&self, parent_transform: &Transform) -> Transform {
        Transform {
            position: parent_transform.position + self.offset.position,
            scale: parent_transform.scale + self.offset.scale,
            rotation: parent_transform.rotation + self.offset.rotation,
        }
    }

    pub fn offset(&self) -> &Transform {
        &self.offset
    }

    pub fn offset_mut(&mut self) -> &mut Transform {
        self.mark_dirty();
        &mut self.offset
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn update(&mut self, g_ctx: &GraphicsContext) {
        if self.dirty {
            self.dirty = false;
            let mut transformed = Vec::new();
            for index in &self.indices {
                transformed.push((self.vertices[*index as usize] * &self.rot) * &self.offset.scale + &self.offset.position);
            }
            if let Some(buffer) = self.buffer.as_ref() {
                // TODO: update to add rotation
                g_ctx.queue.write_buffer(buffer.deref(), 0, bytemuck::cast_slice(&transformed));
            } else {
                let buffer = g_ctx.device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&transformed),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });

                self.buffer = Some(SizedBuffer::new(buffer, transformed.len() as _));
            }
        }
    }
}

pub fn update_dirty_models(vm_models: ViewMut<ModelView>, mut model_map: UniqueViewMut<ModelMap>, g_ctx: UniqueView<GraphicsContext>) {
    for model in (&vm_models).iter() {
        let Some((model, ..)) = model_map.map.get_mut(&model.0) else {
            continue;
        };

        model.update(&g_ctx);
    }
}