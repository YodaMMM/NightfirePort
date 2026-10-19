//! The launcher (`NightfirePort.exe`): the one thing a player supplies is the disc image.
//!
//! First run it asks for the 007: Nightfire (USA, SLUS-20579) image — the system open dialog, or drop the file on
//! the window — checks it ([`nf_formats::identity`]) and remembers it in `launcher.toml` beside the game's
//! `settings.toml`. Later runs show the remembered disc and a PLAY button (Enter). PLAY starts `nf-view.exe <iso>`
//! (the front end) from the launcher's folder with no console window, logs its output to `nf-view.log` in the config
//! dir, and closes the launcher once the game's window is up (or reports the error if the game stops at once).
//!
//! A separate GUI-subsystem binary rather than a mode of `nf-view.exe`: `nf-view` is a console program (its
//! command-line tools and logs need the console), so double-clicking it would always flash one; and winit allows
//! one event loop per process, so the launcher hands over to a fresh game process instead of reusing its own.
//!
//! The screen is drawn on the CPU (canvas.rs) and shown through a one-texture wgpu blit; `--shot out.png` renders
//! the same frame headless:
//! ```text
//! NightfirePort [<iso>]                       open the launcher (an <iso> argument is checked and remembered)
//! NightfirePort --shot out.png [--state first|ready|error|drop|starting] [--iso <path>] [--scale S] [--t SEC]
//! ```

pub mod canvas;
pub mod config;
pub mod platform;
pub mod ui;
pub mod update;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorIcon, Icon, Window, WindowId};

use canvas::{Canvas, Fonts};
use config::LauncherConfig;
use ui::{Btn, Disc, Ui};

const TITLE: &str = "007: Nightfire \u{2014} NightfirePort";
/// Close anyway if the game never shows a window this long after PLAY.
const START_TIMEOUT: Duration = Duration::from_secs(60);

/// The game's console log (`<config dir>/nf-view.log`).
fn game_log_path() -> Option<PathBuf> { crate::settings::config_dir().map(|d| d.join("nf-view.log")) }

/// Loads the remembered disc (if it still checks out). A remembered image that went missing or changed is
/// reported, not forgotten — it may be on a drive that isn't plugged in.
fn initial_ui(arg_iso: Option<&Path>) -> Ui {
    let mut ui = Ui::default();
    let cfg_path = config::default_path();
    if let Some(p) = arg_iso {
        match Disc::check(p) {
            Ok(d) => { remember(&d.path, cfg_path.as_deref()); ui.disc = Some(d); }
            Err(e) => ui.error = Some(e),
        }
        if ui.disc.is_some() { return ui; }
    }
    let cfg = cfg_path.as_deref().map(LauncherConfig::load).unwrap_or_default();
    if let Some(p) = cfg.iso {
        match Disc::check(&p) {
            Ok(d) => ui.disc = Some(d),
            Err(e) if ui.error.is_none() => ui.error = Some(format!("{}: {e}", p.file_name().unwrap_or_default().to_string_lossy())),
            Err(_) => {}
        }
    }
    ui
}

fn remember(iso: &Path, cfg_path: Option<&Path>) -> Option<String> {
    let path = cfg_path?;
    let iso = std::fs::canonicalize(iso).map(strip_verbatim).unwrap_or_else(|_| iso.to_path_buf());
    LauncherConfig { iso: Some(iso) }.save(path).err()
}

/// `\\?\C:\x` → `C:\x` (canonicalize's verbatim prefix; the game's arg parsing and the dialog prefer the plain form).
fn strip_verbatim(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") { Some(rest) if !rest.starts_with("UNC\\") => PathBuf::from(rest), _ => p }
}

pub fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    if let Some(out) = flag("--shot") { return shot(&args, Path::new(&out)); }
    let arg_iso = args.iter().find(|a| !a.starts_with("--")).map(PathBuf::from);
    let fonts = Fonts::load().context("no usable system font (Bahnschrift, Segoe UI, Arial or DejaVu Sans)")?;
    let mut ui = initial_ui(arg_iso.as_deref());
    // The self-update (update.rs): leftovers of the last swap go, then a release build checks for a newer one in the
    // background; PLAY waits while an update downloads and installs (the user's choice: automatic on launch).
    // Right after an update the previous launcher may still be exiting (its `.old` file still locked): retry briefly.
    if let Some(dir) = update::install_dir() {
        std::thread::spawn(move || for _ in 0..20 { update::cleanup_old(&dir); std::thread::sleep(Duration::from_millis(500)); });
    }
    let update_rx = (update::current_version().is_some() && std::env::var_os("NF_NO_UPDATE").is_none()).then(|| {
        ui.updating = true;
        ui.update_note = Some("Checking for updates\u{2026}".into());
        spawn_update()
    });
    let el = EventLoop::new()?;
    let mut app = App { fonts, ui, gfx: None, canvas: Canvas::new(1, 1), bg: Canvas::new(1, 1), bg_key: (0, 0, 0), rgba: Vec::new(), cursor: (0.0, 0.0), start: Instant::now(), child: None, launched_at: None, shown: false, update_rx, autoplayed: false };
    el.run_app(&mut app)?;
    Ok(())
}

/// The update thread's reports.
enum UpdateMsg {
    Note(String),
    /// Nothing newer (or the check failed: the note says so); PLAY is free.
    Done(Option<String>),
    /// The new files are in place: restart as the new launcher.
    Restart,
}

/// Runs check → download → install on a thread (update.rs).
fn spawn_update() -> std::sync::mpsc::Receiver<UpdateMsg> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let msg = match update::check() {
            Ok(None) => UpdateMsg::Done(None),
            Err(e) => UpdateMsg::Done(Some(format!("Couldn't check for updates ({e})."))),
            Ok(Some(rel)) => {
                let _ = tx.send(UpdateMsg::Note(format!("Downloading update {}\u{2026}", rel.tag)));
                match update::download(&rel).and_then(|stage| {
                    let _ = tx.send(UpdateMsg::Note(format!("Installing update {}\u{2026}", rel.tag)));
                    let dir = update::install_dir().ok_or("the launcher's folder is unknown")?;
                    update::install(&stage, &dir)
                }) {
                    Ok(()) => UpdateMsg::Restart,
                    Err(e) => UpdateMsg::Done(Some(format!("Update {} failed: {e}. Playing the installed version.", rel.tag))),
                }
            }
        };
        let _ = tx.send(msg);
    });
    rx
}

/// Headless render of one launcher frame to a PNG (`--shot`).
fn shot(args: &[String], out: &Path) -> Result<()> {
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let fonts = Fonts::load().context("no usable system font")?;
    let scale: f32 = flag("--scale").and_then(|v| v.parse().ok()).unwrap_or(1.0);
    let state = flag("--state").unwrap_or_else(|| "ready".into());
    let mut ui = Ui { t: flag("--t").and_then(|v| v.parse().ok()).unwrap_or(1.0), ..Ui::default() };
    let iso = flag("--iso").map(PathBuf::from).or_else(|| config::default_path().and_then(|p| LauncherConfig::load(&p).iso));
    match state.as_str() {
        "first" => {}
        "error" => {
            let bad = iso.clone().unwrap_or_else(|| PathBuf::from("missing.iso"));
            ui.error = Some(Disc::check(&bad).err().unwrap_or_else(|| "That disc image is \"SOMETHING_ELSE\", not 007: Nightfire.".into()));
        }
        "drop" => ui.drop_hover = true,
        "ready" | "starting" | "hover" | "updating" => {
            let p = iso.context("--state ready needs --iso <path> (or a remembered disc)")?;
            ui.disc = Some(Disc::check(&p).map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?);
            ui.launching = state == "starting";
            if state == "hover" { ui.hover = Some(Btn::Play); }
            if state == "updating" { ui.updating = true; ui.update_note = Some("Downloading update v0.1.8\u{2026}".into()); }
        }
        other => anyhow::bail!("unknown --state {other} (first|ready|error|drop|starting|hover|updating)"),
    }
    let (w, h) = ((ui::W * scale).round() as u32, (ui::H * scale).round() as u32);
    let mut bg = Canvas::new(w, h);
    ui::paint_background(&mut bg, scale);
    let mut c = Canvas::new(w, h);
    ui::paint(&mut c, &bg, &fonts, &ui, scale);
    let mut rgba = Vec::new();
    c.to_rgba(&mut rgba);
    std::fs::write(out, nf_formats::png::encode_rgba(w, h, &rgba)).with_context(|| format!("write {}", out.display()))?;
    eprintln!("wrote {} ({w}x{h}, state {state})", out.display());
    Ok(())
}

struct Gfx {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    tex: Option<(wgpu::Texture, wgpu::BindGroup, u32, u32)>,
}

const BLIT: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
struct VO { @builtin(position) p: vec4<f32> };
@vertex fn vs(@builtin(vertex_index) i: u32) -> VO {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    var o: VO;
    o.p = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return o;
}
@fragment fn fs(v: VO) -> @location(0) vec4<f32> {
    let d = vec2<i32>(textureDimensions(tex)) - vec2<i32>(1, 1);
    return textureLoad(tex, min(vec2<i32>(v.p.xy), d), 0);
}
"#;

impl Gfx {
    fn new(window: Arc<Window>) -> Result<Gfx> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), power_preference: wgpu::PowerPreference::LowPower, ..Default::default() }))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
        device.on_uncaptured_error(Arc::new(|e| eprintln!("launcher GPU error: {e}")));
        let size = window.inner_size();
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).context("surface config")?;
        // The canvas bytes are display values (no gamma step), like the game's own non-sRGB surface.
        config.format = config.format.remove_srgb_suffix();
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("launcher"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("launcher"), source: wgpu::ShaderSource::Wgsl(BLIT.into()) });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("launcher"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None,
            cache: None,
        });
        Ok(Gfx { window, surface, config, device, queue, pipeline, layout, tex: None })
    }

    fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 { return; }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
    }

    fn present(&mut self, rgba: &[u8], w: u32, h: u32) {
        if !matches!(&self.tex, Some((_, _, tw, th)) if (*tw, *th) == (w, h)) {
            let t = self.device.create_texture(&wgpu::TextureDescriptor { label: Some("launcher frame"), size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[] });
            let view = t.create_view(&Default::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &self.layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }] });
            self.tex = Some((t, bg, w, h));
        }
        let Some((tex, bg, _, _)) = &self.tex else { return };
        self.queue.write_texture(wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, rgba, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) }, wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => { self.surface.configure(&self.device, &self.config); return; }
            _ => return,
        };
        let view = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("launcher"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

struct App {
    fonts: Fonts,
    ui: Ui,
    gfx: Option<Gfx>,
    canvas: Canvas,
    bg: Canvas,
    /// (w, h, scale bits) the cached backdrop was painted for.
    bg_key: (u32, u32, u32),
    rgba: Vec<u8>,
    cursor: (f32, f32),
    start: Instant,
    child: Option<std::process::Child>,
    launched_at: Option<Instant>,
    shown: bool,
    update_rx: Option<std::sync::mpsc::Receiver<UpdateMsg>>,
    /// NF_LAUNCHER_AUTOPLAY pressed PLAY already.
    autoplayed: bool,
}

impl App {
    fn scale(&self) -> f32 { self.gfx.as_ref().map_or(1.0, |g| g.window.scale_factor() as f32) }

    fn hwnd(&self) -> Option<*mut std::ffi::c_void> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let g = self.gfx.as_ref()?;
        match g.window.window_handle().ok()?.as_raw() {
            RawWindowHandle::Win32(h) => Some(h.hwnd.get() as *mut std::ffi::c_void),
            _ => None,
        }
    }

    /// Checks a picked/dropped image; a good one replaces the disc and is remembered.
    fn take_iso(&mut self, p: &Path) {
        match Disc::check(p) {
            Ok(d) => {
                self.ui.error = remember(&d.path, config::default_path().as_deref()).map(|e| format!("Couldn't save the launcher settings: {e}"));
                self.ui.disc = Some(d);
            }
            Err(e) => self.ui.error = Some(e),
        }
    }

    fn select(&mut self) {
        let start = self.ui.disc.as_ref().and_then(|d| d.path.parent().map(Path::to_path_buf));
        if let Some(p) = platform::pick_iso(self.hwnd(), start.as_deref()) { self.take_iso(&p); }
    }

    fn play(&mut self) {
        if self.ui.updating { return; }
        let Some(d) = &self.ui.disc else { return self.select() };
        let iso = d.path.clone();
        let exe = match platform::game_exe() { Ok(e) => e, Err(e) => { self.ui.error = Some(e); return; } };
        match platform::launch(&exe, &iso, game_log_path().as_deref()) {
            Ok(child) => { self.child = Some(child); self.launched_at = Some(Instant::now()); self.ui.launching = true; self.ui.error = None; self.ui.hover = None; }
            Err(e) => self.ui.error = Some(e),
        }
    }

    fn act(&mut self, el: &ActiveEventLoop, b: Btn) {
        match b {
            Btn::Play => self.play(),
            Btn::Select | Btn::Change => self.select(),
            Btn::Quit => el.exit(),
        }
        if let Some(g) = &self.gfx { g.window.request_redraw(); }
    }

    fn update_hover(&mut self) {
        let s = self.scale();
        let h = self.ui.hit(self.cursor.0 / s, self.cursor.1 / s);
        if h != self.ui.hover {
            self.ui.hover = h;
            if let Some(g) = &self.gfx { g.window.set_cursor(if h.is_some() { CursorIcon::Pointer } else { CursorIcon::Default }); }
        }
    }

    fn redraw(&mut self) {
        let Some(g) = self.gfx.as_mut() else { return };
        let (w, h) = (g.config.width, g.config.height);
        let s = g.window.scale_factor() as f32;
        if self.bg_key != (w, h, s.to_bits()) {
            self.bg.resize(w, h);
            ui::paint_background(&mut self.bg, s);
            self.bg_key = (w, h, s.to_bits());
        }
        self.canvas.resize(w, h);
        self.ui.t = self.start.elapsed().as_secs_f32();
        ui::paint(&mut self.canvas, &self.bg, &self.fonts, &self.ui, s);
        self.canvas.to_rgba(&mut self.rgba);
        g.present(&self.rgba, w, h);
        if !self.shown { g.window.set_visible(true); g.window.focus_window(); self.shown = true; }
    }

    /// The update thread's progress; a finished install restarts the launcher as the new version.
    fn poll_update(&mut self, el: &ActiveEventLoop) {
        let Some(rx) = &self.update_rx else { return };
        while let Ok(m) = rx.try_recv() {
            match m {
                UpdateMsg::Note(n) => self.ui.update_note = Some(n),
                UpdateMsg::Done(note) => { self.ui.updating = false; self.ui.update_note = note; self.update_rx = None; return; }
                UpdateMsg::Restart => {
                    let args: Vec<String> = std::env::args().skip(1).collect();
                    match std::env::current_exe().and_then(|exe| std::process::Command::new(exe).args(&args).spawn()) {
                        Ok(_) => el.exit(),
                        Err(e) => { self.ui.updating = false; self.ui.update_note = Some(format!("Updated \u{2014} restart the launcher to use it ({e}).")); }
                    }
                    self.update_rx = None;
                    return;
                }
            }
        }
    }

    /// After PLAY: close once the game's window is up; report it if the game stops first.
    fn poll_child(&mut self, el: &ActiveEventLoop) {
        let (Some(child), Some(at)) = (self.child.as_mut(), self.launched_at) else { return };
        match child.try_wait() {
            Ok(Some(status)) => {
                let tail = game_log_path().and_then(|p| platform::log_tail(&p));
                self.ui.launching = false;
                self.child = None;
                if status.success() { el.exit(); return; }
                let log = game_log_path().map(|p| p.display().to_string()).unwrap_or_default();
                self.ui.error = Some(match tail {
                    Some(t) => format!("The game stopped ({status}): {t}"),
                    None => format!("The game stopped ({status}). See {log}."),
                });
            }
            Ok(None) => {
                let up = platform::game_window_up(child.id());
                let waited = at.elapsed();
                if up == Some(true) || (up.is_none() && waited > Duration::from_secs(3)) || waited > START_TIMEOUT { el.exit(); }
            }
            Err(e) => { self.ui.error = Some(format!("The game couldn't be watched: {e}")); self.ui.launching = false; self.child = None; }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gfx.is_some() { return; }
        let (icon, n) = ui::icon_rgba();
        let mut attrs = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(LogicalSize::new(ui::W as f64, ui::H as f64))
            .with_resizable(false)
            .with_visible(false)
            .with_window_icon(Icon::from_rgba(icon, n, n).ok());
        if let Some(mon) = el.primary_monitor() {
            let (mw, mh) = (mon.size().width as f64, mon.size().height as f64);
            let sf = mon.scale_factor();
            let (ww, wh) = (ui::W as f64 * sf, ui::H as f64 * sf);
            attrs = attrs.with_position(PhysicalPosition::new(mon.position().x + ((mw - ww) / 2.0).max(0.0) as i32, mon.position().y + ((mh - wh) / 2.5).max(0.0) as i32));
        }
        let window = match el.create_window(attrs) { Ok(w) => Arc::new(w), Err(e) => { eprintln!("launcher window: {e}"); el.exit(); return; } };
        match Gfx::new(window.clone()) {
            Ok(g) => self.gfx = Some(g),
            Err(e) => { eprintln!("launcher GPU: {e:#}"); el.exit(); return; }
        }
        // Draw the first frame now and then show the window (a hidden window gets no RedrawRequested on Windows,
        // and showing it before a frame flashes white).
        self.redraw();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(sz) => { if let Some(g) = self.gfx.as_mut() { g.resize(sz.width, sz.height); } }
            WindowEvent::ScaleFactorChanged { .. } => { if let Some(g) = &self.gfx { g.window.request_redraw(); } }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => { self.cursor = (position.x as f32, position.y as f32); self.update_hover(); }
            WindowEvent::CursorLeft { .. } => { self.cursor = (-1.0, -1.0); self.update_hover(); self.ui.pressed = None; }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.update_hover();
                match state {
                    ElementState::Pressed => self.ui.pressed = self.ui.hover,
                    ElementState::Released => {
                        let (p, h) = (self.ui.pressed.take(), self.ui.hover);
                        if let (Some(p), Some(h)) = (p, h) { if p == h { self.act(el, h); } }
                    }
                }
            }
            WindowEvent::HoveredFile(_) => self.ui.drop_hover = true,
            WindowEvent::HoveredFileCancelled => self.ui.drop_hover = false,
            WindowEvent::DroppedFile(p) => {
                self.ui.drop_hover = false;
                if !self.ui.launching { self.take_iso(&p); }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && !event.repeat && !self.ui.launching => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space) => { let b = self.ui.default_action(); self.act(el, b); }
                    PhysicalKey::Code(KeyCode::KeyO) => self.act(el, Btn::Change),
                    PhysicalKey::Code(KeyCode::Escape) => el.exit(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.poll_update(el);
        // Test aid (NF_LAUNCHER_AUTOPLAY=1): press PLAY once any update has finished — the end-to-end release check
        // drives install → update → restart → game start with no clicks.
        if !self.ui.updating && self.child.is_none() && self.ui.disc.is_some() && self.ui.error.is_none()
            && std::env::var_os("NF_LAUNCHER_AUTOPLAY").is_some() && !std::mem::replace(&mut self.autoplayed, true) {
            self.play();
        }
        self.poll_child(el);
        let Some(g) = &self.gfx else { return };
        if g.window.is_minimized().unwrap_or(false) {
            el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(250)));
            return;
        }
        // ~60 fps animation (the barrel's turn, the READY light, the button sheen); vsync paces the present.
        g.window.request_redraw();
        el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_follow_the_state() {
        let mut ui = Ui::default();
        assert_eq!(ui.default_action(), Btn::Select);
        let (_, r) = ui.buttons()[0];
        assert_eq!(ui.hit(r.x + 5.0, r.y + 5.0), Some(Btn::Select));
        assert_eq!(ui.hit(5.0, 5.0), None);
        assert!(!ui.buttons().iter().any(|(b, _)| *b == Btn::Change), "no Change before a disc");
        ui.disc = Some(Disc { path: PathBuf::from("x.iso"), bytes: 2_439_315_456, info: nf_formats::identity::DiscInfo { volume_id: "NIGHTFIRE".into(), serial: "SLUS-20579".into(), vmode: "NTSC".into(), version: "1.00".into(), sectors: 1_191_072 } });
        assert_eq!(ui.default_action(), Btn::Play);
        assert_eq!(ui.hit(r.x + 5.0, r.y + 5.0), Some(Btn::Play));
        assert!(ui.buttons().iter().any(|(b, _)| *b == Btn::Change));
        assert_eq!(ui.disc.as_ref().unwrap().details(), "SLUS-20579  \u{b7}  NTSC-U  \u{b7}  v1.00  \u{b7}  2.27 GB");
        ui.launching = true;
        assert_eq!(ui.hit(r.x + 5.0, r.y + 5.0), None, "no clicks while the game starts");
    }

    #[test]
    fn strip_verbatim_keeps_plain_paths() {
        assert_eq!(strip_verbatim(PathBuf::from(r"\\?\C:\PS2\a.iso")), PathBuf::from(r"C:\PS2\a.iso"));
        assert_eq!(strip_verbatim(PathBuf::from(r"\\?\UNC\srv\share\a.iso")), PathBuf::from(r"\\?\UNC\srv\share\a.iso"));
        assert_eq!(strip_verbatim(PathBuf::from("/home/a.iso")), PathBuf::from("/home/a.iso"));
    }

    /// The real disc through the launcher's own path (opt-in NF_ISO): accepted, remembered, reloaded as ready.
    /// Sister: a file that isn't the disc is rejected with the friendly message and not remembered.
    #[test]
    fn real_disc_is_checked_and_remembered() {
        let Some(iso) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
        let dir = std::env::temp_dir().join(format!("nf-launcher-real-{}", std::process::id()));
        let cfg = dir.join("launcher.toml");
        let d = Disc::check(Path::new(&iso)).expect("the user's disc passes");
        assert_eq!(d.info.serial, "SLUS-20579");
        assert_eq!(d.bytes, 2_439_315_456);
        assert_eq!(remember(&d.path, Some(&cfg)), None);
        let back = LauncherConfig::load(&cfg).iso.expect("remembered");
        assert_eq!(Disc::check(&back).map(|d| d.info), Ok(d.info.clone()));
        let junk = dir.join("not-a-disc.iso");
        std::fs::write(&junk, vec![0u8; 40 * 2048]).unwrap();
        let e = Disc::check(&junk).unwrap_err();
        assert!(e.contains("isn't a disc image"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
