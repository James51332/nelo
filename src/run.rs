use crate::context::Context;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(not(target_arch = "wasm32"))]
use env_logger;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<Context>>,
    context: Option<Context>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<Context>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            context: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<Context> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use log::info;
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            info!("test0");
            let window = wgpu::web_sys::window().unwrap_throw();
            info!("test");
            let document = window.document().unwrap_throw();

            info!("test1");
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();

            info!("test2");
            let html_canvas_element = canvas.unchecked_into();

            info!("test3");
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to await the window creation.
            self.context = Some(pollster::block_on(Context::new(window)));
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(Context::new(window).await).is_ok())
                });
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, context: Context) {
        self.context = Some(context);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let context = match &mut self.context {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => context.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                context.render();
            }
            _ => (),
        }
    }
}

/// This is an example flow for how a nelo app might look. We'll consider better ways to abstract this in the future.
pub fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build().unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new();
        event_loop.run_app(&mut app).unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    {
        let app = App::new(&event_loop);
        event_loop.spawn_app(app);
    }
}
