//! glass-guards-14: a guard standing behind one of 07000014's glass windows was invisible
//! through it. The windows are the level's three breakables (0x20 placements ServerRoomWindow,
//! CompRoom1Window, SCCP_Window [D]) and the glass leaves of the Door_Win_Slide_* doors — objects the
//! port poses on the door slot space (Character layer). Every such object drew in the character
//! pass (pass 2), ahead of the characters in material order and WITH depth write (ZBUF_1 ZMSK 0 [D]),
//! so the guard behind the pane failed the depth test.
//!
//! The game's rule: these are objects, and `View_AddObjects` 0x1e62d8:44-52 /
//! `View_AddForcedObjects` 0x1e5fb8:152-160 put an object in list 1 when `obj+0xF0 & 1`
//! (`Control_SetGList` 0x134960 copies the glist flags `celglist+0x44` to obj+0xF0; `Break_Create`
//! 0x12c230:19 and `Door_Create` 0x136100:71 call it) [M]. The windows' glist flags carry bit 0
//! (ServerRoomWindow / CompRoom1Window / SCCP_Window 0x1, Door_Win_Slide_* 0x40001 [D]); the guards'
//! do not. `Game_Draw` 0x1e6f50 draws list 1 (`View_DrawObjList` v,1 0x1e69a0) after list 0, where
//! the characters ride — so the guard is in the depth buffer before the glass blends over him.
//!
//! Opt-in: `NF_ISO`. `NF_GLASS_SHOTS=<dir>` also writes the renders as PNGs.

use nf_view::gpu;
use nf_view::scene::{Layer, Scene, ViewOpts};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Render the full scene from `cam` (the fix_0c_pylon_live harness: shot()'s path) and return RGBA.
fn render_full(scene: &Scene, cam: glam::Vec3, yaw: f32, pitch: f32, fov: f32, ticks: u32) -> (u32, u32, Vec<u8>) {
    let (w, h) = (1280u32, 720u32);
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

fn save(name: &str, w: u32, h: u32, rgba: &[u8]) {
    if let Some(dir) = std::env::var_os("NF_GLASS_SHOTS") {
        let p = std::path::Path::new(&dir).join(name);
        std::fs::write(&p, nf_formats::png::encode_rgba(w, h, rgba)).expect("write png");
        eprintln!("wrote {}", p.display());
    }
}

/// Pixels in the rect whose colour differs by more than 24 in any channel.
fn changed(w: u32, a: &[u8], b: &[u8], x0: u32, y0: u32, x1: u32, y1: u32) -> u32 {
    let mut n = 0;
    for y in y0..y1 { for x in x0..x1 {
        let i = ((y * w + x) * 4) as usize;
        if (0..3).any(|k| (a[i + k] as i32 - b[i + k] as i32).abs() > 24) { n += 1; }
    }}
    n
}

/// The eye west of CompRoom1Window (x 30.62..31.27, y 1.34..2.97, z 30.82..35.68 [D its verts]),
/// looking east (+x) through it.
const EYE: [f32; 3] = [27.5, 1.75, 33.2];
const YAW: f32 = std::f32::consts::FRAC_PI_2;
/// Behind the pane: the raised floor the level's own guard there stands on (drone 5's spawn
/// at (32.22, 1.80, 33.22) [D placement]).
const BEHIND: [f32; 3] = [32.6, 1.8, 33.2];

/// `NF_HIDE` is process-wide: the loads take turns.
static LOAD: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn load(hide_glass: bool) -> Scene {
    let _g = LOAD.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("NF_SILENT", "1");
    if hide_glass { std::env::set_var("NF_HIDE", "CompRoom1Window"); } else { std::env::remove_var("NF_HIDE"); }
    let sc = nf_view::load(&iso().unwrap(), "07000014.bin", false).expect("load 07000014");
    std::env::remove_var("NF_HIDE");
    sc
}

/// The one guard present at load (drone 22 [L the load census]) — moved behind the pane, facing the eye.
fn place_guard(sc: &mut Scene, at: [f32; 3]) {
    let sim = sc.sim.as_mut().unwrap();
    let d = sim.drones.list.iter_mut().find(|d| d.present && d.enabled).expect("a present guard at load");
    d.pos = at;
    d.yaw = -std::f32::consts::FRAC_PI_2;
    d.desired_yaw = d.yaw;
}

fn guard_pixels(sc: &mut Scene, tag: &str) -> u32 {
    let home = sc.sim.as_ref().unwrap().drones.list.iter().find(|d| d.present && d.enabled).unwrap().pos;
    let cam = glam::Vec3::from(EYE);
    let fov = std::f32::consts::FRAC_PI_3;
    place_guard(sc, BEHIND);
    let (w, h, with) = render_full(sc, cam, YAW, 0.0, fov, 30);
    save(&format!("{tag}_guard.png"), w, h, &with);
    place_guard(sc, home);
    let (_, _, without) = render_full(sc, cam, YAW, 0.0, fov, 30);
    save(&format!("{tag}_empty.png"), w, h, &without);
    // The pane's screen band at this eye (the window frame spans x ≈ 180..1100, y ≈ 90..440 in
    // the 1280×720 render); the guard 5.1 u ahead fills the middle of it.
    changed(w, &with, &without, 400, 150, 880, 440)
}

#[test]
fn a_guard_behind_the_comp_room_window_is_visible_through_it() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return }
    let mut sc = load(false);
    let seen = guard_pixels(&mut sc, "glass");
    eprintln!("[glass-guards-14] guard pixels through the pane: {seen}");
    // Sister: the same render with the pane left out of the build — the guard's full footprint,
    // which the old tree also drew. The fix must reach it through the glass.
    let mut open = load(true);
    let bare = guard_pixels(&mut open, "noglass");
    eprintln!("[glass-guards-14] guard pixels with the pane removed: {bare}");
    assert!(bare > 2000, "the guard's footprint without the pane is on screen (sister: the setup can see him), got {bare}");
    assert!(seen * 2 > bare,
        "the guard behind CompRoom1Window shows through the glass (list 1 after the characters [M View_AddObjects 0x1e62d8:44-52, Game_Draw 0x1e6f50]): {seen} vs {bare} without the pane");
}

/// The rule on the material table: every Character-layer posed object whose glist flags carry
/// bit 0 rides the alpha list (pass 3, after the characters); the characters stay in pass 2.
#[test]
fn the_14_windows_ride_the_alpha_list_after_the_characters() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return }
    let sc = load(false);
    let mut windows = 0;
    for m in sc.materials.iter().filter(|m| m.layer == Layer::Character && m.skin.is_none() && m.index_count > 0) {
        let v = sc.indices[m.first_index as usize] as usize - sc.char_vertex_start;
        // The three breakables' slots carry the window textures (ServerRoomWindow's verts at
        // x 50.45..51.10 — tex (54,96) [D]); found by the draw state the disc gives them.
        if sc.char_verts[v].door.is_some() && m.key == Some((54, 96)) {
            windows += 1;
            assert_eq!(m.draw.pass, 3, "a window pane draws in the alpha list (list 1), got {:?}", m.draw);
        }
    }
    assert!(windows > 0, "the window materials exist (sister: the filter finds them)");
    let chars = sc.materials.iter().filter(|m| m.layer == Layer::Character && m.skin.is_some_and(|(_, a)| a.is_none()) && m.node == u32::MAX).count();
    assert!(chars > 0);
    assert!(sc.materials.iter().filter(|m| m.layer == Layer::Character && m.skin.is_some_and(|(_, a)| a.is_none()) && m.node == u32::MAX)
        .all(|m| m.draw.pass == 2), "the characters stay in list 0's character pass");
}
