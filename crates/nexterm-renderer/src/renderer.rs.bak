use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;
use tracing::info;

use nexterm_terminal::screen::Screen;

use crate::font::FontSystem;
use crate::grid::{GridRenderer, Vertex};
use crate::theme::Theme;
use crate::{RendererError, Result};

pub struct Renderer {
    surface:       wgpu::Surface<'static>,
    device:        wgpu::Device,
    queue:         wgpu::Queue,
    config:        wgpu::SurfaceConfiguration,
    pub width:     u32,
    pub height:    u32,
    pipeline:      wgpu::RenderPipeline,
    pub font:      FontSystem,
    pub theme:     Theme,
    grid:          GridRenderer,
    bg_vertex_buf: wgpu::Buffer,
    bg_index_buf:  wgpu::Buffer,
}

impl Renderer {
    pub async fn new(
        window:      Arc<Window>,
        font_name:   &str,
        font_size:   f32,
        line_height: f32,
    ) -> Result<Self> {
        let size = window.inner_size();

        info!("Initializing renderer: {}x{}", size.width, size.height);

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            }
        );

        let surface = instance
            .create_surface(window)
            .map_err(|e| RendererError::Surface(e.to_string()))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference:       wgpu::PowerPreference::HighPerformance,
                compatible_surface:     Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| RendererError::NoDevice(
                "No GPU adapter found".into()
            ))?;

        info!("GPU: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label:             Some("nexterm_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits:   wgpu::Limits::default(),
                    
                },
                None,
            )
            .await
            .map_err(|e| RendererError::NoDevice(e.to_string()))?;

        let surface_caps   = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:       surface_format,
            width:        size.width,
            height:       size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode:   surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label:  Some("nexterm_shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shader.wgsl").into()
                ),
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label:                Some("nexterm_pipeline_layout"),
                bind_group_layouts:   &[],
                push_constant_ranges: &[],
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label:  Some("nexterm_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module:              &shader,
                    entry_point:         "vs_main",
                    buffers:             &[Vertex::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module:      &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format:     config.format,
                        blend:      Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology:           wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face:         wgpu::FrontFace::Ccw,
                    cull_mode:          None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count:                     1,
                    mask:                      !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        );

        let font  = FontSystem::new(font_name, font_size, line_height)
            .map_err(|e| RendererError::Font(e.to_string()))?;

        let theme = Theme::dark();
        let grid  = GridRenderer::new();

        let bg_vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("bg_vertex_buf"),
                contents: &[],
                usage:    wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bg_index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("bg_index_buf"),
                contents: &[],
                usage:    wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST,
            }
        );

        info!("Renderer initialized");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            width:  size.width,
            height: size.height,
            pipeline,
            font,
            theme,
            grid,
            bg_vertex_buf,
            bg_index_buf,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.width         = width;
        self.height        = height;
        self.config.width  = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, screen: &Screen) -> Result<()> {
        self.grid.build(
            screen,
            &self.font,
            &self.theme,
            self.width,
            self.height,
        );

        self.update_buffers();

        let output = self.surface
            .get_current_texture()
            .map_err(|e| RendererError::Render(e.to_string()))?;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("nexterm_encoder"),
            }
        );

        {
            let mut pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("nexterm_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view:           &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load:  wgpu::LoadOp::Clear(
                                    self.theme.background.to_wgpu()
                                ),
                                store: wgpu::StoreOp::Store,
                            },
                        }
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes:         None,
                    occlusion_query_set:      None,
                }
            );

            pass.set_pipeline(&self.pipeline);

            if !self.grid.bg_indices.is_empty() {
                pass.set_vertex_buffer(0, self.bg_vertex_buf.slice(..));
                pass.set_index_buffer(
                    self.bg_index_buf.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                pass.draw_indexed(
                    0..self.grid.bg_indices.len() as u32,
                    0,
                    0..1,
                );
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn grid_size(&self) -> (u16, u16) {
        self.font.calculate_grid_size(self.width, self.height)
    }

    fn update_buffers(&mut self) {
        if self.grid.bg_vertices.is_empty() {
            return;
        }

        let vertex_data = bytemuck::cast_slice(&self.grid.bg_vertices);
        let index_data  = bytemuck::cast_slice(&self.grid.bg_indices);

        self.bg_vertex_buf = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("bg_vertex_buf"),
                contents: vertex_data,
                usage:    wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST,
            }
        );

        self.bg_index_buf = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("bg_index_buf"),
                contents: index_data,
                usage:    wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST,
            }
        );
    }
}