use pollster::FutureExt;
use winit::window::Window;

// TODO: fix visibility
pub struct GraphicsContext<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // must be dropped after the device since it has unsafe references to the window
    pub window: &'a Window,
}

impl<'a> GraphicsContext<'a> {
    pub fn new(window: &'a Window) -> GraphicsContext {
        let size = window.inner_size();

        // handle to the GPU, interfaces with Vulkan, DX12, etc.; main purpose to create adapters and surfaces
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                .. Default::default()
            }
        );

        // surface where rendered frames can be presented, i.e. a window
        let surface = instance.create_surface(window).expect("failed to create surface");

        // handle to the actual GPU, can get info about GPU & create device & queue
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false, // switch to software system instead of hardware
                compatible_surface: Some(&surface), // find an adapter that supports the surface
            }
        ).block_on().expect("failed to request adapter");

        // device is an open connection to gpu, queue executes command buffers
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(), // what additional features of the GPU are needed
                required_limits: wgpu::Limits::default(), // limit properties of the gpu to support different architectures
                label: None,
            },
            None // trace path for api call tracing
        ).block_on().expect("failed to request device");

        // get list of formats, texture usages, presentation modes, etc.
        // presentation mode -> vsync, fifo, etc.
        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        // defines how the surface creates its underlying surface textures
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // texture used as output of a render pass
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2, // max frames that should be queued in advance
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: Vec::new(),
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}