//! Driving mission loader and 60 Hz runtime (`DRIVING/*.VIV` track `.crp` + `.ssh` + `nf_game::driving`).

use crate::scene::{Attachment, CharVertex, Draw, Layer, Material, Scene, TextureImage, Vertex};
use anyhow::{Context, Result};
use nf_formats::anim::Mat4;
use nf_formats::crp::Crp;
use nf_formats::crp_mesh::{CollisionTri, MapTrigger, TrackModule};
use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;
use nf_formats::ssh::Ssh;
use nf_game::driving::{
    ai::AiDriver,

    mission::{Mission, MissionState, SimSnapshot},
    physics::Tuning,
    road::RoadNet,
    suspension::{
        axle_of, rotor_wheelspin, watercraft_handbrake_drag, watercraft_longitudinal_thrust,
        SusParams, SusWheel,
    },
    Car, CarInput, CarParams, FlatWorld, Rigid, DT,
};
use std::collections::HashMap;

/// Map the front end's `0x09xxxxxx` driving level code to its `/DRIVING/<VIV>.VIV` stem.
pub fn driving_viv_for_level(level: u32) -> Option<&'static str> {
    match level {
        0x0900_0001 => Some("MIS01"),
        0x0900_0005 => Some("MIS3"),
        0x0900_0006 => Some("MIS4"),
        0x0900_0002 => Some("MIS11"),
        0x0900_0003 => Some("MIS13A"),
        0x0900_0004 => Some("MIS13B"),
        0x0900_0007 => Some("MIS13C"),
        0x0900_0008 => Some("RACE"),
        _ => None,
    }
}

fn player_car_stem(viv_upper: &str) -> &'static str {
    match viv_upper {
        "MIS11" => "vanquishsub",
        "MIS13A" | "MIS13B" => "ultralight",
        "MIS13C" => "patrolboat",
        // RACE: the archive's only player vehicle file is cobra_player.atr, whose
        // RENDER_FILENAME.s is paradis_car.crp [D RACE.VIV]; RACE carries no vanquish.crp.
        "RACE" => "paradis_car",
        // MIS3 carries no vanquish.crp either; supersnow (8 colours, a bullet-streak weapon type) is
        // taken as the player's over small_snowmobile (1 colour, no weapon) [S: the player vehicle
        // pick is not traced].
        "MIS3" => "supersnow",
        _ => "vanquish",
    }
}

fn drive_index_for_viv(viv_upper: &str) -> u8 {
    match viv_upper {
        "MIS01" => 0,
        "MIS3" => 1,
        "MIS4" => 2,
        "MIS11" => 3,
        "MIS13A" => 4,
        "MIS13B" => 5,
        "MIS13C" => 6,
        "RACE" => 7,
        _ => 0,
    }
}

/// Spatial XZ grid over `.crp` `CDat` collision triangles for O(1) ground raycasts.
pub struct CollisionGrid {
    tris: Vec<CollisionTri>,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl CollisionGrid {
    const CELL: f32 = 16.0;

    /// The nearest hit of the segment a->b on the collision triangles (two-sided), for the chase
    /// camera's pull-in ray (`FUN_002127f0` in the game; this grid test is the port's query).
    pub fn segment_hit(&self, a: [f32; 3], b: [f32; 3]) -> Option<[f32; 3]> {
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
        let (z0, z1) = (a[2].min(b[2]), a[2].max(b[2]));
        let (cx0, cx1) = ((x0 / Self::CELL).floor() as i32, (x1 / Self::CELL).floor() as i32);
        let (cz0, cz1) = ((z0 / Self::CELL).floor() as i32, (z1 / Self::CELL).floor() as i32);
        let mut best: Option<f32> = None;
        for cx in cx0..=cx1 {
            for cz in cz0..=cz1 {
                let Some(list) = self.cells.get(&(cx, cz)) else { continue };
                for &ti in list {
                    let [p0, p1, p2] = self.tris[ti].v;
                    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                    let h = [d[1] * e2[2] - d[2] * e2[1], d[2] * e2[0] - d[0] * e2[2], d[0] * e2[1] - d[1] * e2[0]];
                    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
                    if det.abs() < 1e-8 { continue; }
                    let inv = 1.0 / det;
                    let sv = [a[0] - p0[0], a[1] - p0[1], a[2] - p0[2]];
                    let u = inv * (sv[0] * h[0] + sv[1] * h[1] + sv[2] * h[2]);
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = [sv[1] * e1[2] - sv[2] * e1[1], sv[2] * e1[0] - sv[0] * e1[2], sv[0] * e1[1] - sv[1] * e1[0]];
                    let v = inv * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
                    if (0.0..=1.0).contains(&t) && best.is_none_or(|bt| t < bt) { best = Some(t); }
                }
            }
        }
        best.map(|t| [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t])
    }

    pub fn build(raw_tris: Vec<CollisionTri>) -> Self {
        let mut tris = Vec::new();
        let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for t in raw_tris {
            let e1 = [t.v[1][0] - t.v[0][0], t.v[1][1] - t.v[0][1], t.v[1][2] - t.v[0][2]];
            let e2 = [t.v[2][0] - t.v[0][0], t.v[2][1] - t.v[0][1], t.v[2][2] - t.v[0][2]];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len < 1e-5 || (ny.abs() / len) < 0.25 {
                continue;
            }
            let idx = tris.len();
            let min_x = t.v[0][0].min(t.v[1][0]).min(t.v[2][0]);
            let max_x = t.v[0][0].max(t.v[1][0]).max(t.v[2][0]);
            let min_z = t.v[0][2].min(t.v[1][2]).min(t.v[2][2]);
            let max_z = t.v[0][2].max(t.v[1][2]).max(t.v[2][2]);
            let cx0 = (min_x / Self::CELL).floor() as i32;
            let cx1 = (max_x / Self::CELL).floor() as i32;
            let cz0 = (min_z / Self::CELL).floor() as i32;
            let cz1 = (max_z / Self::CELL).floor() as i32;
            for cx in cx0..=cx1 {
                for cz in cz0..=cz1 {
                    cells.entry((cx, cz)).or_default().push(idx);
                }
            }
            tris.push(t);
        }
        Self { tris, cells }
    }

    /// Vertical downward raycast at `(x, cur_y + 2.5, z)` returning the highest walkable surface
    /// `y` within `[cur_y - 12.0, cur_y + 3.0]`.
    pub fn ground_y(&self, x: f32, cur_y: f32, z: f32) -> Option<f32> {
        let cx = (x / Self::CELL).floor() as i32;
        let cz = (z / Self::CELL).floor() as i32;
        let list = self.cells.get(&(cx, cz))?;
        let mut best: Option<f32> = None;
        for &ti in list {
            let [a, b, c] = self.tris[ti].v;
            let denom = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if denom.abs() < 1e-6 {
                continue;
            }
            let u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / denom;
            let v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / denom;
            let w = 1.0 - u - v;
            if u < -1e-4 || v < -1e-4 || w < -1e-4 {
                continue;
            }
            let y = u * a[1] + v * b[1] + w * c[1];
            if y >= cur_y - 12.0 && y <= cur_y + 3.0 {
                best = Some(match best {
                    Some(prev) => prev.max(y),
                    None => y,
                });
            }
        }
        best
    }
}

/// The player car's moving parts: the game's wheel rig (`FUN_001d2cb0`) posing the Skel-attached
/// items (wheels, hubs) each tick (nf_formats::carpose, docs/formats/car-model.md §11).
pub struct CarRig {
    pub mesh: nf_formats::carmesh::CarMesh,
    pub rig: nf_formats::carpose::WheelRig,
    pub attrs: nf_formats::carpose::WheelAttrs,
    pub mem: nf_formats::carpose::WheelMemory,
    pub pose: nf_formats::carpose::CarPose,
    pub spin: [f32; 2],
    /// `(in#0 item index, palette slot)` of each posed part.
    pub parts: Vec<(usize, u32)>,
}

impl CarRig {
    /// One tick: roll by `speed / (WHEEL_RADIUS·6.32·60)` turns (the vehicle's +0x274 step, docs/
    /// systems/drive-suspension.md §3; signed by the car's forward speed), steer, springs.
    pub fn tick(&mut self, car: &Car, steer_in: f32) {
        let fwd = [car.yaw.sin(), 0.0, car.yaw.cos()];
        let v = car.vel[0] * fwd[0] + car.vel[2] * fwd[2];
        let step = v / (self.attrs.wheel_radius.max(0.01) * 6.32 * 60.0);
        self.spin = nf_formats::carpose::spin_step(self.spin, step);
        let input = nf_formats::carpose::WheelInput {
            spin: self.spin,
            // The steer angle in turns, clamped to +-0.075 by the rig [S: the port's steer input
            // scaled to the clamp, not the car's +0x28c].
            steer: steer_in * 0.075,
            compression: car.spring_prev,
            slow_steer: false,
            fins: None,
        };
        self.pose.frame += 1.0;
        let _ = self.rig.update(&self.attrs, &input, &mut self.mem, &mut self.pose.skel);
    }
}

/// The player car's tyre smoke and surface spray (nf_game::driving::effects: `FUN_001ee4a0`
/// emitters, the 1024-particle pool, the ELF's built-in particle types; docs/systems/driving-effects.md).
pub struct DriveFx {
    pub lib: nf_game::driving::effects::ParticleLib,
    pub veh: nf_game::driving::effects::VehicleFx,
    pub pool: nf_game::driving::effects::ParticleSystem,
    pub rng: nf_game::driving::effects::DriveRng,
    /// The track sheet's 256x256 `MAIN` atlas in the scene's textures.
    pub main_tex: Option<usize>,
}

/// The driving HUD: hudNTSC.gal's layout and sheets, the ported HUD passes
/// (nf_game::driving_hud_draw) and the player's health block they read (nf_game::driving::damage).
pub struct DriveHud {
    pub layout: nf_game::driving_hud_draw::HudLayout,
    pub draw: nf_game::driving_hud_draw::DriveHudDraw,
    /// gallery sheet index -> `Scene::hud_textures` hash.
    pub sheets: Vec<(u32, u32)>,
    pub health: nf_game::driving::damage::PlayerHealth,
    pub last_tick: u32,
    /// The gallery's font (data\loading\loading.sfn) and the English string table (IDL_ENG.loc)
    /// the text passes measure and resolve labels with.
    pub font: nf_formats::sfn::Font,
    pub loc: nf_formats::loc::Loc,
}

/// The player car's specular pass (bondrm.o `MatCar*` pass 2, docs/formats/car-model.md §10):
/// per frame, ST = 0.5 + SpecularSpot·(n·W·basis).xy on the `carS` map, colour Specular·C_w,
/// added over the base pass. The strips are baked at the parts' rest matrices (car space).
pub struct CarShine {
    /// (strip with car-space positions/normals, its CarRender props).
    pub strips: Vec<(nf_formats::carmesh::CarStrip, nf_formats::rendertun::CarMaterialProps)>,
    pub entry: nf_formats::rendertun::CarRenderEntry,
    pub light: nf_formats::rendertun::LightEnv,
    /// `carS` (data\render\common.ssh) in the scene's textures.
    pub tex: Option<usize>,
}

/// The player car's engine sound: the ten layer loops of the track's Engine bank (slot 9) and the
/// game's layer logic (nf_game::driving::audio::EngineSound: FUN_002fa8f0 / FUN_002ff040).
pub struct DriveEngine {
    pub snd: nf_game::driving::audio::EngineSound,
    /// Per layer: the decoded loop and its sample rate.
    pub voices: [Option<(std::sync::Arc<nf_audio::Pcm>, u32)>; 10],
    pub playing: [bool; 10],
    /// Opened by the window app only (`start_audio`): headless shots and tests stay silent.
    pub output: Option<nf_audio::Output>,
}

/// The player car's weapons: the game's weapon table and firing rules (nf_game::driving::weapons,
/// FUN_0020f2b8 / FUN_0020de20), the projectiles in flight and the hits they land on AI cars.
pub struct DriveWeapons {
    pub sys: nf_game::driving::weapons::WeaponSystem,
    pub shots: Vec<nf_game::driving::weapons::Projectile>,
    /// Per AI car (ai_cars order): damage taken; 700 hit points wreck a car [D driving-damage.md].
    pub ai_damage: Vec<f32>,
    /// Edge-triggered inputs the window app sets before a step.
    pub fire: bool,
    pub cycle: bool,
}

/// The track's placed glares (lamp glows, traffic lights, beacons: nf_game::driving::trackfx,
/// FUN_001f2058) and the `flar` sheet they sample.
pub struct DriveGlares {
    pub fx: nf_game::driving::trackfx::TrackFx,
    pub types: Vec<nf_game::driving::trackfx::GlareType>,
    pub tex: usize,
}

/// Live driving mission simulation state.
pub struct DrivingState {
    pub viv: String,
    pub car: Car,
    pub world: FlatWorld,
    pub rigid: Rigid,
    pub crp_bytes: Vec<u8>,
    pub road_nodes: Vec<[f32; 3]>,
    pub collision: CollisionGrid,
    pub ai_cars: Vec<(AiDriver, u32)>,
    pub player_attach_slot: Option<u32>,
    pub mission: Mission,
    pub mission_state: MissionState,
    pub triggers: Vec<MapTrigger>,
    pub triggered: Vec<bool>,
    pub progress: u32,
    pub rotor_spin: f32,
    pub wheels: [SusWheel; 4],
    pub sus_params: SusParams,
    pub ticks: u32,
    /// The driving engine's chase camera (nf_game::driving::camera, `FUN_001b2b48`) and its last view.
    pub cam: Option<(nf_game::driving::camera::ChaseCamera, nf_game::driving::camera::HeliParams, nf_game::driving::camera::ArmParams)>,
    pub cam_view: Option<nf_game::driving::camera::CamView>,
    /// The land car's ground: the track's collision triangles for `Car::step`'s four sprung wheels
    /// (the DRIVING.ELF land pass, docs/systems/drive-physics.md §8). `None` (the watercraft, unit
    /// tests) keeps the flat model + the view's own 4-wheel height follow.
    pub land: Option<nf_game::driving::CollisionGround>,
    /// The player model's bbox centre in car space: the body the land step moves is that box
    /// (`CarParams::half_extents` = the model bbox half extents [M FUN_001c9778]), so the model draws
    /// at `pos − rot·centre`.
    pub car_box_centre: [f32; 3],
    /// The player car's posed parts (wheels, hubs); `None` when the model does not decode.
    pub car_rig: Option<CarRig>,
    pub fx: Option<DriveFx>,
    /// The land car's righting reset (nf_game::driving::reset: tipped and touching for 30 ticks
    /// -> righted where it lies [M 0x188200]) and the triangles it reads.
    pub reset: Option<(nf_game::driving::reset::ResetState, Vec<CollisionTri>)>,
    pub hud: Option<DriveHud>,
    pub shine: Option<CarShine>,
    pub engine: Option<DriveEngine>,
    pub weapons: Option<DriveWeapons>,
    pub glares: Option<DriveGlares>,
    /// The traffic world the AI yields in: the junction lights' shared phase (FUN_0013ef40, one
    /// step per 300 ticks) and the world's one driving generator (*0x339190) [M].
    pub lights: nf_game::driving::ai::JunctionLights,
    pub ai_rng: nf_game::driving::ai::DriveRng,
    /// A SUB_PHYSICS player vehicle (the sub, the ultralights: FUN_00190f88 + the hull against the
    /// track, nf_game::driving::sub_physics / sub_collide); the car mirrors its body each tick.
    pub sub: Option<(nf_game::driving::sub_physics::SubCraft, nf_game::driving::sub_collide::WorldMesh, nf_game::driving::sub_collide::Building)>,
    /// The scripted control and camera layer (nf_game::driving::intro, docs/systems/driving-intros.md):
    /// the race countdown hold, scripted splines, letterbox. Fed by the mission script's events.
    pub intro: Option<(nf_game::driving::intro::IntroState, nf_formats::drivepath::MapPaths)>,
}

const IDENT_MAT: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn yaw_pos_mat(pos: [f32; 3], yaw: f32) -> Mat4 {
    let (s, c) = yaw.sin_cos();
    [
        [c, 0.0, -s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [s, 0.0, c, 0.0],
        [pos[0], pos[1], pos[2], 1.0],
    ]
}

impl DrivingState {
    pub fn step(&mut self, throttle: f32, brake: f32, steer: f32, handbrake: bool) {
        self.ticks = self.ticks.wrapping_add(1);
        // The chase camera, once per tick on the car's pose (the previous tick's: a one-tick lag vs
        // the game's end-of-frame update [S]); the pull-in ray is the collision grid's segment test.
        {
            let pose = nf_game::driving::camera::CarPose::from_yaw(self.car.pos, self.car.yaw, self.car.vel);
            if self.cam.is_none() {
                let (hp, arm) = nf_game::driving::camera::default_heli_cam(player_car_stem(&self.viv.to_ascii_uppercase()));
                let c = nf_game::driving::camera::ChaseCamera::new(&hp, &arm, &pose);
                self.cam = Some((c, hp, arm));
            }
            let grid = &self.collision;
            if let Some((c, hp, arm)) = self.cam.as_mut() {
                let mut ray = |a: [f32; 3], b: [f32; 3]| grid.segment_hit(a, b);
                self.cam_view = Some(c.update(hp, arm, &pose, false, &mut ray));
            }
        }
        let mut input = CarInput {
            throttle: throttle.clamp(0.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            steer: steer.clamp(-1.0, 1.0),
            handbrake,
        };
        // While the script owns the car (countdown hold, a spline, a forced stop, the POV mode),
        // the player's pedals and wheel do nothing [M the driver owner / EForceCarStop, intro.rs].
        if let Some((i, _)) = &self.intro {
            if !i.player_drives() {
                // A forced stop parks the car: the handbrake park, not the brake pedal (a held brake
                // turns into reverse after 30 ticks in the land step) [S: what vtable +0x274(3) does
                // inside the vehicle is not traced].
                input = CarInput { throttle: 0.0, brake: 0.0, steer: 0.0, handbrake: i.stop.is_some() };
            }
        }
        if let Some(r) = self.car_rig.as_mut() { r.tick(&self.car, input.steer); }
        if let Some((c, mesh, b)) = self.sub.as_mut() {
            let si = nf_game::driving::sub_physics::SubInput { throttle: input.throttle, brake: input.brake, steer: input.steer, handbrake: input.handbrake, ..Default::default() };
            let _ = c.step_world(&si, &self.rigid, b, mesh, DT);
            let r = c.body.rows;
            self.car.pos = c.body.pos;
            self.car.vel = c.body.vel;
            self.car.rot = r;
            self.car.yaw = r[2][0].atan2(r[2][2]);
        } else if let Some(g) = &self.land {
            // The land pass owns the height, pitch and roll (its wheels stand on the triangles).
            self.car.step(&input, g, &self.rigid, DT);
            if let Some((rs, tris)) = self.reset.as_mut().filter(|_| self.sub.is_none()) {
                if let Some(r) = nf_game::driving::reset::check(&self.car, tris, rs, None) {
                    r.apply(&mut self.car);
                }
            }
        } else {
            self.step_flat(&input);
        }
        self.step_fx();
        self.step_weapons();
        self.step_engine(input.throttle);
        self.step_ai(&input);
    }

    /// The player car's effect emitters for this tick, then the pool steps [M FUN_001ee4a0,
    /// FUN_001f3188]. The wheel inputs come from the land step: the slip ratio, the contact
    /// triangle's surface byte; [S] which vtable value each input is (driving-effects.md).
    fn step_fx(&mut self) {
        let Some(fx) = self.fx.as_mut() else { return };
        use nf_game::driving::effects::{CarFxInput, WheelFxInput};
        let car = &self.car;
        let wp = car.params.wheel_points();
        let r = car.rot;
        let wheels: [WheelFxInput; 4] = std::array::from_fn(|k| {
            let l = wp[k];
            let pos = [0, 1, 2].map(|c| car.pos[c] + r[0][c] * l[0] + r[1][c] * l[1] + r[2][c] * l[2]);
            let plane = car.wheel_cache[k];
            WheelFxInput { pos, surface: plane.map_or(1, |w| w.surface), flag_a: false, flag_b: plane.is_some(), slip: car.slip[k] }
        });
        let v = car.vel;
        let cin = CarFxInput { vel: v, axis: r[2], speed: (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(), no_snow: false, attach: 0, attach_pos: car.pos };
        fx.veh.update(self.land.is_some(), &cin, &wheels, self.ticks, false, &fx.lib);
        fx.veh.tick(1, &fx.lib, &mut fx.rng, &mut fx.pool);
        fx.pool.run_steps(self.ticks, &fx.lib);
    }

    /// The weapons for one tick: cycle / fire the selected secondary (the game's switch delay,
    /// refire interval and ammo), launch the projectiles from the car's nose along its forward
    /// axis, fly them (homing onto the nearest AI car ahead for the seekers) and land hits on AI
    /// cars within 2 u [S: the game's hit test is the car box; the sphere stands in].
    fn step_weapons(&mut self) {
        use nf_game::driving::weapons as w;
        let now = self.ticks as i32;
        let car_pos = self.car.pos;
        let fwd = self.car.rot[2];
        let speed = (self.car.vel[0].powi(2) + self.car.vel[1].powi(2) + self.car.vel[2].powi(2)).sqrt();
        let ai: Vec<([f32; 3], [f32; 3])> = self.ai_cars.iter().map(|(d, _)| (d.car.pos, d.car.vel)).collect();
        let Some(wp) = self.weapons.as_mut() else { return };
        if std::mem::take(&mut wp.cycle) { wp.sys.select(true, now, 60, false); }
        if std::mem::take(&mut wp.fire) {
            let k = wp.sys.secondary.max(0) as usize;
            if let w::FireResult::Fired { barrels, .. } = wp.sys.fire(k, now, &w::FireCtx { barrel_ready: true, ..Default::default() }) {
                for _ in 0..barrels.len().max(1) {
                    let pp = w::launch_params(k, speed, 0.0, 60, w::Launcher::Player);
                    let pos = [car_pos[0] + fwd[0] * 2.5, car_pos[1] + fwd[1] * 2.5 + 0.5, car_pos[2] + fwd[2] * 2.5];
                    wp.shots.push(w::Projectile::launch(k, pp, pos, fwd, now));
                }
            }
        }
        if wp.ai_damage.len() != ai.len() { wp.ai_damage.resize(ai.len(), 0.0); }
        let mut keep = Vec::new();
        for mut sh in std::mem::take(&mut wp.shots) {
            let target = ai.iter().filter(|(p, _)| {
                let d = [p[0] - sh.pos[0], p[1] - sh.pos[1], p[2] - sh.pos[2]];
                d[0] * sh.fwd[0] + d[1] * sh.fwd[1] + d[2] * sh.fwd[2] > 0.0
            }).min_by(|a, b| {
                let da = (a.0[0] - sh.pos[0]).powi(2) + (a.0[2] - sh.pos[2]).powi(2);
                let db = (b.0[0] - sh.pos[0]).powi(2) + (b.0[2] - sh.pos[2]).powi(2);
                da.total_cmp(&db)
            }).map(|(p, v)| w::Target { pos: *p, vel: *v, prox_kind: false });
            sh.step(target, now, DT, 60);
            let hit = ai.iter().position(|(p, _)| (p[0] - sh.pos[0]).powi(2) + (p[1] - sh.pos[1]).powi(2) + (p[2] - sh.pos[2]).powi(2) < 4.0);
            if let Some(i) = hit {
                wp.ai_damage[i] += sh.p.damage.max(1.0);
                continue;
            }
            if sh.detonate || now - sh.born > sh.p.life.max(60) { continue; }
            keep.push(sh);
        }
        wp.shots = keep;
    }

    /// Opens the audio output for the engine sound (the window app calls this; nothing else does).
    pub fn start_audio(&mut self) {
        if nf_audio::silent() { return; }
        if let Some(e) = self.engine.as_mut() {
            if e.output.is_none() { e.output = nf_audio::Output::open(); }
        }
    }

    /// One tick of the engine layers: the game's rpm/load slew and the ten voices' volume and
    /// pitch, sent as SPU voice commands (pitch 0x1000 = 48 kHz). The chase camera listens from
    /// outside (distance 2: the exterior layers) [M audio.rs Listener default].
    fn step_engine(&mut self, throttle: f32) {
        let ticks = self.ticks;
        let rpm = self.car.rpm * 1.25; // the player's bridge factor [M 0x188200:589]
        let Some(e) = self.engine.as_mut() else { return };
        e.snd.base_update(rpm, 0.0, 0.0, throttle, false, ticks);
        let f = e.snd.layer_update(ticks, &nf_game::driving::audio::Listener::default(), 1.0);
        let Some(out) = e.output.as_ref() else { return };
        let mut cmds = Vec::new();
        for (k, v) in f.voices.iter().enumerate() {
            let Some((pcm, rate)) = &e.voices[k] else { continue };
            let pitch = ((v.pitch * *rate as f32 / 48_000.0) * 4096.0).clamp(0.0, 0x3fff as f32) as u32;
            let vol = v.volume.clamp(0.0, 1.0);
            if vol > 0.0 && !e.playing[k] {
                cmds.push(nf_audio::Cmd::Play { voice: 0xd000 + k as u32, pcm: pcm.clone(), pitch, left: vol, right: vol, delay: 0.0, reverb: false });
                e.playing[k] = true;
            } else if vol > 0.0 {
                cmds.push(nf_audio::Cmd::Set { voice: 0xd000 + k as u32, pitch, left: vol, right: vol });
            } else if e.playing[k] {
                cmds.push(nf_audio::Cmd::Stop { voice: 0xd000 + k as u32 });
                e.playing[k] = false;
            }
        }
        out.send(cmds);
    }

    /// This frame's HUD overlay: the ported passes' sprites on the game's 640x448 screen, kept 4:3
    /// at the window's height (the on-foot HUD's mapping). Text and panels wait for the driving
    /// font (not drawn yet) [S].
    pub fn hud_frame(&mut self, scene: &Scene, aspect: f32) -> crate::gpu::HudFrame {
        let mut f = crate::gpu::HudFrame::default();
        let ticks = self.ticks;
        let Some(h) = self.hud.as_mut() else { return f };
        let (hp, hm, ar, am) = h.health.hud();
        let input = nf_game::driving_hud_draw::HudInput {
            dt: ticks.saturating_sub(h.last_tick) as f32 / 60.0,
            health: hp, health_max: hm, armour: ar, armour_max: am,
            weapon: self.weapons.as_ref().and_then(|w| { let k = w.sys.secondary; (k >= 0).then(|| (k, w.sys.slots[k as usize].ammo)) }),
            timer: None, health_always: false, grunt_on_hit: false, suppressed: false,
            ticks, paused: false,
        };
        h.last_tick = ticks;
        let out = h.draw.frame(&h.layout, &input, nf_game::driving_hud_draw::HudStrings { font: &h.font, loc: &h.loc });
        let s = 2.0 / 640.0 * (4.0 / 3.0) / aspect.max(1e-3);
        let ndc = |x: f32, y: f32| [(x - 320.0) * s, 1.0 - y / 224.0];
        for d in out.draws {
            let nf_game::driving_hud_draw::HudDraw::Sprite { x, y, w, h: hh, uv, sheet, colour, .. } = d else { continue };
            let Some(&(_, hash)) = h.sheets.iter().find(|(i, _)| *i == sheet) else { continue };
            let Some(ti) = scene.hud_textures.iter().position(|(k, _)| *k == hash) else { continue };
            // GS colour word (R G B A bytes, 0x80 = 1.0).
            let c = [0, 8, 16, 24].map(|b| ((colour >> b) & 0xff) as f32 / 128.0);
            let c = [c[0], c[1], c[2], c[3].min(1.0)];
            let (a, b) = (ndc(x, y), ndc(x + w, y + hh));
            let v = |p: [f32; 2], t: [f32; 2]| crate::gpu::HudVertex { p, uv: t, c };
            let first = f.v.len() as u32;
            f.v.extend_from_slice(&[v([a[0], a[1]], [uv[0], uv[1]]), v([b[0], a[1]], [uv[2], uv[1]]), v([b[0], b[1]], [uv[2], uv[3]]), v([a[0], a[1]], [uv[0], uv[1]]), v([b[0], b[1]], [uv[2], uv[3]]), v([a[0], b[1]], [uv[0], uv[3]])]);
            f.runs.push((ti + 1, 0, first, 6));
        }
        // The letterbox bars: y 0 and y 448 - h, the HUD width, colour 0x7f000000 [M 0x245190].
        if let Some(bh) = self.intro.as_ref().and_then(|(i, _)| i.letterbox.bar_height(false)) {
            if bh > 0.0 {
                let c = [0.0, 0.0, 0.0, 1.0];
                for (y0, y1) in [(0.0, bh), (448.0 - bh, 448.0)] {
                    let (a, b) = ([-1.0, 1.0 - y0 / 224.0], [1.0, 1.0 - y1 / 224.0]);
                    let v = |p: [f32; 2]| crate::gpu::HudVertex { p, uv: [0.0, 0.0], c };
                    let first = f.v.len() as u32;
                    f.v.extend_from_slice(&[v([a[0], a[1]]), v([b[0], a[1]]), v([b[0], b[1]]), v([a[0], a[1]]), v([b[0], b[1]]), v([a[0], b[1]])]);
                    f.runs.push((0, 0, first, 6));
                }
            }
        }
        f
    }

    /// The player car's specular triangles this frame (world space) for the additive overlay.
    fn shine_group(&self, right: [f32; 3], up: [f32; 3]) -> Option<(Option<usize>, bool, Vec<Vertex>)> {
        let sh = self.shine.as_ref()?;
        let tex = sh.tex?;
        // The view's forward axis (camera + 0x20, into the scene): up x right for this basis.
        let f = [up[1] * right[2] - up[2] * right[1], up[2] * right[0] - up[0] * right[2], up[0] * right[1] - up[1] * right[0]];
        let basis = sh.light.specular_basis(f);
        let r = self.car.rot;
        let c = self.car_box_centre;
        let place = |q: [f32; 3]| [0, 1, 2].map(|k| self.car.pos[k] + r[0][k] * (q[0] - c[0]) + r[1][k] * (q[1] - c[1]) + r[2][k] * (q[2] - c[2]));
        let mut out = Vec::new();
        for (st, props) in &sh.strips {
            let pv = st.pass_vertices(nf_formats::carmesh::CarPassKind::Specular, &r, &basis, &sh.entry, *props);
            if pv.len() != st.positions.len() || pv.len() < 3 { continue; }
            let vert = |k: usize| { let g = pv[k].rgb.min(255) as u8; Vertex { pos: place(st.positions[k]), uv: pv[k].st, rgba: [g, g, g, 128], nrm: [0; 4] } };
            for k in 0..pv.len() - 2 {
                out.extend_from_slice(&[vert(k), vert(k + 1), vert(k + 2)]);
            }
        }
        Some((Some(tex), true, out))
    }

    /// This frame's particle billboards, grouped by (texture, additive) for the renderer.
    pub fn particle_groups(&mut self, right: [f32; 3], up: [f32; 3]) -> Vec<(Option<usize>, bool, Vec<Vertex>)> {
        let mut shine: Vec<(Option<usize>, bool, Vec<Vertex>)> = self.shine_group(right, up).into_iter().collect();
        // The projectiles in flight: a small additive glow each [S: the game draws the rocket /
        // missile models and their trails; the models are not decoded yet].
        if let Some(w) = self.weapons.as_ref().filter(|w| !w.shots.is_empty()) {
            let mut v = Vec::new();
            for sh in &w.shots {
                let h = 0.35;
                let c = |sx: f32, sy: f32| [0, 1, 2].map(|k| sh.pos[k] + right[k] * h * sx + up[k] * h * sy);
                let q = |p: [f32; 3]| Vertex { pos: p, uv: [0.0, 0.0], rgba: [255, 180, 80, 128], nrm: [0; 4] };
                let (a, b, cc, d) = (q(c(-1.0, 1.0)), q(c(1.0, 1.0)), q(c(1.0, -1.0)), q(c(-1.0, -1.0)));
                v.extend_from_slice(&[a, b, cc, a, cc, d]);
            }
            shine.push((None, true, v));
        }
        // The track glares: halo and spike sprites per kept glare, additive (ALPHA Cs*As + Cd
        // [M FUN_001f14b0]) on the `flar` sheet [M FUN_001f2058].
        if let (Some(g), Some(v)) = (self.glares.as_ref(), self.cam_view.as_ref()) {
            let frame = g.fx.glares(self.ticks, v.eye, v.fwd, 1.0, &g.types, |_| true);
            let mut out = Vec::new();
            for sp in &frame.sprites {
                let h = sp.width * 0.5;
                let (sn, cs) = sp.rotation.sin_cos();
                let ax = [0, 1, 2].map(|k| (right[k] * cs + up[k] * sn) * h);
                let ay = [0, 1, 2].map(|k| (up[k] * cs - right[k] * sn) * h);
                let c = |sx: f32, sy: f32| [0, 1, 2].map(|k| sp.centre[k] + ax[k] * sx + ay[k] * sy);
                let rgba = [0, 8, 16, 24].map(|b| ((sp.rgba >> b) & 0xff) as u8);
                let (u0, v0, sz) = (sp.cell[0], sp.cell[1], sp.cell[2]);
                let q = |p: [f32; 3], u: f32, vv: f32| Vertex { pos: p, uv: [u, vv], rgba, nrm: [0; 4] };
                let (a, b, cc, d) = (q(c(-1.0, 1.0), u0, v0), q(c(1.0, 1.0), u0 + sz, v0), q(c(1.0, -1.0), u0 + sz, v0 + sz), q(c(-1.0, -1.0), u0, v0 + sz));
                out.extend_from_slice(&[a, b, cc, a, cc, d]);
            }
            if !out.is_empty() { shine.push((Some(g.tex), true, out)); }
        }
        let shine = shine;
        let Some(fx) = self.fx.as_mut() else { return shine };
        let mut groups: Vec<(Option<usize>, bool, Vec<Vertex>)> = shine;
        for d in fx.pool.draws(&fx.lib, |_| None) {
            let nf_game::driving::effects::ParticleDraw::Quad { centre, size, rotation, cell, rgba, additive } = d else { continue };
            let h = size * 0.5;
            let (sn, cs) = rotation.sin_cos();
            let ax = [0, 1, 2].map(|k| (right[k] * cs + up[k] * sn) * h);
            let ay = [0, 1, 2].map(|k| (up[k] * cs - right[k] * sn) * h);
            let corner = |sx: f32, sy: f32| [0, 1, 2].map(|k| centre[k] + ax[k] * sx + ay[k] * sy);
            let (u0, v0, uv) = (cell[0], cell[1], cell[2]);
            let vert = |p: [f32; 3], u: f32, v: f32| Vertex { pos: p, uv: [u, v], rgba, nrm: [0; 4] };
            let (a, b, c, e) = (vert(corner(-1.0, 1.0), u0, v0), vert(corner(1.0, 1.0), u0 + uv, v0), vert(corner(1.0, -1.0), u0 + uv, v0 + uv), vert(corner(-1.0, -1.0), u0, v0 + uv));
            let g = match groups.iter().position(|(t, ad, _)| *t == fx.main_tex && *ad == additive) {
                Some(g) => g,
                None => { groups.push((fx.main_tex, additive, Vec::new())); groups.len() - 1 }
            };
            groups[g].2.extend_from_slice(&[a, b, c, a, c, e]);
        }
        groups
    }

    fn step_flat(&mut self, input: &CarInput) {
        let y_before = self.car.pos[1];
        self.car.step(input, &self.world, &self.rigid, DT);
        // `Car::step` floors the height at 0 (its flat-ground model); on tracks that lie below 0
        // (MIS13C's river at y ~-587) that snaps the car up out of the world, out of the ground
        // follow's -12..+3 window. Keep the height the car had; the collision follow below sets it.
        if y_before < 0.0 && self.car.pos[1] == 0.0 { self.car.pos[1] = y_before; }

        // The boundary sheets (DRIVING.ELF 0x331a60) are the SUB_PHYSICS pass (FUN_00190f88 via
        // FUN_00188200 0x188bf8), not the land car's: the vanquish's .atr sets no SUB_PHYSICS, so
        // no sheet pass runs here (docs/systems/driving-world-bounds.md).

        // Ground height from 4-wheel CDat collision raycasts + SusWheel suspension (fallback to RoadNet).
        let (s, c) = self.car.yaw.sin_cos();
        let wheel_offsets = [
            [-0.85f32, self.car.params.wheel_zf_offset],
            [0.85, self.car.params.wheel_zf_offset],
            [-0.85, self.car.params.wheel_zr_offset],
            [0.85, self.car.params.wheel_zr_offset],
        ];
        let mut hit_sum = 0.0f32;
        let mut hit_cnt = 0u32;
        for (wi, &[lx, lz]) in wheel_offsets.iter().enumerate() {
            let wx = self.car.pos[0] + lx * c + lz * s;
            let wz = self.car.pos[2] - lx * s + lz * c;
            let hy = self.collision.ground_y(wx, self.car.pos[1], wz);
            let ray_t = hy.map(|y| (self.car.pos[1] - y).max(0.0));
            let _ = self.wheels[wi].step(0.0, ray_t, &self.sus_params, axle_of(wi));
            self.wheels[wi].end_tick();
            if let Some(y) = hy {
                hit_sum += y;
                hit_cnt += 1;
            }
        }
        if hit_cnt > 0 {
            self.car.pos[1] = hit_sum / hit_cnt as f32;
        } else if let Some(hy) = self.collision.ground_y(self.car.pos[0], self.car.pos[1], self.car.pos[2]) {
            self.car.pos[1] = hy;
        } else if !self.road_nodes.is_empty() {
            let p = self.car.pos;
            let mut best_d2 = f32::MAX;
            let mut best_y = p[1];
            for n in &self.road_nodes {
                let dx = n[0] - p[0];
                let dz = n[2] - p[2];
                let d2 = dx * dx + dz * dz;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best_y = n[1];
                }
            }
            self.car.pos[1] = best_y;
        }
    }

    fn step_ai(&mut self, input: &CarInput) {
        // Step AI traffic cars.
        if !self.ai_cars.is_empty() {
            if let Ok(crp) = Crp::parse(&self.crp_bytes) {
                if let Ok(rn) = RoadNet::parse_crp(&crp) {
                    let now = self.ticks as i32;
                    self.lights.tick(now);
                    // Every vehicle the yield looks at: the traffic cars and the player
                    // (kind < 4, status 2 = live) [M 0x135530 / 0x1377c8].
                    let veh = |pos: [f32; 3], h: [f32; 3]| nf_game::driving::ai::Vehicle { pos, box_x: h[0], box_z: h[2], kind: 1, status: 2, enabled: true, state: 4, alive: 1.0, lane: 0, end: 0 };
                    let mut vehicles: Vec<nf_game::driving::ai::Vehicle> = self.ai_cars.iter().map(|(d, _)| veh(d.car.pos, d.car.params.half_extents)).collect();
                    vehicles.push(veh(self.car.pos, self.car.params.half_extents));
                    let lights = self.lights.clone();
                    for (k, (drv, _)) in self.ai_cars.iter_mut().enumerate() {
                        let world = nf_game::driving::ai::YieldWorld { now, lights: lights.clone(), vehicles: &vehicles, me: Some(k), light_objects: &[] };
                        let _ = drv.step_shared(&rn, DT, Some(&world), &mut self.ai_rng);
                        if let Some(hy) = self.collision.ground_y(drv.car.pos[0], drv.car.pos[1], drv.car.pos[2]) {
                            drv.car.pos[1] = hy;
                        }
                    }
                }
            }
        }

        // Watercraft (`MIS11`, `MIS13C`) and aerial rotor (`MIS13A`, `MIS13B`) passes.
        let mut speed = (self.car.vel[0].powi(2) + self.car.vel[2].powi(2)).sqrt();
        match self.viv.as_str() {
            "MIS11" | "MIS13C" => {
                let thrust = watercraft_longitudinal_thrust(
                    input.throttle,
                    self.car.params.maxacc,
                    input.brake,
                    self.car.params.maxbrake,
                );
                let drag = watercraft_handbrake_drag(
                    speed,
                    self.car.params.handbrake_force,
                    input.handbrake,
                );
                let (sy, cy) = self.car.yaw.sin_cos();
                self.car.vel[0] += sy * thrust * DT * 0.25;
                self.car.vel[2] += cy * thrust * DT * 0.25;
                if drag > 0.0 && speed > 1e-4 {
                    let damp = (1.0 - (drag * DT / speed).min(1.0)).max(0.0);
                    self.car.vel[0] *= damp;
                    self.car.vel[2] *= damp;
                }
                speed = (self.car.vel[0].powi(2) + self.car.vel[2].powi(2)).sqrt();
            }
            "MIS13A" | "MIS13B" => {
                self.rotor_spin = rotor_wheelspin(speed, self.car.params.wheel_radius, DT);
            }
            _ => {}
        }

        // Checkpoint proximity detection against `Map` `Trgr` volumes.
        for (i, t) in self.triggers.iter().enumerate() {
            if self.triggered.get(i).copied().unwrap_or(true) {
                continue;
            }
            let dx = self.car.pos[0] - t.pos[0];
            let dy = self.car.pos[1] - t.pos[1];
            let dz = self.car.pos[2] - t.pos[2];
            let r = t.radius.max(4.0);
            if dx * dx + dy * dy + dz * dz <= r * r {
                self.triggered[i] = true;
                self.progress = self.progress.saturating_add(1);
            }
        }

        // Step mission rules & timers.
        let snap = SimSnapshot {
            health: 100.0,
            damage: 0.0,
            speed,
            pos: self.car.pos,
            died: None,
            collided: false,
            progress: self.progress,
            difficulty: 1,
        };
        if self.mission.script.is_some() {
            let h = self.car.params.half_extents;
            let facts = nf_game::driving::script::Facts {
                cars: self.ai_cars.iter().map(|(d, _)| nf_game::driving::script::CarFacts {
                    kind: 0, // traffic: neither 1 (player) nor 2 [S: +0x6d of a traffic car not traced]
                    pos: d.car.pos,
                    vel: d.car.vel,
                    radius: (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt(),
                    // The traffic cars' own boxes are not built here: the sphere stand-in [S].
                    bbox: None,
                }).collect(),
                player_pos: self.car.pos,
                player_vel: self.car.vel,
                // +0x7c = |half extents| [M 0x1f6e98].
                player_radius: (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt(),
                // The player body's oriented box: its rotation rows and the model-box half extents
                // [M 0x22eb28 -> FUN_001f5cf0].
                player_box: Some(nf_game::driving::script::CarBox { axes: self.car.rot, half: h }),
                difficulty: 1,
                ..Default::default()
            };
            let _ = snap;
            self.mission.step_facts(&facts);
            if let (Some(sc), Some((i, paths))) = (self.mission.script.as_mut(), self.intro.as_mut()) {
                for o in sc.take_output() {
                    match o {
                        nf_game::driving::script::Output::StartMission { .. } => i.start_mission(),
                        nf_game::driving::script::Output::Effect { action, .. } => i.apply(&action, &sc.map, paths),
                        _ => {}
                    }
                }
                i.out.clear();
                i.step();
                // A scripted spline carries the car (the path player's row 3 into the car matrix [S]).
                if let Some(q) = i.spline_position(paths) {
                    let o = self.car.pos;
                    self.car.vel = [(q[0] - o[0]) * 60.0, (q[1] - o[1]) * 60.0, (q[2] - o[2]) * 60.0];
                    self.car.pos = q;
                }
            }
        } else {
            let _events = self.mission.step(DT, &snap);
        }
        self.mission_state = self.mission.state;
    }

    /// Write player and AI car world transforms into `Scene` attachment palette slots.
    pub fn apply_to_palette(&self, attach_base: usize, pal: &mut [Mat4]) {
        if let Some(slot) = self.player_attach_slot {
            let car_m: Mat4 = if self.land.is_some() || self.sub.is_some() {
                let r = self.car.rot;
                let c = self.car_box_centre;
                let o = [0, 1, 2].map(|k| self.car.pos[k] - (r[0][k] * c[0] + r[1][k] * c[1] + r[2][k] * c[2]));
                [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0], [o[0], o[1], o[2], 1.0]]
            } else {
                yaw_pos_mat(self.car.pos, self.car.yaw)
            };
            // The posed parts: part-local -> car (the rig's run-time matrix) -> world, row vectors.
            if let Some(r) = &self.car_rig {
                for &(pi, ps) in &r.parts {
                    let Some(part) = r.mesh.parts.get(pi) else { continue };
                    let Some(m) = pal.get_mut(attach_base + ps as usize) else { continue };
                    *m = match nf_formats::carpose::part_matrix(part, &r.pose) {
                        nf_formats::carpose::PartMatrix::Drawn(pm) => mat_mul(&pm, &car_m),
                        // A part the rig hides (or does not evaluate) folds away.
                        _ => [[0.0; 4]; 4],
                    };
                }
            }
            if let Some(m) = pal.get_mut(attach_base + slot as usize) {
                *m = if self.land.is_some() || self.sub.is_some() {
                    let r = self.car.rot;
                    let c = self.car_box_centre;
                    let o = [0, 1, 2].map(|k| self.car.pos[k] - (r[0][k] * c[0] + r[1][k] * c[1] + r[2][k] * c[2]));
                    [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0], [o[0], o[1], o[2], 1.0]]
                } else {
                    yaw_pos_mat(self.car.pos, self.car.yaw)
                };
            }
        }
        for (drv, slot) in &self.ai_cars {
            if let Some(m) = pal.get_mut(attach_base + *slot as usize) {
                *m = yaw_pos_mat(drv.car.pos, drv.car.yaw);
            }
        }
    }

    /// Third-person chase camera behind the player car: `(pos, yaw, pitch)`.
    pub fn chase_camera(&self) -> ([f32; 3], f32, f32) {
        if let Some(v) = &self.cam_view {
            let yaw = v.fwd[0].atan2(v.fwd[2]);
            let pitch = v.fwd[1].clamp(-1.0, 1.0).asin();
            return (v.eye, yaw, pitch);
        }
        let yaw = self.car.yaw;
        let fwd = [yaw.sin(), 0.0, yaw.cos()];
        let cam_pos = [
            self.car.pos[0] - fwd[0] * 6.5,
            self.car.pos[1] + 2.4,
            self.car.pos[2] - fwd[2] * 6.5,
        ];
        (cam_pos, yaw, -0.14)
    }
}

/// The lowest car-space y of attachment `slot`'s vertices.
fn scene_char_min_y(cv: &[CharVertex], slot: u32) -> Option<f32> {
    cv.iter().filter(|v| v.attachment == Some(slot)).map(|v| v.pos[1]).reduce(f32::min)
}

/// Row-vector product: `v·a·b` = `v·(a·b)`.
fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut o = [[0.0f32; 4]; 4];
    for i in 0..4 { for j in 0..4 { o[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum(); } }
    o
}

fn maybe_decompress(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb {
        Ok(refpack::decompress(raw)?)
    } else {
        Ok(raw.to_vec())
    }
}

/// Triangulates a PS2 triangle strip into `indices`, skipping strip-break jumps
/// whose edges exceed `sqrt(max_edge_sq)`.
fn push_strip_tris(
    positions: &[[f32; 3]],
    base_v: u32,
    max_edge_sq: f32,
    indices: &mut Vec<u32>,
) {
    let dist_sq = |a: [f32; 3], b: [f32; 3]| -> f32 {
        (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
    };
    for k in 0..positions.len().saturating_sub(2) {
        let (p0, p1, p2) = (positions[k], positions[k + 1], positions[k + 2]);
        if dist_sq(p0, p1) > max_edge_sq
            || dist_sq(p1, p2) > max_edge_sq
            || dist_sq(p0, p2) > max_edge_sq
        {
            continue;
        }
        let (i0, i1, i2) = if k % 2 == 0 {
            (base_v + k as u32, base_v + k as u32 + 1, base_v + k as u32 + 2)
        } else {
            (base_v + k as u32 + 1, base_v + k as u32, base_v + k as u32 + 2)
        };
        indices.push(i0);
        indices.push(i1);
        indices.push(i2);
    }
}

fn load_car_into_scene(
    iso: &IsoImage,
    viv_base: u64,
    car_stem: &str,
    crp_off_sz: (u32, u32),
    ssh_off_sz: Option<(u32, u32)>,
    vertices: &mut Vec<Vertex>,
    char_verts: &mut Vec<CharVertex>,
    indices: &mut Vec<u32>,
    materials: &mut Vec<Material>,
    textures: &mut Vec<TextureImage>,
    attachments: &mut Vec<Attachment>,
    light: Option<(&nf_formats::rendertun::LightEnv, &nf_formats::rendertun::CarRender)>,
    mut posed: Option<&mut Vec<(usize, u32)>>,
) -> Option<u32> {
    let crp_raw = iso
        .read_bytes(viv_base + crp_off_sz.0 as u64, crp_off_sz.1 as u64)
        .ok()?;
    let crp_bytes = maybe_decompress(&crp_raw).ok()?;
    let car_crp = Crp::parse(&crp_bytes).ok()?;
    let car_mod = TrackModule::load(&car_crp).ok()?;

    let mut car_tex_by_name: HashMap<String, usize> = HashMap::new();
    if let Some((ssh_off, ssh_sz)) = ssh_off_sz {
        if let Ok(ssh_raw) = iso.read_bytes(viv_base + ssh_off as u64, ssh_sz as u64) {
            if let Ok(ssh_bytes) = maybe_decompress(&ssh_raw) {
                if let Ok(ssh) = Ssh::parse(&ssh_bytes) {
                    for t in ssh.textures {
                        let idx = textures.len();
                        car_tex_by_name.insert(t.name.to_ascii_lowercase(), idx);
                        textures.push(TextureImage {
                            key: (0, idx as u32),
                            width: t.width,
                            height: t.height,
                            rgba: t.rgba,
                            more: Vec::new(),
                            rate: 1,
                        });
                    }
                }
            }
        }
    }

    // The car's parts through the corrected draw chain (nf_formats::carmesh: each placement names its
    // model set [M DRIVING.ELF 0x1ad2e0 +0x1c], the vd lists hold placement indices [M 0x1ad758]),
    // every drawn part placed in car space; normals ride the vertex nrm (the car batches carry no
    // vertex colours [D]). The old reader stays as the fallback when the car file does not decode.
    // The base-pass car light (docs/formats/car-model.md §10, bondrm.o microcode):
    // min(255, 127·(D·L1·max(0,n·d1) + L2·max(0,n·d2) + Ground·max(0,n·up) + A·Sky)) with the strip's
    // CarRender class (A, D) for this vehicle, PS2 scale (0x80 = 1.0). [S] the World light set, shade
    // 1.0 (its writer is not traced), the normals at the car's own frame (the colours are baked once,
    // not re-lit as the car turns); the specular and reflection passes are not drawn.
    let entry = light.and_then(|(_, cr)| cr.entries.iter().find(|e| e.name.eq_ignore_ascii_case(car_stem)).or_else(|| cr.entries.iter().find(|e| e.name == "default")));
    let mut strip_rgba: Vec<Vec<[u8; 4]>> = Vec::new();
    // With `posed`, the Skel-attached parts (wheels, hubs) keep their part-local positions and get a
    // palette slot each, posed per tick by `CarRig` (`(part index, slot)` out); the rest bake.
    let mut strip_part: Vec<Option<usize>> = Vec::new();
    let want_posed = posed.is_some();
    let strips: Vec<nf_formats::crp_mesh::MeshStrip> = match nf_formats::carmesh::car_mesh(&crp_bytes) {
        Ok(car) => {
            let mut v = Vec::new();
            // The list the game draws in normal play (driver kind 0 = the player's car, others 2+:
            // both pick vd#1 outside the dashboard camera and AutoDrive) and each part at its
            // run-time matrix at rest (nf_formats::carpose; docs/formats/car-model.md §11) [M].
            // The wheels' spin/steer/spring pose is baked at rest here [S: per-tick part pose not drawn].
            let state = nf_formats::carpose::DrawState { owner: nf_formats::carpose::DrawOwner::Vehicle { driver_kind: 0 }, interior_view: false, auto_drive: false };
            let pose = nf_formats::carpose::CarPose::rest(&car);
            for part in car.list_for(&state).into_iter().filter(|part| !part.hidden()) {
                let m = match nf_formats::carpose::part_matrix(part, &pose) {
                    nf_formats::carpose::PartMatrix::Drawn(m) => m,
                    nf_formats::carpose::PartMatrix::NotDrawn => continue,
                    nf_formats::carpose::PartMatrix::Unported(_) => nf_formats::carpose::own_matrix(part),
                };
                let live = want_posed && matches!(part.attach, nf_formats::carmesh::Attach::Skel(_));
                let place = |q: [f32; 3]| if live { q } else { [0, 1, 2].map(|k| q[0] * m[0][k] + q[1] * m[1][k] + q[2] * m[2][k] + m[3][k]) };
                for st in &part.strips {
                    let rgba: Vec<[u8; 4]> = match (light, entry, st.lighting) {
                        (Some((env, _)), Some(e), Some(class)) if st.normals.len() == st.positions.len() => {
                            let pr = e.props(class);
                            st.normals.iter().map(|&n| {
                                let c = env.car_light(n, pr.ambient, pr.diffuse, 1.0);
                                // GS RGBAQ = min(255, 127·L) as stored; Vertex.rgba keeps the PS2 scale (the shader maps 0x80 to 1.0).
                                let b = |x: f32| (127.0 * x).clamp(0.0, 255.0) as u8;
                                [b(c[0]), b(c[1]), b(c[2]), 255]
                            }).collect()
                        }
                        _ => Vec::new(),
                    };
                    strip_rgba.push(rgba);
                    strip_part.push(live.then_some(part.index));
                    v.push(nf_formats::crp_mesh::MeshStrip {
                        positions: st.positions.iter().map(|&q| place(q)).collect(),
                        uvs: st.uvs.clone(),
                        colors: st.normals.iter().map(|n| [(n[0] * 127.0) as i8 as u8, (n[1] * 127.0) as i8 as u8, (n[2] * 127.0) as i8 as u8]).collect(),
                        texture: st.texture.clone(),
                    });
                }
            }
            v
        }
        Err(_) => car_mod.extract_car_mesh(&car_crp, car_stem),
    };
    if strips.is_empty() {
        return None;
    }

    let slot = attachments.len() as u32;
    let new_attachment = || Attachment {
        character: 0,
        bone: 0,
        local: IDENT_MAT,
        flash: None,
        tracer: None,
        projectile: None,
        dropped: false,
        script: None,
        aims_light: None,
        gas: None,
        effect: None,
        silencer: false,
        quiver_bolt: None,
        swap_piece: false,
        watch_hand: None,
        pda_digit: None,
    };
    attachments.push(Attachment {
        character: 0,
        bone: 0,
        local: IDENT_MAT,
        flash: None,
        tracer: None,
        projectile: None,
        dropped: false,
        script: None,
        aims_light: None,
        gas: None,
        effect: None,
        silencer: false,
        quiver_bolt: None,
        swap_piece: false,
        watch_hand: None,
        pda_digit: None,
    });

    for (si, strip) in strips.into_iter().enumerate() {
        if strip.positions.len() < 3 {
            continue;
        }
        let slot = match (strip_part.get(si).copied().flatten(), posed.as_deref_mut()) {
            (Some(pi), Some(out)) => match out.iter().find(|(q, _)| *q == pi) {
                Some(&(_, s)) => s,
                None => {
                    let s = attachments.len() as u32;
                    attachments.push(new_attachment());
                    out.push((pi, s));
                    s
                }
            },
            _ => slot,
        };
        let base_v = vertices.len() as u32;
        let first_i = indices.len() as u32;
        for (k, &p) in strip.positions.iter().enumerate() {
            let uv = strip.uvs.get(k).copied().unwrap_or([0.0, 0.0]);
            let n = strip.colors.get(k).copied().unwrap_or([0, 127, 0]);
            vertices.push(Vertex {
                pos: p,
                uv,
                rgba: {
                    let c = strip_rgba.get(si).and_then(|v| v.get(k)).copied().unwrap_or([128, 128, 128, 255]);
                    // An untextured strip (Gouraud/Dash, TME off) shows the vertex colour itself (255 = full),
                    // not texture × colour/128: halve it so the shader's 0x80 = 1.0 scaling lands there.
                    if strip.texture.is_none() { [c[0] / 2, c[1] / 2, c[2] / 2, c[3]] } else { c }
                },
                nrm: [n[0] as i8, n[1] as i8, n[2] as i8, 0],
            });
            char_verts.push(CharVertex {
                character: 0,
                pos: p,
                weight: 1.0,
                slots: [0, 0],
                attachment: Some(slot),
                searchlight: None,
                door: None,
                pickup: None,
                morph: -1,
            });
        }
        push_strip_tris(&strip.positions, base_v, 3.5 * 3.5, indices);
        let index_count = indices.len() as u32 - first_i;
        if index_count == 0 {
            continue;
        }
        let tex_key = strip
            .texture
            .as_ref()
            .and_then(|t| car_tex_by_name.get(&t.to_ascii_lowercase()).copied())
            .map(|idx| (0usize, idx as u32));
        materials.push(Material {
            layer: Layer::Character,
            draw: Draw {
                pass: 1,
                blend: 0,
                zwrite: true,
                atest: 0,
                fog: 0,
            },
            centre: [0.0, 0.0, 0.0],
            radius: 10_000.0,
            cel: None,
            chain: u32::MAX,
            node: u32::MAX,
            key: tex_key,
            lflags: 0,
            skin: None,
            sky_slot: None,
            first_index: first_i,
            index_count,
        });
    }
    Some(slot)
}

/// Load a driving track `.crp` + `.ssh` + player/AI car models + mission rules from `/DRIVING/<viv>.VIV`.
pub fn load_driving_scene(iso_path: &str, viv_name: &str) -> Result<(Scene, DrivingState)> {
    let viv_upper = viv_name
        .trim_end_matches(".VIV")
        .trim_end_matches(".viv")
        .to_ascii_uppercase();
    let iso = IsoImage::open(iso_path)?;
    let entry = iso
        .find(&format!("/DRIVING/{viv_upper}.VIV"))
        .with_context(|| format!("missing /DRIVING/{viv_upper}.VIV"))?;
    let bigf = disc::read_bigf(&iso, entry)?;
    let viv_base = entry.lba as u64 * 2048;

    let crp_entry = bigf
        .entries
        .iter()
        .find(|x| {
            x.name.to_ascii_lowercase().contains("data\\track\\")
                && x.name.to_ascii_lowercase().ends_with(".crp")
        })
        .with_context(|| format!("no track .crp in {viv_upper}.VIV"))?;
    let crp_raw = iso.read_bytes(viv_base + crp_entry.offset as u64, crp_entry.size as u64)?;
    let crp_bytes = maybe_decompress(&crp_raw)?;
    let crp = Crp::parse(&crp_bytes)?;

    let mut textures = Vec::new();
    let mut tex_by_name: HashMap<String, usize> = HashMap::new();
    if let Some(ssh_entry) = bigf.entries.iter().find(|x| {
        x.name.to_ascii_lowercase().contains("data\\track\\")
            && x.name.to_ascii_lowercase().ends_with(".ssh")
    }) {
        let ssh_raw = iso.read_bytes(viv_base + ssh_entry.offset as u64, ssh_entry.size as u64)?;
        let ssh_bytes = maybe_decompress(&ssh_raw)?;
        if let Ok(ssh) = Ssh::parse(&ssh_bytes) {
            for (idx, t) in ssh.textures.into_iter().enumerate() {
                tex_by_name.insert(t.name.to_ascii_lowercase(), idx);
                textures.push(TextureImage {
                    key: (0, idx as u32),
                    width: t.width,
                    height: t.height,
                    rgba: t.rgba,
                    more: Vec::new(),
                    rate: 1,
                });
            }
        }
    }

    let module = TrackModule::load(&crp)?;
    let articles = crp.track_articles()?;

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];

    let push_strip = |strip: &nf_formats::crp_mesh::MeshStrip,
                      inst: Option<&nf_formats::crp_mesh::MapInstance>,
                      vertices: &mut Vec<Vertex>,
                      indices: &mut Vec<u32>,
                      materials: &mut Vec<Material>,
                      bmin: &mut [f32; 3],
                      bmax: &mut [f32; 3]| {
        if strip.positions.len() < 3 {
            return;
        }
        let base_v = vertices.len() as u32;
        let first_i = indices.len() as u32;
        let mut smin = [f32::MAX; 3];
        let mut smax = [f32::MIN; 3];
        for (k, &lp) in strip.positions.iter().enumerate() {
            let p = inst.map_or(lp, |m| m.transform(lp));
            for a in 0..3 {
                bmin[a] = bmin[a].min(p[a]);
                bmax[a] = bmax[a].max(p[a]);
                smin[a] = smin[a].min(p[a]);
                smax[a] = smax[a].max(p[a]);
            }
            let uv = strip.uvs.get(k).copied().unwrap_or([0.0, 0.0]);
            let n = strip.colors.get(k).copied().unwrap_or([0, 127, 0]);
            vertices.push(Vertex {
                pos: p,
                uv,
                // A strip with no colour upload: neutral 0x80 = 1.0 on the PS2 scale [S: what the
                // microcode's colour slot holds without an upload is not traced].
                rgba: [128, 128, 128, 255],
                nrm: [n[0] as i8, n[1] as i8, n[2] as i8, 0],
            });
        }
        push_strip_tris(&strip.positions, base_v, 40.0 * 40.0, indices);
        let index_count = indices.len() as u32 - first_i;
        if index_count == 0 {
            return;
        }
        let centre = [
            0.5 * (smin[0] + smax[0]),
            0.5 * (smin[1] + smax[1]),
            0.5 * (smin[2] + smax[2]),
        ];
        let radius = ((smax[0] - centre[0]).powi(2)
            + (smax[1] - centre[1]).powi(2)
            + (smax[2] - centre[2]).powi(2))
        .sqrt()
        .max(1.0);
        let tex_key = strip
            .texture
            .as_ref()
            .and_then(|t| tex_by_name.get(&t.to_ascii_lowercase()).copied())
            .map(|idx| (0usize, idx as u32));

        materials.push(Material {
            layer: Layer::World,
            draw: Draw {
                pass: 1,
                blend: 0,
                zwrite: true,
                atest: 0,
                fog: 0,
            },
            centre,
            radius,
            cel: None,
            chain: u32::MAX,
            node: u32::MAX,
            key: tex_key,
            lflags: 0,
            skin: None,
            sky_slot: None,
            first_index: first_i,
            index_count,
        });
    };

    let map_instances = crp.extract_map_instances().unwrap_or_default();
    // The track's drawn geometry: every Map in#0 placement's baked model, road, ground, props
    // and sky alike (nf_formats::geoprim, docs/formats/geoprim.md; the forward placement
    // matrices [M DRIVING.ELF 0x22faf8]).
    let geometry = nf_formats::geoprim::track_geometry(&crp_bytes)?;
    let placements = nf_formats::geoprim::map_placements(&crp).unwrap_or_default();
    // Draw batching: strips sorted by (sky, texture, 64-u map cell) so that each run of one
    // texture in one cell merges into a single draw below (paris: ~58,700 strips) [port choice].
    let cell_of = |ws: &nf_formats::geoprim::WorldStrip| -> (i32, i32) {
        let p = ws.positions.first().copied().unwrap_or([0.0; 3]);
        ((p[0] / 64.0).floor() as i32, (p[2] / 64.0).floor() as i32)
    };
    let mut strips = geometry.strips;
    strips.sort_by(|a, b| {
        let ka = (a.article.to_ascii_lowercase().starts_with("sky"), a.texture.clone(), cell_of(a));
        let kb = (b.article.to_ascii_lowercase().starts_with("sky"), b.texture.clone(), cell_of(b));
        ka.cmp(&kb)
    });
    let m0 = materials.len();
    let mut group: Vec<(bool, Option<String>, (i32, i32))> = Vec::new();
    for ws in strips {
        let gkey = (ws.article.to_ascii_lowercase().starts_with("sky"), ws.texture.clone(), cell_of(&ws));
        // The sky articles (`sky*`) are a small dome (paris: +-30 u as placed, docs/formats/geoprim.md):
        // drawn around the viewer like the on-foot camera skies (Layer::SkyCamera, before the world,
        // no depth write) with their vertices relative to their placement [S: the driving engine's
        // sky pass is not traced].
        let sky = ws.article.to_ascii_lowercase().starts_with("sky");
        let origin = if sky { placements.get(ws.placement).map_or([0.0; 3], |p| p.origin) } else { [0.0; 3] };
        let positions = if sky { ws.positions.iter().map(|q| [q[0] - origin[0], q[1] - origin[1], q[2] - origin[2]]).collect() } else { ws.positions };
        let vcols = ws.colors;
        let strip = nf_formats::crp_mesh::MeshStrip { positions, uvs: ws.uvs, colors: Vec::new(), texture: ws.texture };
        let n = materials.len();
        let (b0, b1) = (bmin, bmax);
        let v0 = vertices.len();
        push_strip(&strip, None, &mut vertices, &mut indices, &mut materials, &mut bmin, &mut bmax);
        // The part's per-vertex colours light the track (the night lighting is baked into them;
        // PS2 scale, 0x80 = 1.0) [D docs/formats/geoprim.md; the microcode's pass-through is unproven].
        if !vcols.is_empty() && vertices.len() - v0 == vcols.len() {
            for (v, c) in vertices[v0..].iter_mut().zip(&vcols) {
                // Vertex.rgba keeps the PS2 scale; the shader maps 0x80 to 1.0 (scene.rs module doc).
                v.rgba = [c[0], c[1], c[2], 255];
            }
        }
        if sky {
            (bmin, bmax) = (b0, b1); // the dome does not widen the track bounds
            for m in &mut materials[n..] { m.layer = Layer::SkyCamera; m.draw.pass = 0; m.draw.zwrite = false; m.radius = 1.0e6; }
        }
        for _ in n..materials.len() { group.push(gkey.clone()); }
    }
    // Merge consecutive same-group materials whose index ranges touch (one draw per texture per cell).
    {
        let mut out: Vec<Material> = Vec::with_capacity(materials.len() - m0);
        let mut out_g: Vec<(bool, Option<String>, (i32, i32))> = Vec::new();
        for (m, g) in materials.drain(m0..).zip(group.into_iter()) {
            if let (Some(last), Some(lg)) = (out.last_mut(), out_g.last()) {
                if *lg == g && last.key == m.key && last.layer == m.layer && last.first_index + last.index_count == m.first_index {
                    // Bounding sphere of both.
                    let d = [m.centre[0] - last.centre[0], m.centre[1] - last.centre[1], m.centre[2] - last.centre[2]];
                    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if dist + m.radius > last.radius {
                        if dist + last.radius <= m.radius { last.centre = m.centre; last.radius = m.radius; }
                        else if dist > 0.0 {
                            let r = 0.5 * (dist + last.radius + m.radius);
                            let t = (r - last.radius) / dist;
                            for k in 0..3 { last.centre[k] += d[k] * t; }
                            last.radius = r;
                        }
                    }
                    last.index_count += m.index_count;
                    continue;
                }
            }
            out.push(m);
            out_g.push(g);
        }
        materials.extend(out);
    }

    let col_tris = crp.extract_collision_tris().unwrap_or_default();

    if bmin[0] > bmax[0] {
        bmin = [-100.0; 3];
        bmax = [100.0; 3];
    }

    // The ground query skips class-bit-0x04 triangles [M DRIVING.ELF 0x2138b0].
    let collision = CollisionGrid::build(col_tris.iter().filter(|t| !t.ground_query_skips()).cloned().collect());

    // Parse RoadNet & compute player spawn.
    let mut road_nodes = Vec::new();
    let mut start_pos = [
        0.5 * (bmin[0] + bmax[0]),
        bmin[1] + 2.0,
        0.5 * (bmin[2] + bmax[2]),
    ];
    let mut start_yaw = 0.0f32;
    let rn_opt = RoadNet::parse_crp(&crp).ok();
    if let Some(rn) = &rn_opt {
        road_nodes = rn.nodes.iter().map(|n| n.pos()).collect();
        if let Some(p0) = road_nodes.first().copied() {
            start_pos = p0;
            if let Some(p1) = road_nodes.get(1).copied() {
                start_yaw = (p1[0] - p0[0]).atan2(p1[2] - p0[2]);
            }
        }
    }
    // The game's start: the one Map Trgr whose action list holds EStartMission (0x16), fired at
    // world init [M DRIVING.ELF 0x22cd30, 0x15c920]; the car goes to its x/z at the ground under
    // it + 1.0, facing its +0x30 direction [M 0x15c738, 0x1fee68] (nf_formats::trackstart).
    // The road-node pick above stays only as the fallback when no such trigger decodes.
    if let Ok(sp) = nf_formats::trackstart::track_spawn(&crp, &col_tris) {
        start_pos = sp.pos;
        start_yaw = sp.yaw;
    }

    let find_car_entry = |stem: &str| -> Option<((u32, u32), Option<(u32, u32)>)> {
        let target_crp = format!("data\\car\\model\\{stem}.crp");
        let crp_e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&target_crp))?;
        let target_ssh = format!("data\\car\\model\\{stem}.ssh");
        let ssh_e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&target_ssh));
        Some(((crp_e.offset, crp_e.size), ssh_e.map(|s| (s.offset, s.size))))
    };

    // Load player & AI car 3D meshes into Scene dynamic attachment slots.
    let char_vertex_start = vertices.len();
    let mut char_verts = Vec::new();
    let mut attachments = Vec::new();
    // The mission's light set (Render/Lighting/<track>.tun, World) and the car material table
    // (Render/CarRender/default.tun) for the base-pass car light.
    let read_tun = |want: &str| -> Option<String> {
        let e = bigf.entries.iter().find(|x| { let n = x.name.to_ascii_lowercase(); n.contains("render") && n.contains(want) && n.ends_with(".tun") })?;
        let raw = iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok()?;
        maybe_decompress(&raw).ok().map(|t| String::from_utf8_lossy(&t).into_owned())
    };
    let car_lighting = read_tun("lighting").and_then(|t| nf_formats::rendertun::Lighting::parse(&t).ok());
    let car_render = read_tun("carrender").and_then(|t| nf_formats::rendertun::CarRender::parse(&t).ok());
    let car_light_ref = car_lighting.as_ref().zip(car_render.as_ref()).map(|(l, c)| (&l.world, c));
    let p_stem = player_car_stem(&viv_upper);
    let mut posed_parts: Vec<(usize, u32)> = Vec::new();
    let player_attach_slot = find_car_entry(p_stem).and_then(|(c, s)| {
        load_car_into_scene(
            &iso,
            viv_base,
            p_stem,
            c,
            s,
            &mut vertices,
            &mut char_verts,
            &mut indices,
            &mut materials,
            &mut textures,
            &mut attachments,
            car_light_ref,
            Some(&mut posed_parts),
        )
    });

    // Load CarParams (.atr) and Rigid (.tun).
    let car_params = bigf
        .entries
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case(&format!("data\\sim\\attrib\\pvehicle\\{p_stem}.atr")))
        .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| CarParams::parse_atr(&s))
        .unwrap_or_else(CarParams::vanquish);

    let rigid = bigf
        .entries
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case("data\\tuning\\Physics\\Rigid\\default.tun"))
        .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| Rigid::from_tun(&Tuning::parse(&s)))
        .unwrap_or_else(Rigid::game_defaults);

    // Spawn up to 4 AI traffic cars on RoadNet segments.
    let mut ai_cars = Vec::new();
    if let Some(rn) = &rn_opt {
        if rn.segs.len() >= 4 {
            let traffic_stem = bigf
                .entries
                .iter()
                .filter_map(|e| {
                    let low = e.name.to_ascii_lowercase();
                    let rest = low.strip_prefix("data\\car\\model\\")?.strip_suffix(".crp")?;
                    if rest != p_stem { Some(rest.to_string()) } else { None }
                })
                .next();
            if let Some(t_stem) = traffic_stem {
                if let Some((c, s)) = find_car_entry(&t_stem) {
                    // The traffic car's own attributes (data/sim/attrib/pvehicle/<model>.atr, shipped
                    // beside each model [D]: civilian steering 0.2 against the player's 0.09); the
                    // player's settings only when the file is missing.
                    let t_params = bigf
                        .entries
                        .iter()
                        .find(|x| x.name.eq_ignore_ascii_case(&format!("data\\sim\\attrib\\pvehicle\\{t_stem}.atr")))
                        .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
                        .and_then(|raw| maybe_decompress(&raw).ok())
                        .map(|b| CarParams::parse_atr(&String::from_utf8_lossy(&b)))
                        .unwrap_or_else(|| car_params.clone());
                    let step_seg = (rn.segs.len() / 5).max(1);
                    for k in 1..=4 {
                        let seg = (k * step_seg) % rn.segs.len();
                        if let Ok(drv) = AiDriver::on_segment(t_params.clone(), rn, seg, 0, k as i8) {
                            if let Some(slot) = load_car_into_scene(
                                &iso,
                                viv_base,
                                &t_stem,
                                c,
                                s,
                                &mut vertices,
                                &mut char_verts,
                                &mut indices,
                                &mut materials,
                                &mut textures,
                                &mut attachments,
                                car_light_ref,
                                None,
                            ) {
                                ai_cars.push((drv, slot));
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse mission Rule / MSet from track .crp.
    let rule_bytes = crp
        .find(0x5275_6c65)
        .ok()
        .flatten()
        .or_else(|| crp.pool_find(0x5275_6c65).ok().flatten())
        .and_then(|r| crp.payload(&r).ok())
        .unwrap_or(&[]);
    let mset_bytes = crp
        .find(0x4d53_6574)
        .ok()
        .flatten()
        .or_else(|| crp.pool_find(0x4d53_6574).ok().flatten())
        .and_then(|r| crp.payload(&r).ok());
    // The track's mission script runs the game's own rules (triggers, events, rules, timers,
    // win/lose: nf_game::driving::script, docs/systems/driving-missions.md); the rule-bytes model
    // stays the fallback when the script does not read.
    let mission = match nf_formats::trackstart::map_script(&crp) {
        Ok(ms) => Mission::from_map_script(ms, drive_index_for_viv(&viv_upper)),
        Err(_) => Mission::load(rule_bytes, mset_bytes, drive_index_for_viv(&viv_upper)),
    };
    let mission_state = mission.state;

    // Extract `Map` `Trgr` checkpoint triggers and emit subtle translucent ground beacon rings
    // (spaced >=35m apart, capped at 16, using normal alpha blend so they never blow out the scene).
    let triggers = crp.extract_map_triggers().unwrap_or_default();
    let triggered = vec![false; triggers.len()];
    {
        const SEGMENTS: usize = 12;
        const BEACON_H: f32 = 0.8;
        let first_i = indices.len() as u32;
        let mut placed_beacons: Vec<[f32; 3]> = Vec::new();
        for t in &triggers {
            if !(2.0..=12.0).contains(&t.radius) {
                continue;
            }
            if placed_beacons.iter().any(|p| {
                let dx = p[0] - t.pos[0];
                let dz = p[2] - t.pos[2];
                dx * dx + dz * dz < 35.0 * 35.0
            }) {
                continue;
            }
            placed_beacons.push(t.pos);
            let r = t.radius.clamp(2.5, 6.0);
            let bv = vertices.len() as u32;
            for s in 0..SEGMENTS {
                let ang = (s as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
                let (sa, ca) = ang.sin_cos();
                let x = t.pos[0] + ca * r;
                let z = t.pos[2] + sa * r;
                vertices.push(Vertex {
                    pos: [x, t.pos[1], z],
                    uv: [0.0, 0.0],
                    rgba: [40, 220, 255, 96],
                    nrm: [0, 127, 0, 0],
                });
                vertices.push(Vertex {
                    pos: [x, t.pos[1] + BEACON_H, z],
                    uv: [0.0, 1.0],
                    rgba: [255, 220, 60, 24],
                    nrm: [0, 127, 0, 0],
                });
            }
            for s in 0..SEGMENTS {
                let s1 = (s + 1) % SEGMENTS;
                let b0 = bv + (s * 2) as u32;
                let t0 = b0 + 1;
                let b1 = bv + (s1 * 2) as u32;
                let t1 = b1 + 1;
                indices.extend_from_slice(&[b0, b1, t0, t0, b1, t1]);
            }
            if placed_beacons.len() >= 16 {
                break;
            }
        }
        let index_count = indices.len() as u32 - first_i;
        if index_count > 0 {
            let centre = [
                0.5 * (bmin[0] + bmax[0]),
                0.5 * (bmin[1] + bmax[1]),
                0.5 * (bmin[2] + bmax[2]),
            ];
            let radius = ((bmax[0] - centre[0]).powi(2)
                + (bmax[1] - centre[1]).powi(2)
                + (bmax[2] - centre[2]).powi(2))
            .sqrt()
            .max(1000.0);
            materials.push(Material {
                layer: Layer::World,
                draw: Draw {
                    pass: 2,
                    blend: 2,
                    zwrite: false,
                    atest: 0,
                    fog: 0,
                },
                centre,
                radius,
                cel: None,
                chain: u32::MAX,
                node: u32::MAX,
                key: None,
                lflags: 0,
                skin: None,
                sky_slot: None,
                first_index: first_i,
                index_count,
            });
        }
    }

    let stats = format!(
        "{viv_upper}: {} articles, {} verts, {} tris, {} textures, {} col tris",
        articles.len(),
        vertices.len(),
        indices.len() / 3,
        textures.len(),
        collision.tris.len()
    );
    let mut scene = Scene::from_static_world(
        vertices,
        indices,
        materials,
        textures,
        (bmin, bmax),
        Some((start_pos, start_yaw)),
        stats,
    );
    scene.char_vertex_start = char_vertex_start;
    scene.char_verts = char_verts;
    scene.attachments = attachments;
    // The driving engine's world is left-handed against the on-foot renderer (the car's number plate
    // and the track's sign text read mirrored otherwise): the view mirrors screen x.
    scene.mirror_x = true;
    // The mission's fog (the Render/Fog/<track>.tun tuning text, nf_formats::rendertun): its colour is the
    // frame's background where nothing draws (the open sky overhead, past the sky bands), as the
    // on-foot levels clear to theirs. The track's draws keep fog 0, so only the background changes.
    if let Some(e) = bigf.entries.iter().find(|x| { let n = x.name.to_ascii_lowercase(); n.contains("render") && n.contains("fog") && n.ends_with(".tun") }) {
        if let Ok(raw) = iso.read_bytes(viv_base + e.offset as u64, e.size as u64) {
            if let Ok(t) = maybe_decompress(&raw) {
                if let Ok(f) = nf_formats::rendertun::Fog::parse(&String::from_utf8_lossy(&t)) { scene.fog = Some((f.colour, f.end)); }
            }
        }
    }

    // The land pass for land vehicles (the watercraft MIS11/MIS13C and the aerial MIS13A/MIS13B keep
    // the flat model and their own passes: their SUB_PHYSICS steps are not ported).
    let land = (!matches!(viv_upper.as_str(), "MIS11" | "MIS13C" | "MIS13A" | "MIS13B")).then(|| nf_game::driving::CollisionGround::new(col_tris.clone()));
    // The body box = the player model's article `Base` bbox (model+0xc [M FUN_001c9778], the
    // half extents the land step's hull, wheel points and inertia use); the model draws at
    // `pos − rot·centre`. Falls back to the .atr-side defaults when the box does not read.
    let mut car_params = car_params;
    let mut car_box_centre = [0.0f32; 3];
    let model_box = find_car_entry(p_stem)
        .and_then(|((o, n), _)| iso.read_bytes(viv_base + o as u64, n as u64).ok())
        .and_then(|raw| maybe_decompress(&raw).ok())
        .and_then(|b| Crp::parse(&b).ok().and_then(|c| c.track_articles().ok()))
        .and_then(|arts| arts.into_iter().find(|a| (0..3).all(|k| a.bbox_max[k] > a.bbox_min[k])));
    if let Some(a) = model_box {
        car_params.half_extents = [0, 1, 2].map(|k| (a.bbox_max[k] - a.bbox_min[k]) * 0.5);
    }
    // Where the drawn model sits on that body [S: the game's render offset is not traced]: its
    // lowest vertex at the body's rest contact (`pos.y − rest height`), so the tyres touch the
    // road when the springs rest.
    if let Some(slot) = player_attach_slot {
        let vmin = scene_char_min_y(&scene.char_verts, slot);
        if let Some(v) = vmin { car_box_centre = [0.0, v + car_params.half_extents[1] + car_params.spring_rest, 0.0]; }
    }
    // The effects: the ELF's particle types and the track's particle sheet (`MAIN` atlas) [D].
    let fx = (|| {
        let e = iso.find("/DRIVING.ELF")?;
        let elf = iso.read_bytes(e.lba as u64 * 2048, e.size as u64).ok()?;
        let lib = nf_game::driving::effects::ParticleLib::from_elf_types(&nf_formats::drivefx::elf_types(&elf).ok()?);
        let sheet = match viv_upper.as_str() { "MIS11" => "partic02", "MIS13A" | "MIS13B" | "MIS13C" => "ears_partic01", "MIS3" | "MIS4" => "sav_partic01", _ => "partic01" };
        let want = ["data", "render", &format!("{sheet}.ssh")].join("\\");
        let main_tex = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&want))
            .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
            .and_then(|raw| maybe_decompress(&raw).ok())
            .and_then(|b| Ssh::parse(&b).ok())
            .and_then(|sh| sh.textures.into_iter().find(|t| t.name == "MAIN"))
            .map(|t| {
                let i = scene.textures.len();
                scene.textures.push(TextureImage { key: (0, 0xfee0_0000 | i as u32), width: t.width, height: t.height, rgba: t.rgba, more: Vec::new(), rate: 1 });
                i
            });
        Some(DriveFx { veh: nf_game::driving::effects::VehicleFx::new(&lib), lib, pool: Default::default(), rng: nf_game::driving::effects::DriveRng::seeded(0), main_tex })
    })();
    let car_rig = (!posed_parts.is_empty())
        .then(|| find_car_entry(p_stem))
        .flatten()
        .and_then(|((o, n), _)| iso.read_bytes(viv_base + o as u64, n as u64).ok())
        .and_then(|raw| maybe_decompress(&raw).ok())
        .and_then(|b| nf_formats::carmesh::car_mesh(&b).ok())
        .map(|mesh| {
            let rig = nf_formats::carpose::WheelRig::new(&mesh, false);
            let pose = nf_formats::carpose::CarPose::rest(&mesh);
            let attrs = nf_formats::carpose::WheelAttrs {
                rest: car_params.spring_rest,
                limit_draw: car_params.spring_limit,
                wheel_radius: car_params.wheel_radius,
                is_sub: false,
            };
            CarRig { mesh, rig, attrs, mem: Default::default(), pose, spin: [0.0; 2], parts: posed_parts.clone() }
        });
    // The engine sound: the track's bank list (banks.ini) picks the Engine slot's bank; its .h
    // names the ten layer sounds (ENGINE_VOICES; the vanquish has no midload pair [D]).
    let engine = (|| {
        let read = |n: &str| -> Option<Vec<u8>> {
            let e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(n))?;
            maybe_decompress(&iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok()?).ok()
        };
        let p = |parts: &[&str]| parts.join("\\");
        let track = crp_entry.name.rsplit(|c: char| c == '/' || c as u32 == 0x5c).next()?.trim_end_matches(".crp").to_ascii_lowercase();
        let secs = nf_formats::easnd::parse_banks_ini(&String::from_utf8_lossy(&read(&p(&["data", "audio", "banks.ini"]))?));
        let (_, path) = nf_game::driving::audio::track_banks(&secs, &track).into_iter().find(|(s, _)| *s == nf_game::driving::audio::ENGINE_SLOT)?;
        // banks.ini writes `cars/vanquish.bnk`; the archive names use backslashes [D].
        let bnk_name = p(&["data", "audio", &path.to_ascii_lowercase().replace('/', &p(&["", ""]))]);
        let bank_bytes = read(&bnk_name)?;
        let table = nf_formats::easnd::SfxTable::from_header(&String::from_utf8_lossy(&read(&bnk_name.replace(".bnk", ".h"))?));
        let bank = nf_formats::easnd::Bank::parse(&bank_bytes).ok()?;
        let voices = nf_game::driving::audio::ENGINE_VOICES.map(|k| {
            let id = table.get(k)? as usize;
            let patch = bank.slots.get(id)?.as_ref()?.first()?;
            let pcm = nf_formats::easnd::Bank::decode(&bank_bytes, patch).ok()?;
            let ls = (patch.loop_start >= 0 && (patch.loop_start as usize) < pcm.len()).then_some(patch.loop_start as usize).or(Some(0));
            Some((std::sync::Arc::new(nf_audio::Pcm { data: pcm, loop_start: ls }), patch.rate.max(1)))
        });
        voices.iter().any(|v| v.is_some()).then(|| DriveEngine {
            snd: nf_game::driving::audio::EngineSound::new(nf_game::driving::audio::PLAYER_ENGINE_RPM),
            voices, playing: [false; 10], output: None,
        })
    })();
    // A SUB_PHYSICS spawn vehicle (the mission set's MSet name [M 0x22faf8 -> FUN_00176658]):
    // its own pass and the hull against the track (MIS11 vanquishsub, MIS13B ultralight [D]).
    let sub = (|| {
        let stem = nf_formats::trackstart::mission_vehicle(&crp).ok()??;
        let want = ["data", "sim", "attrib", "pvehicle", &format!("{stem}.atr")].join("\\");
        let e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&want))?;
        let atr = maybe_decompress(&iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok()?).ok()?;
        let attrs = nf_game::driving::sub_physics::VehicleAttrs::parse_atr(&String::from_utf8_lossy(&atr));
        if nf_game::driving::sub_physics::drive_pass(&attrs, 0, false) != nf_game::driving::sub_physics::DrivePass::Sub { return None; }
        let sp = nf_formats::trackstart::track_spawn(&crp, &col_tris).ok()?;
        let mut c = nf_game::driving::sub_physics::SubCraft::new(attrs, car_params.half_extents, sp.pos, sp.yaw, 0);
        c.start_mission(sp.dir, sp.speed_factor);
        let mesh = nf_game::driving::sub_collide::WorldMesh::new(col_tris.iter().map(|t| (t.v, t.flags)));
        Some((c, mesh, nf_game::driving::sub_collide::Building::game_defaults()))
    })();
    // The sub's body is its box centre and the model's Base box is centred on the origin.
    let car_box_centre = if sub.is_some() { [0.0; 3] } else { car_box_centre };
    let car = Car::new(car_params, start_pos, start_yaw);
    let reset = land.is_some().then(|| (nf_game::driving::reset::ResetState::new(&car), col_tris.clone()));
    // The car's specular pass: its Specular-pass strips at rest (car space), the CarRender entry,
    // the World light and `carS` from data\render\common.ssh [M carmesh.rs CarMaterial::pass_slots].
    let shine = (|| {
        let light = car_lighting.as_ref()?.world.clone();
        let cr = car_render.as_ref()?;
        let entry = cr.entries.iter().find(|e| e.name.eq_ignore_ascii_case(p_stem)).or_else(|| cr.entries.iter().find(|e| e.name == "default"))?.clone();
        let ((o, n), _) = find_car_entry(p_stem)?;
        let mesh = nf_formats::carmesh::car_mesh(&maybe_decompress(&iso.read_bytes(viv_base + o as u64, n as u64).ok()?).ok()?).ok()?;
        let state = nf_formats::carpose::DrawState { owner: nf_formats::carpose::DrawOwner::Vehicle { driver_kind: 0 }, interior_view: false, auto_drive: false };
        let pose = nf_formats::carpose::CarPose::rest(&mesh);
        let mut strips = Vec::new();
        for part in mesh.list_for(&state).into_iter().filter(|p| !p.hidden()) {
            let nf_formats::carpose::PartMatrix::Drawn(m) = nf_formats::carpose::part_matrix(part, &pose) else { continue };
            for st in &part.strips {
                let Some(class) = st.lighting else { continue };
                if !st.passes().iter().any(|p| p.kind == nf_formats::carmesh::CarPassKind::Specular) { continue; }
                let mut s2 = st.clone();
                s2.positions = st.positions.iter().map(|&q| [0, 1, 2].map(|k| q[0] * m[0][k] + q[1] * m[1][k] + q[2] * m[2][k] + m[3][k])).collect();
                s2.normals = st.normals.iter().map(|&q| { let v = [0, 1, 2].map(|k| q[0] * m[0][k] + q[1] * m[1][k] + q[2] * m[2][k]); let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6); v.map(|x| x / l) }).collect();
                strips.push((s2, entry.props(class)));
            }
        }
        let want = ["data", "render", "common.ssh"].join("\\");
        let tex = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&want))
            .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
            .and_then(|raw| maybe_decompress(&raw).ok())
            .and_then(|b| Ssh::parse(&b).ok())
            .and_then(|sh| sh.textures.into_iter().find(|t| t.name == "carS"))
            .map(|t| { let i = scene.textures.len(); scene.textures.push(TextureImage { key: (0, 0xfee1_0000 | i as u32), width: t.width, height: t.height, rgba: t.rgba, more: Vec::new(), rate: 1 }); i });
        Some(CarShine { strips, entry, light, tex })
    })();
    // A file from this mission's archive, else from MISC.VIV (the string tables live there [D]),
    // matched by its path suffix.
    let read_any = |suffix: &str| -> Option<Vec<u8>> {
        let suf = suffix.to_ascii_lowercase();
        let from = |g: &nf_formats::bigf::Bigf, base: u64| g.entries.iter().find(|x| x.name.to_ascii_lowercase().ends_with(&suf))
            .and_then(|e| iso.read_bytes(base + e.offset as u64, e.size as u64).ok())
            .and_then(|raw| maybe_decompress(&raw).ok());
        from(&bigf, viv_base).or_else(|| {
            let m = iso.find("/DRIVING/MISC.VIV")?;
            let g = disc::read_bigf(&iso, m).ok()?;
            from(&g, m.lba as u64 * 2048)
        })
    };
    // The HUD gallery (data\gfx\data\hudNTSC.gal, every driving archive [D]) and its sheets.
    let hud = (|| {
        let want = ["data", "gfx", "data", "hudNTSC.gal"].join("\\");
        let e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&want))?;
        let data = maybe_decompress(&iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok()?).ok()?;
        let gal = nf_formats::gal::Gal::parse(&data).ok()?;
        let layout = nf_game::driving_hud_draw::HudLayout::from_gal(&gal, false, 448.0).ok()?;
        let mut sheets = Vec::new();
        for (idx, _, r) in &gal.sheets {
            let Some(t) = data.get(r.clone()).and_then(|b| Ssh::parse(b).ok()).and_then(|sh| sh.textures.into_iter().max_by_key(|t| t.width * t.height)) else { continue };
            // (each sheet also carries 1x1 entries; the atlas the elements' UVs index is the big one [D])
            let hash = 0xd71d_0000 | *idx as u32;
            scene.hud_textures.push((hash, TextureImage { key: (0, hash), width: t.width, height: t.height, rgba: t.rgba, more: Vec::new(), rate: 1 }));
            sheets.push((*idx as u32, hash));
        }
        let set = mset_bytes.map(nf_game::driving::damage::HealthSet::parse).unwrap_or_default();
        let health = nf_game::driving::damage::PlayerHealth::start(&set, &nf_game::driving::damage::StartInputs { difficulty: 1, ..Default::default() });
        let font = nf_formats::sfn::Font::parse(&read_any(&["data", "loading", "loading.sfn"].join("\\"))?).ok()?;
        let loc = nf_formats::loc::Loc::parse(&read_any("IDL_ENG.loc")?).ok()?;
        Some(DriveHud { layout, draw: nf_game::driving_hud_draw::DriveHudDraw::new(), sheets, health, last_tick: 0, font, loc })
    })();
    let state = DrivingState {
        fx,
        reset,
        hud,
        shine,
        engine,
        glares: (|| {
            let placed = nf_formats::trackfx::placed_effects(&crp).ok()?;
            let gt = read_any(&["Render", "Glare", "default.tun"].join("\\"))?;
            let types = nf_game::driving::trackfx::glare_types(&nf_formats::drivefx::parse_glare(&String::from_utf8_lossy(&gt)));
            let mut lib = nf_game::driving::effects::ParticleLib::default();
            let fx = nf_game::driving::trackfx::TrackFx::new(&placed, |_| None, &mut lib);
            if fx.glares.is_empty() { return None; }
            let flar = ["partic01.ssh", "partic02.ssh", "ears_partic01.ssh", "sav_partic01.ssh", "common.ssh"].iter().find_map(|n| {
                read_any(n).and_then(|b| Ssh::parse(&b).ok()).and_then(|sh| sh.textures.into_iter().find(|t| t.name == "flar"))
            })?;
            let tex = scene.textures.len();
            scene.textures.push(TextureImage { key: (0, 0xfee2_0000 | tex as u32), width: flar.width, height: flar.height, rgba: flar.rgba, more: Vec::new(), rate: 1 });
            Some(DriveGlares { fx, types, tex })
        })(),
        lights: nf_game::driving::ai::JunctionLights::new(0),
        ai_rng: nf_game::driving::ai::DriveRng::game_reset(),
        weapons: Some({
            let mut sys = nf_game::driving::weapons::WeaponSystem::new(60, false);
            nf_game::driving::weapons::default_loadout(&mut sys, false);
            DriveWeapons { sys, shots: Vec::new(), ai_damage: Vec::new(), fire: false, cycle: false }
        }),
        sub,
        intro: nf_formats::drivepath::map_paths(&crp).ok().map(|p| (nf_game::driving::intro::IntroState::new(60), p)),
        viv: viv_upper,
        car,
        world: FlatWorld::paved(),
        rigid,
        crp_bytes,
        road_nodes,
        collision,
        ai_cars,
        player_attach_slot,
        mission,
        mission_state,
        triggers,
        triggered,
        progress: 0,
        rotor_spin: 0.0,
        wheels: [SusWheel::default(); 4],
        sus_params: SusParams::VANQUISH,
        ticks: 0,
        cam: None,
        cam_view: None,
        land,
        car_box_centre,
        car_rig,
    };
    Ok((scene, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_all_driving_mission_codes() {
        assert_eq!(driving_viv_for_level(0x0900_0001), Some("MIS01"));
        assert_eq!(driving_viv_for_level(0x0900_0005), Some("MIS3"));
        assert_eq!(driving_viv_for_level(0x0900_0006), Some("MIS4"));
        assert_eq!(driving_viv_for_level(0x0900_0002), Some("MIS11"));
        assert_eq!(driving_viv_for_level(0x0900_0003), Some("MIS13A"));
        assert_eq!(driving_viv_for_level(0x0900_0008), Some("RACE"));
    }

    #[test]
    fn collision_grid_raycasts_sloped_triangle() {
        let tri = CollisionTri {
            v: [[0.0, 0.0, 0.0], [10.0, 5.0, 0.0], [0.0, 0.0, 10.0]],
            flags: 1,
        };
        let grid = CollisionGrid::build(vec![tri]);
        let y = grid.ground_y(5.0, 3.0, 2.0).expect("hit sloped tri");
        assert!((y - 2.5).abs() < 1e-4);
    }

    #[test]
    fn checkpoints_and_watercraft_rotor_passes_step_cleanly() {
        let mut state = DrivingState {
            viv: "MIS11".into(),
            car: Car::new(CarParams::vanquish(), [0.0, 0.0, 0.0], 0.0),
            world: FlatWorld::paved(),
            rigid: Rigid::game_defaults(),
            crp_bytes: Vec::new(),
            road_nodes: Vec::new(),
            collision: CollisionGrid::build(Vec::new()),
            ai_cars: Vec::new(),
            player_attach_slot: None,
            mission: Mission::load(&[], None, 3),
            mission_state: MissionState::Playing,
            triggers: vec![MapTrigger {
                pos: [0.0, 0.0, 1.0],
                radius: 4.0,
                flags: 0x0407_0101,
                id: 0x656c_0001,
            }],
            triggered: vec![false],
            progress: 0,
            rotor_spin: 0.0,
            wheels: [SusWheel::default(); 4],
            sus_params: SusParams::VANQUISH,
            ticks: 0,
            cam: None,
            cam_view: None,
            land: None,
            car_box_centre: [0.0; 3],
            car_rig: None,
            fx: None,
            reset: None,
            hud: None,
            shine: None,
            engine: None,
            weapons: None,
            glares: None,
            lights: nf_game::driving::ai::JunctionLights::new(0),
            ai_rng: nf_game::driving::ai::DriveRng::game_reset(),
            sub: None,
            intro: None,
        };
        state.step(1.0, 0.0, 0.0, false);
        assert_eq!(state.progress, 1);
        assert!(state.triggered[0]);
        assert!(state.car.vel[2] > 0.0);

        // The wheel roll is turns per frame at the integer frame rate, floored at 0.01 and capped
        // at 1.0 [M DRIVING.ELF 0x192498:126-142]: a car two ticks off the line rolls under the
        // floor, so build speed first.
        state.viv = "MIS13A".into();
        for _ in 0..120 { state.step(1.0, 0.0, 0.0, false); }
        assert!(state.rotor_spin > 0.0 && state.rotor_spin <= 1.0, "spin {}", state.rotor_spin);
    }
}
