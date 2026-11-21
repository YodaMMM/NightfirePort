//! The driving view's car reflection map (docs/formats/car-model.md §10.5 pass 3, §10.6.3,
//! §10.6.4; crate::drive_reflection): each frame one 256 x 256 map is drawn from the reflection
//! camera over the player car (eye = car + (0, Refl HEIGHT, 0), forward = normalize(0.8 f + up)),
//! holding the `cref` sky layers, the track items `ReflectionItemQuery::keeps` keeps and up to 4
//! vehicles; the car's Paint / Chrome / Glass strips sample it additively with
//! ST = (0.5 + v.x, 0.5 + v.z). Opt-in: NF_ISO. The GPU half skips without an adapter.

use nf_formats::carmesh::{CarMaterial, CarPassKind};
use nf_view::drive_reflection::{MapSource, MapStep, MAP_KEY};

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

fn near(a: [f32; 3], b: [f32; 3], what: &str) {
    for k in 0..3 {
        assert!((a[k] - b[k]).abs() < 1e-4, "{what}: {a:?} vs {b:?}");
    }
}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
    // The window's recipe: the bindless features where the adapter has them (the batched path).
    let want = wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let desc = wgpu::DeviceDescriptor { required_features: adapter.features() & want, required_limits: adapter.limits(), ..Default::default() };
    pollster::block_on(adapter.request_device(&desc)).ok()
}

/// Renders one frame and reads the 256 x 256 map back (RGBA bytes).
fn render_map(dq: &(wgpu::Device, wgpu::Queue), scene: &nf_view::scene::Scene) -> Vec<u8> {
    let (device, queue) = dq;
    let mut r = nf_view::gpu::Renderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm, scene, false, false);
    r.set_reflection(queue, scene.drive_reflection.as_ref().map(|m| &m.frame));
    let (w, h) = (64u32, 64u32);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let depth = nf_view::gpu::depth_view(device, w, h);
    let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut enc = r.render(device, queue, &view, &depth, id, id, [0.0; 3], 0, &[], &Default::default(), None, None);
    let map = r.reflection_target().expect("the map target");
    assert_eq!((map.width(), map.height()), (256, 256));
    let row = 256 * 4;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * 256) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: map, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(256) } },
        wgpu::Extent3d { width: 256, height: 256, depth_or_array_layers: 1 });
    queue.submit([enc.finish()]);
    buf.map_async(wgpu::MapMode::Read, .., |r| r.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let out = buf.get_mapped_range(..).expect("range").to_vec();
    out
}

#[test]
fn driving_reflection_map_follows_the_documented_camera_and_the_car_samples_it() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let (mut scene, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let rf = d.reflection.as_ref().expect("the reflection pass");
    let map_tex = rf.map_tex;
    let t = &scene.textures[map_tex];
    assert_eq!((t.key, t.width, t.height), ((0, MAP_KEY), 256, 256));
    // The steps of §10.6.3 for the shipped CarRender (Sky ? 1, Desat ? 0, Sky blend 24 -> two cref
    // quads round the scene, no rmsk quad) [D carshade_disc every_tracks_reflection_map_inputs_decode].
    let ms = scene.drive_reflection.as_ref().expect("the map scene");
    let s = rf.render.sky_level();
    match ms.frame.steps.as_slice() {
        [MapStep::Quad { tex: a, rgba: ra, subtract: false }, MapStep::Scene, MapStep::Quad { tex: b, rgba: rb, subtract: false }] => {
            assert_eq!(a, b);
            assert_eq!(*ra, [s, s, s, 0x40]);
            assert_eq!(*rb, [s, s, s, rf.render.sky_blend]);
            assert!(rf.render.sky_blend != 0 && !rf.render.desat_on);
            // snow2a_mis4.ssh carries its own 128 x 128 cref [D].
            let c = &scene.textures[a - 1];
            assert_eq!((c.width, c.height), (128, 128));
        }
        other => panic!("steps {other:?}"),
    }

    // A known pose: the car 3 units beside a tall kept-able item (its top more than 10 over the car).
    let it = rf.items.iter().find(|it| nf_formats::rendertun::item_bounds(it.bounds).1 >= 15.0).expect("a tall item").clone();
    d.car.pos = [it.pos[0] + 3.0, it.pos[1], it.pos[2]];
    let (right, up) = ([0.6f32, 0.0, -0.8], [0.0f32, 1.0, 0.0]);
    let f = cross(up, right);
    scene.drive_particles = d.particle_groups(right, up);
    d.reflection_frame(&mut scene, right, up);
    let rf = d.reflection.as_ref().unwrap();
    let fr = &scene.drive_reflection.as_ref().unwrap().frame;
    assert!(fr.active);

    // The camera: CarRender::reflection_view and its documented formulas [M FUN_001c1b18].
    let v = fr.view.as_ref().expect("the view");
    let want = rf.render.reflection_view(d.car.pos, f);
    near(v.eye, want.eye, "eye");
    near(v.forward, want.forward, "forward");
    near(v.right, want.right, "right");
    near(v.up, want.up, "up");
    assert_eq!(v.projection, want.projection);
    near(v.eye, [d.car.pos[0], d.car.pos[1] + rf.render.refl_height, d.car.pos[2]], "eye = p + (0, HEIGHT, 0)");
    let fw = [0.8 * f[0], 0.8 * f[1] + 1.0, 0.8 * f[2]];
    let l = (fw[0] * fw[0] + fw[1] * fw[1] + fw[2] * fw[2]).sqrt();
    near(v.forward, fw.map(|c| c / l), "forward = normalize(0.8 f + up)");
    // The mirror: R x U = +F (docs §10.6.4), kept in the map.
    near(cross(v.right, v.up), v.forward, "R x U = F");
    // The GPU matrix lands a world point on ReflectionProjection::map_texel's texel.
    for q in [[it.pos[0], it.pos[1] + 12.0, it.pos[2]], [d.car.pos[0] + 5.0, d.car.pos[1] + 20.0, d.car.pos[2] - 4.0]] {
        let m = fr.view_proj;
        let c: [f32; 4] = [0, 1, 2, 3].map(|r| m[0][r] * q[0] + m[1][r] * q[1] + m[2][r] * q[2] + m[3][r]);
        let t = want.map_texel(q).expect("in front");
        let got = [128.0 + 128.0 * c[0] / c[3], 128.0 - 128.0 * c[1] / c[3]];
        assert!((got[0] - t[0]).abs() < 0.01 && (got[1] - t[1]).abs() < 0.01, "{q:?}: {got:?} vs {t:?}");
        assert!(c[2] / c[3] > 0.0 && c[2] / c[3] < 1.0, "depth in 0..1");
    }

    // The kept items: exactly the decoded Map in#0 items with drawn geometry that `keeps` keeps.
    let crp = nf_formats::crp::Crp::parse(&d.crp_bytes).unwrap();
    let words = nf_formats::geoprim::map_item_word15(&crp).unwrap();
    let drawn: std::collections::HashSet<usize> = nf_formats::geoprim::track_geometry(&d.crp_bytes).unwrap().strips.iter()
        .filter(|s| !s.article.to_ascii_lowercase().starts_with("sky")).map(|s| s.placement).collect();
    let mut expect: Vec<usize> = nf_formats::geoprim::map_placements(&crp).unwrap().iter()
        .filter(|p| drawn.contains(&p.index) && want.items.keeps(p.origin, words[p.index], p.flags))
        .map(|p| p.index).collect();
    expect.sort();
    expect.dedup();
    assert!(fr.kept_items.contains(&it.index), "the tall item is kept");
    assert_eq!(fr.kept_items, expect, "kept items");
    // The scene step's draws: the kept items' (table order), then each drawn vehicle's.
    let want_draws: Vec<_> = rf.items.iter().filter(|i| expect.contains(&i.index)).flat_map(|i| rf.item_draws[i.draws.clone()].iter().copied())
        .chain(fr.vehicles.iter().flat_map(|s| { let r = rf.vehicles.iter().find(|v| v.0 == *s).unwrap().1.clone(); rf.vehicle_draws[r].iter().copied() }))
        .collect();
    assert_eq!(fr.draws, want_draws);
    assert!(fr.draws.iter().all(|d| d.source == MapSource::Items));
    let item_draws = rf.items.iter().filter(|i| expect.contains(&i.index)).map(|i| i.draws.len()).sum::<usize>();
    // Vehicles: at most 4, never the player's own car.
    assert!(fr.vehicles.len() <= 4);
    assert!(d.player_attach_slot.is_none_or(|s| !fr.vehicles.contains(&s)));
    eprintln!("[refl] kept {} items, {} item draws, {} vehicles, {} draws", fr.kept_items.len(), item_draws, fr.vehicles.len(), fr.draws.len());

    // The car binds the map: an additive group on the map texture from exactly the strips whose
    // material has a reflection pass (Texture / TexturePacked / Chrome / GlassPacked) and Reflect > 0.
    let sh = d.shine.as_ref().expect("the car strips");
    let refl_mats = [CarMaterial::Texture, CarMaterial::TexturePacked, CarMaterial::Chrome, CarMaterial::GlassPacked];
    let want_strips: Vec<usize> = (0..sh.strips.len())
        .filter(|&i| refl_mats.contains(&sh.strips[i].0.material) && sh.strips[i].1.reflect > 0.0).collect();
    assert_eq!(rf.car_strips, want_strips);
    assert!(!want_strips.is_empty());
    assert!(sh.strips.iter().any(|(s, _)| s.material == CarMaterial::Dash), "the vanquish has dash strips, which do not reflect");
    let g = scene.drive_particles.iter().find(|g| g.0 == Some(map_tex)).expect("the reflection group");
    assert!(g.1, "additive (envmap ALPHA 0x48)");
    let n: usize = want_strips.iter().map(|&i| sh.strips[i].0.positions.len().saturating_sub(2) * 3).sum();
    assert_eq!(g.2.len(), n);
    // ST and colour of the first vertex: carmesh::reflection_vertex with the car's rotation rows.
    let (st0, pr0) = &sh.strips[want_strips[0]];
    let pv = nf_formats::carmesh::reflection_vertex(st0.normals[0], st0.positions[0], &d.car.rot, sh.entry.fresnel, sh.entry.reflection_warp, pr0.reflect, 127.0);
    assert!((g.2[0].uv[0] - pv.st[0]).abs() < 1e-5 && (g.2[0].uv[1] - pv.st[1]).abs() < 1e-5);
    assert_eq!(g.2[0].rgba, [pv.rgb.min(255) as u8, pv.rgb.min(255) as u8, pv.rgb.min(255) as u8, 127]);
    let _ = CarPassKind::Reflection;

    // Frame cost (240 Hz = 4.17 ms a frame): the per-frame CPU side, the query over every item
    // plus the car overlay's vertices, stays far under it.
    let mut ms_runs: Vec<f64> = (0..50).map(|_| {
        let t0 = std::time::Instant::now();
        d.reflection_frame(&mut scene, right, up);
        let e = t0.elapsed().as_secs_f64() * 1000.0;
        scene.drive_particles.retain(|g| g.0 != Some(map_tex));
        e
    }).collect();
    ms_runs.sort_by(f64::total_cmp);
    eprintln!("[refl] reflection_frame median {:.3} ms over {} items", ms_runs[25], d.reflection.as_ref().unwrap().items.len());
    assert!(ms_runs[25] < 0.5, "reflection_frame {:.3} ms", ms_runs[25]);
    d.reflection_frame(&mut scene, right, up);

    // GPU: the map holds the kept items (a render without the scene's draws differs) and the cref sky.
    let Some(dq) = device() else { eprintln!("no GPU adapter - the map render is skipped"); return };
    let with = render_map(&dq, &scene);
    if let Ok(p) = std::env::var("NF_REFL_MAP_OUT") {
        std::fs::write(&p, nf_formats::png::encode_rgba(256, 256, &with)).unwrap();
    }
    let saved = std::mem::take(&mut scene.drive_reflection.as_mut().unwrap().frame.draws);
    let without = render_map(&dq, &scene);
    scene.drive_reflection.as_mut().unwrap().frame.draws = saved;
    // Without the scene step the map is the two cref layers over the clear (black, alpha 1):
    // M = min(C·s/128, 1)·(0x40/128), then M·(1 − b) + min(C·s/128, 1)·b with b = Sky blend/128
    // (ALPHA 0x44, vertex 0x80 = 1.0; cref's alpha is 255 in every texel [D]).
    let ci = match scene.drive_reflection.as_ref().unwrap().frame.steps[0] { MapStep::Quad { tex, .. } => tex - 1, _ => unreachable!() };
    let c = &scene.textures[ci];
    assert!(c.rgba.chunks(4).all(|p| p[3] == 255));
    let (k, b) = (s as f32 / 128.0, rf.render.sky_blend as f32 / 128.0);
    let mut good = 0;
    for y in 0..256usize {
        for x in 0..256usize {
            let o = (y * 256 + x) * 4;
            let ok = (0..3).all(|ch| {
                let cv = c.rgba[((y / 2) * 128 + x / 2) * 4 + ch] as f32 / 255.0;
                let l = (cv * k).min(1.0);
                let want = (l * 0.5 * (1.0 - b) + l * b) * 255.0;
                (want - without[o + ch] as f32).abs() <= 6.0
            });
            good += ok as usize;
        }
    }
    assert!(good > 256 * 256 * 9 / 10, "the sky-only map is the two cref layers ({good} of 65536 texels match)");
    let changed = with.chunks(4).zip(without.chunks(4)).filter(|(a, b)| a[..3] != b[..3]).count();
    eprintln!("[refl] {changed} texels from the scene step");
    assert!(changed > 200, "the kept items are drawn into the map ({changed} texels)");
    // The tall item's base column, 12 units up, lands on a changed texel neighbourhood.
    let t = want.map_texel([it.pos[0], it.pos[1] + 12.0, it.pos[2]]).unwrap();
    let (cx, cy) = (t[0] as i32, t[1] as i32);
    let hit = (-6..=6).flat_map(|dy| (-6..=6).map(move |dx| (cx + dx, cy + dy)))
        .filter(|&(x, y)| (0..256).contains(&x) && (0..256).contains(&y))
        .any(|(x, y)| { let o = (y as usize * 256 + x as usize) * 4; with[o..o + 3] != without[o..o + 3] });
    assert!(hit, "the item is drawn where map_texel puts it ({t:?})");
}
