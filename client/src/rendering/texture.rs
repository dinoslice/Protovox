use image::{GenericImageView, ImageError};

#[allow(dead_code)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str
    ) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok( Self::from_image(device, queue, &img, Some(label)) )
    }

    pub fn from_images_2d(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        images: &[image::DynamicImage],
        label: Option<&str>,
    ) -> Option<Self> {
        Self::from_images_2d_inner(device, queue, images, label, &wgpu::TextureViewDescriptor::default())
    }

    pub fn new_cubemap(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        images: &[image::DynamicImage; 6],
        label: Option<&str>,
    ) -> Option<Self> {
        let view_descriptor = wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            aspect: wgpu::TextureAspect::All,
            ..Default::default()
        };

        Self::from_images_2d_inner(device, queue, images, label, &view_descriptor)
    }

    fn from_images_2d_inner(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        images: &[image::DynamicImage],
        label: Option<&str>,
        view_descriptor: &wgpu::TextureViewDescriptor,
    ) -> Option<Self> {
        let dimensions = images.first()?.dimensions();

        if images.iter().any(|img| img.dimensions() != dimensions) {
            return None;
        }

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: images.len() as _,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1, // multisampling
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // texture_binding -> use texture in shaders, copy_dst -> copy data into texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );


        for (i, image) in images.into_iter().enumerate() {
            let rgba = image.to_rgba8();

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        z: i as u32,
                        .. wgpu::Origin3d::ZERO
                    },
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(size_of::<image::Rgba<u8>>() as u32 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                wgpu::Extent3d {
                    depth_or_array_layers: 1,
                        .. size
                }
            );
        }

        // view into the texture to read from it
        let view = texture.create_view(view_descriptor);

        // sampler uses tex-coords (u, v) and sampler returns data from the texture
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge, // repeats the last pixel, to the end
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest, // when close to the texture, use nearest when displaying
                min_filter: wgpu::FilterMode::Nearest, // when far, use nearest when displaying
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Some(Self { texture, view, sampler })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>
    ) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1, // represent 2d texture as 1, since stored as 3d texture
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1, // multisampling
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // texture_binding -> use texture in shaders, copy_dst -> copy data into texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture( // writes data to the texture
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba, // image data
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0), // 4 bytes per pixel, rgba
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        // view into the texture to read from it
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // sampler uses tex-coords (u, v) and sampler returns data from the texture
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge, // repeats the last pixel, to the end
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest, // when close to the texture, use nearest when displaying
                min_filter: wgpu::FilterMode::Nearest, // when far, use nearest when displaying
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // store depth as f32

    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        // depth texture is same size as screen, values taken from surface config
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // rendering to the depth texture
                | wgpu::TextureUsages::TEXTURE_BINDING, // use texture in shaders
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // sampler not needed for depth texture, needs rewrite
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // for rendering the texture?
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}
