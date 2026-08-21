//! Resizeable render target with fixed aspect ratio.

use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

pub struct Canvas {
    target: Texture,

    format: TextureFormat,
    view: TextureView,

    ui_format: TextureFormat,
    ui_view: TextureView,

    width: u32,
    height: u32,
}

impl Canvas {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        ui_format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        dbg!(&format, &ui_format);
        let view_formats = &[ui_format];
        let texture_desc = Self::texture_desc(format, view_formats, width, height);
        let target = device.create_texture(&texture_desc);
        let view = target.create_view(&TextureViewDescriptor::default());
        let ui_view = target.create_view(&TextureViewDescriptor {
            label: Some("nelo scene ui view"),
            format: Some(ui_format),
            ..Default::default()
        });

        Self {
            target,

            format,
            view,

            ui_format,
            ui_view,

            width,
            height,
        }
    }

    /// Resize keeps a fixed aspect ratio. Just accept the width and compute from the aspect ratio.
    pub fn resize(&mut self, device: &Device, height: u32) {
        let aspect = self.aspect();
        let width = (aspect * height as f32) as u32;
        let view_formats = &[self.ui_format];
        let texture_desc = Self::texture_desc(self.format, view_formats, width, height);

        self.target = device.create_texture(&texture_desc);
        self.view = self.target.create_view(&TextureViewDescriptor::default());
        self.ui_view = self.target.create_view(&TextureViewDescriptor {
            label: Some("nelo scene ui view"),
            format: Some(self.ui_format),
            ..Default::default()
        });
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn ui_view(&self) -> &TextureView {
        &self.ui_view
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.width
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    fn texture_desc(
        format: TextureFormat,
        view_formats: &[TextureFormat],
        width: u32,
        height: u32,
    ) -> TextureDescriptor<'_> {
        let (width, height) = (width.max(1), height.max(1));
        TextureDescriptor {
            label: Some("nelo scene render texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats,
        }
    }
}
