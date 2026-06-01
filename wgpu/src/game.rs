use std::mem;

use wgpu::util::{self, DeviceExt};

use crate::{
    application, camera,
    engine::{
        inputs, model,
        render::{self, CameraUniform, GpuVertex},
        texture,
    },
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl render::GpuVertex for Vertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1=>Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub struct GameState {
    pub objects: Vec<u32>,
    pub camera: camera::Camera,
}

impl application::Application for GameState {
    fn setup(context: &mut render::RenderContext, renderer: &mut render::Renderer) -> Self {
        let camera_buffer = context.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&glam::Mat4::IDENTITY.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let global_binding_layout =
            context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Game global bindings"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let global_binding = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game global bindings"),
            layout: &global_binding_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(camera_buffer.as_entire_buffer_binding()),
            }],
        });
        renderer.global_bindings.push(render::Uniform {
            buffer: Some(camera_buffer),
            binding: global_binding,
            group: 0,
        });

        let simple_color_pipeline = {
            let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/simple_color.wgsl").into()),
            });

            let render_pipline_layout =
                context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render pipeline layout"),
                    bind_group_layouts: &[Some(&global_binding_layout)],
                    immediate_size: 0,
                });
            context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&render_pipline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Vertex::descriptor()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pid1 = renderer.register_pipeline(simple_color_pipeline);

        let texture_binding_layout =
            context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture binding"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let model_binding_layout =
            context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model matrix binding"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_pipeline = {
            let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Texture shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/simple_texture.wgsl").into()),
            });
            let render_pipeline_layout =
                context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline layout"),
                    bind_group_layouts: &[
                        Some(&global_binding_layout),
                        Some(&model_binding_layout),
                        Some(&texture_binding_layout),
                    ],
                    immediate_size: 0,
                });
            context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Texture pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[model::ModelVertex::descriptor()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pid2 = renderer.register_pipeline(texture_pipeline);

        let mut objects = Vec::new();
        let obj1 = render::Renderable {
            mesh: render::Mesh::new(
                bytemuck::cast_slice(&[
                    Vertex { position: [0.2, -0.5, 0.0], color: [0.0, 1.0, 1.0] },
                    Vertex { position: [0.8, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                    Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0] },
                ]),
                &[0, 1, 2],
                &context.device,
            ),
            uniforms: Vec::new(),
            pipeline: pid1,
            texture: None,
        };
        objects.push(renderer.register_object(obj1));

        let obj2 = render::Renderable {
            mesh: render::Mesh::new(
                bytemuck::cast_slice(&[
                    Vertex { position: [-0.8, -0.5, 1.0], color: [1.0, 0.0, 0.0] },
                    Vertex { position: [-0.2, -0.5, 1.0], color: [1.0, 0.0, 1.0] },
                    Vertex { position: [-0.5, 0.5, 1.0], color: [1.0, 0.0, 0.0] },
                ]),
                &[0, 1, 2],
                &context.device,
            ),
            uniforms: Vec::new(),
            pipeline: pid1,
            texture: None,
        };
        objects.push(renderer.register_object(obj2));

        let penguin_matrix_buffer = context.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Penguin model matrix"),
            contents: bytemuck::cast_slice(
                &glam::Mat4::from_rotation_x(-90.0f32.to_radians()).to_cols_array(),
            ),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let penguin_matrix_binding = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Penguin model matrix"),
            layout: &model_binding_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(penguin_matrix_buffer.as_entire_buffer_binding()),
            }],
        });
        let penguin = render::Renderable {
            mesh: model::load_model_mesh("../assets/emperor.obj", context)
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
            uniforms: vec![render::Uniform {
                buffer: Some(penguin_matrix_buffer),
                binding: penguin_matrix_binding,
                group: 1,
            }],
            texture: Some(
                texture::Texture::new("../assets/emperor.jpg", Some("Penguin diffuse"), context).unwrap(),
            ),
            pipeline: pid2,
        };
        objects.push(renderer.register_object(penguin));

        let camera = camera::Camera::builder()
            .pos((0.0, 0.0, 1.0).into())
            .aspect(context.config.width as f32 / context.config.height as f32)
            .build();

        Self { objects, camera }
    }

    fn physics_frame(
        &mut self,
        inputs: &mut inputs::Inputs,
        context: &render::RenderContext,
        _: &render::Renderer,
    ) {
        self.camera.aspect = context.config.width as f32 / context.config.height as f32;
        let look = glam::Vec2::from(inputs.consume_mouse_delta());
        self.camera.update_rotation(-look.y * 0.0025, -look.x * 0.0025);

        let [mut dx, mut dy, mut dz] = [0.0; 3];
        if inputs.key_press("keyw") {
            dz += 1.0
        };
        if inputs.key_press("keys") {
            dz -= 1.0
        };
        if inputs.key_press("keya") {
            dx -= 1.0
        };
        if inputs.key_press("keyd") {
            dx += 1.0
        };
        if inputs.key_press("space") {
            dy += 1.0;
        }
        if inputs.key_press("shiftleft") {
            dy -= 1.0;
        }
        self.camera.update_translation(dx * 0.1, dy * 0.1, dz * 0.1);

        if inputs.key_press("escape") {
            inputs.request_quit = true;
        }
    }

    fn render_frame(
        &self,
        _: &inputs::Inputs,
        context: &mut render::RenderContext,
        renderer: &mut render::Renderer,
    ) {
        let Some(buf) = &mut renderer.global_bindings[0].buffer
        else {
            log::error!("Camera buffer not bound to slot 0");
            return;
        };
        context.queue.write_buffer(
            buf,
            0,
            bytemuck::cast_slice(&self.camera.view_proj_matrix().to_cols_array()),
        );

        for &obj in &self.objects {
            renderer.queue_command(render::RenderCommand { id: obj });
        }
    }
}
