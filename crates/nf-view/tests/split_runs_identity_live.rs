//! perf-240f: the split effect-pool draws (`Renderer::sub_runs`, the runs whose palette slot is
//! collapsed left out of the frame) render exactly the pixels of the whole draws. Per level, at the
//! start view and its opposite, the frame's RGBA bytes with the split on and off must be equal.
//! Sisters: the split must actually leave indices out (else the equality is vacuous), and the two
//! views' frames must differ (the comparison can fail). Opt-in: `NF_ISO`; the levels from
//! `NF_SPLIT_LEVELS` (default the three heaviest effect-pool levels).

use nf_view::scene::ViewOpts;

fn view_proj(pos: [f32; 3], yaw: f32, aspect: f32) -> [[f32; 4]; 4] {
    use glam::{Mat4, Vec3};
    let (near, far, fov) = (0.05f32, 4000.0f32, std::f32::consts::FRAC_PI_3);
    let f = 1.0 / (fov * 0.5).tan();
    let proj = Mat4::from_cols_array_2d(&[[f / aspect, 0.0, 0.0, 0.0], [0.0, f, 0.0, 0.0], [0.0, 0.0, far / (near - far), -1.0], [0.0, 0.0, near * far / (near - far), 0.0]]);
    let fw = Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
    let s = fw.cross(Vec3::Y).normalize();
    let u = s.cross(fw);
    let p = Vec3::from(pos);
    let view = Mat4::from_cols_array_2d(&[[s.x, u.x, -fw.x, 0.0], [s.y, u.y, -fw.y, 0.0], [s.z, u.z, -fw.z, 0.0], [-s.dot(p), -u.dot(p), fw.dot(p), 1.0]]);
    (proj * view).to_cols_array_2d()
}

#[test]
#[cfg(not(debug_assertions))]
fn split_effect_pool_draws_render_the_same_pixels() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())) else { eprintln!("no adapter"); return };
    let want = wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let desc = wgpu::DeviceDescriptor { required_features: adapter.features() & want, required_limits: adapter.limits(), ..Default::default() };
    let (device, queue) = pollster::block_on(adapter.request_device(&desc)).expect("device");
    let (w, h) = (640u32, 360u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[] });
    let view = target.create_view(&Default::default());
    let depth = nf_view::gpu::depth_view(&device, w, h);
    let row = (4 * w).next_multiple_of(256);
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let levels = std::env::var("NF_SPLIT_LEVELS").unwrap_or("0700001b,07000016,07000007".into());
    for lv in levels.split(',') {
        let name = format!("{}.bin", lv.trim());
        let mut scene = nf_view::load(&iso, &name, false).expect("level");
        if let Some(sim) = scene.sim.as_mut() { for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); } }
        let (pos, yaw0) = scene.start.clone().unwrap_or(([0.0; 3], 0.0));
        let mut r = nf_view::gpu::Renderer::new(&device, &queue, format, &scene, false, false);
        let (layout, _) = scene.skin_layout();
        let mut per_view = Vec::new();
        for q in [0u32, 2] {
            let yaw = yaw0 + q as f32 * std::f32::consts::FRAC_PI_2;
            let eye = [pos[0], pos[1] + 1.6, pos[2]];
            let vp = view_proj(eye, yaw, w as f32 / h as f32);
            let mut opts = ViewOpts::default();
            let fw = glam::Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
            let s = fw.cross(glam::Vec3::Y).normalize();
            opts.right = s.to_array(); opts.up = s.cross(fw).to_array();
            let (mut p, mut c) = (Vec::new(), Vec::new());
            scene.skin_palette_into(&layout, None, 120.0, opts, None, &mut p, &mut c);
            let world = scene.sim.as_ref().map(|s| &s.world);
            let doors = scene.sim.as_ref().map(|s| s.drones.doors.closed_flags());
            let mut frames = Vec::new();
            for split in [true, false] {
                r.split_runs = split;
                let _ = r.cel_visibility(eye, vp, world, doors.as_deref());
                r.char_light_centre = scene.char_light_centres();
                r.upload_skin(&queue, &p, &c);
                r.lights = scene.frame_lights();
                let mut e = r.render(&device, &queue, &view, &depth, vp, vp, eye, 120, &[], &nf_view::gpu::HudFrame::default(), None, None);
                e.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
                    wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
                queue.submit([e.finish()]);
                buf.map_async(wgpu::MapMode::Read, .., |r| r.expect("map"));
                device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                let bytes = buf.get_mapped_range(..).expect("range").to_vec();
                buf.unmap();
                frames.push((bytes, r.last_indices, r.order_len()));
            }
            let (on, off) = (&frames[0], &frames[1]);
            eprintln!("[split] {name} q{q} draws {} indices split {} whole {}", on.2, on.1, off.1);
            assert_eq!(on.2, off.2, "{name} q{q}: the same draws are submitted");
            assert!(on.1 < off.1, "{name} q{q}: the split left no index out ({} vs {}) — the identity below would prove nothing", on.1, off.1);
            assert!(on.0 == off.0, "{name} q{q}: the split frame's pixels differ from the whole draws'");
            per_view.push(on.0.clone());
        }
        r.split_runs = true;
        // Sister: the two views' frames differ, so a byte comparison of frames can fail.
        assert!(per_view[0] != per_view[1], "{name}: the opposite views rendered identical frames");
        drop(r); let _ = queue.submit([]); device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}
