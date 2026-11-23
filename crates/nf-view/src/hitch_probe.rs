//! frame-hitches: count the WORK each frame does (not its wall time) so the frames that do far more
//! than the median stand out on a loaded machine.
//!
//! The probe runs the window's frame headless: `HeadlessApp::frame` (the RedrawRequested logic half:
//! ticks, sound_tick, scripts) at a 240 Hz cadence (one sim tick every fourth frame), then a replica
//! of `App::draw_frame`'s draw half on a headless wgpu device (palette rebuild on tick frames, cel
//! visibility, blend + upload, HUD, lights, shards/particles, the render encode and submit). Per frame
//! it records:
//! - heap allocations (count, bytes, and those of 64 KiB or more) from the counting allocator below,
//!   split by draw phase,
//! - the `nf_formats::workcount` counters (wgpu creations and queue writes in gpu.rs, the sound
//!   driver's first-use sample/stream decodes).
//!
//! The allocator counts only while `ON` is set (the probe sets it); every other test in this binary
//! pays one relaxed atomic load per allocation.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

pub const PHASES: usize = 10;
pub const PHASE_NAMES: [&str; PHASES] = ["other", "logic", "palette", "vis", "upload", "hud", "misc", "fx", "encode", "submit"];
/// Set once any probe has counted (the fast exit for every other test in this binary).
static ANY: AtomicBool = AtomicBool::new(false);
pub const BIG: usize = 64 * 1024;

thread_local! {
    // Per thread: only the probe's own thread counts (a worker's allocations, the sound predecode's
    // or wgpu's, are not the frame's work), and probes on parallel test threads do not mix.
    static ON: Cell<bool> = const { Cell::new(false) };
    static PHASE: Cell<usize> = const { Cell::new(0) };
    static N: [Cell<u64>; PHASES] = const { [const { Cell::new(0) }; PHASES] };
    static BYTES: [Cell<u64>; PHASES] = const { [const { Cell::new(0) }; PHASES] };
    static BIG_N: Cell<u64> = const { Cell::new(0) };
    static BIG_BYTES: Cell<u64> = const { Cell::new(0) };
    /// Allocations at least this large record a backtrace (usize::MAX = none).
    static TRACE_MIN: Cell<usize> = const { Cell::new(usize::MAX) };
    static IN_TRACE: Cell<bool> = const { Cell::new(false) };
    static TRACES: std::cell::RefCell<Vec<(usize, usize, std::backtrace::Backtrace)>> = const { std::cell::RefCell::new(Vec::new()) };
}

struct Counting;

#[inline]
fn note(size: usize) {
    if !ANY.load(Ordering::Relaxed) { return; }
    if !ON.try_with(|c| c.get()).unwrap_or(false) { return; }
    let p = PHASE.with(|c| c.get());
    N.with(|n| n[p].set(n[p].get() + 1));
    BYTES.with(|b| b[p].set(b[p].get() + size as u64));
    if size >= BIG {
        BIG_N.with(|c| c.set(c.get() + 1));
        BIG_BYTES.with(|c| c.set(c.get() + size as u64));
    }
    if size >= TRACE_MIN.with(|c| c.get()) && !IN_TRACE.with(|f| f.get()) {
        IN_TRACE.with(|f| f.set(true));
        let bt = std::backtrace::Backtrace::force_capture();
        let _ = TRACES.try_with(|t| t.borrow_mut().push((size, p, bt)));
        IN_TRACE.with(|f| f.set(false));
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 { note(l.size()); unsafe { System.alloc(l) } }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 { note(l.size()); unsafe { System.alloc_zeroed(l) } }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { unsafe { System.dealloc(p, l) } }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        if new > l.size() { note(new); }
        unsafe { System.realloc(p, l, new) }
    }
}

/// Counting on or off for the calling thread.
pub fn counting(on: bool) { ANY.store(true, Ordering::Relaxed); ON.with(|c| c.set(on)); }

#[global_allocator]
static A: Counting = Counting;

pub fn set_phase(p: usize) { PHASE.with(|c| c.set(p)); }

/// One frame's counted work.
#[derive(Clone, Debug, Default)]
pub struct FrameWork {
    pub n: [u64; PHASES],
    pub bytes: [u64; PHASES],
    pub big_n: u64,
    pub big_bytes: u64,
    pub wc: Vec<(&'static str, u64, u64)>,
}

impl FrameWork {
    pub fn allocs(&self) -> u64 { self.n.iter().sum() }
    pub fn alloc_bytes(&self) -> u64 { self.bytes.iter().sum() }
    pub fn wc(&self, name: &str) -> (u64, u64) {
        self.wc.iter().filter(|e| e.0 == name).fold((0, 0), |a, e| (a.0 + e.1, a.1 + e.2))
    }
    /// Every event name containing `pat` (events, bytes).
    pub fn wc_like(&self, pat: &str) -> (u64, u64) {
        self.wc.iter().filter(|e| e.0.contains(pat)).fold((0, 0), |a, e| (a.0 + e.1, a.1 + e.2))
    }
}

/// The counters since the last take, cleared.
pub fn take() -> FrameWork {
    let mut f = FrameWork::default();
    for i in 0..PHASES {
        f.n[i] = N.with(|n| n[i].replace(0));
        f.bytes[i] = BYTES.with(|b| b[i].replace(0));
    }
    f.big_n = BIG_N.with(|c| c.replace(0));
    f.big_bytes = BIG_BYTES.with(|c| c.replace(0));
    let was = ON.with(|c| c.replace(false));
    f.wc = nf_formats::workcount::take();
    ON.with(|c| c.set(was));
    f
}

pub fn trace_from(min: usize) { TRACE_MIN.with(|c| c.set(min)); }

/// The backtraces recorded since the last call: (size, phase, the first few crate frames).
pub fn take_traces() -> Vec<(usize, usize, String)> {
    let was = ON.with(|c| c.replace(false));
    let raw = TRACES.with(|t| std::mem::take(&mut *t.borrow_mut()));
    let out = raw.into_iter().map(|(sz, p, bt)| {
        let s = format!("{bt}");
        if std::env::var_os("NF_PROBE_RAW").is_some() && sz >= 200 * 1024 { eprintln!("[raw] {sz}
{s}"); }
        let lines: Vec<&str> = s.lines().map(str::trim).collect();
        let mut frames: Vec<String> = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            if l.starts_with("at ") || !l.contains("::") || !l.contains("nf_") || l.contains("hitch_probe") || l.contains("alloc::") { continue; }
            let name = l.trim_start_matches(|c: char| c.is_ascii_digit() || c == ':' || c == ' ');
            let at = lines.get(i + 1).filter(|n| n.starts_with("at ")).map(|n| n.rsplit(|c: char| c == '/' || c == char::from(92u8)).next().unwrap_or("").to_string()).unwrap_or_default();
            frames.push(if at.is_empty() { name.to_string() } else { format!("{name} ({at})") });
            if frames.len() == 4 { break; }
        }
        (sz, p, frames.join(" < "))
    }).collect();
    ON.with(|c| c.set(was));
    out
}

#[cfg(not(debug_assertions))]
pub mod probe {
    use super::*;
    use crate::headless_app_harness::HeadlessApp;
    use crate::{gpu, App};

    pub struct Gpu { pub device: wgpu::Device, pub queue: wgpu::Queue, pub view: wgpu::TextureView, pub depth: wgpu::TextureView, _t: wgpu::Texture }

    /// The window's device recipe on a headless adapter (frame_path_probe_live's `app_device`), 1280x720.
    pub fn gpu() -> Option<Gpu> {
        let instance = crate::gpu_instance_headless();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
        let want = wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::INDIRECT_FIRST_INSTANCE;
        let desc = wgpu::DeviceDescriptor { required_features: adapter.features() & want, required_limits: adapter.limits(), ..Default::default() };
        let (device, queue) = pollster::block_on(adapter.request_device(&desc)).ok()?;
        let (w, h) = (1280u32, 720u32);
        let t = device.create_texture(&wgpu::TextureDescriptor {
            label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] });
        let view = t.create_view(&Default::default());
        let depth = gpu::depth_view(&device, w, h);
        Some(Gpu { device, queue, view, depth, _t: t })
    }

    /// `App::draw_frame`'s draw half, replicated for a windowless App (the lines between the gfx
    /// borrow and the present, minus the menu/movie branches: the probe drives missions only).
    pub fn draw(app: &mut App, r: &mut gpu::Renderer, g: &Gpu) {
        let aspect = 16.0f32 / 9.0;
        app.fps_meter.frame();
        set_phase(2);
        let (rr, u) = crate::cam_axes(&app.cam);
        app.opts.right = rr;
        app.opts.up = u;
        let interp0 = !app.noclip && !app.paused && app.scene.sim.is_some();
        let pal_rebuilt = app.ticked || app.pal_cur.is_none();
        if pal_rebuilt {
            let (mut np, mut nc) = app.pal_prev.take().or_else(|| app.pal_spare.pop()).unwrap_or_default();
            app.scene.skin_palette_into(&r.skin_layout, app.choice, app.anim_time as f32, app.opts, None, &mut np, &mut nc);
            let new = (np, nc);
            r.char_light_centre = app.scene.char_light_centres();
            app.pal_prev = app.pal_cur.take();
            app.pal_moving.clear();
            let made = app.scene.sim.as_ref().map_or((0, 0), |s| (s.drones.effects.made, s.drones.effects.debris_made));
            let prev = app.fx_made.unwrap_or(made);
            let mut snap = app.scene.decal_reused_entries(&r.skin_layout, prev.0);
            snap.extend(app.scene.debris_reused_entries(&r.skin_layout, prev.1));
            app.fx_made = Some(made);
            app.decal_cells = app.scene.decal_cell_entries(&r.skin_layout);
            if let Some((pp, _)) = &app.pal_prev { app.pal_moving.extend(crate::moving_entries(pp, &new.0, &snap)); }
            app.pal_blend.clear();
            app.pal_blend.extend_from_slice(&new.0);
            app.pal_cur = Some(new);
            app.beam_entries = app.scene.beam_entries(&r.skin_layout);
        }
        let (draw_cam, alpha) = crate::draw_bracket(&mut app.cam_prev, &mut app.cam_cur, app.cam, pal_rebuilt, app.anim_time.fract() as f32, interp0);
        let cutscene = app.scene.sim.as_ref().is_some_and(|s| s.script_camera().is_some());
        let (proj_aspect, band) = if cutscene { crate::cutscene_frame(1280.0, 720.0, app.game_aspect) } else { (aspect, [0.0; 4]) };
        let (vp0, _) = draw_cam.view_proj_for(proj_aspect, app.scene.mirror_x);
        set_phase(3);
        let door_closed = app.scene.sim.as_ref().map(|s| s.drones.doors.closed_flags());
        let vis = r.cel_visibility(draw_cam.pos.to_array(), vp0, app.scene.sim.as_ref().map(|s| &s.world), door_closed.as_deref());
        set_phase(4);
        if let Some((pal, cols)) = &app.pal_cur {
            if app.pal_blend.len() == pal.len() {
                for &e in &app.pal_hidden { if let Some(m) = pal.get(e as usize) { app.pal_blend[e as usize] = *m; } }
            }
            app.pal_hidden.clear();
            let hide_any = vis.is_some() && !app.decal_cells.is_empty();
            match &app.pal_prev {
                Some((pp, _)) if interp0 => {
                    let vm_entries = crate::view_model_entries(&app.scene, &r.skin_layout);
                    let blended = &mut app.pal_blend;
                    if pp.len() == pal.len() && blended.len() == pal.len() {
                        crate::lerp_palette_moving(blended, pp, pal, &app.pal_moving, vm_entries.as_deref().unwrap_or(&[]), alpha);
                    } else {
                        *blended = crate::lerp_palette(pp, pal, alpha);
                    }
                    if app.scene.sim.as_ref().map_or(true, |s| s.script_camera().is_none()) {
                        if let (Some(entries), Some(cp), Some(cc)) = (vm_entries.as_ref(), app.cam_prev, app.cam_cur) {
                            crate::reattach_view_model(blended, pp, pal, alpha, &cp, &cc, &draw_cam, entries);
                            crate::reattach_beams(blended, pp, pal, alpha, &cp, &cc, &draw_cam, &app.beam_entries);
                        }
                    }
                    if let (true, Some(v)) = (hide_any, vis.as_deref()) {
                        app.pal_hidden = app.scene.hide_unseen_decals(blended, &app.decal_cells, v);
                    }
                    r.upload_skin(&g.queue, blended, cols);
                }
                _ if hide_any && app.pal_blend.len() == pal.len() => {
                    app.pal_blend.copy_from_slice(pal);
                    if let Some(v) = vis.as_deref() { app.scene.hide_unseen_decals(&mut app.pal_blend, &app.decal_cells, v); }
                    r.upload_skin(&g.queue, &app.pal_blend, cols);
                }
                _ => r.upload_skin(&g.queue, pal, cols),
            }
            if pal_rebuilt || app.scene.sim.is_none() { r.upload_morph_weights(&g.queue, &app.scene); }
        }
        set_phase(5);
        let (_, vp_rot) = draw_cam.view_proj_for(proj_aspect, app.scene.mirror_x);
        crate::snow_for_into(&app.scene, &draw_cam, &mut app.snow_buf);
        let snow = std::mem::take(&mut app.snow_buf);
        crate::hud::set_view_h(720.0);
        let mut hud = crate::overlay_for(&app.scene, proj_aspect);
        if app.fps_meter.shown > 0.0 { hud.append(&crate::dev::fps_counter_hud(&app.scene, proj_aspect, app.fps_meter.shown)); }
        if crate::scene_overlay_up(&app.scene, false) {
            let mut f = app.scene.entry_fade_hud(proj_aspect);
            f.append(&hud);
            hud = f;
            hud.append(&app.scene.script_overlay_hud(proj_aspect));
        }
        set_phase(6);
        r.lights = app.scene.frame_lights();
        r.screen_aspect = 16.0 / 9.0;
        r.destroy_dead = crate::dead_destroy_centres(&app.scene);
        r.sky_flash_mask = app.scene.sim.as_ref().map_or(0, |s| s.drones.sky.flash_mask);
        r.sky_spin = app.scene.sim.as_ref().map_or(0.0, |s| s.drones.sky.spin());
        r.destroy_wreck_draw = crate::wreck_draw_centres(&app.scene);
        r.destroy_tint = crate::destroy_tints(&app.scene);
        set_phase(7);
        r.set_shards(&g.queue, &app.scene.shard_draws());
        r.set_particles(&g.queue, &crate::particles_for(&app.scene, &draw_cam));
        set_phase(8);
        let enc = r.render(&g.device, &g.queue, &g.view, &g.depth, vp0, vp_rot, draw_cam.pos.to_array(), app.anim_time as u32, &snow, &hud, None, if cutscene { Some(band) } else { None });
        app.snow_buf = snow;
        set_phase(9);
        g.queue.submit([enc.finish()]);
        set_phase(0);
    }

    /// The scripted input for frame `f` of the run (240 Hz frames): idle for the first 20 s (the
    /// opening scene and the first look), then W held with a slow turn (one turn ~ 6 s), and a
    /// trigger tap every 2 s (the first shot's sound, flash and hole).
    pub fn input(app: &mut App, f: u32) {
        use winit::keyboard::KeyCode;
        let s = f as f32 / 240.0;
        if s >= 20.0 {
            app.keys.insert(KeyCode::KeyW);
            app.turn += 0.0044;
            app.trigger = (f % 480) < 24;
        }
    }

    pub struct Run { pub level: String, pub frames: Vec<(bool, FrameWork)>, pub traces: Vec<(u32, usize, usize, String)> }

    /// Loads `level` as a mission and runs `frames` 240 Hz frames, counting each. Frames whose
    /// allocated bytes reach `trace_min` record where (the backtraces).
    pub fn run(iso: &str, level: &str, g: &Gpu, frames: u32, trace_min: usize, mut each: impl FnMut(&mut App, u32)) -> Run {
        let mut h = HeadlessApp::mission(iso, level);
        let mut r = gpu::Renderer::new(&g.device, &g.queue, wgpu::TextureFormat::Rgba8Unorm, &h.app.scene, false, false);
        let _ = take();
        let _ = take_traces();
        trace_from(trace_min);
        let mut out = Vec::with_capacity(frames as usize);
        let mut traces = Vec::new();
        for f in 0..frames {
            each(&mut h.app, f);
            counting(true);
            set_phase(1);
            h.frame(1.0 / 240.0);
            let ticked = h.app.ticked;
            draw(&mut h.app, &mut r, g);
            counting(false);
            out.push((ticked, take()));
            for (sz, p, s) in take_traces() { traces.push((f, sz, p, s)); }
            if f % 32 == 31 { let _ = g.device.poll(wgpu::PollType::wait_indefinitely()); }
            if h.app.async_load.is_some() { break; }
        }
        trace_from(usize::MAX);
        drop(r);
        let _ = g.device.poll(wgpu::PollType::wait_indefinitely());
        Run { level: level.to_string(), frames: out, traces }
    }

    /// A committed walk recording (tests/replays) replayed through the sim at the 240 Hz cadence:
    /// each recorded tick steps the sim and the window's `sound_tick`, then four draws (the tick
    /// frame and three in-between frames), each counted like `run`'s.
    pub fn run_replay(iso: &str, path: &str, g: &Gpu, max_ticks: u32, trace_min: usize) -> Run {
        let replay = crate::replay::Replay::load(path).expect("recording");
        let level = replay.header.level.clone();
        let mut h = HeadlessApp::mission(iso, &level);
        if let (Some(o), Some(sim)) = (replay.header.options, h.app.scene.sim.as_mut()) {
            sim.auto_aim = o.auto_aim; sim.drones.auto_switch = o.auto_switch; sim.drones.crouch_toggle = o.crouch_toggle; sim.drones.crosshair = o.crosshair;
        }
        let mut r = gpu::Renderer::new(&g.device, &g.queue, wgpu::TextureFormat::Rgba8Unorm, &h.app.scene, false, false);
        let _ = take();
        let _ = take_traces();
        trace_from(trace_min);
        let (mut out, mut traces) = (Vec::new(), Vec::new());
        let mut f = 0u32;
        for t in replay.ticks.iter().take(max_ticks as usize) {
            for sub in 0..4 {
                counting(true);
                set_phase(1);
                if sub == 0 {
                    let app = &mut h.app;
                    let Some(sim) = app.scene.sim.as_mut() else { break };
                    let (input, trigger, reload, use_key) = t.to_parts();
                    if t.skip { sim.drones.scripts.skip_request = true; }
                    sim.step_bond(input, trigger, reload, use_key);
                    if let Some(snd) = app.sound.as_mut() { crate::sound_tick(snd, sim); }
                    let (p, yaw, pitch) = (sim.bond.head(), sim.bond.yaw, sim.bond.pitch);
                    app.cam.pos = glam::Vec3::from(p);
                    app.cam.yaw = yaw;
                    app.cam.pitch = pitch;
                    app.ticked = true;
                } else { h.app.ticked = false; }
                h.app.anim_time += 0.25;
                draw(&mut h.app, &mut r, g);
                counting(false);
                out.push((sub == 0, take()));
                for (sz, p, s) in take_traces() { traces.push((f, sz, p, s)); }
                if f % 32 == 31 { let _ = g.device.poll(wgpu::PollType::wait_indefinitely()); }
                f += 1;
            }
        }
        trace_from(usize::MAX);
        drop(r);
        let _ = g.device.poll(wgpu::PollType::wait_indefinitely());
        Run { level: format!("{level} (replay)"), frames: out, traces }
    }

    pub fn median(mut v: Vec<u64>) -> u64 { if v.is_empty() { return 0; } v.sort(); v[v.len() / 2] }
}

#[cfg(all(test, not(debug_assertions)))]
mod census {
    use super::probe::*;
    use super::*;

    fn on_foot(iso: &str) -> Vec<String> {
        let all = ["07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007", "07000008", "07000009",
            "0700000a", "0700000b", "0700000c", "0700000d", "07000011", "07000012", "07000013", "07000014", "07000015", "07000016", "0700001b"];
        let _ = iso;
        let only = std::env::var("NF_PROBE_LEVELS").ok();
        all.iter().filter(|l| only.as_deref().map_or(true, |o| o.split(',').any(|x| l.starts_with(x.trim())))).map(|l| format!("{l}.bin")).collect()
    }

    fn kb(b: u64) -> String { if b >= 1 << 20 { format!("{:.1}M", b as f64 / 1048576.0) } else { format!("{}K", b / 1024) } }

    /// Measurement only: every on-foot level's first 60 s (20 s idle, then a walk with a slow turn
    /// and a shot every 2 s) at the 240 Hz cadence, every frame's work counted; the frames far above
    /// their kind's median (tick frames against tick frames) are printed with their causes.
    #[test]
    #[ignore]
    fn hitch_census_per_level() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let Some(g) = gpu() else { eprintln!("no adapter"); return };
        nf_formats::workcount::force(true);
        let secs: u32 = std::env::var("NF_PROBE_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60);
        for level in on_foot(&iso) {
            let run = run(&iso, &level, &g, secs * 240, 256 * 1024, input);
            report(&run);
        }
        nf_formats::workcount::force(false);
    }

    /// Measurement only: the committed 0a/0b walk recordings (whole missions on foot) replayed with
    /// every frame's work counted (`NF_PROBE_TICKS` caps the ticks, default 9000 = 150 s).
    #[test]
    #[ignore]
    fn hitch_census_replays() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let Some(g) = gpu() else { return };
        nf_formats::workcount::force(true);
        let ticks: u32 = std::env::var("NF_PROBE_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(9000);
        for f in ["0700000a_operative.bin", "0700000b_operative.bin"] {
            let path = format!("{}/tests/replays/{f}", env!("CARGO_MANIFEST_DIR"));
            report(&run_replay(&iso, &path, &g, ticks, 256 * 1024));
        }
        nf_formats::workcount::force(false);
    }

    /// Measurement only: what the sound predecode holds per on-foot level (PCM bytes, decode time).
    #[test]
    #[ignore]
    fn hitch_predecode_sizes() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        for level in on_foot(&iso) {
            let Ok(mut scene) = crate::load(&iso, &level, false) else { continue };
            crate::attach_headless_sound(&mut scene);
            let Some(sim) = scene.sim.as_mut() else { continue };
            let Some(snd) = sim.headless_sound.as_mut() else { continue };
            let t0 = std::time::Instant::now();
            snd.0.driver.predecode_now();
            eprintln!("[predecode] {level} {:.1} MB in {:.0} ms", snd.0.driver.cached_pcm_bytes() as f64 / 1048576.0, t0.elapsed().as_secs_f64() * 1000.0);
        }
    }

    pub fn report(run: &Run) {
        for kind in [true, false] {
            let fr: Vec<(usize, &FrameWork)> = run.frames.iter().enumerate().filter(|(_, (t, _))| *t == kind).map(|(i, (_, w))| (i, w)).collect();
            let mb = median(fr.iter().map(|(_, w)| w.alloc_bytes()).collect());
            let mn = median(fr.iter().map(|(_, w)| w.allocs()).collect());
            let mw = median(fr.iter().map(|(_, w)| w.wc("gpu: write_buffer").1).collect());
            eprintln!("[hitch] {} {} frames={} median allocs {} bytes {} gpu-write {}", run.level, if kind { "tick" } else { "draw" }, fr.len(), mn, kb(mb), kb(mw));
            let mut spikes: Vec<(usize, &FrameWork, u64)> = fr.iter().filter(|(i, w)| {
                *i > 0 && (w.alloc_bytes() > 4 * mb + 256 * 1024 || w.allocs() > 3 * mn + 400 || w.wc("gpu: write_buffer").1 > 4 * mw + 256 * 1024
                    || w.wc_like("first use").0 > 0 || w.wc_like("create_").0 > 0 || w.wc("gpu: write_texture").0 > 0)
            }).map(|(i, w)| (*i, *w, w.alloc_bytes())).collect();
            let total = spikes.len();
            spikes.sort_by(|a, b| b.2.cmp(&a.2));
            for (i, w, _) in spikes.iter().take(12) {
                let mut ph: Vec<(usize, u64)> = (0..PHASES).map(|p| (p, w.bytes[p])).filter(|x| x.1 > 0).collect();
                ph.sort_by(|a, b| b.1.cmp(&a.1));
                let ph: Vec<String> = ph.iter().take(3).map(|(p, b)| format!("{}={}", PHASE_NAMES[*p], kb(*b))).collect();
                let wc: Vec<String> = w.wc.iter().filter(|e| e.0 != "gpu: write_buffer").map(|e| format!("{}x{}({})", e.0, e.1, kb(e.2))).collect();
                eprintln!("[hitch]   f{} t{:.2}s allocs {} bytes {} big {}({}) gpu-write {} | {} | {}", i, *i as f32 / 240.0, w.allocs(), kb(w.alloc_bytes()), w.big_n, kb(w.big_bytes),
                    kb(w.wc("gpu: write_buffer").1), ph.join(" "), wc.join(" "));
                for (f, sz, p, s) in run.traces.iter().filter(|t| t.0 as usize == *i).take(4) {
                    eprintln!("[hitch]     f{f} {} in {}: {s}", kb(*sz as u64), PHASE_NAMES[*p]);
                }
            }
            if total > 12 { eprintln!("[hitch]   ... {} more", total - 12); }
        }
    }
}

#[cfg(all(test, not(debug_assertions)))]
mod sites {
    use super::probe::*;

    /// Measurement only: where the 64 KiB-and-up allocations of a level's frames come from, summed
    /// over the run (the steady per-frame ones show as counts near the frame count).
    #[test]
    #[ignore]
    fn hitch_alloc_sites_per_level() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let Some(g) = gpu() else { return };
        let levels = std::env::var("NF_PROBE_LEVELS").unwrap_or("07000008".into());
        let secs: u32 = std::env::var("NF_PROBE_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);
        for l in levels.split(',') {
            let min = std::env::var("NF_PROBE_MIN").ok().and_then(|s| s.parse().ok()).unwrap_or(super::BIG);
            let run = run(&iso, &format!("{}.bin", l.trim()), &g, secs * 240, min, input);
            let mut agg: std::collections::HashMap<(usize, String), (u64, u64)> = Default::default();
            for (_, sz, p, s) in &run.traces { let e = agg.entry((*p, s.clone())).or_default(); e.0 += 1; e.1 += *sz as u64; }
            let mut v: Vec<_> = agg.into_iter().collect();
            v.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
            eprintln!("[sites] {} frames {}", run.level, run.frames.len());
            for ((p, s), (n, b)) in v.iter().take(14) { eprintln!("[sites]   {n:6}x {:8.1} MB in {}: {s}", *b as f64 / 1048576.0, super::PHASE_NAMES[*p]); }
        }
    }
}

/// frame-hitches: the counter proofs. Each spike class the census found is re-run on its level and
/// its counted work asserted down (each assert fails on the code before the fix: the numbers in the
/// messages are the census's before-values).
#[cfg(all(test, not(debug_assertions)))]
mod fixes {
    use super::probe::*;
    use super::*;

    fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

    /// Backtraces of `min` bytes and up in `phase`, over a run's TICK frames (or draw frames).
    fn big_in(run: &Run, phase: usize, min: usize, tick: bool) -> usize {
        run.traces.iter().filter(|t| t.2 == phase && t.1 >= min && run.frames.get(t.0 as usize).is_some_and(|f| f.0 == tick)).count()
    }

    fn median_phase(run: &Run, phase: usize, tick: bool) -> u64 {
        median(run.frames.iter().skip(8).filter(|f| f.0 == tick).map(|f| f.1.bytes[phase]).collect())
    }

    /// The draw half's steady allocations: the snow quads (05: a fresh ~670 KB Vec every frame) and
    /// the draw sort's scratch buffer (0c: ~80 KB every frame) are gone.
    #[test]
    fn the_draw_half_reuses_its_snow_and_sort_buffers() {
        let Some(iso) = iso() else { return };
        let Some(g) = gpu() else { eprintln!("no adapter: skipped"); return };
        let r05 = run(&iso, "07000005.bin", &g, 1200, BIG, |_, _| {});
        let hud = median_phase(&r05, 5, false);
        eprintln!("[fix] 05 draw frames: median hud-phase alloc {hud} B, {} allocs >= 64K in it", big_in(&r05, 5, BIG, false));
        // The HUD overlay's own build stays (~150 KB of small vectors); the snow's ~670 KB is gone.
        assert!(hud < 400 * 1024, "05's draw frames allocate {hud} B a frame in the HUD/snow phase (before: ~740 KB, the snow quads)");
        assert!(big_in(&r05, 5, 512 * 1024, false) <= 2, "the snow quads still allocate fresh every frame on 05");
        let r0c = run(&iso, "0700000c.bin", &g, 1200, BIG, |_, _| {});
        let enc = big_in(&r0c, 8, BIG, false);
        eprintln!("[fix] 0c draw frames: {enc} allocs >= 64K in the encode, median encode alloc {} B", median_phase(&r0c, 8, false));
        // A few first-frame capacity growths (the light memo, the order list) stay; the per-frame one is gone.
        // (The encode's remaining ~75 KB a frame is wgpu's own pass recording, each piece under 64 KB.)
        assert!(enc <= 8, "0c's encode allocates >= 64 KB on {enc} draw frames (before: every frame, the stable sort's scratch)");
    }

    /// The sim half's steady allocations: the copter path list clone (08: ~230 KB every tick) and
    /// the moving-collision rebuild (1b: ~1.4 MB of fresh buffers every tick) are gone.
    #[test]
    fn the_tick_reuses_its_collision_and_copter_buffers() {
        let Some(iso) = iso() else { return };
        let Some(g) = gpu() else { eprintln!("no adapter: skipped"); return };
        let r08 = run(&iso, "07000008.bin", &g, 2400, BIG, |_, _| {});
        let n08 = big_in(&r08, 1, 200 * 1024, true);
        eprintln!("[fix] 08 tick frames: {n08} logic allocs >= 200K over {} ticks", r08.frames.iter().filter(|f| f.0).count());
        assert!(n08 <= 12, "08 allocates >= 200 KB in {n08} ticks of 600 (before: every tick, the copter paths clone)");
        let r1b = run(&iso, "0700001b.bin", &g, 2400, BIG, |_, _| {});
        let n1b = big_in(&r1b, 1, 1024 * 1024, true);
        eprintln!("[fix] 1b tick frames: {n1b} logic allocs >= 1M over {} ticks", r1b.frames.iter().filter(|f| f.0).count());
        assert!(n1b <= 12, "1b allocates >= 1 MB in {n1b} ticks of 600 (before: every tick, set_dynamic's fresh rebuild)");
    }

    /// The level bank's samples and its effects' streams decode before play: 01's first 60 s (the
    /// census: 30+ first-use decodes, up to 0.5 MB of PCM in one frame) run with none from the level
    /// bank, and the sound the driver plays is the same tick for tick (the decode only moved).
    #[test]
    fn the_level_bank_decodes_before_play_and_plays_the_same() {
        let Some(iso) = iso() else { return };
        let run_sound = |pre: bool| {
            let mut scene = crate::load(&iso, "07000001.bin", false).expect("01");
            crate::attach_headless_sound(&mut scene);
            let sim = scene.sim.as_mut().unwrap();
            if pre { sim.headless_sound.as_mut().unwrap().0.driver.predecode_now(); }
            nf_formats::workcount::force(true);
            let _ = nf_formats::workcount::take();
            let mut prints = Vec::new();
            let (mut bank, mut stream, mut bytes) = (0u64, 0u64, 0u64);
            for t in 0..3600u32 {
                let mut input = nf_game::player::Input::default();
                if t >= 1200 { input.forward = 1.0; input.turn = 0.0044 * 4.0; }
                sim.step_bond(input, t >= 1200 && t % 120 < 6, false, false);
                for (name, n, b) in nf_formats::workcount::take() {
                    if name == "snd: bank sample decode (first use)" { bank += n; bytes += b; }
                    if name == "snd: stream decode (first use)" { stream += n; bytes += b; }
                }
                let d = &sim.headless_sound.as_ref().unwrap().0.driver;
                let mut h = 0xcbf2_9ce4_8422_2325u64;
                for &(id, handle) in &d.active {
                    for v in [id as u64, handle as u64, d.played_sample_hash(id).unwrap_or(0)] { h = (h ^ v).wrapping_mul(0x1000_0000_01b3); }
                }
                prints.push(h);
            }
            nf_formats::workcount::force(false);
            (bank, stream, bytes, prints, (sim.drones.rng.x, sim.drones.rng.y))
        };
        let (b0, s0, by0, p0, rng0) = run_sound(false);
        let (b1, s1, by1, p1, rng1) = run_sound(true);
        eprintln!("[fix] 01 60 s: first-use decodes without the predecode: {b0} bank + {s0} stream ({by0} B); with: {b1} + {s1} ({by1} B)");
        assert!(b0 + s0 > 0, "the sister run must see first-use decodes (else the walk plays nothing and proves nothing)");
        assert_eq!(b1, 0, "a level-bank sample still decoded mid-play");
        assert!(s1 < s0.max(1), "the streams the level bank names still decode mid-play ({s1} of {s0})");
        assert_eq!(p0, p1, "the predecode changed what the driver plays");
        assert_eq!(rng0, rng1, "the predecode moved the random stream");
    }
}
