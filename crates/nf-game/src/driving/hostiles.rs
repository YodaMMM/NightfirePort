//! The hostile runtime: one object the driving view owns and steps once per world frame. It
//! wires the ported pieces together and invents no rule of its own; every rule below is a call
//! into the module that ports it:
//!
//! - the element database `hostile::ElementDb` (`FUN_0012bec8`, spawn `FUN_0012cb08`, despawn
//!   `FUN_0012c740`) turns the track's `AIEl` records into Spawn / Despawn actions;
//! - a spawned ground vehicle is a `physics::Car` built from its own pvehicle `.atr`
//!   (`FUN_001862c0`), given its spawn command (AIEl +0xf8 → `AICo`, applied as `FUN_00145ff8`
//!   does) and driven by the chase brain: `hostile::sight_tick` / `ram_tick`,
//!   `patterns::aim_point`, `pursue_switch`, `hostile::chase_target_speed`, the road carrot
//!   (`ai::RoadNav`, kinds 3/4 as `FUN_0012e468` sets them), `patterns::dispatch`,
//!   `avoid_chase`, `chase_drive`, then `Car::step` on the caller's `Ground`;
//! - its weapons: `patterns::turret_step`, `hostile::line_clear`, `hostile::fire_step`; a shot is
//!   resolved through `weapons::delivery` (the damage) and, for rockets and missiles,
//!   `weapons::launch_params` + `weapons::Projectile::step`;
//! - its riders: `riders::seat_class` and the window / snowmobile / sunroof sight and fire
//!   cadence;
//! - a helicopter element is a `heli::Heli` controller (`FUN_0013fa28` attach, `FUN_00138fa8`
//!   command, `FUN_00138ef0` update);
//! - henchmen elements are tracked live at their record position (the on-foot character class is
//!   not part of the driving port).
//!
//! What is a stand-in, not the game, is marked [S] at the place it is used and listed in
//! docs/systems/driving-ai.md §13. Random draws: the driving generator only (`ai::DriveRng`),
//! passed in by the caller so the traffic and the hostiles share the one stream the game has.

use crate::driving::ai::{gated_stop, lookahead, DriveCommand, DriveRng, NavHint, RoadNav, SimCar};
use crate::driving::damage::Hit;
use crate::driving::heli::{Heli, HeliActor, HeliEnv, HeliEvent, PathPose};
use crate::driving::hostile::{
    chase_target_speed, fire_step, line_clear, ram_tick, sight_tick, AiAction, ChaseSpeedInputs, ElemState, ElementDb,
    FireInputs, LiveStatus, LofCar, RamState, SightState, Spawn, WeaponState, WorldAttr,
};
use crate::driving::patterns::{
    aim_point, avoid_chase, chase_drive, dispatch, finish_rejoin, pursue_switch, road_heading_latch, turret_step, AimInputs, Arm,
    Avoid, Body, ChaseDriveInputs, DispatchInputs, OtherCar, ReverseState, Slots, SwitchInputs, Turret, PATTERN_PATH,
    PATTERN_ROAD, PATTERN_ROAD_FOLLOW,
};
use crate::driving::physics::{Car, CarInput, CarParams, Ground, Rigid, DT};
use crate::driving::riders::{
    aim_test, range_sq, seat_class, snow_fire, snow_sight, sunroof_fire, sunroof_setup, sunroof_sight, window_acquire,
    window_fire, window_sight, window_tick, Fire, Gunner, SeatClass,
};
use crate::driving::road::{LaneSet, RoadNet};
use crate::driving::weapons::{
    ai_damage_scale, aim_is_exact, aim_spread, delivery, launch_params, Delivery, Launcher, Projectile, Shooter, Target,
    WeaponSystem, CANNON, GUN, MISSILE, ROCKET, WEAPONS,
};
use nf_formats::driveai::{AiCommand, MapAi, TYPE_HENCHMAN, TYPE_VEHICLE};

/// Ticks per second in play (DAT_00339f4c; FRAME_RATE 60 in play) — the rate every
/// `fps·seconds` rule below is evaluated at.
pub const FPS: i32 = 60;
/// AIVehicle state 5, the chase state `FUN_0013f748` creates a sim-kind-2 car in [M 0x13f748].
pub const STATE_CHASE: u32 = 5;
/// [S] A launcher animation channel is taken as busy this many ticks after it fires
/// (`FUN_001a2d10` reads the model's animation channel, which the port does not play) — the
/// same stand-in the helicopter disc test uses.
pub const LAUNCHER_BUSY_TICKS: i32 = 30;
/// [S] The contact radius added to the player's box half length for a projectile hit (the game
/// tests the projectile's 0.1 × 0.1 × 0.5 box against the target's box, `FUN_001f5eb0`
/// [M 0x182978]; the port tests a sphere).
pub const PROJECTILE_CONTACT_PAD: f32 = 0.5;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn len(a: [f32; 3]) -> f32 {
    dot(a, a).max(0.0).sqrt()
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { [0.0; 3] }
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// The ground height under `p` from the caller's `Ground` (the wheel query's plane).
pub fn ground_y<G: Ground + ?Sized>(g: &G, p: [f32; 3]) -> Option<f32> {
    let pl = g.wheel_plane(p)?;
    let n = pl.normal;
    if n[1].abs() < 1e-4 {
        return None;
    }
    Some(pl.v0[1] - ((p[0] - pl.v0[0]) * n[0] + (p[2] - pl.v0[2]) * n[2]) / n[1])
}

// ------------------------------------------------------------------------------------------
// The vehicle attributes
// ------------------------------------------------------------------------------------------

/// What the runtime reads from one pvehicle `.atr` (`data\sim\attrib\pvehicle\<name>.atr`):
/// the car parameters (`CarParams::parse_atr`), the model (`RENDER_FILENAME`), the weapon
/// declarations `FUN_00145ff8` tests before it honours a command's weapon bytes (`HAS_MACHINEGUNS`
/// / `HAS_CANNONS`, `HAS_ROCKETS`, `HAS_MISSLES` / `HAS_TORPEDOS` [M 0x145ff8 tail, strings
/// 0x36e390.. D]), `HAS_TURRET` (AI +0x12c, `FUN_0013f748` [M]), `TURRET_RANGE`, `IS_HELI`
/// (the pattern-4 bob height `FUN_0012e468` [M], string 0x36b528) and the rider seat vectors
/// (`AIC_*_POS`, read by `FUN_0012cb08` through `FUN_00169e48` [M]). The attribute system's
/// fallback to `default.atr` is not modelled [S]: a key the file lacks reads as absent.
#[derive(Debug, Clone)]
pub struct VehicleAttrs {
    pub name: String,
    pub params: CarParams,
    /// `RENDER_FILENAME`, e.g. `paris_henchbigbox.crp`.
    pub render_file: Option<String>,
    pub machine_guns: bool,
    pub cannons: bool,
    pub rockets: bool,
    pub missiles: bool,
    pub torpedos: bool,
    pub turret: bool,
    pub turret_range: Option<[f32; 4]>,
    pub is_heli: bool,
    /// `(attribute name, x, y, z)` for every `AIC_*_POS` vector.
    pub seats: Vec<(String, [f32; 3])>,
}

impl Default for VehicleAttrs {
    fn default() -> VehicleAttrs {
        VehicleAttrs {
            name: String::new(),
            params: CarParams::game_defaults(),
            render_file: None,
            machine_guns: false,
            cannons: false,
            rockets: false,
            missiles: false,
            torpedos: false,
            turret: false,
            turret_range: None,
            is_heli: false,
            seats: Vec::new(),
        }
    }
}

impl VehicleAttrs {
    pub fn parse(name: &str, text: &str) -> VehicleAttrs {
        let mut a = VehicleAttrs { name: name.to_string(), params: CarParams::parse_atr(text), ..Default::default() };
        let mut sections = 0;
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            if line.starts_with('[') {
                sections += 1;
                if sections > 1 {
                    break;
                }
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            let key = k.trim().split('.').next().unwrap_or("").trim();
            let v = v.trim();
            let truthy = matches!(v.to_ascii_lowercase().as_str(), "1" | "true");
            let floats: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect();
            match key {
                "RENDER_FILENAME" => a.render_file = Some(v.to_string()),
                "HAS_MACHINEGUNS" => a.machine_guns = truthy,
                "HAS_CANNONS" => a.cannons = truthy,
                "HAS_ROCKETS" => a.rockets = truthy,
                "HAS_MISSLES" => a.missiles = truthy,
                "HAS_TORPEDOS" => a.torpedos = truthy,
                "HAS_TURRET" => a.turret = truthy,
                "IS_HELI" => a.is_heli = truthy,
                "TURRET_RANGE" if floats.len() >= 4 => a.turret_range = Some([floats[0], floats[1], floats[2], floats[3]]),
                k if k.starts_with("AIC_") && k.ends_with("_POS") && floats.len() >= 3 => a.seats.push((k.to_string(), [floats[0], floats[1], floats[2]])),
                _ => {}
            }
        }
        a
    }

    /// The model file stem (`RENDER_FILENAME` without `.crp`); the vehicle name when the file
    /// has no `RENDER_FILENAME` [S: the attribute fallback is not modelled].
    pub fn model(&self) -> String {
        match &self.render_file {
            Some(f) => f.trim_end_matches(".crp").trim_end_matches(".CRP").to_string(),
            None => self.name.clone(),
        }
    }

    pub fn seat(&self, attr: &str) -> Option<[f32; 3]> {
        self.seats.iter().find(|(k, _)| k == attr).map(|(_, v)| *v)
    }
}

/// One model the view must have loaded before the mission runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelNeed {
    /// The AIEl +0x54 vehicle name.
    pub vehicle: String,
    /// The model file stem (`data\car\model\<stem>.crp`).
    pub model: String,
    /// The vehicle set has no `.atr` for this name.
    pub missing_atr: bool,
}

/// Every vehicle model the track's elements can spawn, in first-seen order (`MapAi::vehicle_set`
/// → each name's `RENDER_FILENAME`) [M 0x12b8b0 reads the rendername of each element's +0x54
/// attribute at load; D the names]. Run [`snow_hench_swap`] first: the snow2a_mis4
/// `paris_hench` → `junglesnow` swap changes the set (0x12b8b0:96-125).
pub fn models_needed(map: &MapAi, set: &[VehicleAttrs]) -> Vec<ModelNeed> {
    map.vehicle_set()
        .into_iter()
        .map(|v| match set.iter().find(|a| a.name.eq_ignore_ascii_case(&v)) {
            Some(a) => ModelNeed { vehicle: v.clone(), model: a.model(), missing_atr: false },
            None => ModelNeed { model: v.clone(), vehicle: v, missing_atr: true },
        })
        .collect()
}

/// The track whose henchman cars `FUN_0012b8b0` swaps [D DRIVING.ELF 0x36aea8].
pub const HENCH_SWAP_TRACK: &str = "snow2a_mis4";
/// The rendername prefix the swap matches, 11 bytes [D 0x36b0c0; M 0x12b8b0:103 strncmp 0xb].
pub const HENCH_SWAP_RENDER: &str = "paris_hench";
/// The vehicle attribute the swapped records name [D 0x36b0d0].
pub const HENCH_SWAP_VEHICLE: &str = "junglesnow";

/// `FUN_0012b8b0`'s load-time swap [M DRIVING 0x12b8b0:96-125]: when the track key
/// (`DAT_0033aac8`+0xc) is exactly `snow2a_mis4` and the PTP word 0x100984 reads 2 (DRIVING
/// main has already remapped it, so 2 = the hardest band: raw 3 or 4, see
/// `ptp::driving_difficulty`), every vehicle record (+0x44 == 1) whose pvehicle attribute's
/// rendername starts with `paris_hench` gets its +0x54 name overwritten with `junglesnow` and
/// its rider count +0x7c zeroed. The game swaps before the health scale loop, so the riders it
/// drops are never scaled. `render_of(name)` is the rendername `FUN_001967a8` hands back for an
/// attribute name (the `RENDER_FILENAME` value; the caller's fallback for an attribute with no
/// such key is [S]). Returns the number of records swapped.
pub fn snow_hench_swap(map: &mut MapAi, track_key: &str, ptp_word: u32, render_of: impl Fn(&str) -> String) -> usize {
    // strcmp is exact (case-sensitive) [M 0x12b8b0:96 FUN_002a9508 == 0].
    if track_key != HENCH_SWAP_TRACK || ptp_word != 2 {
        return 0;
    }
    let mut n = 0;
    for e in map.elements.iter_mut().filter(|e| e.kind == TYPE_VEHICLE) {
        if render_of(&e.name).as_bytes().starts_with(HENCH_SWAP_RENDER.as_bytes()) {
            e.name = HENCH_SWAP_VEHICLE.to_string();
            e.riders.clear();
            n += 1;
        }
    }
    n
}

// ------------------------------------------------------------------------------------------
// The live objects
// ------------------------------------------------------------------------------------------

/// The player the hostiles chase: the first controller's sim (DAT_00339f5c[0]) [M 0x12bec8].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerState {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// The forward row.
    pub fwd: [f32; 3],
    /// The car's box half length (sim +0x5c → +0x4c8) used by the chase speed and the hit tests.
    pub box_z: f32,
}

/// The chase brain's per-car state: the AIVehicle fields the ported functions read and write
/// (offsets on each field in `hostile.rs` / `patterns.rs`).
#[derive(Debug, Clone)]
pub struct ChaseBrain {
    /// +0x70: 5 for a sim-kind-2 car [M 0x13f748].
    pub state: u32,
    /// +0x78 / +0x84 / +0x80 (target mode).
    pub pattern: u32,
    pub profile: u32,
    pub target_mode: u32,
    /// +0x1a4 ≠ 0: the target is the player (target mode 0 → manager handle 0 [M 0x12e670]).
    pub has_target: bool,
    pub goal: [f32; 3],
    /// The command's numbers (AICo → AI, `FUN_00145ff8`).
    pub follow: f32,
    pub catchup: f32,
    pub cap: f32,
    pub fire_rate: f32,
    pub ram_chance: f32,
    pub lead: f32,
    pub avoid_on: bool,
    pub follow_target: bool,
    pub accuracy: i8,
    pub damage_mod: i8,
    /// +0x12d / +0x12e / +0x12f: the weapon bytes after the `.atr` test.
    pub guns: bool,
    pub rockets: bool,
    pub missiles: bool,
    /// +0x224 weapons on (1 at construction [M 0x12ded8]).
    pub weapons_on: bool,
    /// The command record applied last (AI +0x88; the same record is not re-applied [M 0x145ff8]).
    pub command: Option<usize>,
    pub nav: Option<RoadNav>,
    pub ram: RamState,
    pub sight: SightState,
    pub weapons: WeaponState,
    pub reverse: ReverseState,
    pub slots: Slots,
    pub turret: Turret,
    pub lost: bool,
    pub prev_aim: [f32; 3],
    pub prev_dist: f32,
    /// +0x1e0 as last tick left it.
    pub dir: [f32; 3],
    /// +0x74 the switch hold, +0x27c the road latch.
    pub hold_until: i32,
    pub latch: bool,
    /// [S] launcher busy timers for channels 5, 6, 7, 8.
    pub busy: [i32; 4],
    /// This tick's +0x8c target speed and drive command (for inspection).
    pub last_speed: f32,
    pub last_cmd: DriveCommand,
}

impl ChaseBrain {
    /// The AIVehicle as `FUN_0012ded8` + `FUN_0013f748` leave it before any command: state 5,
    /// no pattern/profile (the dispatch idles) [S: the ctor's +0x78/+0x84 values were not read;
    /// 0 makes `dispatch` idle and `chase_target_speed` return 0].
    fn new() -> ChaseBrain {
        ChaseBrain {
            state: STATE_CHASE,
            pattern: 0,
            profile: 0,
            target_mode: 0,
            has_target: false,
            goal: [0.0; 3],
            follow: 0.0,
            catchup: 0.0,
            cap: 0.0,
            fire_rate: -1.0,
            ram_chance: 0.0,
            lead: 0.0,
            avoid_on: true,
            follow_target: false,
            accuracy: 0,
            damage_mod: 0,
            guns: false,
            rockets: false,
            missiles: false,
            weapons_on: true,
            command: None,
            nav: None,
            ram: RamState::new(FPS),
            sight: SightState::default(),
            weapons: WeaponState::default(),
            reverse: ReverseState::default(),
            slots: Slots::default(),
            turret: Turret::default(),
            lost: false,
            prev_aim: [0.0; 3],
            prev_dist: 0.0,
            dir: [0.0, 0.0, 1.0],
            hold_until: 0,
            latch: false,
            busy: [0; 4],
            last_speed: 0.0,
            last_cmd: DriveCommand::IDLE,
        }
    }
}

/// A seated rider gunner.
#[derive(Debug, Clone)]
pub struct RiderRt {
    /// Index into the element's riders.
    pub index: usize,
    pub class: SeatClass,
    pub gunner: Gunner,
    pub range_sq: f32,
    pub weapon: u32,
    /// The seat vector from the car's `.atr` (car-local).
    pub offset: [f32; 3],
    /// +0x154 the window class's sweep yaw.
    pub sweep_yaw: f32,
}

/// A helicopter element's controller and the actor values it drives.
#[derive(Debug, Clone)]
pub struct HeliUnit {
    pub heli: Heli,
    pub actor: HeliActor,
    /// [S] the stand-in integration's heading.
    pub heading: f32,
    pub nav: Option<RoadNav>,
    /// [S] launchers 7 / 8 busy timers.
    pub busy: [i32; 2],
}

#[derive(Debug, Clone)]
pub enum LiveKind {
    /// A ground vehicle; `brain` is `None` for sim kind 4 (no AIVehicle controller [M 0x13f118]).
    Car { car: Car, brain: Option<ChaseBrain>, wheel_spin: f32 },
    Heli(HeliUnit),
    /// A henchman element (on foot): held at its record position [S: the character class is the
    /// on-foot layer, not ported here].
    Henchman { pos: [f32; 3], facing: [f32; 3] },
}

/// One live element.
#[derive(Debug, Clone)]
pub struct LiveVehicle {
    pub element: usize,
    pub name: String,
    pub model: String,
    pub health: f32,
    pub sim_kind: u32,
    pub kind: LiveKind,
    pub riders: Vec<RiderRt>,
    /// The tick it spawned.
    pub born: i32,
}

impl LiveVehicle {
    pub fn pos(&self) -> [f32; 3] {
        match &self.kind {
            LiveKind::Car { car, .. } => car.pos,
            LiveKind::Heli(h) => h.actor.body.pos,
            LiveKind::Henchman { pos, .. } => *pos,
        }
    }
    pub fn vel(&self) -> [f32; 3] {
        match &self.kind {
            LiveKind::Car { car, .. } => car.vel,
            LiveKind::Heli(h) => h.actor.body.vel,
            LiveKind::Henchman { .. } => [0.0; 3],
        }
    }
    /// Rows right, up, forward.
    pub fn rows(&self) -> [[f32; 3]; 3] {
        match &self.kind {
            LiveKind::Car { car, .. } => car.rot,
            LiveKind::Heli(h) => [h.actor.body.side, h.actor.body.up, h.actor.body.fwd],
            LiveKind::Henchman { facing, .. } => {
                let f = norm([facing[0], 0.0, facing[2]]);
                let f = if len(f) > 0.0 { f } else { [0.0, 0.0, 1.0] };
                [[f[2], 0.0, -f[0]], [0.0, 1.0, 0.0], f]
            }
        }
    }
}

/// A hostile projectile in flight.
#[derive(Debug, Clone)]
pub struct HostileProjectile {
    pub element: usize,
    /// The weapon id (`weapons::ROCKET`, `MISSILE`, …).
    pub weapon: usize,
    pub p: Projectile,
}

/// What the view needs to draw one live vehicle.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleDraw {
    pub element: usize,
    pub name: String,
    pub model: String,
    /// Rows right / up / forward / position (row-major, translation in row 3).
    pub matrix: [[f32; 4]; 4],
    /// Wheel centres (world), the front steer angle and the accumulated roll angle (radians).
    pub wheels: [[f32; 3]; 4],
    pub steer: f32,
    pub wheel_spin: f32,
    /// A helicopter's actor +0x74 (the spin the crash steps) [M 0x139be8 case 16].
    pub rotor: f32,
    pub heli: bool,
    pub henchman: bool,
}

/// One tick's outcome.
#[derive(Debug, Clone, Default)]
pub struct StepOut {
    /// The element database's actions this tick [M 0x12bec8], plus the despawns the world made
    /// (helicopter `FUN_0012da78`, `deactivate`).
    pub actions: Vec<AiAction>,
    /// Damage calls for `damage::inflict` on the player's car (kind 2 [M]).
    pub hits: Vec<HostileHit>,
    /// Traffic cars within `their own box half length + r` of a point must be parked
    /// (`FUN_00132ba8`): a vehicle spawn (r = 6) or a command with +0x7b (r = the AI car's box)
    /// [M 0x12cb08:166-195, 0x145ff8:194-232].
    pub park_traffic: Vec<([f32; 3], f32)>,
    /// Shots and blasts for sound / effects.
    pub events: Vec<HostileEvent>,
}

/// A damage call against the player.
#[derive(Debug, Clone, PartialEq)]
pub struct HostileHit {
    pub element: usize,
    pub weapon: usize,
    pub hit: Hit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostileEvent {
    /// A machine-gun / cannon shot left `from` along `dir`; `hit` = it struck the player.
    Shot { element: usize, weapon: usize, from: [f32; 3], dir: [f32; 3], hit: bool },
    /// A rocket / missile was launched.
    Launch { element: usize, weapon: usize, from: [f32; 3] },
    /// A projectile ended at `pos` (a contact, or its life ran out).
    Impact { element: usize, weapon: usize, pos: [f32; 3], on_player: bool },
    /// A rider gunner's fire animation started (`riders::Fire::Shot`). The round it fires comes
    /// out of the character animation layer and is not traced: no damage is booked for it.
    RiderShot { element: usize, rider: usize, anim: u32, from: [f32; 3] },
    /// A helicopter's blast (`HeliEvent::Blast`).
    Blast { element: usize, pos: [f32; 3] },
}

// ------------------------------------------------------------------------------------------
// The world
// ------------------------------------------------------------------------------------------

/// The mission's hostile runtime.
pub struct HostileWorld {
    pub db: ElementDb,
    pub map: MapAi,
    pub attrs: Vec<VehicleAttrs>,
    pub live: Vec<LiveVehicle>,
    pub projectiles: Vec<HostileProjectile>,
    /// The world tick (DAT_00339f54).
    pub now: i32,
    /// The mission speed factor (`FUN_00208468`, 1.0 on every shipped track [D]).
    pub speed_factor: f32,
    /// The weapon table the AI shots read their damage from: the *player's* table
    /// (`DAT_0033a530+0x10`) [M 0x14ca60]. The caller keeps it in step with the player's
    /// pickups; built from `FUN_0020bbe0`'s defaults.
    pub table: WeaponSystem,
    pub rigid: Rigid,
    /// (segment, t, point) samples of the road for the nearest-piece join [S: the game's grid
    /// query `FUN_00225490` / `FUN_00225c48` is not ported; a brute-force nearest sample stands in].
    road_samples: Vec<(usize, f32, [f32; 3])>,
}

impl HostileWorld {
    /// `FUN_0012b8b0`'s load: the database from the track's `AIEl` with the world `.atr` slot
    /// budgets and the difficulty's health factor [M]. `attrs` = the mission's AI vehicle set
    /// (one per pvehicle `.atr`). `net` = the track's road network (for the road patterns).
    pub fn new(map: &MapAi, attrs: Vec<VehicleAttrs>, world: &WorldAttr, difficulty: u32, health_factor: f32, speed_factor: f32, net: Option<&RoadNet>) -> HostileWorld {
        let mut road_samples = Vec::new();
        if let Some(net) = net {
            for k in 0..net.segs.len() {
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    if let Ok(p) = net.seg_point(k, t) {
                        road_samples.push((k, t, p));
                    }
                }
            }
        }
        HostileWorld {
            db: ElementDb::new(map, world.slots(), difficulty, health_factor, 0),
            map: map.clone(),
            attrs,
            live: Vec::new(),
            projectiles: Vec::new(),
            now: 0,
            speed_factor,
            table: WeaponSystem::new(FPS, false),
            rigid: Rigid::game_defaults(),
            road_samples,
        }
    }

    pub fn attrs_of(&self, name: &str) -> Option<&VehicleAttrs> {
        self.attrs.iter().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    /// `EActivateAIElement` (`FUN_001476b8`): clear the element's HELD bit so the range manager
    /// may spawn it [M]. The handler's gate (`DAT_00327c2c` set, a non-persistent vehicle, and
    /// `FUN_0012c5d0` false) is taken as open [S: no writer of DAT_00327c2c was found].
    pub fn activate(&mut self, element: usize) {
        if let Some(e) = self.db.elements.get_mut(element) {
            e.rec.flags &= !nf_formats::driveai::flag::HELD;
        }
    }

    /// `EDeactivateAIElement` (`FUN_0014b510`): set HELD and, for a vehicle, despawn it at once
    /// (`FUN_0012da78` → `FUN_0012c740`) [M].
    pub fn deactivate(&mut self, element: usize) -> Option<AiAction> {
        let e = self.db.elements.get_mut(element)?;
        e.rec.flags |= nf_formats::driveai::flag::HELD;
        if e.rec.kind == TYPE_VEHICLE && e.state == ElemState::Live {
            let st = self.live_status(element);
            let a = self.db.despawn(element, st);
            self.live.retain(|v| v.element != element);
            return Some(a);
        }
        None
    }

    /// `EAICommand` (`FUN_00145ff8`) on a live element: apply AICo record `command` [M]. Returns
    /// the traffic-park request the command's +0x7b makes.
    pub fn command(&mut self, element: usize, command: usize, net: Option<&RoadNet>) -> Option<([f32; 3], f32)> {
        let cmd = self.map.commands.get(command)?.clone();
        let samples = std::mem::take(&mut self.road_samples);
        let attrs = self.live.iter().find(|v| v.element == element).and_then(|v| self.attrs_of(&v.name).cloned());
        let path = self.map.path_index(cmd.pattern_arg).is_some();
        let out = self.live.iter_mut().find(|v| v.element == element).and_then(|v| apply_command(v, command, &cmd, attrs.as_ref(), net, &samples, path));
        self.road_samples = samples;
        out
    }

    /// The player's (or a script's) damage to a live element: the actor's hit points, which
    /// point at AIEl +0x4c (`FUN_00195a08`) [M 0x12cb08]. `FUN_00195a58` floors them at 0 [M].
    pub fn damage(&mut self, element: usize, amount: f32) {
        if let Some(v) = self.live.iter_mut().find(|v| v.element == element) {
            v.health = (v.health - amount.max(0.0)).max(0.0);
            if let LiveKind::Heli(h) = &mut v.kind {
                h.actor.health = v.health;
            }
        }
    }

    fn live_status(&self, element: usize) -> Option<LiveStatus> {
        let v = self.live.iter().find(|v| v.element == element)?;
        // Wheels never come off in the port's car (the wheel detach of FUN_001879c8 is not
        // ported) [S]; +0x3e8 is taken as clear [S, its meaning was not traced].
        Some(LiveStatus { pos: v.pos(), health: v.health, wheels: 4, vehicle_3e8: false, hench_returns: true })
    }

    /// One world frame. Order [S: the relative order of the element manager and the vehicle
    /// controllers inside `FUN_00159d10` was not traced]: the element database (`FUN_0012bec8`,
    /// once per frame [M]), its spawns and despawns, every live vehicle's controller and car,
    /// then the projectiles.
    ///
    /// `ground` = what `Car::step` drives on. `surface` = the track's ground for heights when
    /// `ground` is the legacy rail stand-in (`FlatWorld`): a rail car is then held at the
    /// surface height + its rest height each tick, as the view does for its traffic [S], and a
    /// spawn is lifted onto it. `ray(a, b)` = the world ray's hit count from `a` to `b`
    /// (`FUN_00215cc8`, the sight test `FUN_001330d0` [M]).
    #[allow(clippy::too_many_arguments)]
    pub fn step<G: Ground + ?Sized>(&mut self, net: Option<&RoadNet>, ground: &G, surface: Option<&dyn Ground>, player: &PlayerState, rng: &mut DriveRng, ray: &mut dyn FnMut([f32; 3], [f32; 3]) -> u32) -> StepOut {
        self.now += 1;
        let now = self.now;
        let mut out = StepOut::default();

        // 1. The element database.
        let status: Vec<Option<LiveStatus>> = (0..self.db.elements.len()).map(|i| self.live_status(i)).collect();
        let actions = self.db.tick(now, player.pos, player.vel, &|i| status.get(i).copied().flatten());
        for a in actions {
            match &a {
                AiAction::Spawn(s) => {
                    if let Some(p) = self.spawn(s, net, ground, surface) {
                        out.park_traffic.push(p);
                    }
                }
                AiAction::Despawn { index, .. } => self.live.retain(|v| v.element != *index),
            }
            out.actions.push(a);
        }
        // 2. The live vehicles.
        let snapshot: Vec<Snap> = self.live.iter().map(Snap::of).collect();
        let samples = std::mem::take(&mut self.road_samples);
        let mut despawn_by_actor = Vec::new();
        let difficulty = self.db.difficulty as i32;
        for k in 0..self.live.len() {
            let attrs = self.attrs_of(&self.live[k].name).cloned().unwrap_or_default();
            let mut cx = Cx {
                net,
                player,
                rng: &mut *rng,
                ray: &mut *ray,
                now,
                others: &snapshot,
                samples: &samples,
                speed_factor: self.speed_factor,
                table: &self.table,
                difficulty,
                out: &mut out,
                projectiles: &mut self.projectiles,
            };
            let v = &mut self.live[k];
            match &mut v.kind {
                LiveKind::Car { .. } => step_car(v, &attrs, ground, surface, &self.rigid, &mut cx),
                LiveKind::Heli(_) => {
                    let hit = match surface {
                        Some(sf) => step_heli(v, sf, &mut cx),
                        None => step_heli(v, ground, &mut cx),
                    };
                    if hit {
                        despawn_by_actor.push(v.element);
                    }
                }
                LiveKind::Henchman { .. } => {}
            }
            step_riders(&mut self.live[k], &mut cx);
        }
        self.road_samples = samples;
        for e in despawn_by_actor {
            // `FUN_0012da78`: the actor despawns its element (`FUN_0012c740`) [M 0x13b2a8].
            let st = self.live_status(e);
            out.actions.push(self.db.despawn(e, st));
            self.live.retain(|v| v.element != e);
        }

        // 3. The projectiles: `Projectile::step` (homing + thrust + life) [M 0x181e70 /
        // 0x182978], then the contact with the player [S sphere, see PROJECTILE_CONTACT_PAD].
        let mut keep = Vec::new();
        for mut pr in std::mem::take(&mut self.projectiles) {
            let tgt = Target { pos: player.pos, vel: player.vel, prox_kind: false };
            pr.p.step(Some(tgt), now, DT, FPS);
            let age = now - pr.p.born;
            let near = len(sub(pr.p.pos, player.pos)) <= player.box_z + PROJECTILE_CONTACT_PAD;
            if age >= pr.p.p.arm_delay && near {
                // The contact: `FUN_00200088(damage, …, 2, …)` — the explosion with the
                // projectile's +0x718 damage [M 0x182978:307]. The explosion's radius rule
                // (FUN_00159fb8 → FUN_001847e8) is not ported: the full damage is booked on the
                // player as one kind-2 call with dent 0 [S].
                let from = sub(pr.p.pos, scale(pr.p.vel, DT));
                out.hits.push(HostileHit {
                    element: pr.element,
                    weapon: pr.weapon,
                    hit: Hit { amount: pr.p.p.damage, dent: 0.0, kind: 2, at: pr.p.pos, from, from_player: false },
                });
                out.events.push(HostileEvent::Impact { element: pr.element, weapon: pr.weapon, pos: pr.p.pos, on_player: true });
                continue;
            }
            if pr.p.detonate {
                out.events.push(HostileEvent::Impact { element: pr.element, weapon: pr.weapon, pos: pr.p.pos, on_player: false });
                continue;
            }
            keep.push(pr);
        }
        self.projectiles = keep;
        out
    }

    /// `FUN_0012cb08`'s create, after `ElementDb::spawn` decided it [M 0x12cb08:152-260]: a
    /// ground vehicle (`FUN_001ff850` with sim kind 2 / 4 at the record's +0x30 facing +0x20,
    /// the controller `FUN_0013f118`), a helicopter (`FUN_002003c8` → `FUN_0013f2e8` → the
    /// AIHelicopter), or a henchman; the health pointer at AIEl +0x4c; the spawn command +0xf8
    /// handed to `FUN_00145fc8` [M]. Returns the traffic park request.
    fn spawn<G: Ground + ?Sized>(&mut self, s: &Spawn, net: Option<&RoadNet>, ground: &G, surface: Option<&dyn Ground>) -> Option<([f32; 3], f32)> {
        let e = &self.db.elements[s.index];
        let rec = e.rec.clone();
        let attrs = self.attrs_of(&rec.name).cloned();
        let model = attrs.as_ref().map(|a| a.model()).unwrap_or_else(|| rec.name.clone());
        let f = norm([s.facing[0], 0.0, s.facing[2]]);
        let f = if len(f) > 0.0 { f } else { [0.0, 0.0, 1.0] };
        let yaw = f[0].atan2(f[2]);
        let cmd_idx = self.map.command_index(rec.command_ref);
        let mut park = None;
        let kind = if rec.kind == TYPE_HENCHMAN {
            LiveKind::Henchman { pos: s.pos, facing: s.facing }
        } else if s.heli {
            // `FUN_0013fa28`: the controller is attached (+0x58) [M]; the command sets +0x5c.
            let side = cross([0.0, 1.0, 0.0], f);
            let body = Body { pos: s.pos, vel: [0.0; 3], side, up: [0.0, 1.0, 0.0], fwd: f, box_z: 2.0, box_x: 1.0 };
            let mut actor = HeliActor::new(body);
            actor.health = rec.health;
            let mut heli = Heli::reset();
            heli.attached = true;
            if let Some(ci) = cmd_idx {
                let c = &self.map.commands[ci];
                heli.apply_command(c, &mut actor, self.map.path_index(c.pattern_arg).is_some());
            }
            LiveKind::Heli(HeliUnit { heli, actor, heading: yaw, nav: None, busy: [0; 2] })
        } else {
            let params = attrs.as_ref().map(|a| a.params.clone()).unwrap_or_else(CarParams::game_defaults);
            let mut pos = s.pos;
            // [S] The record's +0x30 is taken as the body centre the sim is placed at
            // (vtbl+0x164); a car whose record point sits below its rest height on the ground
            // under it is lifted to that height so the wheels do not start inside the road.
            let gy = match surface {
                Some(sf) => ground_y(sf, pos),
                None => ground_y(ground, pos),
            };
            if let Some(gy) = gy {
                let rest = params.half_extents[1] + params.spring_rest;
                if pos[1] < gy + rest {
                    pos[1] = gy + rest;
                }
            }
            let car = Car::new(params, pos, yaw);
            park = s.clear_traffic_pad.map(|pad| (pos, pad));
            let brain = (s.sim_kind == 2).then(ChaseBrain::new);
            LiveKind::Car { car, brain, wheel_spin: 0.0 }
        };
        let mut riders = Vec::new();
        for &k in &s.riders {
            let r = &rec.riders[k];
            let Some(class) = seat_class(r.seat) else { continue };
            let mut g = Gunner::base();
            if r.seat == 4 {
                let (clip, shots, state) = sunroof_setup(r.weapon as i64);
                g.clip = clip;
                g.shots = shots;
                g.state = state;
            } else {
                g.state = class.first_state;
            }
            let offset = attrs.as_ref().and_then(|a| a.seat(class.attr)).unwrap_or([0.0; 3]);
            riders.push(RiderRt { index: k, class, gunner: g, range_sq: range_sq(r.skill), weapon: r.weapon, offset, sweep_yaw: 0.0 });
        }
        let mut v = LiveVehicle { element: s.index, name: rec.name.clone(), model, health: rec.health, sim_kind: s.sim_kind, kind, riders, born: self.now };
        if let (Some(ci), LiveKind::Car { .. }) = (cmd_idx, &v.kind) {
            let c = self.map.commands[ci].clone();
            let path = self.map.path_index(c.pattern_arg).is_some();
            if let Some(p) = apply_command(&mut v, ci, &c, attrs.as_ref(), net, &self.road_samples, path) {
                park = Some(p);
            }
        }
        self.live.push(v);
        park
    }

    /// The draw list: every live element's model, transform, wheel and rotor state.
    pub fn draw_list(&self) -> Vec<VehicleDraw> {
        self.live
            .iter()
            .map(|v| {
                let r = v.rows();
                let p = v.pos();
                let matrix = [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0], [p[0], p[1], p[2], 1.0]];
                let (wheels, steer, wheel_spin) = match &v.kind {
                    LiveKind::Car { car, wheel_spin, .. } => ([car.wheel_world(0), car.wheel_world(1), car.wheel_world(2), car.wheel_world(3)], car.steer_angle, *wheel_spin),
                    _ => ([p; 4], 0.0, 0.0),
                };
                let rotor = match &v.kind {
                    LiveKind::Heli(h) => h.actor.spin,
                    _ => 0.0,
                };
                VehicleDraw {
                    element: v.element,
                    name: v.name.clone(),
                    model: v.model.clone(),
                    matrix,
                    wheels,
                    steer,
                    wheel_spin,
                    rotor,
                    heli: matches!(v.kind, LiveKind::Heli(_)),
                    henchman: matches!(v.kind, LiveKind::Henchman { .. }),
                }
            })
            .collect()
    }

    pub fn live_vehicles(&self) -> usize {
        self.live.iter().filter(|v| matches!(v.kind, LiveKind::Car { .. })).count()
    }
    pub fn live_helis(&self) -> usize {
        self.live.iter().filter(|v| matches!(v.kind, LiveKind::Heli(_))).count()
    }
}

// ------------------------------------------------------------------------------------------
// The command apply (FUN_00145ff8 → AIVehicle)
// ------------------------------------------------------------------------------------------

/// The nearest road sample to `p` [S, see `HostileWorld::road_samples`], joined as a navigator of
/// `kind` heading along `fwd` (`FUN_00225c48(nav, pos, fwd, 0)` [M 0x12e468]).
fn join_nearest(net: &RoadNet, samples: &[(usize, f32, [f32; 3])], p: [f32; 3], fwd: [f32; 3], kind: u32) -> Option<RoadNav> {
    let mut best = (f32::MAX, 0usize, 0.0f32);
    for &(k, t, q) in samples {
        let d = (q[0] - p[0]).powi(2) + (q[2] - p[2]).powi(2) + (q[1] - p[1]).powi(2) * 0.25;
        if d < best.0 {
            best = (d, k, t);
        }
    }
    if best.0 == f32::MAX {
        return None;
    }
    let dir = net.seg(best.1).ok()?.dir();
    let forward = dir[0] * fwd[0] + dir[2] * fwd[2] >= 0.0;
    RoadNav::join(net, best.1, best.2, 0, forward, kind, LaneSet::Default).ok()
}

/// `FUN_00145ff8`'s AIVehicle arm [M 0x145ff8:186-377]: skipped when the record is the one held
/// (+0x88); +0x7b parks overlapping traffic and puts the car at the record's +0x40 facing its
/// +0x20 row; +0x154 = (+0x7c == 0); the target mode `FUN_0012e670` (0 = the player, 1 = the
/// goal +0x40); the pattern `FUN_0012e468` (3 → navigator kind 3, 4 → kind 4, joined at the car
/// along its forward row); the profile `FUN_0012e6f8`; +0x220/+0x138/+0x13c/+0x14c/+0x150/
/// +0x228 from +0x4c/+0x60/+0x5c/+0x68/+0x6c/+0x64; the +0x70 launch (velocity = forward ×
/// +0x70 unless +0x70 is 0 or 1.5); +0x158 = (+0x7d ≠ 0); the weapon bytes when the `.atr`
/// declares the weapon; +0x130/+0x131 = +0x93/+0x94. The +0x74 impulse (`FUN_001f8588` along
/// the constant at 0x36e380) and the +0x50 element target (`FUN_0012e770`) are not applied [S].
fn apply_command(v: &mut LiveVehicle, idx: usize, c: &AiCommand, attrs: Option<&VehicleAttrs>, net: Option<&RoadNet>, samples: &[(usize, f32, [f32; 3])], _path: bool) -> Option<([f32; 3], f32)> {
    let LiveKind::Car { car, brain: Some(b), .. } = &mut v.kind else { return None };
    if b.command == Some(idx) {
        return None;
    }
    b.command = Some(idx);
    let mut park = None;
    if c.clear_traffic != 0 {
        let row2 = [c.placement[2][0], c.placement[2][1], c.placement[2][2]];
        let f = norm([row2[0], 0.0, row2[2]]);
        let f = if len(f) > 0.0 { f } else { car.rot[2] };
        // vtbl+0x164(&goal, &row2): the placement [M]; the park radius is both boxes, the
        // caller adds the traffic car's own [M 0x145ff8:210-232].
        car.place(c.goal, [cross([0.0, 1.0, 0.0], f), [0.0, 1.0, 0.0], f]);
        park = Some((c.goal, car.params.half_extents[2]));
    }
    b.avoid_on = c.no_avoid == 0;
    match c.target_mode {
        0 => b.has_target = true,
        1 => {
            b.has_target = false;
            b.goal = c.goal;
        }
        _ => {}
    }
    b.target_mode = c.target_mode as u32;
    if c.pattern != 0 {
        b.pattern = c.pattern as u32;
        let fwd = car.rot[2];
        b.nav = match (b.pattern, net) {
            (PATTERN_ROAD, Some(n)) => join_nearest(n, samples, car.pos, fwd, 3),
            (PATTERN_ROAD_FOLLOW, Some(n)) => join_nearest(n, samples, car.pos, fwd, 4),
            (PATTERN_PATH, _) => None,
            (_, _) => b.nav.take(),
        };
    }
    if c.profile != 0 {
        b.profile = c.profile as u32;
    }
    b.follow = c.follow;
    b.cap = c.speed_cap;
    b.catchup = c.catchup;
    b.ram_chance = c.ram_chance;
    b.lead = c.lead;
    b.fire_rate = c.fire_rate;
    if c.launch_a != 0.0 && c.launch_a != 1.5 {
        car.vel = scale(norm(car.rot[2]), c.launch_a);
    }
    b.follow_target = c.follow_target != 0;
    if let Some(a) = attrs {
        if a.machine_guns || a.cannons {
            b.guns = c.guns.0 != 0 || c.guns.1 != 0;
        }
        if a.rockets {
            b.rockets = c.rockets != 0;
        }
        if a.missiles || a.torpedos {
            b.missiles = c.missiles != 0;
        }
    }
    b.accuracy = c.accuracy;
    b.damage_mod = c.damage_mod;
    park
}

// ------------------------------------------------------------------------------------------
// The per-vehicle step
// ------------------------------------------------------------------------------------------

/// The other live vehicles as this tick began.
#[derive(Debug, Clone, Copy)]
struct Snap {
    id: i16,
    pos: [f32; 3],
    side: [f32; 3],
    box_z: f32,
    chase: bool,
    car: bool,
}

impl Snap {
    fn of(v: &LiveVehicle) -> Snap {
        let r = v.rows();
        let (box_z, chase, car) = match &v.kind {
            LiveKind::Car { car, brain, .. } => (car.params.half_extents[2], brain.as_ref().is_some_and(|b| b.state == STATE_CHASE), true),
            _ => (2.0, false, false),
        };
        Snap { id: v.element as i16, pos: v.pos(), side: r[0], box_z, chase, car }
    }
}

struct Cx<'a> {
    net: Option<&'a RoadNet<'a>>,
    player: &'a PlayerState,
    rng: &'a mut DriveRng,
    ray: &'a mut dyn FnMut([f32; 3], [f32; 3]) -> u32,
    now: i32,
    others: &'a [Snap],
    samples: &'a [(usize, f32, [f32; 3])],
    speed_factor: f32,
    table: &'a WeaponSystem,
    difficulty: i32,
    out: &'a mut StepOut,
    projectiles: &'a mut Vec<HostileProjectile>,
}

fn body_of(car: &Car) -> Body {
    Body {
        pos: car.pos,
        vel: car.vel,
        side: car.rot[0],
        up: car.rot[1],
        fwd: car.rot[2],
        // sim +0x4c8 / +0x4c0 [M 0x1f6e98]; the port reads the car's half extents [S].
        box_z: car.params.half_extents[2],
        box_x: car.params.half_extents[0],
    }
}

/// The weapon a fire channel belongs to: the first of Gun, Cannon, Rocket, Missile whose barrel
/// list (0x33a0b0 [ELF data], `weapons::WEAPONS`) holds the channel.
pub fn weapon_of_channel(ch: u8) -> Option<usize> {
    [GUN, CANNON, ROCKET, MISSILE].into_iter().find(|&w| WEAPONS[w].barrels.contains(&ch))
}

/// One shell: the damage from `weapons::delivery` (the player's table × `ai_damage_scale`) [M
/// 0x14ca60]; the aim roll `aim_is_exact` (one draw) [M]. [S] The shell's flight
/// (`FUN_001fffa8`) is not traced: an exact shot hits the player; a wobbled one leaves along the
/// shot direction turned by `2·aim_spread·sin(now·0.02)` on x and z and hits when that line passes
/// within the player's box half length. The damage call is kind 2 with dent 0.25 (the bullet
/// caller `FUN_0015de48` [M]).
#[allow(clippy::too_many_arguments)]
fn fire_shell(cx: &mut Cx, element: usize, weapon: usize, from: [f32; 3], dir: [f32; 3], accuracy: i8, damage_mod: i8) {
    let shooter = Shooter { kind10: false, pov_or_remote: false, scale: ai_damage_scale(damage_mod) };
    let amount = match delivery(cx.table, weapon, &shooter) {
        Delivery::Shell { a, .. } => a,
        Delivery::RayShell { damage, .. } => damage,
        _ => return,
    };
    let r = cx.rng.next() as i32;
    let to_player = sub(cx.player.pos, from);
    let exact = aim_is_exact(accuracy, 0, r);
    let shot = if exact {
        norm(to_player)
    } else {
        let w = 2.0 * aim_spread(cx.difficulty) * (cx.now as f32 * 0.02).sin();
        norm([dir[0] + w, dir[1], dir[2] + w])
    };
    let along = dot(to_player, shot);
    let miss = len(sub(to_player, scale(shot, along)));
    let hit = exact || (along > 0.0 && miss <= cx.player.box_z);
    cx.out.events.push(HostileEvent::Shot { element, weapon, from, dir: shot, hit });
    if hit {
        cx.out.hits.push(HostileHit { element, weapon, hit: Hit { amount, dent: 0.25, kind: 2, at: cx.player.pos, from, from_player: false } });
    }
}

/// A rocket / missile: `weapons::delivery` → `launch_params` (an AI launcher) → `Projectile::launch`
/// [M 0x14ca60 / 0x180bc8]. The launch point is the vehicle's position [S: the barrel's bone is
/// the model's].
fn fire_projectile(cx: &mut Cx, element: usize, weapon: usize, from: [f32; 3], dir: [f32; 3], shooter_speed: f32, alpha_sub: bool, damage_mod: i8) {
    let shooter = Shooter { kind10: false, pov_or_remote: false, scale: ai_damage_scale(damage_mod) };
    let Delivery::Projectile { kind, damage } = delivery(cx.table, weapon, &shooter) else { return };
    let d2 = dot(sub(cx.player.pos, from), sub(cx.player.pos, from));
    let p = launch_params(kind, shooter_speed, damage, FPS, Launcher::AiCar { alpha_sub, lock_dist2: Some(d2) });
    cx.projectiles.push(HostileProjectile { element, weapon, p: Projectile::launch(kind, p, from, dir, cx.now) });
    cx.out.events.push(HostileEvent::Launch { element, weapon, from });
}

/// One ground vehicle: the brain (sim kind 2) and the car.
fn step_car<G: Ground + ?Sized>(v: &mut LiveVehicle, attrs: &VehicleAttrs, ground: &G, surface: Option<&dyn Ground>, rigid: &Rigid, cx: &mut Cx) {
    let element = v.element;
    let health = v.health;
    let name_alpha = v.name.eq_ignore_ascii_case("alpha_sub");
    let LiveKind::Car { car, brain, wheel_spin } = &mut v.kind else { return };
    let b0 = body_of(car);
    let mut cmd = DriveCommand::IDLE;
    let mut shots = None;
    if let Some(b) = brain.as_mut() {
        let player = *cx.player;
        let has_target = b.has_target;
        let nav_repeat = b.nav.as_ref().is_some_and(|n| n.repeated);
        // FUN_00131770: sight (every fps/4 ticks), the ram timers, the aim point, the switch.
        let td = len(sub(player.pos, b0.pos));
        let hits = (cx.ray)(b0.pos, player.pos);
        b.lost = sight_tick(&mut b.sight, cx.now, FPS, nav_repeat, has_target, b.target_mode, td, hits);
        ram_tick(&mut b.ram, false, b.lost, has_target, true, b.ram_chance, FPS, cx.rng);
        let carrot = b.nav.as_ref().map(|n| n.pos).unwrap_or(b0.pos);
        let aim = aim_point(&AimInputs {
            profile: b.profile,
            pattern: b.pattern,
            target_mode: b.target_mode,
            goal: b.goal,
            has_target,
            target_pos: player.pos,
            target_vel: player.vel,
            lost: b.lost,
            self_body: b0,
            lead: b.lead,
            follow: b.follow,
            prev_dist: b.prev_dist,
            prev_aim: b.prev_aim,
            nav_carrot: carrot,
            nav_10: carrot,
        });
        b.lost = aim.lost;
        b.prev_aim = aim.point;
        b.prev_dist = aim.dist;
        b.dir = aim.dir;
        if let (Some(net), Some(nav)) = (cx.net, b.nav.as_ref()) {
            // The profile-0xd pursue/road switch. The road under the target (`FUN_001351d0`) is
            // not ported: its lane test is given a zero half width, so pattern 2 re-joins only
            // when the target is lost [S].
            if let Some(sw) = pursue_switch(&SwitchInputs {
                profile: b.profile,
                pattern: b.pattern,
                lost: b.lost,
                now: cx.now,
                hold_until: b.hold_until,
                nav_repeat,
                dist: aim.dist,
                dir: aim.dir,
                follow: b.follow,
                nav_dir_30: nav.dir,
                target_speed: len(player.vel),
                self_fwd: b0.fwd,
                self_speed: b0.speed(),
                target_lane: 0.0,
                target_road_half_width: 0.0,
                target_road_dir: nav.dir,
            }) {
                let mut sw = sw;
                if let Some(h) = sw.hold_until {
                    b.hold_until = h;
                }
                if sw.rejoin {
                    // Re-join at the own position, kind 3, then advance 7.5 [M 0x131770].
                    if let Some(mut n) = join_nearest(net, cx.samples, b0.pos, b0.fwd, 3) {
                        let _ = n.advance(net, 7.5, &mut || cx.rng.next());
                        let fd = dot(b0.fwd, norm(sub(n.pos, b0.pos)));
                        b.nav = Some(n);
                        finish_rejoin(&mut sw, fd);
                    }
                }
                if let Some(p) = sw.pattern {
                    b.pattern = p;
                }
            }
            if matches!(b.pattern, PATTERN_ROAD | PATTERN_ROAD_FOLLOW) {
                let (latch, rejoin) = road_heading_latch(b.latch, dot(b0.fwd, aim.nav_dir));
                b.latch = latch;
                if rejoin {
                    let kind = if b.pattern == PATTERN_ROAD { 3 } else { 4 };
                    let back = scale(b0.fwd, -1.0);
                    if let Some(n) = join_nearest(net, cx.samples, b0.pos, back, kind) {
                        b.nav = Some(n);
                    }
                }
            }
        }
        let seg_limit = match (cx.net, b.nav.as_ref()) {
            (Some(net), Some(n)) => net.seg(n.seg).map(|s| s.speed_limit()).unwrap_or(0.0),
            _ => 0.0,
        };
        let sp = chase_target_speed(&ChaseSpeedInputs {
            state: b.state,
            profile: b.profile,
            pattern: b.pattern,
            alive: health,
            stop_query: false,
            stop_byte: false,
            self_pos: b0.pos,
            self_vel: b0.vel,
            self_forward: b0.fwd,
            self_box: b0.box_z,
            has_target,
            target_is_player: has_target,
            target_vel: player.vel,
            target_box: player.box_z,
            aim: aim.point,
            dist: aim.dist,
            dir: aim.dir,
            dir_210: aim.nav_dir,
            follow: b.follow,
            catchup: b.catchup,
            cap: b.cap,
            speed_mul: 1.0,
            ramming: b.ram.ram > 0,
            lost: b.lost,
            element_target: false,
            follow_target: b.follow_target,
            spawn_delay: b.ram.spawn_delay,
            seg_limit,
            speed_factor: cx.speed_factor,
        });
        if sp.ram_cleared {
            b.ram.ram = 0;
        }
        // The carrot (`FUN_00132f80` → `FUN_00130140`) for the road patterns: keep it a
        // lookahead ahead; the hint is n(target − car) for profile 0xd with a target, else
        // +0x1e0 [M 0x130140]; the target piece +0x1fc is not tracked (−1) [S].
        if let (Some(net), Some(n)) = (cx.net, b.nav.as_mut()) {
            let la = lookahead(b0.speed(), b0.box_z);
            let gap = len(sub(b0.pos, n.pos));
            if gap < la {
                let hint = if b.profile == 0xd && has_target { norm(sub(player.pos, b0.pos)) } else { aim.dir };
                let _ = n.advance_with(net, la - gap, NavHint { dir: hint, target: -1 }, &mut || cx.rng.next());
            }
        }
        let carrot = b.nav.as_ref().map(|n| n.pos).unwrap_or(b0.pos);
        let arm = dispatch(&DispatchInputs {
            state: b.state,
            pattern: b.pattern,
            alive: health,
            stop_query: false,
            stop_byte: false,
            target_speed: sp.speed,
            avoid_on: b.avoid_on,
            nav_repeat: b.nav.as_ref().is_some_and(|n| n.repeated),
            self_pos: b0.pos,
            dir: aim.dir,
            lookahead: lookahead(b0.speed(), b0.box_z),
            carrot,
        });
        cmd = match arm {
            Arm::Chase { speed, mut aim, avoid } => {
                if avoid == Avoid::Chase {
                    let others: Vec<OtherCar> = cx
                        .others
                        .iter()
                        .filter(|o| o.car)
                        .map(|o| OtherCar { id: o.id, pos: o.pos, sim_6d: 1, sim_6e: 2, side: o.side, box_z: o.box_z, width: 3, is_target: false })
                        .collect();
                    let d = norm(sub(aim, b0.pos));
                    // The two 32-unit side rays from the aim (`FUN_001369a0`) are not cast [S].
                    avoid_chase(&mut b.slots, &b0, element as i16, d, nav_repeat, (None, None), &others, &mut aim);
                }
                let i = ChaseDriveInputs {
                    profile: b.profile,
                    pattern: b.pattern,
                    spawn_delay: b.ram.spawn_delay,
                    hold_160: false,
                    hold_15c: false,
                    nav_repeat,
                    has_target,
                    target_is_player: has_target,
                    target_pos: player.pos,
                    target_vel: player.vel,
                    target_fwd: player.fwd,
                    dir: b.dir,
                    held_steer: 0.0,
                    fps: FPS,
                };
                // The bump probe needs the contact flag +0xdc / +0xe0 (not raised by the port's
                // car) [S]: no bump.
                chase_drive(&b0, aim, speed, &mut b.reverse, &i, &mut |_| false).drive()
            }
            Arm::Gated => gated_stop(&SimCar { pos: b0.pos, vel: b0.vel, forward: b0.fwd, right: b0.side, alive: health }),
            _ => DriveCommand::IDLE,
        };
        b.last_speed = sp.speed;
        b.last_cmd = cmd;
        // FUN_00130050: the turret (`FUN_0012f170`, before the fire when +0x12c), the line of
        // fire, the fire step.
        if attrs.turret {
            turret_step(&mut b.turret, b.lost, b.target_mode, aim.dir, b0.fwd, attrs.turret_range);
        }
        let shot_dir = if attrs.turret && b.turret.on_target { aim.dir } else { b0.fwd };
        let lof: Vec<LofCar> = cx.others.iter().filter(|o| o.car).map(|o| LofCar { id: o.id, pos: o.pos, sim_6d: 1, sim_6e: 2, chase_state: o.chase }).collect();
        let clear = line_clear(aim.dist, element as i16, shot_dir, b0.pos, &lof);
        for t in b.busy.iter_mut() {
            *t = (*t - 1).max(0);
        }
        let s = fire_step(
            &mut b.weapons,
            &FireInputs {
                alive: health,
                stop_byte: false,
                weapons_on: b.weapons_on,
                has_target,
                lost: b.lost,
                seen: b.sight.seen,
                profile: b.profile,
                turret: attrs.turret,
                turret_on_target: b.turret.on_target,
                guns: b.guns,
                rockets: b.rockets,
                missiles: b.missiles,
                aim_dot: dot(aim.dir, shot_dir),
                line_clear: clear,
                rate: b.fire_rate,
                fps: FPS,
                rocket_busy: [b.busy[1] > 0, b.busy[0] > 0],
                missile_busy: [b.busy[3] > 0, b.busy[2] > 0],
            },
        );
        shots = Some((s, shot_dir, b.accuracy, b.damage_mod));
    }
    // The car (`Car::step`) on the caller's ground.
    let mut input = CarInput { throttle: cmd.throttle, brake: cmd.brake, steer: cmd.steer, handbrake: cmd.handbrake };
    let (pre_s, pre_c) = car.yaw.sin_cos();
    let rail = ground.legacy_flat().is_some();
    if rail {
        // The rail model has no reverse state of its own: the land controller's rule is applied
        // here — brake held more than 29 ticks at a longitudinal speed ≤ 0 → reversing, the
        // counter reset when the brake is up and cleared at ≥ 2.5 forward [M 0x18cef8
        // 0x18d0e0..0x18d18c, as `physics::Car::controller`]. The rail yaw ignores the travel
        // sign, so a reversing car's steer is mirrored [S].
        let long_v = car.vel[0] * pre_s + car.vel[2] * pre_c;
        car.reversing = false;
        if input.brake == 0.0 {
            car.brake_ticks = 0x1e;
        } else if long_v >= 2.5 {
            car.brake_ticks = 0;
        } else if car.brake_ticks > 0x1d {
            car.reversing = long_v <= 0.0;
        } else {
            car.brake_ticks += 1;
            if long_v < 0.5 {
                input.brake = 0.0;
            }
        }
        if car.reversing {
            input.steer = -input.steer;
        }
    }
    car.step(&input, ground, rigid, DT);
    if rail {
        // The rail model's two stand-ins, as `ai::AiDriver::step` [S] (the first only while
        // not reversing).
        if !car.reversing && input.throttle == 0.0 && input.brake > 0.0 && car.vel[0] * pre_s + car.vel[2] * pre_c < 0.0 {
            car.vel = [0.0, car.vel[1], 0.0];
        }
        let (s, c) = car.yaw.sin_cos();
        let along = car.vel[0] * s + car.vel[2] * c;
        car.vel[0] = s * along;
        car.vel[2] = c * along;
        if let Some(y) = surface.and_then(|sf| ground_y(sf, car.pos)) {
            car.pos[1] = y + car.params.half_extents[1] + car.params.spring_rest;
        }
    }
    let r = car.params.wheel_radius.max(0.05);
    *wheel_spin += dot(car.vel, car.rot[2]) / r * DT;
    let pos = car.pos;
    let speed = len(car.vel);
    if let Some((s, dir, acc, dm)) = shots {
        if s.gun {
            let w = if attrs.cannons && !attrs.machine_guns { CANNON } else { GUN };
            fire_shell(cx, element, w, pos, dir, acc, dm);
        }
        if let Some(ch) = s.rocket {
            if let LiveKind::Car { brain: Some(b), .. } = &mut v.kind {
                b.busy[(ch - 5) as usize] = LAUNCHER_BUSY_TICKS;
            }
            if let Some(w) = weapon_of_channel(ch) {
                fire_projectile(cx, element, w, pos, dir, speed, name_alpha, dm);
            }
        }
        if let Some(ch) = s.missile {
            if let LiveKind::Car { brain: Some(b), .. } = &mut v.kind {
                b.busy[(ch - 5) as usize] = LAUNCHER_BUSY_TICKS;
            }
            if let Some(w) = weapon_of_channel(ch) {
                fire_projectile(cx, element, w, pos, dir, speed, name_alpha, dm);
            }
        }
    }
}

/// The helicopter's world for `heli::Heli` [S where noted].
struct HeliWorld<'a, G: Ground + ?Sized> {
    ground: &'a G,
    net: Option<&'a RoadNet<'a>>,
    samples: &'a [(usize, f32, [f32; 3])],
    target: Body,
    nav: &'a mut Option<RoadNav>,
    busy: [i32; 2],
    last_y: f32,
}

impl<G: Ground + ?Sized> HeliEnv for HeliWorld<'_, G> {
    fn ground(&mut self, p: [f32; 3]) -> f32 {
        // [S] No triangle under the helicopter: the last height found.
        if let Some(y) = ground_y(self.ground, p) {
            self.last_y = y;
        }
        self.last_y
    }
    fn target(&self) -> Body {
        self.target
    }
    fn fps(&self) -> i32 {
        FPS
    }
    /// The EAGL path playback is not ported [S]: no pose, the path mode holds.
    fn path_pose(&mut self) -> Option<PathPose> {
        None
    }
    fn nav_carrot(&self) -> [f32; 3] {
        self.nav.as_ref().map_or(self.target.pos, |n| n.pos)
    }
    fn nav_dir(&self) -> [f32; 3] {
        self.nav.as_ref().map_or([0.0, 0.0, 1.0], |n| n.dir)
    }
    fn nav_road(&self) -> u16 {
        self.nav.as_ref().map_or(0xffff, |n| n.seg as u16)
    }
    fn nav_rejoin(&mut self, pos: [f32; 3], dir: [f32; 3]) {
        if let Some(net) = self.net {
            *self.nav = join_nearest(net, self.samples, pos, dir, 3);
        }
    }
    fn nav_reset(&mut self) {}
    fn nav_advance(&mut self, dist: f32, dir: [f32; 3], _road: Option<u16>) {
        if let (Some(net), Some(n)) = (self.net, self.nav.as_mut()) {
            let _ = n.advance_with(net, dist, NavHint { dir, target: -1 }, &mut || 0u16);
        }
    }
    /// `FUN_00135018` (the target's road) is not ported [S]: the held road.
    fn target_road(&mut self, _dir: [f32; 3]) -> u16 {
        self.nav_road()
    }
    fn route_differs(&mut self, _road: u16) -> bool {
        false
    }
    fn probe_carrot(&mut self, _pos: [f32; 3], _fwd: [f32; 3]) -> [f32; 3] {
        self.nav_carrot()
    }
    fn crash_hit(&mut self, a: [f32; 3], b: [f32; 3]) -> bool {
        self.ground.wall_hit(a, b).is_some()
    }
    fn launcher_busy(&self, ch: u8) -> bool {
        self.busy[(ch.saturating_sub(7) as usize).min(1)] > 0
    }
}

/// One helicopter tick: `Heli::update` [M 0x138ef0], then the stand-in integration of its four
/// flight inputs [S: the helicopter physics class `FUN_002f1428` is not ported — the same
/// integration as the helicopter disc test: vertical acceleration lift − 9.8, yaw rate = yaw
/// clamped to ±1 rad/s, the flat velocity easing toward forward × roll·50]. Returns whether the
/// actor despawned its element.
fn step_heli<G: Ground + ?Sized>(v: &mut LiveVehicle, ground: &G, cx: &mut Cx) -> bool {
    let element = v.element;
    let health = v.health;
    let LiveKind::Heli(h) = &mut v.kind else { return false };
    h.actor.health = health;
    let p = *cx.player;
    let target = Body { pos: p.pos, vel: p.vel, side: cross([0.0, 1.0, 0.0], p.fwd), up: [0.0, 1.0, 0.0], fwd: p.fwd, box_z: p.box_z, box_x: 1.0 };
    for t in h.busy.iter_mut() {
        *t = (*t - 1).max(0);
    }
    let mut ev = Vec::new();
    let last_y = h.actor.body.pos[1];
    let mut env = HeliWorld { ground, net: cx.net, samples: cx.samples, target, nav: &mut h.nav, busy: h.busy, last_y };
    h.heli.update(&mut h.actor, &mut env, cx.rng, &mut ev);
    if h.actor.a8 {
        let b = &mut h.actor.body;
        b.vel[1] += (h.actor.lift - 9.8) * DT;
        h.heading += h.actor.yaw.clamp(-1.0, 1.0) * DT;
        let (s, c) = h.heading.sin_cos();
        b.fwd = [s, 0.0, c];
        b.side = [c, 0.0, -s];
        let want = [b.fwd[0] * h.actor.roll * 50.0, b.fwd[2] * h.actor.roll * 50.0];
        b.vel[0] += (want[0] - b.vel[0]) * DT;
        b.vel[2] += (want[1] - b.vel[2]) * DT;
        for k in 0..3 {
            b.pos[k] += b.vel[k] * DT;
        }
    }
    let pos = h.actor.body.pos;
    let speed = h.actor.body.speed();
    let mut gone = false;
    for e in ev {
        match e {
            // `FUN_001a27a0(weapon, 9, 10)`: channel 9 = the gun [M 0x1387a0; ELF data].
            HeliEvent::Gun { aim } => fire_shell(cx, element, GUN, pos, aim, 0, 0),
            HeliEvent::Rocket { channel, target } => {
                h.busy[(channel.saturating_sub(7) as usize).min(1)] = LAUNCHER_BUSY_TICKS;
                if let Some(w) = weapon_of_channel(channel) {
                    fire_projectile(cx, element, w, pos, norm(sub(target, pos)), speed, false, 0);
                }
            }
            HeliEvent::Blast { pos } => cx.out.events.push(HostileEvent::Blast { element, pos }),
            HeliEvent::Despawn => gone = true,
        }
    }
    gone
}

/// The seated gunners: the window (`FUN_00122d28` / `FUN_00122780` / `FUN_00122920`), the
/// snowmobile (`FUN_0011f528` / `FUN_0011e9a0`) and the sunroof (`FUN_00121818` /
/// `FUN_00120fe0`) classes, aiming at `riders::pick_target` = the player [M]. [S] The order of
/// the window class's tick / acquire / fire inside its state machine was not traced: each tick
/// runs the bookkeeping, acquires outside states 2 / 0xb, and runs the fire state in them. The
/// mount is the seat vector turned by the car's rows (the game's mount is the seat bone).
fn step_riders(v: &mut LiveVehicle, cx: &mut Cx) {
    if v.riders.is_empty() {
        return;
    }
    let element = v.element;
    let rows = v.rows();
    let pos = v.pos();
    let health = v.health;
    let disabled = health <= 0.0;
    let target = cx.player.pos;
    for r in &mut v.riders {
        let o = r.offset;
        let mount = add(pos, add(add(scale(rows[0], o[0]), scale(rows[1], o[1])), scale(rows[2], o[2])));
        let fwd = rows[2];
        let (on, side) = aim_test(mount, fwd, target);
        let rng = &mut *cx.rng;
        let mut rand = || rng.next();
        let fire = match r.class.seat {
            3 => {
                window_tick(&mut r.gunner, health);
                let sight = window_sight(mount, fwd, target, r.range_sq);
                if !matches!(r.gunner.state, 2 | 0xb) {
                    window_acquire(&mut r.gunner, disabled, sight, FPS);
                }
                if matches!(r.gunner.state, 2 | 0xb) {
                    let yaw = &mut r.sweep_yaw;
                    window_fire(&mut r.gunner, cx.now, disabled, sight, &mut || {
                        crate::driving::patterns::gunner_sweep(yaw, false, side as u32, 0);
                    }, &mut rand)
                } else {
                    Fire::Wait
                }
            }
            0xe => {
                let (sight, _) = snow_sight(mount, fwd, target, r.range_sq);
                snow_fire(&mut r.gunner, cx.now, sight, on, &mut rand)
            }
            4 => {
                let sight = sunroof_sight(mount, target, r.range_sq, [false, false]);
                let w = (r.weapon as i64).clamp(1, 2) as u32;
                sunroof_fire(&mut r.gunner, cx.now, disabled, sight, false, on, w, &mut || {}, &mut rand)
            }
            _ => Fire::Wait,
        };
        if let Fire::Shot(anim) = fire {
            cx.out.events.push(HostileEvent::RiderShot { element, rider: r.index, anim, from: mount });
        }
    }
}

#[cfg(test)]
mod swap_tests {
    use super::*;
    use nf_formats::driveai::AiElement;

    fn vehicle(name: &str, riders: i32) -> AiElement {
        let mut b = vec![0u8; 0x100];
        b[0x44..0x48].copy_from_slice(&TYPE_VEHICLE.to_le_bytes());
        b[0x54..0x54 + name.len()].copy_from_slice(name.as_bytes());
        b[0x7c..0x80].copy_from_slice(&riders.to_le_bytes());
        for k in 0..riders as usize { b[0x80 + k * 0x20..0x84 + k * 0x20].copy_from_slice(&3u32.to_le_bytes()); }
        AiElement::parse(&b).unwrap()
    }

    fn map() -> MapAi {
        let mut hench = vehicle("", 0);
        hench.kind = TYPE_HENCHMAN;
        MapAi { elements: vec![vehicle("paris_hench", 2), vehicle("alpine_police", 1), hench], ..Default::default() }
    }

    // The MIS4 attribute renders [D MIS4.VIV pvehicle\paris_hench.atr RENDER_FILENAME.s=paris_hench.crp].
    fn render(name: &str) -> String { format!("{name}.crp") }

    #[test]
    fn swap_only_on_snow2a_mis4_at_word_2() {
        // [M DRIVING 0x12b8b0:96-125]
        for (key, word, want) in [
            ("snow2a_mis4", 2, 1),
            ("snow2a_mis4", 1, 0),
            ("snow2a_mis4", 0, 0),
            ("snow2a_mis4", 3, 0), // a raw 3 never reaches the reader: DRIVING main remaps it to 2 first
            ("snow2a_race", 2, 0),
            ("snow1a_mis3", 2, 0),
            ("SNOW2A_MIS4", 2, 0),
        ] {
            let mut m = map();
            assert_eq!(snow_hench_swap(&mut m, key, word, render), want, "{key} {word}");
            let e = &m.elements[0];
            if want == 1 {
                assert_eq!(e.name, "junglesnow");
                assert!(e.riders.is_empty(), "+0x7c zeroed");
            } else {
                assert_eq!(e.name, "paris_hench");
                assert_eq!(e.riders.len(), 2);
            }
            assert_eq!(m.elements[1].name, "alpine_police");
            assert_eq!(m.elements[1].riders.len(), 1);
        }
    }

    #[test]
    fn swap_matches_the_rendername_prefix_not_the_attribute_name() {
        let mut m = MapAi { elements: vec![vehicle("henchcar", 0), vehicle("paris_hench", 0)], ..Default::default() };
        let n = snow_hench_swap(&mut m, HENCH_SWAP_TRACK, 2, |n| if n == "henchcar" { "paris_henchbigbox.crp".into() } else { "other.crp".into() });
        assert_eq!(n, 1);
        assert_eq!(m.elements[0].name, "junglesnow");
        assert_eq!(m.elements[1].name, "paris_hench");
    }
}
