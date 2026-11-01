//! A small in-process sampling profiler for the frame path (Windows x64 only): a helper thread
//! suspends the measured thread every ~50 µs, unwinds its stack (RtlVirtualUnwind) and records
//! the return addresses; dbghelp names them from the PDB. The window's per-tick chain runs N
//! times per level — tick, palette build, visibility, upload, lights, render, finish, submit,
//! wait — with each phase tagged, and the top functions per phase are printed (self and
//! inclusive). Measurement only; asserts nothing. `#[ignore]`; `NF_PROBE_LEVELS` picks levels.

#![cfg(all(windows, target_arch = "x86_64"))]

use nf_view::scene::ViewOpts;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

type H = isize;
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentThread() -> H;
    fn GetCurrentProcess() -> H;
    fn DuplicateHandle(sp: H, s: H, tp: H, t: *mut H, access: u32, inherit: i32, opts: u32) -> i32;
    fn SuspendThread(h: H) -> u32;
    fn ResumeThread(h: H) -> u32;
    fn GetThreadContext(h: H, ctx: *mut u8) -> i32;
    fn RtlLookupFunctionEntry(pc: u64, base: *mut u64, hist: *mut u8) -> *mut u8;
    fn RtlVirtualUnwind(ht: u32, base: u64, pc: u64, fe: *mut u8, ctx: *mut u8, hd: *mut *mut u8, ef: *mut u64, cp: *mut u8) -> *mut u8;
}
#[link(name = "dbghelp")]
extern "system" {
    fn SymInitialize(p: H, path: *const u8, invade: i32) -> i32;
    fn SymFromAddr(p: H, addr: u64, disp: *mut u64, sym: *mut u8) -> i32;
    fn SymSetOptions(o: u32) -> u32;
    fn SymGetLineFromAddr64(p: H, addr: u64, disp: *mut u32, line: *mut u8) -> i32;
}

fn line_of(a: u64) -> String {
    let mut buf = [0u8; 40];
    buf[0..4].copy_from_slice(&40u32.to_le_bytes());
    let mut disp = 0u32;
    unsafe {
        if SymGetLineFromAddr64(GetCurrentProcess(), a, &mut disp, buf.as_mut_ptr()) == 0 { return "?".into(); }
        let ln = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let fp = u64::from_le_bytes(buf[24..32].try_into().unwrap()) as *const i8;
        let f = std::ffi::CStr::from_ptr(fp).to_string_lossy();
        let f = f.rsplit(['\\', '/']).next().unwrap_or("").to_string();
        format!("{f}:{ln}")
    }
}
#[link(name = "kernel32")]
extern "system" {
    fn GetLastError() -> u32;
}

#[repr(C, align(16))]
struct Ctx([u8; 1232]);
const CONTEXT_FULL: u32 = 0x10_000B;
fn rd(c: &Ctx, off: usize) -> u64 { u64::from_le_bytes(c.0[off..off + 8].try_into().unwrap()) }
fn wr(c: &mut Ctx, off: usize, v: u64) { c.0[off..off + 8].copy_from_slice(&v.to_le_bytes()); }
const RSP: usize = 0x98;
const RIP: usize = 0xF8;
const DEPTH: usize = 24;

struct Sampler { stop: Arc<AtomicBool>, tag: Arc<AtomicU32>, join: Option<std::thread::JoinHandle<Vec<(u32, [u64; DEPTH])>>> }

impl Sampler {
    fn start() -> Sampler {
        let mut h: H = 0;
        unsafe { DuplicateHandle(GetCurrentProcess(), GetCurrentThread(), GetCurrentProcess(), &mut h, 0, 0, 2) };
        let stop = Arc::new(AtomicBool::new(false));
        let tag = Arc::new(AtomicU32::new(0));
        let (s2, t2) = (stop.clone(), tag.clone());
        let join = std::thread::spawn(move || {
            let mut out: Vec<(u32, [u64; DEPTH])> = Vec::with_capacity(4_000_000);
            let mut ctx = Box::new(Ctx([0; 1232]));
            while !s2.load(Ordering::Relaxed) {
                let t0 = std::time::Instant::now();
                while t0.elapsed().as_micros() < 50 { std::hint::spin_loop(); }
                let tg = t2.load(Ordering::Relaxed);
                if tg == 0 || out.len() == out.capacity() { continue; }
                unsafe {
                    if SuspendThread(h) == u32::MAX { continue; }
                    ctx.0.fill(0);
                    ctx.0[0x30..0x34].copy_from_slice(&CONTEXT_FULL.to_le_bytes());
                    let mut st = [0u64; DEPTH];
                    if GetThreadContext(h, ctx.0.as_mut_ptr()) != 0 {
                        // The tag is read again under suspension (the phase the thread is in now).
                        let tg = t2.load(Ordering::Relaxed);
                        for d in 0..DEPTH {
                            let pc = rd(&ctx, RIP);
                            if pc == 0 { break; }
                            st[d] = pc;
                            let mut base = 0u64;
                            let fe = RtlLookupFunctionEntry(pc, &mut base, std::ptr::null_mut());
                            if fe.is_null() {
                                let sp = rd(&ctx, RSP);
                                if sp == 0 { break; }
                                let ret = *(sp as *const u64);
                                wr(&mut ctx, RIP, ret); wr(&mut ctx, RSP, sp + 8);
                            } else {
                                let mut hd = std::ptr::null_mut();
                                let mut ef = 0u64;
                                RtlVirtualUnwind(0, base, pc, fe, ctx.0.as_mut_ptr(), &mut hd, &mut ef, std::ptr::null_mut());
                            }
                        }
                        if tg != 0 { out.push((tg, st)); }
                    }
                    ResumeThread(h);
                }
            }
            out
        });
        Sampler { stop, tag, join: Some(join) }
    }
    fn set(&self, t: u32) { self.tag.store(t, Ordering::Relaxed); }
    fn finish(mut self) -> Vec<(u32, [u64; DEPTH])> { self.stop.store(true, Ordering::Relaxed); self.join.take().unwrap().join().unwrap() }
}

fn sym(cache: &mut std::collections::HashMap<u64, String>, a: u64) -> String {
    if let Some(s) = cache.get(&a) { return s.clone(); }
    let mut buf = vec![0u8; 88 + 1024];
    buf[0..4].copy_from_slice(&88u32.to_le_bytes());
    buf[80..84].copy_from_slice(&1000u32.to_le_bytes());
    let mut disp = 0u64;
    let name = unsafe {
        if SymFromAddr(GetCurrentProcess(), a, &mut disp, buf.as_mut_ptr()) != 0 {
            let n = u32::from_le_bytes(buf[76..80].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&buf[84..84 + n.min(1000)]).into_owned()
        } else { format!("?{a:x} e{}", GetLastError()) }
    };
    // Trim generic noise: keep the path's last three segments.
    let short = { let s = name.split('<').next().unwrap_or(&name).to_string(); let parts: Vec<&str> = s.split("::").collect(); parts[parts.len().saturating_sub(3)..].join("::") };
    cache.insert(a, short.clone());
    short
}

fn report(level: &str, samples: &[(u32, [u64; DEPTH])], names: &[&str]) {
    let mut cache = std::collections::HashMap::new();
    for (ti, tn) in names.iter().enumerate() {
        let tag = ti as u32 + 1;
        let s: Vec<&[u64; DEPTH]> = samples.iter().filter(|x| x.0 == tag).map(|x| &x.1).collect();
        if s.is_empty() { continue; }
        let mut selfc: std::collections::HashMap<String, u32> = Default::default();
        let mut incl: std::collections::HashMap<String, u32> = Default::default();
        for st in &s {
            let mut seen = std::collections::HashSet::new();
            for (d, &a) in st.iter().enumerate() {
                if a == 0 { break; }
                let n = sym(&mut cache, a);
                if d == 0 { *selfc.entry(n.clone()).or_default() += 1; }
                if seen.insert(n.clone()) { *incl.entry(n).or_default() += 1; }
            }
        }
        let top = |m: &std::collections::HashMap<String, u32>, k: usize| { let mut v: Vec<_> = m.iter().collect(); v.sort_by(|a, b| b.1.cmp(a.1)); v.into_iter().take(k).map(|(n, c)| format!("{:5.1}% {n}", 100.0 * *c as f64 / s.len() as f64)).collect::<Vec<_>>() };
        eprintln!("[sample] {level} {tn}: {} samples", s.len());
        for l in top(&selfc, 25) { eprintln!("[sample]   self {l}"); }
        for l in top(&incl, 45) { eprintln!("[sample]   incl {l}"); }
        let mut lines: std::collections::HashMap<String, u32> = Default::default();
        for st in &s { if st[0] != 0 { *lines.entry(line_of(st[0])).or_default() += 1; } }
        for l in top(&lines, 30) { eprintln!("[sample]   line {l}"); }
        // `NF_SAMPLE_LEAF=<substring>`: the call chains (source lines) under the leaf functions
        // whose name contains it.
        if let Ok(leaf) = std::env::var("NF_SAMPLE_LEAF") {
            let mut chains: std::collections::HashMap<String, u32> = Default::default();
            for st in &s {
                if st[0] == 0 || !sym(&mut cache, st[0]).contains(leaf.as_str()) { continue; }
                let c: Vec<String> = st[1..].iter().take_while(|&&a| a != 0).map(|&a| line_of(a)).filter(|l| !l.starts_with('?')).take(5).collect();
                *chains.entry(c.join(" < ")).or_default() += 1;
            }
            for l in top(&chains, 15) { eprintln!("[sample]   leaf {l}"); }
        }
    }
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

#[test]
#[ignore]
#[cfg(not(debug_assertions))]
fn sample_the_tick_frame() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { return };
    let dir = std::env::current_exe().unwrap().parent().unwrap().to_string_lossy().into_owned() + " ";
    unsafe { SymSetOptions(0x2 | 0x4 | 0x10); let ok = SymInitialize(GetCurrentProcess(), dir.as_ptr(), 1); eprintln!("[sample] SymInitialize {ok} {dir}"); }
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    let want = wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let desc = wgpu::DeviceDescriptor { required_features: adapter.features() & want, required_limits: adapter.limits(), ..Default::default() };
    let (device, queue) = pollster::block_on(adapter.request_device(&desc)).expect("device");
    let levels: Vec<String> = std::env::var("NF_PROBE_LEVELS").unwrap_or("07000007,0700000c,07000016".into()).split(',').map(|s| format!("{}.bin", s.trim())).collect();
    let iters: u32 = std::env::var("NF_SAMPLE_ITERS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let (w, h) = (1280u32, 720u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] });
    let view = target.create_view(&Default::default());
    let depth = nf_view::gpu::depth_view(&device, w, h);
    let names = ["tick", "palette", "vis", "upload", "lights+hud", "render", "finish", "submit+wait"];
    for name in &levels {
        let Ok(mut scene) = nf_view::load(&iso_path, name, false) else { continue };
        if let Some(sim) = scene.sim.as_mut() { for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); } }
        let (pos, yaw0) = scene.start.clone().unwrap_or(([0.0; 3], 0.0));
        let mut r = nf_view::gpu::Renderer::new(&device, &queue, format, &scene, false, false);
        let (layout, _) = scene.skin_layout();
        let (mut p, mut c) = (Vec::new(), Vec::new());
        let sampler = Sampler::start();
        for it in 0..iters {
            let yaw = yaw0 + (it / 150) as f32 * std::f32::consts::FRAC_PI_2;
            let eye = [pos[0], pos[1] + 1.6, pos[2]];
            let vp = view_proj(eye, yaw, 0.0, 16.0 / 9.0);
            let mut opts = ViewOpts::default();
            let fw = glam::Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
            let s = fw.cross(glam::Vec3::Y).normalize();
            opts.right = s.to_array(); opts.up = s.cross(fw).to_array();
            sampler.set(1);
            if let Some(sim) = scene.sim.as_mut() {
                let mut i = nf_game::player::Input::default();
                match (it / 60) % 3 { 0 => i.forward = 1.0, 1 => i.turn = 0.5, _ => i.strafe = 1.0 }
                sim.step_bond(i, false, false, false);
            }
            sampler.set(2);
            scene.skin_palette_into(&layout, None, it as f32, opts, None, &mut p, &mut c);
            sampler.set(3);
            let world = scene.sim.as_ref().map(|s| &s.world);
            let doors = scene.sim.as_ref().map(|s| s.drones.doors.closed_flags());
            let _ = r.cel_visibility(eye, vp, world, doors.as_deref());
            sampler.set(4);
            r.char_light_centre = scene.char_light_centres();
            r.upload_skin(&queue, &p, &c);
            sampler.set(5);
            r.lights = scene.frame_lights();
            let hf = nf_view::hud::overlay(&scene, 16.0 / 9.0);
            sampler.set(6);
            let e = r.render(&device, &queue, &view, &depth, vp, vp, eye, it, &[], &hf, None, None);
            sampler.set(7);
            let cb = e.finish();
            sampler.set(8);
            queue.submit([cb]);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            sampler.set(0);
        }
        let samples = sampler.finish();
        report(name, &samples, &names);
        drop(r); let _ = queue.submit([]); device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}
