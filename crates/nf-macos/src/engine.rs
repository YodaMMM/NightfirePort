//! Stateful macOS Metal engine wrapping `nf-view`, `nf-game`, `nf-audio`, and
//! headless `wgpu` rendering with full parity across FrontEnd, On-Foot,
//! Multiplayer, and Driving modes.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use glam::Vec3;
use crate::inspect::{BenchReport, PerfStats};
use nf_view::driving::{
    driving_viv_for_level, load_driving_scene_at, DrivingState, BOOT_DIFFICULTY,
};
use nf_view::gpu::{self, Renderer};
use nf_view::menu::{self, Action as MenuAction, FrontEnd};
use nf_view::scene::{Scene, ViewOpts};
use nf_view::{
    billboard_axes, build_sound, cam_axes, cutscene_frame, dead_destroy_centres, destroy_tints,
    device, gpu_instance_headless, load_front_end, load_mp_carry, load_with_carry_difficulty,
    overlay_for, particles_for, shot_hud, snow_for, sound_tick, view_fov, wreck_draw_centres,
    Camera, GAME_FOV,
};

pub const BTN_FIRE: i32 = 1 << 0;
pub const BTN_AIM: i32 = 1 << 1;
pub const BTN_RELOAD: i32 = 1 << 2;
pub const BTN_JUMP: i32 = 1 << 3;
pub const BTN_CROUCH: i32 = 1 << 4;
pub const BTN_USE: i32 = 1 << 5;
pub const BTN_WEAP_NEXT: i32 = 1 << 6;
pub const BTN_WEAP_PREV: i32 = 1 << 7;
pub const BTN_PAUSE: i32 = 1 << 8;

/// Normalized input state mirrored from `nf-android`'s `NativeBridge`.
#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub move_x: f32,
    pub move_y: f32,
    pub look_dx: f32,
    pub look_dy: f32,
    pub buttons: i32,
}

/// Optional parameters when loading a level in [`MacEngine`].
#[derive(Clone, Copy, Debug)]
pub struct LoadOptions {
    /// Game difficulty (`GameState+0x28`, 1..=4; default [`BOOT_DIFFICULTY`] = 2).
    pub difficulty: u32,
    /// Optional multiplayer match `(humans, bots)` for MP arenas.
    pub mp_match: Option<(u32, u32)>,
    /// Start on a specific FrontEnd page when loading `07000048.bin`.
    pub fe_page: Option<menu::Page>,
    /// Unlock all missions in FrontEnd mode.
    pub unlock_all: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            difficulty: BOOT_DIFFICULTY,
            mp_match: None,
            fe_page: Some(menu::Page::Main),
            unlock_all: true,
        }
    }
}

/// Snapshot returned after stepping one frame.
#[derive(Clone, Debug)]
pub struct FrameTelemetry {
    pub archive: String,
    pub tick: u32,
    pub mode: String,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub speed: Option<f32>,
    pub player_drives: Option<bool>,
    pub mission_state: Option<String>,
    pub health: Option<f32>,
    pub weapon: Option<u8>,
    pub cels: usize,
    pub textures: usize,
    pub pending_level: Option<String>,
}

impl FrameTelemetry {
    /// Formats this snapshot as a compact JSON object (no external serializer needed).
    pub fn to_json(&self) -> String {
        let mut out = format!(
            "{{\"archive\":{},\"tick\":{},\"mode\":{},\"pos\":[{:.3},{:.3},{:.3}],\"yaw\":{:.4},\"pitch\":{:.4},\"cels\":{},\"textures\":{}",
            json_escape(&self.archive),
            self.tick,
            json_escape(&self.mode),
            self.pos[0],
            self.pos[1],
            self.pos[2],
            self.yaw,
            self.pitch,
            self.cels,
            self.textures,
        );
        if let Some(s) = self.speed {
            out.push_str(&format!(",\"speed\":{s:.3}"));
        }
        if let Some(pd) = self.player_drives {
            out.push_str(&format!(",\"player_drives\":{pd}"));
        }
        if let Some(ms) = &self.mission_state {
            out.push_str(&format!(",\"mission_state\":{}", json_escape(ms)));
        }
        if let Some(h) = self.health {
            out.push_str(&format!(",\"health\":{h:.2}"));
        }
        if let Some(w) = self.weapon {
            out.push_str(&format!(",\"weapon\":{w}"));
        }
        if let Some(p) = &self.pending_level {
            out.push_str(&format!(",\"pending_level\":{}", json_escape(p)));
        }
        out.push('}');
        out
    }
}

/// Pixel stats returned after rendering a Metal frame to PNG.
#[derive(Clone, Copy, Debug)]
pub struct RenderStats {
    pub width: u32,
    pub height: u32,
    pub mean_luma: f64,
    pub std_luma: f64,
}

struct MetalContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    format: wgpu::TextureFormat,
}

impl MetalContext {
    fn new(scene: &Scene) -> Result<Self> {
        let instance = gpu_instance_headless();
        let (_adapter, device, queue) = pollster::block_on(device(&instance, None))?;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let renderer = Renderer::new(&device, &queue, format, scene, false, false);
        Ok(Self {
            device,
            queue,
            renderer,
            format,
        })
    }

    fn rebuild(&mut self, scene: &Scene) {
        self.renderer = Renderer::new(&self.device, &self.queue, self.format, scene, false, false);
    }
}

/// Stateful macOS harness engine supporting FrontEnd, On-Foot, Multiplayer, and Driving levels.
pub struct MacEngine {
    pub data_path: PathBuf,
    pub archive: String,
    pub scene: Scene,
    pub driving: Option<DrivingState>,
    pub cam: Camera,
    pub fe: Option<FrontEnd>,
    pub lang: Option<nf_formats::hud::Lang>,
    pub sound: Option<nf_audio::sfx::Sound>,
    pub enable_audio: bool,
    pub auto_transition: bool,
    pub difficulty: u32,
    pub ticks: u32,
    pub input: InputState,
    pub perf: PerfStats,
    prev_buttons: i32,
    pending_menu_actions: Vec<MenuAction>,
    pending_level: Option<String>,
    driving_end_ticks: Option<u32>,
    metal: Option<MetalContext>,
}

impl MacEngine {
    /// Initializes the macOS engine against an ISO image or carved asset directory (audio off).
    pub fn init(data_path: impl AsRef<Path>) -> Result<Self> {
        Self::init_with_audio(data_path, false)
    }

    /// Initializes the macOS engine with optional live CoreAudio output.
    pub fn init_with_audio(data_path: impl AsRef<Path>, enable_audio: bool) -> Result<Self> {
        let data_path = data_path.as_ref().to_path_buf();
        if !data_path.exists() {
            return Err(anyhow!("data path does not exist: {}", data_path.display()));
        }
        let data_str = data_path.to_string_lossy().to_string();
        let lang = load_lang(&data_str);
        let opts = LoadOptions::default();
        let (mut scene, mut driving, fe, cam, archive) =
            load_scene_for(&data_str, "07000048.bin", opts)?;
        let sound = if enable_audio {
            if let Some(d) = driving.as_mut() {
                d.start_audio();
            }
            build_sound(&mut scene)
        } else {
            None
        };
        Ok(Self {
            data_path,
            archive,
            scene,
            driving,
            cam,
            fe,
            lang,
            sound,
            enable_audio,
            auto_transition: false,
            difficulty: opts.difficulty,
            ticks: 0,
            input: InputState::default(),
            perf: PerfStats::default(),
            prev_buttons: 0,
            pending_menu_actions: Vec::new(),
            pending_level: None,
            driving_end_ticks: None,
            metal: None,
        })
    }

    /// Loads an on-foot archive (`0700xxxx.bin`), front-end (`07000048.bin`),
    /// or driving mission (`09000001.bin`..`09000008.bin` / `MIS01`..`RACE`) with default options.
    pub fn load_level(&mut self, archive: &str) -> Result<()> {
        self.load_level_with(
            archive,
            LoadOptions {
                difficulty: self.difficulty,
                ..Default::default()
            },
        )
    }

    /// Loads a level archive with explicit [`LoadOptions`] (difficulty, MP match settings, FE page).
    pub fn load_level_with(&mut self, archive: &str, opts: LoadOptions) -> Result<()> {
        if let Some(s) = self.sound.as_mut() {
            s.reset();
        }
        let data_str = self.data_path.to_string_lossy().to_string();
        let (mut scene, mut driving, fe, cam, resolved) = load_scene_for(&data_str, archive, opts)?;
        let sound = if self.enable_audio {
            if let Some(d) = driving.as_mut() {
                d.start_audio();
            }
            build_sound(&mut scene)
        } else {
            None
        };
        self.scene = scene;
        self.driving = driving;
        self.fe = fe;
        self.cam = cam;
        self.sound = sound;
        self.archive = resolved;
        self.difficulty = opts.difficulty;
        self.ticks = 0;
        self.prev_buttons = 0;
        self.pending_menu_actions.clear();
        self.pending_level = None;
        self.driving_end_ticks = None;
        if let Some(m) = self.metal.as_mut() {
            m.rebuild(&self.scene);
        }
        Ok(())
    }

    /// Updates the current virtual controller / keyboard input state.
    pub fn set_input(&mut self, input: InputState) {
        self.input = input;
    }

    /// Queues a FrontEnd menu action (`Up`, `Down`, `Left`, `Right`, `Confirm`, `Back`, `Side`, `Extra`).
    pub fn menu_action(&mut self, action: MenuAction) {
        self.pending_menu_actions.push(action);
    }

    /// Returns any level transition requested by FrontEnd, on-foot exit, or driving mission completion.
    pub fn pending_level(&self) -> Option<&str> {
        self.pending_level.as_deref()
    }

    /// Teleports the active player (Bond or driving vehicle) for rapid bug reproduction.
    pub fn teleport(&mut self, pos: [f32; 3], yaw: Option<f32>, pitch: Option<f32>) {
        if let Some(drv) = self.driving.as_mut() {
            drv.car.pos = pos;
            drv.car.vel = [0.0; 3];
            if let Some(y) = yaw {
                drv.car.yaw = y;
            }
            let (p, cy, cp) = drv.chase_camera();
            self.cam.pos = Vec3::new(p[0], p[1], p[2]);
            self.cam.yaw = cy;
            self.cam.pitch = cp;
        } else if let Some(sim) = self.scene.sim.as_mut() {
            sim.bond.pos = pos;
            if let Some(y) = yaw {
                sim.bond.yaw = y;
            }
            if let Some(p) = pitch {
                sim.bond.pitch = p;
            }
            self.cam.pos = Vec3::new(
                sim.bond.pos[0],
                sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop,
                sim.bond.pos[2],
            );
            self.cam.yaw = sim.bond.yaw;
            self.cam.pitch = sim.bond.pitch;
        } else {
            self.cam.pos = Vec3::new(pos[0], pos[1], pos[2]);
            if let Some(y) = yaw {
                self.cam.yaw = y;
            }
            if let Some(p) = pitch {
                self.cam.pitch = p;
            }
        }
    }

    /// Sets the camera directly for inspecting any scene angle.
    pub fn set_camera(&mut self, pos: [f32; 3], yaw: f32, pitch: f32) {
        self.cam = Camera {
            pos: Vec3::new(pos[0], pos[1], pos[2]),
            yaw,
            pitch,
            fov: GAME_FOV,
        };
    }

    /// Aims Bond's view directly at world-space `target` and updates `self.cam`.
    pub fn aim_at(&mut self, target: [f32; 3]) -> Result<(f32, f32, f32)> {
        let sim = self
            .scene
            .sim
            .as_mut()
            .ok_or_else(|| anyhow!("aim_at requires an active on-foot/MP simulation"))?;
        let eye = sim.bond.head();
        let dx = target[0] - eye[0];
        let dy = target[1] - eye[1];
        let dz = target[2] - eye[2];
        let horiz = (dx * dx + dz * dz).sqrt().max(1e-5);
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        let yaw = dx.atan2(dz);
        let pitch_rad = dy.atan2(horiz);
        sim.bond.yaw = yaw;
        sim.bond.pitch = (pitch_rad / std::f32::consts::FRAC_PI_2).clamp(-0.99, 0.99);
        self.cam = sim_camera(sim);
        Ok((yaw, pitch_rad, dist))
    }

    /// Aims Bond directly at drone `#idx`'s torso/head.
    pub fn aim_drone(&mut self, idx: usize) -> Result<(f32, f32, f32)> {
        let target = {
            let sim = self
                .scene
                .sim
                .as_ref()
                .ok_or_else(|| anyhow!("aim_drone requires an active on-foot simulation"))?;
            let d = sim
                .drones
                .list
                .get(idx)
                .ok_or_else(|| anyhow!("drone index {idx} out of range (total {})", sim.drones.list.len()))?;
            [d.pos[0], d.pos[1] + 1.35, d.pos[2]]
        };
        self.aim_at(target)
    }

    /// Steers Bond toward `target` (XZ) for up to `max_ticks`, stopping once within `tolerance` meters.
    pub fn walk_to(&mut self, target: [f32; 3], max_ticks: u32, tolerance: f32) -> Result<(u32, f32, bool)> {
        let tol = tolerance.max(0.1);
        let saved_input = self.input;
        let mut used = 0u32;
        for _ in 0..max_ticks.max(1) {
            let pos = self
                .scene
                .sim
                .as_ref()
                .ok_or_else(|| anyhow!("walk_to requires an active on-foot/MP simulation"))?
                .bond
                .pos;
            let dx = target[0] - pos[0];
            let dz = target[2] - pos[2];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= tol {
                self.input = saved_input;
                return Ok((used, dist, true));
            }
            if let Some(sim) = self.scene.sim.as_mut() {
                sim.bond.yaw = dx.atan2(dz);
            }
            self.input = InputState {
                move_y: 1.0,
                ..Default::default()
            };
            self.step(1.0 / 60.0);
            used += 1;
        }
        self.input = saved_input;
        let pos = self.scene.sim.as_ref().map(|s| s.bond.pos).unwrap_or([0.0; 3]);
        let dist = ((target[0] - pos[0]).powi(2) + (target[2] - pos[2]).powi(2)).sqrt();
        Ok((used, dist, dist <= tol))
    }

    /// Requests the game's native cut-scene skip (`scripts.skip_request = true`).
    pub fn skip_cutscene(&mut self) -> bool {
        if let Some(sim) = self.scene.sim.as_mut() {
            sim.drones.scripts.skip_request = true;
            true
        } else {
            false
        }
    }

    /// Steps the active simulation (FrontEnd, On-Foot, or Driving) by `dt_secs`.
    pub fn step(&mut self, dt_secs: f32) -> FrameTelemetry {
        let step_t0 = Instant::now();
        let dt = dt_secs.clamp(1.0 / 240.0, 1.0 / 15.0);
        let n_ticks = ((dt * 60.0).round() as u32).clamp(1, 4);
        let just = self.input.buttons & !self.prev_buttons;
        self.prev_buttons = self.input.buttons;

        let menu_actions: Vec<MenuAction> = self.pending_menu_actions.drain(..).collect();

        if let Some(fe) = self.fe.as_mut() {
            for a in menu_actions {
                fe.action(a);
            }
            if (just & BTN_FIRE) != 0 || (just & BTN_USE) != 0 {
                fe.action(MenuAction::Confirm);
            } else if (just & BTN_CROUCH) != 0 {
                fe.action(MenuAction::Back);
            }
            for _ in 0..n_ticks {
                fe.tick();
                self.ticks = self.ticks.wrapping_add(1);
                if let Some(req) = fe.requested_level.take() {
                    self.pending_level = Some(format!("{req:08x}.bin"));
                }
            }
            if let Some(s) = self.sound.as_mut() {
                s.batch(|_, _| {});
            }
        } else if let Some(drv) = self.driving.as_mut() {
            let throttle = self.input.move_y.max(0.0).clamp(0.0, 1.0);
            let brake = (-self.input.move_y).max(0.0).clamp(0.0, 1.0);
            let steer = -self.input.move_x.clamp(-1.0, 1.0);
            let handbrake = (self.input.buttons & BTN_JUMP) != 0;
            if let Some(w) = drv.weapons.as_mut() {
                w.fire |= (self.input.buttons & BTN_FIRE) != 0;
                w.cycle |= (just & BTN_WEAP_NEXT) != 0;
            }
            for _ in 0..n_ticks {
                drv.step(throttle, brake, steer, handbrake);
                self.ticks = self.ticks.wrapping_add(1);
            }
            let (p, yaw, pitch) = drv.chase_camera();
            self.cam = Camera {
                pos: Vec3::new(p[0], p[1], p[2]),
                yaw,
                pitch,
                fov: GAME_FOV,
            };
            if matches!(
                drv.mission_state,
                nf_game::driving::mission::MissionState::Complete
                    | nf_game::driving::mission::MissionState::Failed
            ) {
                let t = self.driving_end_ticks.get_or_insert(0);
                *t += n_ticks;
                if (*t >= 120 || (*t >= 15 && (just & BTN_USE) != 0))
                    && self.pending_level.is_none()
                {
                    let _ret = drv.mission.drive_return(
                        nf_game::ptp::DrivingStats::default(),
                        self.difficulty,
                    );
                    self.pending_level = Some("07000048.bin".to_string());
                    self.driving_end_ticks = None;
                }
            } else {
                self.driving_end_ticks = None;
            }
        } else if let Some(sim) = self.scene.sim.as_mut() {
            let b = self.input.buttons;
            let mut weap_cycle = if (just & BTN_WEAP_NEXT) != 0 {
                1
            } else if (just & BTN_WEAP_PREV) != 0 {
                -1
            } else {
                0
            };
            let mut turn = self.input.look_dx;
            let mut look = self.input.look_dy;
            let mut jump = (just & BTN_JUMP) != 0;
            let mut crouch = (just & BTN_CROUCH) != 0;
            let mut reload = (just & BTN_RELOAD) != 0;
            let mut use_key = (just & BTN_USE) != 0;
            let fire = (b & BTN_FIRE) != 0;

            for _ in 0..n_ticks {
                let pi = nf_game::player::Input {
                    forward: self.input.move_y.clamp(-1.0, 1.0),
                    strafe: -self.input.move_x.clamp(-1.0, 1.0),
                    lift: if (b & BTN_JUMP) != 0 { 1.0 } else { 0.0 }
                        - if (b & BTN_CROUCH) != 0 { 1.0 } else { 0.0 },
                    stabilise: false,
                    turn,
                    look,
                    jump,
                    crouch,
                    crouch_hold: false,
                    crouch_held: (b & BTN_CROUCH) != 0,
                    aiming: (b & BTN_AIM) != 0,
                    weapon_cycle: weap_cycle,
                    gadget_cycle: 0,
                    zoom_cycle: 0,
                    zoom_axis: 0.0,
                    switch_to: None,
                    fire,
                    zoom: (b & BTN_AIM) != 0,
                    scope: (b & BTN_AIM) != 0,
                    mode_switch: false,
                    goggles_action: false,
                    keypad_confirm: false,
                    keypad_back: false,
                    keypad_nav: 0,
                };
                sim.step_bond(pi, fire, reload, use_key);
                if let Some(s) = self.sound.as_mut() {
                    sound_tick(s, sim);
                }
                if let Some(next) = sim.drones.level_exit.take() {
                    if sim.drones.exit_hold.is_some() {
                        sim.drones.level_exit = Some(next);
                    } else {
                        let wrapped = nf_game::triggers::wrap_intro(next);
                        let resolved = nf_game::triggers::resolve_exit_code(wrapped);
                        self.pending_level = Some(format!("{resolved:08x}.bin"));
                    }
                }
                weap_cycle = 0;
                turn = 0.0;
                look = 0.0;
                jump = false;
                crouch = false;
                reload = false;
                use_key = false;
                self.ticks = self.ticks.wrapping_add(1);
            }
            self.cam = sim_camera(sim);
        } else {
            self.ticks = self.ticks.wrapping_add(n_ticks);
        }

        if self.auto_transition {
            if let Some(next_arch) = self.pending_level.take() {
                let _ = self.load_level(&next_arch);
            }
        }

        self.perf.last_step_ms = step_t0.elapsed().as_secs_f64() * 1000.0;
        self.perf.total_cels = self.scene.cels.len();
        self.telemetry()
    }

    /// Renders the current frame offscreen via wgpu Metal and writes a PNG to `out_path`.
    pub fn render_png(
        &mut self,
        width: u32,
        height: u32,
        out_path: impl AsRef<Path>,
    ) -> Result<RenderStats> {
        let w = width.max(64);
        let h = height.max(64);
        let aspect = w as f32 / h as f32;

        // Refresh driving particles, reflection map, and HUD frame before GPU upload.
        if let Some(drv) = self.driving.as_mut() {
            let (r, u) = billboard_axes(&self.cam);
            self.scene.drive_particles = drv.particle_groups(r, u);
            drv.reflection_frame(&mut self.scene, r, u);
            self.scene.drive_hud = drv.hud_frame(&self.scene, aspect);
        }

        if self.metal.is_none() {
            self.metal = Some(MetalContext::new(&self.scene)?);
        }
        let m = self.metal.as_mut().unwrap();

        let cam = self
            .scene
            .sim
            .as_ref()
            .and_then(|s| s.view_override())
            .map(|(p, d)| Camera {
                pos: Vec3::new(p[0], p[1], p[2]),
                yaw: d[0].atan2(d[2]),
                pitch: d[1].clamp(-1.0, 1.0).asin(),
                fov: GAME_FOV,
            })
            .unwrap_or(self.cam);

        let cut = self
            .scene
            .sim
            .as_ref()
            .is_some_and(|s| s.script_camera().is_some());
        let game_aspect = self.scene.sim.as_ref().map_or(4.0 / 3.0, |s| s.view_aspect);
        let (proj_aspect, band) = if cut {
            cutscene_frame(w as f32, h as f32, game_aspect)
        } else {
            (aspect, [0.0; 4])
        };

        let (vp, vp_rot) = cam.view_proj_for(proj_aspect, self.scene.mirror_x);
        let (opts_right, opts_up) = cam_axes(&cam);
        let opts = ViewOpts {
            reinforcements: false,
            firing: (self.input.buttons & BTN_FIRE) != 0,
            right: opts_right,
            up: opts_up,
        };

        let skin_t0 = Instant::now();
        let door_closed = self
            .scene
            .sim
            .as_ref()
            .map(|s| s.drones.doors.closed_flags());
        let vis = m.renderer.cel_visibility(
            cam.pos.to_array(),
            vp,
            self.scene.sim.as_ref().map(|s| &s.world),
            door_closed.as_deref(),
        );
        let visible_cels = vis
            .as_ref()
            .map(|v| v.iter().filter(|&&b| b).count())
            .unwrap_or(self.scene.cels.len());

        m.renderer.destroy_dead = dead_destroy_centres(&self.scene);
        m.renderer.destroy_wreck_draw = wreck_draw_centres(&self.scene);
        m.renderer.destroy_tint = destroy_tints(&self.scene);
        m.renderer.sky_flash_mask = self
            .scene
            .sim
            .as_ref()
            .map_or(0, |s| s.drones.sky.flash_mask);
        m.renderer.sky_spin = self.scene.sim.as_ref().map_or(0.0, |s| s.drones.sky.spin());

        m.renderer
            .update_skin(&m.queue, &self.scene, None, self.ticks as f32, opts, None);

        let decal_cells = self.scene.decal_cell_entries(&m.renderer.skin_layout);
        if self.driving.is_some() || (vis.is_some() && !decal_cells.is_empty()) {
            let mut pal = self
                .scene
                .skin_palette(&m.renderer.skin_layout, None, self.ticks as f32, opts, None)
                .0;
            if let Some(v) = vis.as_deref() {
                self.scene.hide_unseen_decals(&mut pal, &decal_cells, v);
            }
            if let Some(drv) = self.driving.as_ref() {
                drv.apply_to_palette_eye(
                    m.renderer.skin_layout.attach_base as usize,
                    &mut pal,
                    [cam.pos.x, cam.pos.y, cam.pos.z],
                );
            }
            m.renderer.upload_palette(&m.queue, &pal);
        }

        m.renderer.upload_morph_weights(&m.queue, &self.scene);
        m.renderer.set_char_light_centres(&self.scene);
        m.renderer.lights = self.scene.frame_lights();
        m.renderer.screen_aspect = aspect;
        m.renderer.set_shards(&m.queue, &self.scene.shard_draws());
        m.renderer
            .set_particles(&m.queue, &particles_for(&self.scene, &cam));
        m.renderer
            .set_reflection(&m.queue, self.scene.drive_reflection.as_ref().map(|r| &r.frame));
        let skin_ms = skin_t0.elapsed().as_secs_f64() * 1000.0;
        let render_t0 = Instant::now();

        let target = m.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nf-macos-target"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: m.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let depth = gpu::depth_view(&m.device, w, h);

        let snow = snow_for(&self.scene, &cam);
        let menu_hud = self.fe.as_mut().map(|fe| {
            let lang = self
                .lang
                .as_ref()
                .or_else(|| self.scene.sim.as_ref().and_then(|s| s.lang.as_ref()));
            fe.view_h = h as f32;
            match &self.scene.hud {
                Some(tables) => menu::menu_overlay_fonts(
                    fe,
                    &tables.font,
                    &tables.font_header,
                    Some(&tables.font_medium),
                    lang,
                    &self.scene.hud_textures,
                    proj_aspect,
                ),
                None => overlay_for(&self.scene, proj_aspect),
            }
        });
        let hud = shot_hud(&self.scene, proj_aspect, menu_hud.as_ref(), false);

        let mut enc = m.renderer.render(
            &m.device,
            &m.queue,
            &view,
            &depth,
            vp,
            vp_rot,
            cam.pos.to_array(),
            self.ticks,
            &snow,
            &hud,
            None,
            if cut { Some(band) } else { None },
        );

        let row_bytes = (4 * w).next_multiple_of(256);
        let buf = m.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nf-macos-readback"),
            size: (row_bytes * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row_bytes),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        m.queue.submit([enc.finish()]);

        let slice = buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = m.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .context("readback channel closed")?
            .map_err(|e| anyhow!("buffer map failed: {e:?}"))?;

        let mapped = slice
            .get_mapped_range()
            .map_err(|e| anyhow!("get_mapped_range failed: {e:?}"))?;
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h as usize {
            let start = y * row_bytes as usize;
            rgba.extend_from_slice(&mapped[start..start + (w as usize * 4)]);
        }
        drop(mapped);
        buf.unmap();
        let render_ms = render_t0.elapsed().as_secs_f64() * 1000.0;

        let enc_t0 = Instant::now();
        let (mean_luma, std_luma) = luma_stats(&rgba, w, h);
        let png_bytes = nf_formats::png::encode_rgba(w, h, &rgba);
        let out_path = out_path.as_ref();
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(out_path, png_bytes)?;
        let encode_ms = enc_t0.elapsed().as_secs_f64() * 1000.0;

        self.perf.last_skin_ms = skin_ms;
        self.perf.last_render_ms = render_ms;
        self.perf.last_encode_ms = encode_ms;
        self.perf.last_visible_cels = visible_cels;
        self.perf.total_cels = self.scene.cels.len();

        Ok(RenderStats {
            width: w,
            height: h,
            mean_luma,
            std_luma,
        })
    }

    /// Benchmarks `frames` consecutive 60 Hz sim + Metal GPU frames (no PNG disk write)
    /// and returns real frame-time percentiles and estimated FPS.
    pub fn bench_frames(&mut self, frames: u32, width: u32, height: u32) -> Result<BenchReport> {
        let frames = frames.max(1);
        let w = width.max(64);
        let h = height.max(64);
        let aspect = w as f32 / h as f32;
        if self.metal.is_none() {
            self.metal = Some(MetalContext::new(&self.scene)?);
        }
        let target = {
            let m = self.metal.as_ref().unwrap();
            m.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("nf-macos-bench-target"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: m.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        };
        let view = target.create_view(&Default::default());
        let depth = {
            let m = self.metal.as_ref().unwrap();
            gpu::depth_view(&m.device, w, h)
        };

        let mut totals = Vec::with_capacity(frames as usize);
        let mut sim_sum = 0.0f64;
        let mut skin_sum = 0.0f64;
        let mut gpu_sum = 0.0f64;

        for _ in 0..frames {
            let f0 = Instant::now();
            self.step(1.0 / 60.0);
            let sim_ms = self.perf.last_step_ms;
            sim_sum += sim_ms;

            let skin_t0 = Instant::now();
            if let Some(drv) = self.driving.as_mut() {
                let (r, u) = billboard_axes(&self.cam);
                self.scene.drive_particles = drv.particle_groups(r, u);
                drv.reflection_frame(&mut self.scene, r, u);
                self.scene.drive_hud = drv.hud_frame(&self.scene, aspect);
            }
            let m = self.metal.as_mut().unwrap();
            let cam = self
                .scene
                .sim
                .as_ref()
                .and_then(|s| s.view_override())
                .map(|(p, d)| Camera {
                    pos: Vec3::new(p[0], p[1], p[2]),
                    yaw: d[0].atan2(d[2]),
                    pitch: d[1].clamp(-1.0, 1.0).asin(),
                    fov: GAME_FOV,
                })
                .unwrap_or(self.cam);
            let cut = self
                .scene
                .sim
                .as_ref()
                .is_some_and(|s| s.script_camera().is_some());
            let game_aspect = self.scene.sim.as_ref().map_or(4.0 / 3.0, |s| s.view_aspect);
            let (proj_aspect, band) = if cut {
                cutscene_frame(w as f32, h as f32, game_aspect)
            } else {
                (aspect, [0.0; 4])
            };
            let (vp, vp_rot) = cam.view_proj_for(proj_aspect, self.scene.mirror_x);
            let (opts_right, opts_up) = cam_axes(&cam);
            let opts = ViewOpts {
                reinforcements: false,
                firing: (self.input.buttons & BTN_FIRE) != 0,
                right: opts_right,
                up: opts_up,
            };
            let door_closed = self
                .scene
                .sim
                .as_ref()
                .map(|s| s.drones.doors.closed_flags());
            let vis = m.renderer.cel_visibility(
                cam.pos.to_array(),
                vp,
                self.scene.sim.as_ref().map(|s| &s.world),
                door_closed.as_deref(),
            );
            m.renderer.destroy_dead = dead_destroy_centres(&self.scene);
            m.renderer.destroy_wreck_draw = wreck_draw_centres(&self.scene);
            m.renderer.destroy_tint = destroy_tints(&self.scene);
            m.renderer.sky_flash_mask = self
                .scene
                .sim
                .as_ref()
                .map_or(0, |s| s.drones.sky.flash_mask);
            m.renderer.sky_spin = self.scene.sim.as_ref().map_or(0.0, |s| s.drones.sky.spin());
            m.renderer
                .update_skin(&m.queue, &self.scene, None, self.ticks as f32, opts, None);
            let decal_cells = self.scene.decal_cell_entries(&m.renderer.skin_layout);
            if self.driving.is_some() || (vis.is_some() && !decal_cells.is_empty()) {
                let mut pal = self
                    .scene
                    .skin_palette(&m.renderer.skin_layout, None, self.ticks as f32, opts, None)
                    .0;
                if let Some(v) = vis.as_deref() {
                    self.scene.hide_unseen_decals(&mut pal, &decal_cells, v);
                }
                if let Some(drv) = self.driving.as_ref() {
                    drv.apply_to_palette_eye(
                        m.renderer.skin_layout.attach_base as usize,
                        &mut pal,
                        [cam.pos.x, cam.pos.y, cam.pos.z],
                    );
                }
                m.renderer.upload_palette(&m.queue, &pal);
            }
            m.renderer.upload_morph_weights(&m.queue, &self.scene);
            m.renderer.set_char_light_centres(&self.scene);
            m.renderer.lights = self.scene.frame_lights();
            m.renderer.screen_aspect = aspect;
            m.renderer.set_shards(&m.queue, &self.scene.shard_draws());
            m.renderer
                .set_particles(&m.queue, &particles_for(&self.scene, &cam));
            m.renderer
                .set_reflection(&m.queue, self.scene.drive_reflection.as_ref().map(|r| &r.frame));
            let skin_ms = skin_t0.elapsed().as_secs_f64() * 1000.0;
            skin_sum += skin_ms;

            let gpu_t0 = Instant::now();
            let snow = snow_for(&self.scene, &cam);
            let menu_hud = self.fe.as_mut().map(|fe| {
                let lang = self
                    .lang
                    .as_ref()
                    .or_else(|| self.scene.sim.as_ref().and_then(|s| s.lang.as_ref()));
                fe.view_h = h as f32;
                match &self.scene.hud {
                    Some(tables) => menu::menu_overlay_fonts(
                        fe,
                        &tables.font,
                        &tables.font_header,
                        Some(&tables.font_medium),
                        lang,
                        &self.scene.hud_textures,
                        proj_aspect,
                    ),
                    None => overlay_for(&self.scene, proj_aspect),
                }
            });
            let hud = shot_hud(&self.scene, proj_aspect, menu_hud.as_ref(), false);
            let enc = m.renderer.render(
                &m.device,
                &m.queue,
                &view,
                &depth,
                vp,
                vp_rot,
                cam.pos.to_array(),
                self.ticks,
                &snow,
                &hud,
                None,
                if cut { Some(band) } else { None },
            );
            m.queue.submit([enc.finish()]);
            let _ = m.device.poll(wgpu::PollType::wait_indefinitely());
            let gpu_ms = gpu_t0.elapsed().as_secs_f64() * 1000.0;
            gpu_sum += gpu_ms;

            totals.push(f0.elapsed().as_secs_f64() * 1000.0);
        }

        totals.sort_by(|a, b| a.total_cmp(b));
        let n = totals.len() as f64;
        let avg_ms = totals.iter().sum::<f64>() / n;
        let p50_ms = totals[(totals.len() / 2).min(totals.len() - 1)];
        let p95_idx = ((totals.len() as f64 * 0.95).floor() as usize).min(totals.len() - 1);
        let p95_ms = totals[p95_idx];
        let min_ms = totals[0];
        let max_ms = *totals.last().unwrap();
        let est_fps = if avg_ms > 0.001 { 1000.0 / avg_ms } else { 0.0 };

        Ok(BenchReport {
            frames,
            width: w,
            height: h,
            avg_ms,
            p50_ms,
            p95_ms,
            min_ms,
            max_ms,
            est_fps,
            avg_sim_ms: sim_sum / n,
            avg_skin_ms: skin_sum / n,
            avg_gpu_ms: gpu_sum / n,
        })
    }

    /// Returns whether the engine is currently in the FrontEnd menu.
    pub fn is_in_menu(&self) -> bool {
        self.fe.is_some()
    }

    /// Discovers all available on-foot (`0700xxxx.bin`) and driving (`0900xxxx.bin`)
    /// archives directly from the ISO image or carved asset folder.
    pub fn list_levels(&self) -> Vec<String> {
        discover_levels(&self.data_path)
    }

    /// Returns the live telemetry snapshot.
    pub fn telemetry(&self) -> FrameTelemetry {
        let pos = self
            .driving
            .as_ref()
            .map(|d| d.car.pos)
            .or_else(|| self.scene.sim.as_ref().map(|s| s.bond.pos))
            .unwrap_or([0.0, 0.0, 0.0]);
        let mode = self
            .fe
            .as_ref()
            .map(|f| format!("{:?}", f.page))
            .unwrap_or_else(|| {
                if self.driving.is_some() {
                    "Driving".to_string()
                } else {
                    "InGame".to_string()
                }
            });
        let speed = self
            .driving
            .as_ref()
            .map(|d| (d.car.vel[0].powi(2) + d.car.vel[2].powi(2)).sqrt());
        let player_drives = self
            .driving
            .as_ref()
            .map(|d| d.intro.as_ref().map_or(true, |(i, _)| i.player_drives()));
        let mission_state = self
            .driving
            .as_ref()
            .map(|d| format!("{:?}", d.mission_state));
        let health = self.scene.sim.as_ref().map(|s| s.drones.health.health);
        let weapon = self.scene.sim.as_ref().map(|s| s.gun.weapon as u8);
        FrameTelemetry {
            archive: self.archive.clone(),
            tick: self.ticks,
            mode,
            pos,
            yaw: self.cam.yaw,
            pitch: self.cam.pitch,
            speed,
            player_drives,
            mission_state,
            health,
            weapon,
            cels: self.scene.cels.len(),
            textures: self.scene.textures.len(),
            pending_level: self.pending_level.clone(),
        }
    }

    /// Formats a human-readable status line matching `nf-android`'s `nativeGetStatus`.
    pub fn status_line(&self) -> String {
        let t = self.telemetry();
        format!(
            "{} | tick {} | {} | ({:.1}, {:.1}, {:.1})",
            t.archive, t.tick, t.mode, t.pos[0], t.pos[1], t.pos[2]
        )
    }
}

/// Discovers on-foot (`0700xxxx.bin`) and driving (`0900xxxx.bin`) level archives
/// from either an ISO image or a carved asset directory.
pub fn discover_levels(data_path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(iso) = nf_formats::iso9660::IsoImage::open(data_path) {
        if let Ok((entries, _)) = nf_formats::disc::files_bin_entries(&iso) {
            for e in entries {
                let lower = e.name.to_ascii_lowercase();
                if lower.starts_with("0700") && lower.ends_with(".bin") {
                    out.push(lower);
                }
            }
        }
        for (viv, code) in DRIVING_LEVEL_TABLE {
            let disc_path = format!("DRIVING/{viv}.VIV");
            if iso.find(&disc_path).is_some() {
                out.push(code.to_string());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

const DRIVING_LEVEL_TABLE: [(&str, &str); 8] = [
    ("MIS01", "09000001.bin"),
    ("MIS11", "09000002.bin"),
    ("MIS13A", "09000003.bin"),
    ("MIS13B", "09000004.bin"),
    ("MIS3", "09000005.bin"),
    ("MIS4", "09000006.bin"),
    ("MIS13C", "09000007.bin"),
    ("RACE", "09000008.bin"),
];

fn load_lang(data_path: &str) -> Option<nf_formats::hud::Lang> {
    let iso = nf_formats::iso9660::IsoImage::open(data_path).ok()?;
    let (list, files_bin) = nf_formats::disc::files_bin_entries(&iso).ok()?;
    let entry = list
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("USATxt.dat"))?;
    let base = files_bin.lba as u64 * 2048;
    let bytes = iso
        .read_bytes(base + entry.offset as u64, entry.size as u64)
        .ok()?;
    nf_formats::hud::Lang::parse(&bytes).ok()
}

pub fn resolve_driving_viv(archive: &str) -> Option<&'static str> {
    let trimmed = archive.trim();
    let stem = trimmed
        .trim_end_matches(".bin")
        .trim_end_matches(".BIN")
        .trim_end_matches(".viv")
        .trim_end_matches(".VIV");
    if let Ok(code) = u32::from_str_radix(stem, 16) {
        if let Some(viv) = driving_viv_for_level(code) {
            return Some(viv);
        }
    }
    match stem.to_ascii_uppercase().as_str() {
        "MIS01" => Some("MIS01"),
        "MIS3" => Some("MIS3"),
        "MIS4" => Some("MIS4"),
        "MIS11" => Some("MIS11"),
        "MIS13A" => Some("MIS13A"),
        "MIS13B" => Some("MIS13B"),
        "MIS13C" => Some("MIS13C"),
        "RACE" => Some("RACE"),
        _ => None,
    }
}

fn load_scene_for(
    data_path: &str,
    archive: &str,
    opts: LoadOptions,
) -> Result<(Scene, Option<DrivingState>, Option<FrontEnd>, Camera, String)> {
    if let Some(viv) = resolve_driving_viv(archive) {
        let (scene, drv) = load_driving_scene_at(data_path, viv, opts.difficulty)?;
        let (p, yaw, pitch) = drv.chase_camera();
        let cam = Camera {
            pos: Vec3::new(p[0], p[1], p[2]),
            yaw,
            pitch,
            fov: GAME_FOV,
        };
        return Ok((scene, Some(drv), None, cam, viv.to_string()));
    }

    if archive.eq_ignore_ascii_case("07000048.bin") {
        let scene = load_front_end(data_path, archive, true, None, false, opts.difficulty, 0)?;
        let mut fe = FrontEnd::new();
        fe.unlock_all = opts.unlock_all;
        if let Some(page) = opts.fe_page {
            fe.page = page;
        }
        let cam = Camera {
            pos: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            fov: GAME_FOV,
        };
        return Ok((scene, None, Some(fe), cam, archive.to_string()));
    }

    let scene = if let Some((humans, bots)) = opts.mp_match {
        let code = u32::from_str_radix(archive.trim_end_matches(".bin"), 16).unwrap_or(0x0700_0023);
        let mut st = nf_game::mp::MpSettings::deathmatch(code);
        st.human_count = humans.clamp(1, nf_game::mp::MP_MAX_SLOTS as u32);
        st.bot_count = bots;
        st.player_count = st.human_count as u16;
        st.time_limit = 10 * 60;
        load_mp_carry(data_path, archive, false, None, st)?
    } else {
        load_with_carry_difficulty(data_path, archive, false, None, opts.difficulty)?
    };
    let cam = scene
        .sim
        .as_ref()
        .map(sim_camera)
        .unwrap_or(Camera {
            pos: Vec3::new(0.0, 1.6, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: GAME_FOV,
        });
    Ok((scene, None, None, cam, archive.to_string()))
}

/// Computes the exact camera matching `nf-view`'s priority:
/// `bond.head()` + `view_fov(sim)` -> `script_camera()` -> `wire_camera()` -> `view_override()`.
fn sim_camera(sim: &nf_view::scene::Sim) -> Camera {
    let h = sim.bond.head();
    let mut cam = Camera {
        pos: Vec3::new(h[0], h[1], h[2]),
        yaw: sim.bond.yaw,
        pitch: sim.bond.pitch * std::f32::consts::FRAC_PI_2,
        fov: view_fov(sim),
    };
    if let Some((p, yaw, pitch, fov)) = sim.script_camera() {
        cam = Camera {
            pos: Vec3::new(p[0], p[1], p[2]),
            yaw,
            pitch,
            fov,
        };
    }
    if let Some((p, yaw, pitch, fov)) = sim.wire_camera() {
        cam = Camera {
            pos: Vec3::new(p[0], p[1], p[2]),
            yaw,
            pitch,
            fov: GAME_FOV / fov,
        };
    }
    if let Some((p, d)) = sim.view_override() {
        cam = Camera {
            pos: Vec3::new(p[0], p[1], p[2]),
            yaw: d[0].atan2(d[2]),
            pitch: d[1].clamp(-1.0, 1.0).asin(),
            fov: GAME_FOV,
        };
    }
    cam
}

fn luma_stats(rgba: &[u8], w: u32, h: u32) -> (f64, f64) {
    let mut lumas = Vec::new();
    for y in (0..h as usize).step_by(8) {
        for x in (0..w as usize).step_by(8) {
            let o = (y * w as usize + x) * 4;
            let l = 0.299 * rgba[o] as f64 + 0.587 * rgba[o + 1] as f64 + 0.114 * rgba[o + 2] as f64;
            lumas.push(l);
        }
    }
    let n = lumas.len().max(1) as f64;
    let mean = lumas.iter().sum::<f64>() / n;
    let var = lumas.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    (mean, var.sqrt())
}

pub(crate) fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_driving_codes_and_short_names() {
        assert_eq!(resolve_driving_viv("09000001.bin"), Some("MIS01"));
        assert_eq!(resolve_driving_viv("MIS01"), Some("MIS01"));
        assert_eq!(resolve_driving_viv("mis13a.viv"), Some("MIS13A"));
        assert_eq!(resolve_driving_viv("09000008.bin"), Some("RACE"));
        assert_eq!(resolve_driving_viv("07000005.bin"), None);
    }

    #[test]
    fn computes_luma_stats_cleanly() {
        let mut rgba = vec![0u8; 64 * 64 * 4];
        for i in 0..(64 * 64) {
            let v = if i % 2 == 0 { 200 } else { 40 };
            rgba[i * 4] = v;
            rgba[i * 4 + 1] = v;
            rgba[i * 4 + 2] = v;
            rgba[i * 4 + 3] = 255;
        }
        let (mean, std) = luma_stats(&rgba, 64, 64);
        assert!((mean - 200.0).abs() < 1e-6);
        assert!(std.abs() < 1e-6);
        // Sister check: a varied checkerboard at 8-px stride has non-zero variance.
        for y in 0..64 {
            for x in 0..64 {
                let v = if ((x / 8) + (y / 8)) % 2 == 0 { 220 } else { 20 };
                let o = (y * 64 + x) * 4;
                rgba[o..o + 3].copy_from_slice(&[v, v, v]);
            }
        }
        let (mean2, std2) = luma_stats(&rgba, 64, 64);
        assert!((mean2 - 120.0).abs() < 1e-6);
        assert!((std2 - 100.0).abs() < 1e-6);
    }

    #[test]
    fn telemetry_json_escapes_strings_and_includes_optional_fields() {
        let t = FrameTelemetry {
            archive: "MIS01".into(),
            tick: 60,
            mode: "Driving".into(),
            pos: [1.0, 2.0, 3.0],
            yaw: 0.5,
            pitch: -0.1,
            speed: Some(12.5),
            player_drives: Some(true),
            mission_state: Some("Playing".into()),
            health: None,
            weapon: None,
            cels: 0,
            textures: 10,
            pending_level: Some("07000048.bin".into()),
        };
        let j = t.to_json();
        assert!(j.contains("\"archive\":\"MIS01\""));
        assert!(j.contains("\"speed\":12.500"));
        assert!(j.contains("\"pending_level\":\"07000048.bin\""));
        assert!(!j.contains("\"health\""));
    }
}
