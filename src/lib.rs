// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub use rayca_core::*;
pub use rayca_pipe::*;

pipewriter!(Main, "shaders/main.vert.slang", "shaders/main.frag.slang");
pipewriter!(Line, "shaders/line.vert.slang", "shaders/line.frag.slang");

#[repr(C, align(16))]
struct PushConstant {
    pretransform: Mat4,
}

impl AsBytes for PushConstant {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl RenderPipeline for PipelineLine {
    fn render(
        &self,
        frame: &mut Frame,
        scene: &RenderScene,
        camera_infos: &[CameraDrawInfo],
        infos: Vec<DrawInfo>,
    ) {
        self.bind(&frame.cache);
        frame.set_viewport_and_scissor(1.0, true);

        // Apply pre-rotation transform
        let pretransform = Swapchain::get_prerotation_trs(frame.current_transform);
        let constant = PushConstant {
            pretransform: pretransform.to_mat4(),
        };
        self.push_constant(&frame.cache.command_buffer, &constant);

        for camera_info in camera_infos {
            let camera_key = DescriptorKey::builder()
                .layout(self.get_layout())
                .model(camera_info.model)
                .node(camera_info.node)
                .camera(camera_info.camera)
                .build();

            let view_key = ViewMatrixKey {
                model: camera_info.model,
                node: camera_info.node,
            };
            let view = frame.cache.view_buffers.get(&view_key).unwrap();

            let proj_key = ProjMatrixKey {
                model: camera_info.model,
                camera: camera_info.camera,
            };
            let proj = frame.cache.proj_buffers.get(&proj_key).unwrap();
            self.bind_view_and_proj(
                &frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                camera_key,
                view,
                proj,
            );

            for info in &infos {
                let model_key = ModelMatrixKey {
                    model: info.model,
                    node: info.node,
                };
                let model_buffer = frame.cache.model_buffers.get(&model_key).unwrap();
                let model_key = DescriptorKey::builder()
                    .layout(self.get_layout())
                    .model(info.model)
                    .node(info.node)
                    .build();
                self.bind_model(
                    &frame.cache.command_buffer,
                    &mut frame.cache.descriptors,
                    model_key,
                    model_buffer,
                );

                let model = scene.get_model(info.model).unwrap();
                let primitive = model.primitives.get(info.primitive.id.into()).unwrap();
                self.draw(&frame.cache, primitive);
            }
        }
    }
}

impl RenderPipeline for PipelineMain {
    fn render(
        &self,
        frame: &mut Frame,
        scene: &RenderScene,
        camera_infos: &[CameraDrawInfo],
        infos: Vec<DrawInfo>,
    ) {
        self.bind(&frame.cache);
        frame.set_viewport_and_scissor(1.0, true);

        // Apply pre-rotation transform
        let pretransform = Swapchain::get_prerotation_trs(frame.current_transform);
        let constant = PushConstant {
            pretransform: pretransform.to_mat4(),
        };
        self.push_constant(&frame.cache.command_buffer, &constant);

        for camera_info in camera_infos {
            let camera_key = DescriptorKey::builder()
                .layout(self.get_layout())
                .model(camera_info.model)
                .node(camera_info.node)
                .camera(camera_info.camera)
                .build();

            let view_key = ViewMatrixKey {
                model: camera_info.model,
                node: camera_info.node,
            };
            let view = frame.cache.view_buffers.get(&view_key).unwrap();

            let proj_key = ProjMatrixKey {
                model: camera_info.model,
                camera: camera_info.camera,
            };
            let proj = frame.cache.proj_buffers.get(&proj_key).unwrap();
            self.bind_view_and_proj(
                &frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                camera_key,
                view,
                proj,
            );

            let camera_node = scene
                .get_model(camera_info.model)
                .unwrap()
                .get_node(camera_info.node)
                .unwrap();

            for info in &infos {
                let model = scene.get_model(info.model).unwrap();

                // Bind material
                {
                    let primitive = model.get_primitive(info.primitive).unwrap();
                    let material = match model.get_material(primitive.material) {
                        Some(material) => material,
                        None => &frame.dev.fallback.white_material,
                    };
                    let material_key = MaterialKey {
                        model: info.model,
                        material: primitive.material,
                    };
                    let color_buffer = match frame.cache.material_buffers.get(&material_key) {
                        Some(color_buffer) => color_buffer,
                        None => &frame.dev.fallback.white_buffer,
                    };
                    let albedo = match model.textures.get(material.albedo.id.into()) {
                        Some(texture) => texture,
                        None => &frame.dev.fallback.white_texture,
                    };
                    // The problem here is that this is caching descriptor set for index 1
                    // with the s key as descriptor set index 1.
                    // Need to fix (what?)
                    let image_key = DescriptorKey::builder()
                        .layout(self.get_layout())
                        .model(info.model)
                        .material(primitive.material)
                        .build();
                    self.bind_color_and_albedo(
                        &frame.cache.command_buffer,
                        &mut frame.cache.descriptors,
                        image_key,
                        color_buffer,
                        albedo,
                    );
                }

                // Bind matrices
                {
                    let model_key = ModelMatrixKey {
                        model: info.model,
                        node: info.node,
                    };
                    let model_buffer = frame.cache.model_buffers.get(&model_key).unwrap();

                    let node = model.get_node(info.node).unwrap();
                    let normal_matrix = camera_node.trs.to_view() * &node.trs;
                    let normal_matrix = Mat4::from(normal_matrix.get_inversed()).get_transpose();

                    let normal_matrix_key = NormalMatrixKey {
                        model: info.model,
                        node: info.node,
                        view: camera_info.node,
                    };
                    let normal_matrix_buffer = frame
                        .cache
                        .normal_buffers
                        .get_or_create::<Mat4>(normal_matrix_key);
                    normal_matrix_buffer.upload(&normal_matrix);

                    let model_key = DescriptorKey::builder()
                        .layout(self.get_layout())
                        .model(info.model)
                        .node(info.node)
                        .build();
                    self.bind_model_and_normal_matrix(
                        &frame.cache.command_buffer,
                        &mut frame.cache.descriptors,
                        model_key,
                        model_buffer,
                        normal_matrix_buffer,
                    );
                }

                let primitive = model.primitives.get(info.primitive.id.into()).unwrap();
                self.draw(&frame.cache, primitive);
            }
        }
    }
}
