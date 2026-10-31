//! frame-path probe: the window's per-frame chain (lib.rs `draw_frame`) timed headless per level at a
//! few views, phase by phase, on the `--shot` machinery's wgpu device (no window, no surface).
//! Measurement only — prints one `[probe]` line per view and one `[probe-enc]` line with the
//! `enc:` prof scopes; asserts nothing. `#[ignore]`: run it by name with `--ignored`.
//!
//! Phases (the draw_frame order): `tick` = one `Sim::step_bond` (median over 30 ticks),
//! `pal` = `Scene::skin_palette` with no mask (lib.rs rebuilds it once per tick that way),
//! `vis` = `Renderer::cel_visibility`, `upl` = `upload_skin` + `char_light_centres`,
//! `lights` = `frame_lights`, `hud` = `hud::overlay`, `shards` = `shard_draws` + `set_shards`,
//! `enc` = `Renderer::render`, `gpu` = finish + submit + wait (the GPU's own time at 1280x720).
//! `frame` = vis+upl+lights+hud+shards+enc+gpu (a frame without a tick); `tickf` adds tick+pal.

use nf_view::scene::ViewOpts;

fn median_ms(warmup: u32, runs: u32, mut f: impl FnMut()) -> f64 {
    for _ in 0..warmup { f(); }
    let mut v: Vec<f64> = (0..runs).map(|_| { let t0 = std::time::Instant::now(); f(); t0.elapsed().as_secs_f64() * 1000.0 }).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn view_proj(pos: [f32; 3], yaw: f32, pitch: f32, aspect: f32) -> [[f32; 4]; 4] {
    use glam::{Mat4, Vec3};
    let (near, far, fov) = (0.05f32, 4000.0f32, std::f32::consts::FRAC_PI_3);
    let f = 1.0 / (fov * 0.5).tan();
    let proj = Mat4::from_cols_array_2d(&[[f / aspect, 0.0, 0.0, 0.0], [0.0, f, 0.0, 0.0], [0.0, 0.0, far / (near - far), -1.0], [0.0, 0.0, near * far / (near - far), 0.0]]);
    let cp = pitch.cos();
    let fw = Vec3::new(cp * yaw.sin(), pitch.sin(), cp * yaw.cos()).normalize();
    let s = fw.cross(Vec3::Y).normalize();
    let u = s.cross(fw);
    let view = Mat4::from_cols_array_2d(&[[s.x, u.x, -fw.x, 0.0], [s.y, u.y, -fw.y, 0.0], [s.z, u.z, -fw.z, 0.0],
        [-s.dot(Vec3::from(pos)), -u.dot(Vec3::from(pos)), fw.dot(Vec3::from(pos)), 1.0]]);
    (proj * view).to_cols_array_2d()
}

/// The window's device recipe (lib.rs `device`): the bindless/indirect features where the adapter
/// has them, and the adapter's limits — so the batched world pass is the one timed.
fn app_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let want = wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let desc = wgpu::DeviceDescriptor { required_features: adapter.features() & want, required_limits: adapter.limits(), ..Default::default() };
    pollster::block_on(adapter.request_device(&desc)).expect("device")
}

fn on_foot_levels(iso_path: &str) -> Vec<String> {
    let img = nf_formats::iso9660::IsoImage::open(iso_path).expect("iso");
    let (entries, _) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let mut names: Vec<String> = entries.iter().map(|e| e.name.to_ascii_lowercase())
        .filter(|n| n.starts_with("0700") && n.ends_with(".bin"))
        .filter(|n| { let code = u32::from_str_radix(&n[..8], 16).unwrap_or(0) & 0xff;
            !(0x23..=0x29).contains(&code) && code != 0x00 && n != "07000500.bin" && n != "07000999.bin" })
        .collect();
    names.sort(); names.dedup(); names
}

#[test]
#[ignore]
#[cfg(not(debug_assertions))]
fn probe_the_frame_path_per_level() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())) else { eprintln!("no adapter"); return };
    let (device, queue) = app_device(&adapter);
    nf_game::prof::force_enable();
    let only = std::env::var("NF_PROBE_LEVELS").ok();
    let mut levels = on_foot_levels(&iso_path);
    if let Some(o) = &only { levels.retain(|l| o.split(',').any(|x| l.starts_with(x.trim()))); }
    let (w, h) = (1280u32, 720u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] });
    let view = target.create_view(&Default::default());
    let depth = nf_view::gpu::depth_view(&device, w, h);
    for name in &levels {
        let mut scene = match nf_view::load(&iso_path, name, false) { Ok(s) => s, Err(e) => { eprintln!("[probe] {name} not loadable: {e}"); continue } };
        if let Some(sim) = scene.sim.as_mut() { for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); } }
        let _ = nf_game::prof::take_samples();
        nf_game::prof::capture(true);
        let tick = match scene.sim.as_mut() {
            Some(sim) => median_ms(2, 30, || { sim.step_bond(Default::default(), false, false, false); }),
            None => 0.0,
        };
        nf_game::prof::capture(false);
        {
            let ts = nf_game::prof::take_samples();
            let mut agg: Vec<(&str, f64)> = Vec::new();
            for (nm, ns) in &ts { match agg.iter_mut().find(|e| e.0 == *nm) { Some(e) => e.1 += *ns as f64 / 1e6 / 32.0, None => agg.push((nm, *ns as f64 / 1e6 / 32.0)) } }
            agg.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let top: Vec<String> = agg.iter().take(8).map(|(n, v)| format!("{n}={v:.3}")).collect();
            eprintln!("[probe-tick] {name} {}", top.join(" "));
        }
        let t = 152.0f32;
        let (pos, yaw0) = scene.start.clone().unwrap_or(([0.0; 3], 0.0));
        let mut spots: Vec<(String, [f32; 3], f32)> = (0..4).map(|q| (format!("start{}", q * 90), [pos[0], pos[1] + 1.6, pos[2]], yaw0 + q as f32 * std::f32::consts::FRAC_PI_2)).collect();
        if let Some(sim) = scene.sim.as_ref() {
            let mut picked = 0;
            for (di, d) in sim.drones.list.iter().enumerate() {
                if picked >= 2 { break; }
                if !d.present { continue; }
                let p = [d.pos[0], d.pos[1] + 1.6, d.pos[2]];
                if (p[0] - pos[0]).hypot(p[2] - pos[2]) < 10.0 { continue; }
                // Look back toward the start (the way a player would face a guard's area).
                let yaw = (pos[0] - p[0]).atan2(pos[2] - p[2]);
                spots.push((format!("drone{di}"), p, yaw));
                picked += 1;
            }
        }
        let mut r = nf_view::gpu::Renderer::new(&device, &queue, format, &scene, false, false);
        let (layout, _) = scene.skin_layout();
        let mut worst = (0.0f64, String::new());
        for (label, eye, yaw) in &spots {
            let vp = view_proj(*eye, *yaw, 0.0, 16.0 / 9.0);
            let mut opts = ViewOpts::default();
            let fw = glam::Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
            let s = fw.cross(glam::Vec3::Y).normalize();
            opts.right = s.to_array(); opts.up = s.cross(fw).to_array();
            let mut palc = None;
            let _ = nf_game::prof::take_samples();
            nf_game::prof::capture(true);
            // The window's path: the previous buffers refilled (lib.rs recycles the tick before last's pair).
            let mut bufs = (Vec::new(), Vec::new());
            let pal = median_ms(1, 5, || { scene.skin_palette_into(&layout, None, t, opts, None, &mut bufs.0, &mut bufs.1); });
            palc = Some(bufs.clone());
            nf_game::prof::capture(false);
            let ps = nf_game::prof::take_samples();
            let mut names: Vec<&str> = ps.iter().map(|s| s.0).collect(); names.sort(); names.dedup();
            let line: Vec<String> = names.iter().map(|nm| {
                let mut v: Vec<f64> = ps.iter().filter(|s| s.0 == *nm).map(|s| s.1 as f64 / 1e6).collect();
                let calls = v.len() / 6;
                // per palette build: total of this scope over the 6 builds / 6
                let tot: f64 = v.iter().sum::<f64>() / 6.0; v.clear();
                format!("{nm}={tot:.3}({calls}x)")
            }).collect();
            eprintln!("[probe-pal] {name} {label:>9} {}", line.join(" "));
            let (pl, cols) = palc.unwrap();
            let door_closed = scene.sim.as_ref().map(|s| s.drones.doors.closed_flags());
            let world = scene.sim.as_ref().map(|s| &s.world);
            let _ = nf_game::prof::take_samples();
            nf_game::prof::capture(true);
            let vis = median_ms(2, 7, || { let _ = r.cel_visibility(*eye, vp, world, door_closed.as_deref()); });
            // A moving eye (walking: a new eye every frame — no memo hits).
            let mut step = 0.0f32;
            let vis_move = median_ms(2, 7, || { step += 0.013; let e = [eye[0] + step, eye[1], eye[2] + step * 0.5]; let _ = r.cel_visibility(e, vp, world, door_closed.as_deref()); });
            let _ = r.cel_visibility(*eye, vp, world, door_closed.as_deref());
            nf_game::prof::capture(false);
            {
                let vs = nf_game::prof::take_samples();
                let mut agg: Vec<(&str, f64)> = Vec::new();
                for (nm, ns) in &vs { match agg.iter_mut().find(|e| e.0 == *nm) { Some(e) => e.1 += *ns as f64 / 1e6 / 19.0, None => agg.push((nm, *ns as f64 / 1e6 / 19.0)) } }
                let top: Vec<String> = agg.iter().map(|(n, v)| format!("{n}={v:.3}")).collect();
                eprintln!("[probe-vis] {name} {label:>9} still {vis:.3} moving {vis_move:.3} | {}", top.join(" "));
            }
            let upl = median_ms(2, 7, || { r.char_light_centre = scene.char_light_centres(); r.upload_skin(&queue, &pl, &cols); });
            let lights = median_ms(2, 7, || { r.lights = scene.frame_lights(); });
            let hud = median_ms(2, 7, || { std::hint::black_box(nf_view::hud::overlay(&scene, 16.0 / 9.0)); });
            let shards = median_ms(2, 7, || { r.set_shards(&queue, &scene.shard_draws()); });
            let hf = nf_view::hud::overlay(&scene, 16.0 / 9.0);
            let _ = nf_game::prof::take_samples();
            nf_game::prof::capture(true);
            let mut encs = Vec::new();
            let enc = median_ms(2, 7, || { encs.push(r.render(&device, &queue, &view, &depth, vp, vp, *eye, t as u32, &[], &hf, None, None)); });
            nf_game::prof::capture(false);
            // The enc: scopes' medians over the timed renders.
            let samples = nf_game::prof::take_samples();
            let mut scopes: Vec<(&str, f64)> = Vec::new();
            for nm in ["enc: cull filter", "enc: sort", "enc: uniforms + light picks", "enc: record draws"] {
                let mut v: Vec<f64> = samples.iter().filter(|s| s.0 == nm).map(|s| s.1 as f64 / 1e6).collect();
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                scopes.push((nm, v.get(v.len() / 2).copied().unwrap_or(0.0)));
            }
            let drawn = r.order_len();
            // A tick frame's encode: the sim steps and the frame's lights are re-resolved first.
            let mut tick_encs = Vec::new();
            let mut tv: Vec<f64> = Vec::new();
            for _ in 0..7 {
                if let Some(sim) = scene.sim.as_mut() { sim.step_bond(Default::default(), false, false, false); }
                r.lights = scene.frame_lights();
                let t0 = std::time::Instant::now();
                tick_encs.push(r.render(&device, &queue, &view, &depth, vp, vp, *eye, t as u32, &[], &hf, None, None));
                tv.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            tv.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let enc_tick = tv[3];
            for e in tick_encs.drain(..) { queue.submit([e.finish()]); }
            // Submit the queued encoders (keeps the pending writes bounded), then the GPU's time for one frame.
            for e in encs.drain(..) { queue.submit([e.finish()]); }
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            let gpu = median_ms(1, 5, || {
                let e = r.render(&device, &queue, &view, &depth, vp, vp, *eye, t as u32, &[], &hf, None, None);
                queue.submit([e.finish()]);
                device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            }) - enc;
            let frame = vis + upl + lights + hud + shards + enc + gpu.max(0.0);
            let tickf = frame + tick + pal + (enc_tick - enc).max(0.0);
            if tickf > worst.0 { worst = (tickf, label.clone()); }
            eprintln!("[probe] {name} {label:>9} draws={drawn:5} | tick {tick:.3} pal {pal:.3} vis {vis:.3} upl {upl:.3} lights {lights:.3} hud {hud:.3} shards {shards:.3} enc {enc:.3} enc_tick {enc_tick:.3} gpu {gpu:.3} | frame {frame:.3} tickframe {tickf:.3}");
            eprintln!("[probe-enc] {name} {label:>9} cull {:.3} sort {:.3} lights {:.3} record {:.3}", scopes[0].1, scopes[1].1, scopes[2].1, scopes[3].1);
        }
        eprintln!("[probe-worst] {name} {:.3} ms at {}", worst.0, worst.1);
        drop(r);
        let _ = queue.submit([]);
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}

/// The GPU half split: `finish`, `submit` (CPU) and the wait, per frame, and the back-to-back
/// throughput (8 frames submitted, one wait) at the start view — measurement only.
#[test]
#[ignore]
#[cfg(not(debug_assertions))]
fn probe_the_gpu_half() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { return };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    eprintln!("[gpu] adapter {:?}", adapter.get_info());
    let (device, queue) = app_device(&adapter);
    let levels: Vec<String> = std::env::var("NF_PROBE_LEVELS").unwrap_or("07000004,0700004a,07000015".into()).split(',').map(|s| format!("{}.bin", s.trim())).collect();
    let (w, h) = (1280u32, 720u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] });
    let view = target.create_view(&Default::default());
    let depth = nf_view::gpu::depth_view(&device, w, h);
    for name in &levels {
        let Ok(mut scene) = nf_view::load(&iso_path, name, false) else { continue };
        if let Some(sim) = scene.sim.as_mut() { for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); } }
        let (pos, yaw) = scene.start.clone().unwrap_or(([0.0; 3], 0.0));
        let eye = [pos[0], pos[1] + 1.6, pos[2]];
        let vp = view_proj(eye, yaw, 0.0, 16.0 / 9.0);
        let mut r = nf_view::gpu::Renderer::new(&device, &queue, format, &scene, false, false);
        let vis = r.cel_visibility(eye, vp, scene.sim.as_ref().map(|s| &s.world), None);
        r.update_skin(&queue, &scene, None, 120.0, ViewOpts::default(), vis.as_deref());
        r.lights = scene.frame_lights();
        let hf = nf_view::gpu::HudFrame::default();
        let mut rows = Vec::new();
        for i in 0..12 {
            let t0 = std::time::Instant::now();
            let e = r.render(&device, &queue, &view, &depth, vp, vp, eye, 120, &[], &hf, None, None);
            let t1 = std::time::Instant::now();
            let cb = e.finish();
            let t2 = std::time::Instant::now();
            queue.submit([cb]);
            let t3 = std::time::Instant::now();
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            let t4 = std::time::Instant::now();
            if i >= 2 { rows.push([(t1 - t0).as_secs_f64(), (t2 - t1).as_secs_f64(), (t3 - t2).as_secs_f64(), (t4 - t3).as_secs_f64()]); }
        }
        let med = |k: usize| { let mut v: Vec<f64> = rows.iter().map(|r| r[k] * 1000.0).collect(); v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] };
        let t0 = std::time::Instant::now();
        for _ in 0..8 { let e = r.render(&device, &queue, &view, &depth, vp, vp, eye, 120, &[], &hf, None, None); queue.submit([e.finish()]); }
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let thr = t0.elapsed().as_secs_f64() * 1000.0 / 8.0;
        eprintln!("[gpu] {name} draws={} enc {:.3} finish {:.3} submit {:.3} wait {:.3} | back-to-back {:.3} ms/frame", r.order_len(), med(0), med(1), med(2), med(3), thr);
        drop(r); let _ = queue.submit([]); device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}

/// Output identity across a change: per level at the four start views, the palette/colour hash
/// (FNV over the f32 bits) and the rendered frame's RGBA hash — the first render and a repeat
/// (a frame without a tick: the per-frame memos' hit path). Run before and after, diff the lines.
#[test]
#[ignore]
#[cfg(not(debug_assertions))]
fn probe_output_hashes() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { return };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    let (device, queue) = app_device(&adapter);
    let mut levels = on_foot_levels(&iso_path);
    if let Ok(o) = std::env::var("NF_PROBE_LEVELS") { levels.retain(|l| o.split(',').any(|x| l.starts_with(x.trim()))); }
    let fnv = |bytes: &[u8]| { let mut h = 0xcbf2_9ce4_8422_2325u64; for &b in bytes { h ^= b as u64; h = h.wrapping_mul(0x1000_0000_01b3); } h };
    let (w, h) = (640u32, 360u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[] });
    let view = target.create_view(&Default::default());
    let depth = nf_view::gpu::depth_view(&device, w, h);
    let row = (4 * w).next_multiple_of(256);
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    for name in &levels {
        let Ok(mut scene) = nf_view::load(&iso_path, name, false) else { continue };
        if let Some(sim) = scene.sim.as_mut() { for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); } }
        let (pos, yaw0) = scene.start.clone().unwrap_or(([0.0; 3], 0.0));
        let mut r = nf_view::gpu::Renderer::new(&device, &queue, format, &scene, false, false);
        let (layout, _) = scene.skin_layout();
        for q in 0..4 {
            let yaw = yaw0 + q as f32 * std::f32::consts::FRAC_PI_2;
            let eye = [pos[0], pos[1] + 1.6, pos[2]];
            let vp = view_proj(eye, yaw, 0.0, w as f32 / h as f32);
            let mut opts = ViewOpts::default();
            let fw = glam::Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
            let s = fw.cross(glam::Vec3::Y).normalize();
            opts.right = s.to_array(); opts.up = s.cross(fw).to_array();
            let (mut p, mut c) = (Vec::new(), Vec::new());
            scene.skin_palette_into(&layout, None, 120.0, opts, None, &mut p, &mut c);
            let ph = fnv(bytemuck::cast_slice(&p)) ^ fnv(bytemuck::cast_slice(&c)).rotate_left(1);
            let world = scene.sim.as_ref().map(|s| &s.world);
            let doors = scene.sim.as_ref().map(|s| s.drones.doors.closed_flags());
            let mut frames = Vec::new();
            for _ in 0..2 {
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
                let hh = { let d = buf.get_mapped_range(..).expect("range"); fnv(&d) };
                buf.unmap();
                frames.push(hh);
            }
            eprintln!("[hash] {name} q{q} palette {ph:016x} frame {:016x} repeat {:016x} draws {}", frames[0], frames[1], r.order_len());
        }
        drop(r); let _ = queue.submit([]); device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}
