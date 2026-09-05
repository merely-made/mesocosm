// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use mesocosm_core::{PartId, VolumeRef};
use mesocosm_mesh::{Volume, place_point};

fn gpu() -> Option<crate::Renderer> {
    match crate::Renderer::headless(16, 16) {
        Ok(renderer) => Some(renderer),
        Err(crate::RenderError::NoAdapter) => None,
        Err(error) => panic!("headless renderer failed: {error:?}"),
    }
}

fn attachments(device: &wgpu::Device) -> (wgpu::Texture, wgpu::Texture) {
    let size = wgpu::Extent3d {
        width: 16,
        height: 16,
        depth_or_array_layers: 1,
    };
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("live body test colour"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("live body test depth"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    (colour, depth)
}

fn clear(
    encoder: &mut wgpu::CommandEncoder,
    colour: &wgpu::TextureView,
    depth: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("live body test clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: colour,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

#[test]
fn one_greedy_cube_keeps_six_quads_of_triangles() {
    let mesh = mesocosm_mesh::mesh_volume(&Volume::solid([4, 3, 2], 7));
    assert_eq!(part_vertices(&mesh).len(), 36);
}

#[test]
fn quarter_turn_uses_the_mesh_placement_convention() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let body = LiveBody::new(&mesh, [10.0, 0.0, 0.0]);
    let transformed = model_matrix(body, Yaw::Quarter, [0; 3], [2, 0, 0])
        .transform_point3(glam::Vec3::X)
        .round();
    assert_eq!(transformed.to_array(), [12.0, 0.0, -1.0]);
    assert_eq!(
        place_point([1, 0, 0], Yaw::Quarter, [0; 3], [2, 0, 0]),
        [2, 0, -1]
    );
}

#[test]
fn an_unchanged_frame_reuses_all_gpu_uploads() {
    let Some(host) = gpu() else { return };
    let (colour, depth) = attachments(host.device());
    let colour_view = colour.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    let volume = Volume::solid([1, 1, 1], 1);
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &volume);
    let body = [LiveBody::new(&mesh, [0.0; 3])];
    let mut live = LiveBodyRenderer::new(host.device(), wgpu::TextureFormat::Rgba8Unorm, 4);
    let clip = Mat4::IDENTITY.to_cols_array_2d();

    let mut first = host.device().create_command_encoder(&Default::default());
    clear(&mut first, &colour_view, &depth_view);
    let uploaded = live
        .draw(
            host.device(),
            host.queue(),
            &mut first,
            &colour_view,
            &depth_view,
            clip,
            None,
            &body,
        )
        .unwrap();
    host.queue().submit(Some(first.finish()));
    assert!(uploaded.mesh_upload_bytes > 0);
    assert!(uploaded.instance_upload_bytes > 0);
    assert!(uploaded.frame_upload_bytes > 0);

    let mut second = host.device().create_command_encoder(&Default::default());
    let reused = live
        .draw(
            host.device(),
            host.queue(),
            &mut second,
            &colour_view,
            &depth_view,
            clip,
            None,
            &body,
        )
        .unwrap();
    host.queue().submit(Some(second.finish()));
    assert_eq!(reused.mesh_upload_bytes, 0);
    assert_eq!(reused.instance_upload_bytes, 0);
    assert_eq!(reused.frame_upload_bytes, 0);
    assert_eq!(reused.draw_parts, 1);
    eprintln!("VB1 static receipt: {reused:?}");

    let moved_body = [LiveBody::new(&mesh, [0.25, 0.0, 0.0])];
    let mut movement = host.device().create_command_encoder(&Default::default());
    let moved = live
        .draw(
            host.device(),
            host.queue(),
            &mut movement,
            &colour_view,
            &depth_view,
            clip,
            None,
            &moved_body,
        )
        .unwrap();
    host.queue().submit(Some(movement.finish()));
    assert_eq!(
        moved.mesh_upload_bytes, 0,
        "movement reuses immutable volume geometry"
    );
    assert!(
        moved.instance_upload_bytes > 0,
        "movement updates only instance data"
    );
    assert_eq!(moved.draw_parts, 1);
    eprintln!("VB1 movement receipt: {moved:?}");

    let mut attached_mesh = mesh.clone();
    let mut attached = mesh.placements[0].clone();
    attached.part = PartId(1);
    attached.pivot_at = [2, 0, 0];
    attached_mesh.placements.push(attached);
    let attached_body = [LiveBody::new(&attached_mesh, [0.25, 0.0, 0.0])];
    let mut attachment = host.device().create_command_encoder(&Default::default());
    let grown = live
        .draw(
            host.device(),
            host.queue(),
            &mut attachment,
            &colour_view,
            &depth_view,
            clip,
            None,
            &attached_body,
        )
        .unwrap();
    host.queue().submit(Some(attachment.finish()));
    assert_eq!(
        grown.mesh_upload_bytes, 0,
        "a shared volume needs no immutable upload"
    );
    assert!(
        grown.instance_upload_bytes > 0,
        "attachment changes the instance batch"
    );
    assert_eq!(grown.draw_parts, 2, "attachment adds one rigid placement");
    eprintln!("VB1 attachment receipt: {grown:?}");

    let mut severing = host.device().create_command_encoder(&Default::default());
    let severed = live
        .draw(
            host.device(),
            host.queue(),
            &mut severing,
            &colour_view,
            &depth_view,
            clip,
            None,
            &moved_body,
        )
        .unwrap();
    host.queue().submit(Some(severing.finish()));
    assert_eq!(
        severed.mesh_upload_bytes, 0,
        "severing does not rebuild shared geometry"
    );
    assert!(
        severed.instance_upload_bytes > 0,
        "severing changes the instance batch"
    );
    assert_eq!(
        severed.draw_parts, 1,
        "severing removes the added placement"
    );
    eprintln!("VB1 sever receipt: {severed:?}");
}
