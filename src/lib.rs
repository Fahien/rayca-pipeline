// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub use rayca_pipe::*;
pub use rayca_core::*;

pipewriter!(Main, "shaders/main.vert.slang", "shaders/main.frag.slang");
pipewriter!(Line, "shaders/line.vert.slang", "shaders/line.frag.slang");

impl RenderPipeline for PipelineLine {
    fn render(
        &self,
        frame: &mut Frame,
        model: &RenderModel,
        camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    ) {
        self.bind(&frame.cache);
        frame.set_viewport_and_scissor(1.0);

        for camera_node_handle in camera_nodes.iter().copied() {
            let camera_node = model.gltf.nodes.get(camera_node_handle).unwrap();
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

                let node = model.gltf.nodes.get(node_handle).unwrap();
                let mesh = model.gltf.meshes.get(node.mesh).unwrap();
                let primitive = model.primitives.get(mesh.primitive.id.into()).unwrap();
                self.draw(&frame.cache, primitive);
            }
        }
    }
}

impl RenderPipeline for PipelineMain {
    fn render(
        &self,
        frame: &mut Frame,
        model: &RenderModel,
        camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    ) {
        self.bind(&frame.cache);
        frame.set_viewport_and_scissor(1.0);

        for camera_node_handle in camera_nodes.iter().copied() {
            let camera_node = model.gltf.nodes.get(camera_node_handle).unwrap();
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

            // Supposedly, the material is the same for all nodes
            let node = model.gltf.nodes.get(nodes[0]).unwrap();
            let mesh = model.gltf.meshes.get(node.mesh).unwrap();
            let primitive = model.gltf.primitives.get(mesh.primitive).unwrap();
            let material = model.gltf.materials.get(primitive.material).unwrap();
            let texture = model.textures.get(material.texture.id.into()).unwrap();
            // The problem here is that this is caching descriptor set for index 1
            // with the s key as descriptor set index 1.
            // Need to fix
            let image_key = DescriptorKey::builder()
                .layout(self.get_layout())
                .material(primitive.material)
                .build();
            self.bind_texture(
                &frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                image_key,
                texture,
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

                let node = model.gltf.nodes.get(node_handle).unwrap();
                let mesh = model.gltf.meshes.get(node.mesh).unwrap();
                let primitive = model.primitives.get(mesh.primitive.id.into()).unwrap();
                self.draw(&frame.cache, primitive);
            }
        }
    }
}
