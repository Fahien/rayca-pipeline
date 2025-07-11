// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub use rayca_core::*;
pub use rayca_pipe::*;

pipewriter!(Main, "shaders/main.vert.slang", "shaders/main.frag.slang");
pipewriter!(Line, "shaders/line.vert.slang", "shaders/line.frag.slang");

impl RenderPipeline for PipelineLine {
    fn render(
        &self,
        frame: &mut Frame,
        model: Option<&RenderModel>,
        camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    ) {
        let model = model.as_ref().unwrap();

        self.bind(&frame.cache);
        frame.set_viewport_and_scissor(1.0);

        for camera_node_handle in camera_nodes.iter().copied() {
            let camera_node = model.get_node(camera_node_handle).unwrap();
            let camera_key = DescriptorKey::builder()
                .layout(self.get_layout())
                .node(camera_node_handle)
                .camera(camera_node.camera)
                .build();
            let view = frame.cache.view_buffers.get(&camera_node_handle).unwrap();
            let proj = frame.cache.proj_buffers.get(&camera_node.camera).unwrap();
            self.bind_view_and_proj(
                &frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                camera_key,
                view,
                proj,
            );

            for node_handle in nodes.iter().cloned() {
                let model_buffer = frame.cache.model_buffers.get(&node_handle).unwrap();
                let model_key = DescriptorKey::builder()
                    .layout(self.get_layout())
                    .node(node_handle)
                    .build();
                self.bind_model(
                    &frame.cache.command_buffer,
                    &mut frame.cache.descriptors,
                    model_key,
                    model_buffer,
                );

                let node = model.get_node(node_handle).unwrap();
                let mesh = model.get_mesh(node.mesh).unwrap();
                for primitive_handle in mesh.primitives.iter().copied() {
                    let primitive = model.primitives.get(primitive_handle.id.into()).unwrap();
                    self.draw(&frame.cache, primitive);
                }
            }
        }
    }
}

impl RenderPipeline for PipelineMain {
    fn render(
        &self,
        frame: &mut Frame,
        model: Option<&RenderModel>,
        camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    ) {
        let model = model.as_ref().unwrap();

        self.bind(&frame.cache);
        frame.set_viewport_and_scissor(1.0);

        for camera_node_handle in camera_nodes.iter().copied() {
            let camera_node = model.get_node(camera_node_handle).unwrap();
            let camera_key = DescriptorKey::builder()
                .layout(self.get_layout())
                .node(camera_node_handle)
                .camera(camera_node.camera)
                .build();
            let view = frame.cache.view_buffers.get(&camera_node_handle).unwrap();
            let proj = frame.cache.proj_buffers.get(&camera_node.camera).unwrap();
            self.bind_view_and_proj(
                &frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                camera_key,
                view,
                proj,
            );

            for node_handle in nodes.iter().cloned() {
                let model_buffer = frame.cache.model_buffers.get(&node_handle).unwrap();

                let node = model.get_node(node_handle).unwrap();
                let normal_matrix = camera_node.trs.to_view() * &node.trs;
                let normal_matrix = Mat4::from(normal_matrix.get_inversed()).get_transpose();

                let normal_matrix_key = NormalMatrixKey {
                    node: node_handle,
                    view: camera_node_handle,
                };
                let normal_matrix_buffer = frame
                    .cache
                    .normal_buffers
                    .get_or_create::<Mat4>(normal_matrix_key);
                normal_matrix_buffer.upload(&normal_matrix);

                let model_key = DescriptorKey::builder()
                    .layout(self.get_layout())
                    .node(node_handle)
                    .build();
                self.bind_model_and_normal_matrix(
                    &frame.cache.command_buffer,
                    &mut frame.cache.descriptors,
                    model_key,
                    model_buffer,
                    normal_matrix_buffer,
                );

                let mesh = model.get_mesh(node.mesh).unwrap();
                for primitive_handle in mesh.primitives.iter().copied() {
                    let primitive = model.get_primitive(primitive_handle).unwrap();
                    let material = match model.get_material(primitive.material) {
                        Some(material) => material,
                        None => &frame.cache.fallback.white_material,
                    };
                    let color_buffer = match frame.cache.material_buffers.get(&primitive.material) {
                        Some(color_buffer) => color_buffer,
                        None => &frame.cache.fallback.white_buffer,
                    };
                    let albedo = match model.textures.get(material.albedo.id.into()) {
                        Some(texture) => texture,
                        None => &frame.cache.fallback.white_texture,
                    };
                    // The problem here is that this is caching descriptor set for index 1
                    // with the s key as descriptor set index 1.
                    // Need to fix
                    let image_key = DescriptorKey::builder()
                        .layout(self.get_layout())
                        .material(primitive.material)
                        .build();
                    self.bind_color_and_albedo(
                        &frame.cache.command_buffer,
                        &mut frame.cache.descriptors,
                        image_key,
                        color_buffer,
                        albedo,
                    );
                    let primitive = model.primitives.get(primitive_handle.id.into()).unwrap();
                    self.draw(&frame.cache, primitive);
                }
            }
        }
    }
}
