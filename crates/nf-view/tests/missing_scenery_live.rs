//! missing-scenery: two pieces of disc scenery the port did not show.
//!
//! 1. 0700000c's mountain range ("Mountains_new1", a type-0x2A sky object in sky group 1 — placement
//!    #260 param 4 = 1 [D]). `View_DrawSky` 0x1e5ad0 writes no Z register [M]; the glist's own ZBUF_1
//!    is 0x31000000 (ZMSK 0, depth write) [D], so the mountains write depth before the world, and
//!    the post-fog skybox ("SkyBox1-terra", group 0, ZBUF_1 0x1_31000000) only fills what is left.
//!    The port drew the group-1 sky with no depth write, so its far-pinned skybox painted over it.
//! 2. 07000004's courtyard trees (two 0xF3 placements, skin 0x05000003). The skin has no skinned
//!    mesh, only six rigid parts on bones 0..5 [D]; `AnimObjectNew` 0x1be910 makes one object per
//!    rigid part and `AnimObjectUpdate` 0x1befd8 copies the bone matrix into each [M]. The port only
//!    built characters from main meshes, so the tree drew nothing.
//!
//! Each test renders the full scene through the renderer and compares it with the same frame
//! minus the object's materials (the sister: the comparison must see the object). Opt-in NF_ISO.

use nf_view::gpu;
use nf_view::scene::{Layer, Scene, ViewOpts};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Render the full scene from `cam` and return the RGBA frame (the fix_0c_pylon_live path).
fn render_full(scene: &Scene, cam: glam::Vec3, yaw: f32, pitch: f32, ticks: u32) -> (u32, u32, Vec<u8>) {
    let (w, h) = (640u32, 360u32);
    let fov = std::f32::consts::FRAC_PI_3;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.expect("adapter");
        adapter.request_device(&Default::default()).await.expect("device")
    });
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut r = gpu::Renderer::new(&device, &queue, format, scene, false, false);
    let cp = pitch.cos();
    let fw = glam::Vec3::new(cp * yaw.sin(), pitch.sin(), cp * yaw.cos()).normalize();
    let s = fw.cross(glam::Vec3::Y).normalize();
    let u = s.cross(fw);
    let f = 1.0 / (fov * 0.5).tan();
    let proj = glam::Mat4::from_cols_array_2d(&[
        [f / (w as f32 / h as f32), 0.0, 0.0, 0.0], [0.0, f, 0.0, 0.0],
        [0.0, 0.0, 4000.0 / (0.05 - 4000.0), -1.0], [0.0, 0.0, 0.05 * 4000.0 / (0.05 - 4000.0), 0.0]]);
    let view = glam::Mat4::from_cols_array_2d(&[
        [s.x, u.x, -fw.x, 0.0], [s.y, u.y, -fw.y, 0.0], [s.z, u.z, -fw.z, 0.0],
        [-s.dot(cam), -u.dot(cam), fw.dot(cam), 1.0]]);
    let vp = (proj * view).to_cols_array_2d();
    let vis = r.cel_visibility(cam.to_array(), vp, scene.sim.as_ref().map(|s| &s.world), None);
    let mut opts = ViewOpts::default();
    opts.right = s.to_array();
    opts.up = u.to_array();
    r.update_skin(&queue, scene, None, ticks as f32, opts, vis.as_deref());
    r.lights = scene.frame_lights();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let tview = target.create_view(&Default::default());
    let depth = gpu::depth_view(&device, w, h);
    let mut enc = r.render(&device, &queue, &tview, &depth, vp, vp, cam.to_array(), ticks, &[], &gpu::HudFrame::default(), None, None);
    let row = (4 * w).next_multiple_of(256);
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
    queue.submit([enc.finish()]);
    buf.map_async(wgpu::MapMode::Read, .., |r| r.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let data = buf.get_mapped_range(..).expect("map range");
    let mut rgba = Vec::with_capacity((4 * w * h) as usize);
    for y in 0..h as usize { rgba.extend_from_slice(&data[y * row as usize..][..4 * w as usize]); }
    (w, h, rgba)
}

/// Pixels whose RGB differs by more than 24 in any channel.
fn changed(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| (0..3).any(|c| (p[c] as i32 - q[c] as i32).abs() > 24)).count()
}

/// Render `scene` with the materials `drop` selects removed (the scene is restored afterwards).
fn render_without(scene: &mut Scene, drop: impl Fn(&nf_view::scene::Material) -> bool, cam: glam::Vec3, yaw: f32, pitch: f32) -> Vec<u8> {
    let all = scene.materials.clone();
    scene.materials.retain(|m| !drop(m));
    assert!(scene.materials.len() < all.len(), "the selection must remove materials");
    let out = render_full(scene, cam, yaw, pitch, 30).2;
    scene.materials = all;
    out
}

/// 0700000c, the walkway behind the silos (`--bond 13.5,16.8,-150.0,3.14159,0`): the mountains
/// fill part of the frame beside and behind the silos.
#[test]
fn the_0c_mountain_range_draws_behind_the_plant() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc: Scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    // The disc state the fix reads: the group-1 mountains write depth, the group-0 skybox does not.
    let mountains: Vec<_> = sc.materials.iter().filter(|m| m.layer == Layer::SkyFixed).collect();
    assert!(!mountains.is_empty(), "Mountains_new1 is a fixed sky object [D placement #260]");
    assert!(mountains.iter().all(|m| m.draw.pass == 0 && m.draw.zwrite), "the group-1 mountains: pre-world sky pass, ZBUF_1 0x31000000 (write) [D]");
    assert!(sc.materials.iter().filter(|m| m.layer == Layer::SkyCamera).all(|m| m.draw.pass == 4 && !m.draw.zwrite),
        "SkyBox1-terra: post-fog sky pass, ZBUF_1 0x1_31000000 (no write) [D]");
    let (cam, yaw, pitch) = (glam::Vec3::new(13.5, 17.5, -150.0), std::f32::consts::PI, 0.0);
    let full = render_full(&sc, cam, yaw, pitch, 30).2;
    let without = render_without(&mut sc, |m| m.layer == Layer::SkyFixed, cam, yaw, pitch);
    let n = changed(&full, &without);
    eprintln!("pixels the mountains change: {n}");
    assert!(n > 4000, "the 0c mountains are not on screen (pixels changed by removing them: {n})");
    // Sister: with the mountains' depth write off (the old pre-world sky state) the skybox paints
    // over them again — the frame matches the one without mountains. This is the regression.
    for m in sc.materials.iter_mut().filter(|m| m.layer == Layer::SkyFixed) { m.draw.zwrite = false; }
    let nowrite = render_full(&sc, cam, yaw, pitch, 30).2;
    let n2 = changed(&nowrite, &without);
    eprintln!("pixels changed with no mountain depth write: {n2}");
    assert!(n2 < n / 10, "without the depth write the skybox should cover the mountains ({n2} vs {n})");
}

/// 07000004, the courtyard (`--bond 8.0,5.5,37.0,3.14,0`): the tree at placement #30 stands left of
/// the view centre, its trunk lit by the ground lamp.
#[test]
fn the_04_courtyard_tree_draws() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc: Scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
    // The disc shape: the tree skin carries rigid parts only, so the main-mesh character path
    // builds nothing for the tree placements (the old draw path's input was empty).
    let level_trees: Vec<usize> = sc.tree_chars.iter().map(|&(_, pi)| pi).collect();
    assert_eq!(level_trees.len(), 2, "both 0xF3 Tree placements get a drawn character [D #30, #31]");
    for &(ci, _) in &sc.tree_chars {
        let c = &sc.characters[ci];
        assert_eq!(c.skin.hashcode, 0x0500_0003, "the Tree model [D param 0]");
        assert!(c.skin.main.iter().flatten().next().is_none(), "no skinned main mesh [D]");
        assert_eq!(sc.attachments.iter().filter(|a| a.character as usize == ci).count(), 6, "six rigid parts on bones 0..5 [D]");
    }
    // The parts are posed by the paused sway script: every part sits around the placement, and the
    // crown (bone 2, tree_top) is above the trunk base (bone 0).
    let fs = sc.frame_state(None, 30.0, ViewOpts::default());
    for &(ci, pi) in &sc.tree_chars {
        let p = sc.characters[ci].matrix[3];
        let parts: Vec<(usize, [f32; 4])> = sc.attachments.iter().enumerate().filter(|(_, a)| a.character as usize == ci).map(|(k, a)| (a.bone, fs.attach[k][3])).collect();
        for (bone, t) in &parts {
            let d = ((t[0] - p[0]).powi(2) + (t[1] - p[1]).powi(2) + (t[2] - p[2]).powi(2)).sqrt();
            assert!(d < 12.0, "tree {pi} bone {bone} part sits {d} from the placement (collapsed or thrown away)");
        }
        let y = |b: usize| parts.iter().find(|(bb, _)| *bb == b).map(|(_, t)| t[1]).unwrap();
        assert!(y(2) > y(0) + 1.0, "tree {pi}: the crown (bone 2) must stand above the base (bone 0): {} vs {}", y(2), y(0));
    }
    let tree_ci: Vec<u32> = sc.tree_chars.iter().map(|&(ci, _)| ci as u32).collect();
    let (cam, yaw, pitch) = (glam::Vec3::new(8.0, 6.2, 37.0), 3.14, 0.0);
    let full = render_full(&sc, cam, yaw, pitch, 30).2;
    let without = render_without(&mut sc, |m| m.skin.is_some_and(|(ci, _)| tree_ci.contains(&ci)), cam, yaw, pitch);
    let n = changed(&full, &without);
    eprintln!("pixels the tree changes: {n}");
    assert!(n > 3000, "the 04 courtyard tree is not on screen (pixels changed by removing it: {n})");
    // Sister: facing away from the tree (yaw 0) the same removal leaves the frame unchanged —
    // the comparison only reacts to a tree that is drawn in view.
    let away_full = render_full(&sc, cam, 0.0, pitch, 30).2;
    let away_without = render_without(&mut sc, |m| m.skin.is_some_and(|(ci, _)| tree_ci.contains(&ci)), cam, 0.0, pitch);
    let n0 = changed(&away_full, &away_without);
    eprintln!("pixels the tree changes facing away: {n0}");
    assert!(n0 < 200, "facing away from the tree its removal changed {n0} pixels");
}
