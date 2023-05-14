use only_pos::OnlyPos;
use wgpu::TextureFormat;
use winit::{event::Event, event_loop::ControlFlow};

use log::info;
use log::warn;
use winit::{
    event::{KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

mod only_pos;

struct RenderContext {
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain_format: wgpu::TextureFormat,
}
impl RenderContext {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        info!("{instance:?}");
        let surface_result = unsafe { instance.create_surface(window) };
        let surface = surface_result.expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        info!("{adapter:?}");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        info!("{device:?}");

        let caps = surface.get_capabilities(&adapter);
        info!("{caps:?}");

        let swapchain_format = *caps
            .formats
            .first()
            .expect("No supported swap-chain texture formats");
        let present_mode = *caps
            .present_modes
            .first()
            .expect("No supported present modes");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![swapchain_format],
        };

        surface.configure(&device, &config);

        Self {
            config,
            surface,
            device,
            queue,
            swapchain_format,
        }
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let context = RenderContext::new(&window).await;
    let only_pos = OnlyPos::new(&context);

    event_loop.run(move |event, _, control_flow| {
        log::trace!("Event: {:?}", event);

        // Request an immediate redraw, if not changed by the event handler.
        *control_flow = ControlFlow::Poll;

        // Handle window events.
        match event {
            Event::RedrawRequested(_) => {
                // Get next frame
                let frame = context
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                // Get frame texture view
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let secondary_pass = {
                    let mut secondary_pass = context.device.create_render_bundle_encoder(
                        &wgpu::RenderBundleEncoderDescriptor {
                            label: None,
                            color_formats: &[Some(context.config.format)],
                            depth_stencil: None,
                            sample_count: 1,
                            multiview: None,
                        },
                    );

                    // Render the objects
                    only_pos.render(&mut secondary_pass);

                    secondary_pass.finish(&Default::default())
                };

                // Create a command encoder (to record draw calls)
                let mut encoder = context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    // Create a render pass
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    pass.execute_bundles([&secondary_pass]);
                }

                // Submit command buffer and present frame
                context.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode, ..
                            },
                        ..
                    },
                ..
            } => {
                if let Some(vk) = virtual_keycode {
                    match vk {
                        VirtualKeyCode::F12 | VirtualKeyCode::Escape => {
                            warn!("F12 pressed, quit!");
                            std::process::exit(0);
                        }
                        _ => (),
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Handle the main events cleared event
            Event::MainEventsCleared => {
                // Manually request Redraw
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let builder = winit::window::WindowBuilder::new().with_title("Railroad Scheduler");

    let window = builder.build(&event_loop).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        use log::Level;
        use winit::platform::web::WindowBuilderExtWebSys;
        use winit::platform::web::WindowExtWebSys;

        // Set the panic hook to print to the console.
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Setup the logger to print to the console as well.
        console_log::init_with_level(Level::Warn).expect("could not initialize logger");

        // Append the canvas to the document body
        {
            let win = web_sys::window().expect("no global `window` exists");
            let doc = win.document().expect("should have a document on window");
            let body = doc.body().expect("document should have a body");

            // Remove all children from the body
            while let Some(child) = body.first_child() {
                body.remove_child(&child).ok();
            }

            // Get the winit canvas and append it to the body
            let canvas = window.canvas();
            canvas.style().set_css_text("background-color: black;");
            body.append_child(&web_sys::Element::from(canvas))
                .expect("couldn't append canvas to document body");
        }

        // Spawn the main loop.
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
