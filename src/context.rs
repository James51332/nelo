/// The GPU context owns the device and queue. It has no surface and no
/// pipelines, so it can be created headlessly (for export and tests) or
/// alongside a window. Renderers and targets borrow it.
pub struct Gpu {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Gpu {
    /// Create a headless context — no window required.
    pub async fn headless() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        Self::from_instance(instance, None).await
    }

    /// Create a context together with a surface for `window`. The adapter is
    /// selected to be compatible with the surface. Returns the context and the
    /// raw surface, which the caller wraps in a `WindowTarget`.
    pub async fn with_surface(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
    ) -> (Self, wgpu::Surface<'static>) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window).unwrap();
        let gpu = Self::from_instance(instance, Some(&surface)).await;
        (gpu, surface)
    }

    async fn from_instance(
        instance: wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface,
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Nelo Render Device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
}
