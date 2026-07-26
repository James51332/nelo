//! Build and configure the gpu for rendering

pub mod pipeline;
pub mod target;

pub use target::{Frame, Target, TextureTarget, WindowTarget};

use wgpu::{
    Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits,
    MemoryHints, Queue, RequestAdapterOptions, Surface, SurfaceTarget, TextureFormat, Trace,
};

/// Context which holds device and queue.
pub struct Gpu {
    device: Device,
    queue: Queue,
    format: TextureFormat,
}

impl Gpu {
    /// Create a headless context.
    pub fn headless() -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        pollster::block_on(Self::from_instance(instance, None))
    }

    /// Create a GPU from a surface target. Usually a window.
    pub fn with_surface<'a, T: Into<SurfaceTarget<'a>>>(window: T) -> (Self, Surface<'a>) {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window).unwrap();
        let gpu = pollster::block_on(Self::from_instance(instance, Some(&surface)));
        (gpu, surface)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    async fn from_instance(instance: Instance, surface: Option<&Surface<'_>>) -> Self {
        // Get the handle to physical device.
        let adapter_options = RequestAdapterOptions {
            compatible_surface: surface,
            ..Default::default()
        };
        let adapter = instance.request_adapter(&adapter_options).await.unwrap();

        // Obtain device and queue.
        let required_limits = if cfg!(target_arch = "wasm32") {
            Limits::downlevel_webgl2_defaults()
        } else {
            Limits::default()
        };
        let device_desc = DeviceDescriptor {
            label: Some("nelo render device"),
            required_features: Features::empty(),
            experimental_features: ExperimentalFeatures::disabled(),
            required_limits,
            memory_hints: MemoryHints::default(),
            trace: Trace::Off,
        };
        let (device, queue) = adapter.request_device(&device_desc).await.unwrap();

        // Determine a texture format.
        let format = match surface {
            Some(surface) => {
                let caps = surface.get_capabilities(&adapter);
                caps.formats
                    .iter()
                    .find(|f| f.is_srgb())
                    .copied()
                    .unwrap_or(caps.formats[0])
            }
            None => TextureFormat::Rgba8UnormSrgb,
        };

        Self {
            device,
            queue,
            format,
        }
    }
}
