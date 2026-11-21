//! The driving missions' car physics (DRIVING.ELF's CARP layer), headless — the tuning DB, the
//! per-car parameters, and the player car's per-tick step at the game's fixed 1/60 s.
//!
//! Evidence base (all [M] unless noted — DRIVING.ELF decompile under `work/decomp/DRIVING.ELF/`,
//! addresses = EE; docs/systems/drive-physics.md is the full map):
//! - Tick rate: `FUN_001feda0` ("Reset Sim") stores `fps = FUN_002d6b90()` (the {60,60,50,60}
//!   table [M 0x2d6b90]) at sim+0xdbc and `1/fps` at sim+0xdc0 = `DAT_00339f50` [M :9-12]; the
//!   frame tick `FUN_00159d10` adds 0.016666668 per pass [M :29]. NTSC = 60 Hz.
//! - Tuning: the `.tun` files are ASCII `KEY=VALUE` text [D], looked up per registered variable
//!   by substring (`FUN_001426a8`: find the name, next char `' '`/`'='`, skip `=#([`, sscanf
//!   `%d/%u/%f`); no match keeps the compiled-in default [M]. The Physics namespaces' registered
//!   defaults equal the shipped `data\tuning\Physics\{Rigid,Physical,Friction}\default.tun`
//!   values, which are byte-identical in all 8 mission VIVs [D].
//! - Engine/gears/steering: `FUN_0018cef8` + the auto-shift/RPM `FUN_0018aa88`; tyre/lateral
//!   forces `FUN_00190f88`; gravity + ground damping `FUN_001f7c60`; the force API
//!   `FUN_001f87d8` (accel·mass accumulators); the integration is VU0 microcode
//!   (`FUN_001f89e0` → `.vutext`) — modelled as semi-implicit Euler here [S], see the doc §6.
//! - The land car (`Car::step`): FUN_00188200's SUB_PHYSICS 0 / IS_SNOWMOBILE 0 / player branch
//!   runs `FUN_0018cef8` + the per-wheel `FUN_0018c1b0` against the ground triangles
//!   (`CollisionGround`); docs/systems/drive-physics.md §8 has the function-by-function table.
//!   `FlatWorld` keeps the flat no-wheel model (`Car::step_rail`) for callers without collision.
//! - Per-car parameters: the pvehicle attribute schema [M `FUN_001968f8`]; the shipped
//!   `data\sim\attrib\pvehicle\vanquish.atr` (Bond's Aston Martin) [D].

use std::collections::BTreeMap;

/// The game's fixed physics step [M `FUN_002d6b90`'s {60,60,50,60} table; sim+0xdc0 = 1/fps].
pub const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------------------
// The .tun text database — `KEY=VALUE` lines [M FUN_001426a8's lookup semantics]
// ---------------------------------------------------------------------------

/// One open tuning file: the parsed `KEY=VALUE` pairs. The game keeps the text in memory and
/// substring-searches it per registered variable name (`FUN_001790f8`), then sscanf's the number
/// [M 0x1426a8]; the parser here keeps the same case-sensitivity and the same "first line whose
/// start matches" semantics at the granularity the port's variables need. The map is a
/// `BTreeMap` (cross-process-determinism): `get`'s prefix fallback breaks equal-length ties by
/// iteration order, and a `HashMap`'s order is randomly seeded per process — two processes could
/// resolve the same ambiguous prefix to different values and drive the car sim apart. A
/// `BTreeMap` iterates in key order, so the tie-break (shortest key, then lexicographically
/// smallest) is a pure function of the text.
#[derive(Debug, Default, Clone)]
pub struct Tuning {
    values: BTreeMap<String, f32>,
}

impl Tuning {
    /// Parses `KEY=VALUE` text ([D] layout: CRLF lines, no sections; `//` text after a value is
    /// not part of the .tun form — the sscanf `%f` stops at the first non-numeric char, so a
    /// trailing space or letter is ignored exactly like `%f` does).
    pub fn parse(text: &str) -> Tuning {
        let mut values = BTreeMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            // sscanf("%f") semantics: leading whitespace skip, then the longest float prefix.
            let v = value.trim();
            let num: &str = v.split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')))
                .next().unwrap_or("");
            if let Ok(f) = num.parse::<f32>() {
                values.insert(key.to_string(), f);
            }
        }
        Tuning { values }
    }

    /// The substring lookup the game does: the registered variable name matches when the file
    /// holds a key that starts with it followed by ` ` or `=` [M 0x1426a8/0x1790f8]. The port's
    /// keys are exact (the .tun files spell them the way the code registers them), so an exact
    /// hit first, then the longest registered-prefix match.
    pub fn get(&self, name: &str) -> Option<f32> {
        if let Some(v) = self.values.get(name) { return Some(*v); }
        let mut best: Option<(&String, &f32)> = None;
        for (k, v) in &self.values {
            if k.starts_with(name) && k.len() > name.len() {
                let rest = &k[name.len()..];
                if rest.starts_with(' ') || rest.starts_with('=') || rest.starts_with('[') {
                    if best.map_or(true, |(bk, _)| (k.len(), k.as_str()) < (bk.len(), bk.as_str())) { best = Some((k, v)); }
                }
            }
        }
        best.map(|(_, v)| *v)
    }
}

// ---------------------------------------------------------------------------
// The Physics:Rigid globals [M FUN_001f6600 registers; .data initials 0x3390d0-0x339164]
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Rigid {
    pub gravity: f32,
    pub bodge_player_gravity: f32,
    pub sleep_vel: f32,
    pub ground_friction_coeff: f32,
    pub natural_angular_damping: f32,
    pub am_limit: f32,
    pub m_limit: f32,
    pub speed_force_limit: f32,
    pub speed_scale: f32,
    pub depth_force_scale: f32,
    pub total_force_limit: f32,
}

impl Rigid {
    /// The compiled-in initials — verified equal to the shipped `Rigid\default.tun` [M .data
    /// 0x3390d0-0x339164 via the decompile's DAT_0033xxxx reads; D the shipped file].
    pub fn game_defaults() -> Rigid {
        Rigid {
            gravity: -34.0,
            bodge_player_gravity: -13.0,
            sleep_vel: 5.0,
            ground_friction_coeff: 0.05,
            natural_angular_damping: 0.995,
            am_limit: 10.0,
            m_limit: 100.0,
            speed_force_limit: 10.0,
            speed_scale: 0.25,
            depth_force_scale: 4.5,
            total_force_limit: 40.0,
        }
    }

    /// The tun override pass — the game assigns each registered var from the file, keeping the
    /// initial when the name is absent [M 0x1426a8's no-match arm].
    pub fn from_tun(t: &Tuning) -> Rigid {
        let mut r = Rigid::game_defaults();
        let g = |t: &Tuning, k: &str| t.get(k);
        if let Some(v) = g(t, "GRAVITY") { r.gravity = v; }
        if let Some(v) = g(t, "BODGE_PLAYER_GRAVITY") { r.bodge_player_gravity = v; }
        if let Some(v) = g(t, "SLEEP_VEL") { r.sleep_vel = v; }
        if let Some(v) = g(t, "GROUND_FRICTION_COEFF") { r.ground_friction_coeff = v; }
        if let Some(v) = g(t, "NATURAL_ANGULAR_DAMPING") { r.natural_angular_damping = v; }
        if let Some(v) = g(t, "AM_LIMIT") { r.am_limit = v; }
        if let Some(v) = g(t, "M_LIMIT") { r.m_limit = v; }
        if let Some(v) = g(t, "SPEED_FORCE_LIMIT") { r.speed_force_limit = v; }
        if let Some(v) = g(t, "SPEED_SCALE") { r.speed_scale = v; }
        if let Some(v) = g(t, "DEPTH_FORCE_SCALE") { r.depth_force_scale = v; }
        if let Some(v) = g(t, "TOTAL_FORCE_LIMIT") { r.total_force_limit = v; }
        r
    }
}

// ---------------------------------------------------------------------------
// Physics:Friction — the 13-surface tables [M FUN_00185668's tail; .data 0x3319d8/0x331a18]
// ---------------------------------------------------------------------------

/// The `WSurface_k*` order the code registers: kNODRIVE, kPAVED, kGRAVEL, kGRASS, kCOBBLE,
/// kDIRT, kWATER, kWOOD, kICE, kSNOW, kPAVED_ROUGH, kPAVED_GRATE, kRAILROAD [M the enum's
/// registration order; the array bases 0x3319d8/0x331a18 stride 4].
pub const SURFACES: [&str; 13] = [
    "NODRIVE", "PAVED", "GRAVEL", "GRASS", "COBBLE", "DIRT", "WATER", "WOOD", "ICE", "SNOW",
    "PAVED_ROUGH", "PAVED_GRATE", "RAILROAD",
];

#[derive(Debug, Clone)]
pub struct Friction {
    pub friction: [f32; 13],
    pub lateral_loss: [f32; 13],
}

impl Friction {
    /// The compiled-in initials: all 1.0 [M .data 0x3319d8..; the shipped Friction\default.tun's
    /// `frictionCoefficients[WSurface_k*]` keys cannot match the registered `friction[k*]` names
    /// under the substring rule — dead values, defaults apply [S; live check requested]].
    pub fn game_defaults() -> Friction {
        Friction { friction: [1.0; 13], lateral_loss: [1.0; 13] }
    }
}

// ---------------------------------------------------------------------------
// Physics:Physical — the globals the land car's controller reads
// ---------------------------------------------------------------------------

/// `ROLLING_RESISTANCE` (0x3319a4): the 0x1d0 rolling vector is `vel · this`, subtracted once per
/// contacting wheel [M FUN_0018cef8 0x18d0xx (`0x1d0 = 0x1c0·DAT_003319a4`), FUN_0018c1b0's
/// `0x280 −= param_7`]; .data 0.1 = the shipped Physical\default.tun [D].
pub const ROLLING_RESISTANCE: f32 = 0.1;
/// `MIN_BUTTON_VALUE` (0x3319a8): the pedal threshold of the brake/accel state pick
/// [M FUN_0018cef8 `DAT_003319a8 < brake/throttle`]; .data 0.1 = the shipped tun [D].
pub const MIN_BUTTON_VALUE: f32 = 0.1;
/// `MAX_WHEEL_SPIN_RATE` (0x3319c0): the wheel-spin clamp [M FUN_0018cef8]; the .data initial
/// is 0.013 but the shipped Physical\default.tun sets 0.05 [D], and the tun pass overrides it.
pub const MAX_WHEEL_SPIN_RATE: f32 = 0.05;
/// The integer frame rate at 0x339f4c (written at sim reset from the {60,60,50,60} table,
/// NTSC 60 [M FUN_001feda0]); the wheel-roll term divides by it [M FUN_0018cef8].
pub const FPS: f32 = 60.0;
/// `DAT_00331970`: the wheel's attitude gate `0.2 < up_y · n_y` [M FUN_0018c1b0; .data 0.2].
pub const UPRIGHT_GATE: f32 = 0.2;

// ---------------------------------------------------------------------------
// The per-car parameters — the pvehicle attribute schema [M FUN_001968f8] + the shipped .atr [D]
// ---------------------------------------------------------------------------

/// The pvehicle attributes the land step reads, by their schema offset [M FUN_001968f8: the
/// name ↔ offset pairs are the `FUN_0016e590(.., name, .., offset, ..)` calls; names read from
/// the ELF strings 0x37adb8..0x37b278].
#[derive(Debug, Clone)]
pub struct CarParams {
    /// MASS (the ctor's attr read, FUN_001862c0 → inst+0x4c)
    pub mass: f32,
    /// +0x14
    pub maxacc: f32,
    /// +0x18
    pub maxrevacc: f32,
    /// +0x1c
    pub maxbrake: f32,
    /// +0x20
    pub maxsteering: f32,
    /// +0x24
    pub modsteer_speed: f32,
    /// +0x28
    pub min_steer: f32,
    /// [GEAR_LIMITR, LIMIT1..LIMIT5] [M the +0xd0 base indexed by gear 0..5]
    pub gear_limit: [f32; 6],
    /// [GEAR_RATIOR, RATIO1..RATIO5] [M the +0xe8 base indexed by gear 0..5]
    pub gear_ratio: [f32; 6],
    /// +0x40
    pub friction_limit_front: f32,
    /// +0x44
    pub friction_limit_rear: f32,
    /// +0x48
    pub tyre_grip_factor: f32,
    /// +0x6c
    pub wheel_radius: f32,
    /// +0x70
    pub braking_friction: f32,
    /// +0x8c
    pub front_slipping_scale: f32,
    /// +0x90
    pub rear_slipping_scale: f32,
    /// +0x94
    pub slipping_range: f32,
    /// +0x08
    pub speed_cutoff_reverse: f32,
    /// +0x0c
    pub speed_cutoff_rate_reverse: f32,
    /// +0x98
    pub handbrake_force: f32,
    /// +0x9c
    pub total_handbrake_friction_scale: f32,
    /// BOOST_EXTRA_ACC (the boost pickup path; the land step does not run boosts yet)
    pub boost_extra_acc: f32,
    /// CAR_WHEEL_ZF_OFFSET — added to the box's +z half extent for the front wheels
    /// [M FUN_001f72d8 `param_3[2] + ZF`]
    pub wheel_zf_offset: f32,
    /// CAR_WHEEL_ZR_OFFSET — added to −z half extent for the rear wheels [M FUN_001f72d8]
    pub wheel_zr_offset: f32,
    /// CAR_WHEEL_X_OFFSET — pulled in from the ±x half extent [M FUN_001f72d8 `−x + X`, `x − X`]
    pub wheel_x_offset: f32,
    /// The render model's half extents (model+0xc bbox (max−min)/2 [M FUN_001c9778]): the hull
    /// box, the wheel base points and the inertia [M FUN_001862c0 → FUN_00210938/FUN_001f6e98].
    pub half_extents: [f32; 3],
    /// +0x00 IS_4X4 (not read by the land step)
    pub is_4x4: bool,
    /// +0x04 IS_RALLY: front drive under throttle, the low-speed grip scale [M FUN_0018cef8,
    /// FUN_0018c1b0]
    pub is_rally: bool,
    /// +0x10 WHEEL_FORCE_APP_SCALE: the force application point's y scale [M FUN_0018c1b0]
    pub wheel_force_app_scale: f32,
    /// +0x2c / +0x30 / +0x34 FRIC_MOD_SPEED / RANGE / MIN [M FUN_0018cef8 → actor+0x2e8]
    pub fric_mod_speed: f32,
    pub fric_mod_range: f32,
    pub fric_mod_min: f32,
    /// +0x38 YAW_STABILITY_FACTOR (0 = the pass is skipped [M FUN_0018c1b0 tail])
    pub yaw_stability: f32,
    /// +0x4c SLOPE_SCALE
    pub slope_scale: f32,
    /// +0x50/+0x54 SPRING_STIFFNESS_FRONT/REAR
    pub spring_stiffness: [f32; 2],
    /// +0x58/+0x5c SPRING_DAMPING_FRONT/REAR
    pub spring_damping: [f32; 2],
    /// +0x60 SPRING_REST_LENGTH
    pub spring_rest: f32,
    /// +0x64 SPRING_COMPRESSION_LIMIT
    pub spring_limit: f32,
    /// TYRE_RADIUS, read by name (`FUN_00169d80(actor+0x5c, "TYRE_RADIUS", 0)`) by the righting
    /// reset `FUN_00187b00` and the placement `FUN_00187eb8` for the placed height [M]
    /// (vanquish.atr 0.065 [D])
    pub tyre_radius: f32,
    /// +0x74 / +0x78 / +0x7c / +0x80 / +0x84 / +0x88 ACCEL / COAST / BRAKE / REVERSE /
    /// HANDBRAKE / REV_HANDBRAKE _FRW_RATIO (the front share of the tyre grip)
    pub accel_frw: f32,
    pub coast_frw: f32,
    pub brake_frw: f32,
    pub reverse_frw: f32,
    pub handbrake_frw: f32,
    pub rev_handbrake_frw: f32,
    /// +0xa0 HANDBRAKE_RW_FRIC_SCALE
    pub handbrake_rw_fric_scale: f32,
    /// +0xa4 WHEELSPINSCALE
    pub wheelspin_scale: f32,
    /// +0xac WS_ACCEL
    pub ws_accel: f32,
    /// +0xb0 / +0xb4 COUNTER_STEER_SCALE / _LIMIT
    pub counter_steer_scale: f32,
    pub counter_steer_limit: f32,
}

impl CarParams {
    /// The values the earlier pass called "schema defaults". ⚠ The second float of each
    /// `FUN_0016ec90` registration call is read here as the default; for the ratio fields it
    /// reads 0.1/0.9 pairs (ACCEL_FRW_RATIO (0.1, 0.9)), which look like a (min, max) range, so
    /// these may be the attribute editor's maxima, not defaults [S — see drive-physics.md §2].
    /// The land-step fields use the vanquish values here; no other car is ported.
    pub fn game_defaults() -> CarParams {
        CarParams {
            mass: 2000.0,
            maxacc: 30.0,
            maxrevacc: 30.0,
            maxbrake: 30.0,
            maxsteering: 0.2,
            modsteer_speed: 50.0,
            min_steer: 0.0,
            gear_limit: [100.0; 6],
            gear_ratio: [1.0; 6],
            friction_limit_front: 50.0,
            friction_limit_rear: 50.0,
            tyre_grip_factor: 16.0,
            wheel_radius: 1.0,
            braking_friction: 3.0,
            front_slipping_scale: 1.0,
            rear_slipping_scale: 1.0,
            slipping_range: 10.0,
            speed_cutoff_reverse: 200.0,
            speed_cutoff_rate_reverse: 1000.0,
            handbrake_force: 30.0,
            total_handbrake_friction_scale: 1.0,
            boost_extra_acc: 0.0,
            wheel_zf_offset: -3.0,
            wheel_zr_offset: 3.0,
            ..CarParams::vanquish()
        }
    }

    /// Bond's Aston Martin Vanquish — the shipped `data\sim\attrib\pvehicle\vanquish.atr` [D;
    /// the file is RefPack-compressed in every mission VIV, byte-identical everywhere]; the half
    /// extents are the MIS01 `vanquish.crp` article bbox [D] (read as model+0xc's bbox by
    /// FUN_001c9778 — that the article Base payload is model+0xc is [S]).
    pub fn vanquish() -> CarParams {
        CarParams {
            mass: 1660.0,
            maxacc: 3.0,
            maxrevacc: 2.4,
            maxbrake: 3.15,
            maxsteering: 0.09,
            modsteer_speed: 10.0,
            min_steer: 0.425,
            gear_limit: [10.0, 18.0, 29.0, 50.0, 54.0, 69.0],
            gear_ratio: [1.0, 0.65, 0.59, 0.52, 0.45, 0.44],
            friction_limit_front: 19.0,
            friction_limit_rear: 19.0,
            tyre_grip_factor: 8.0,
            wheel_radius: 0.365,
            braking_friction: 0.3,
            front_slipping_scale: 0.421,
            rear_slipping_scale: 0.425,
            slipping_range: 25.0,
            speed_cutoff_reverse: 10.0,
            speed_cutoff_rate_reverse: 0.075,
            handbrake_force: 0.1,
            total_handbrake_friction_scale: 0.15,
            boost_extra_acc: 1.75,
            wheel_zf_offset: -0.894,
            wheel_zr_offset: 1.006,
            wheel_x_offset: 0.16,
            half_extents: [0.919_357_66, 0.577_840_7, 2.316_776_3],
            is_4x4: false,
            is_rally: false,
            wheel_force_app_scale: 0.5,
            fric_mod_speed: 10.0,
            fric_mod_range: 75.0,
            fric_mod_min: 0.45,
            // absent from vanquish.atr; the step tests `== 0.0` to skip the pass, read as 0 [S]
            yaw_stability: 0.0,
            slope_scale: 0.5,
            spring_stiffness: [80.0, 98.0],
            spring_damping: [400.0, 350.0],
            spring_rest: 0.24,
            spring_limit: 0.75,
            tyre_radius: 0.065,
            accel_frw: 0.25,
            coast_frw: 0.35,
            brake_frw: 0.65,
            reverse_frw: 0.6,
            handbrake_frw: 0.9,
            rev_handbrake_frw: 0.6,
            handbrake_rw_fric_scale: 1.0,
            wheelspin_scale: 0.015,
            ws_accel: 30.0,
            counter_steer_scale: 1.72,
            counter_steer_limit: 2.0,
        }
    }

    /// Parses the pvehicle `.atr` INI text: `[section]` headers (ignored — one car per file),
    /// `KEY.suffix=VALUE` (the suffix `.f/.i/.u/.s/.v` selects the attribute's type [D]; the
    /// port reads the numeric ones), `//` comments, whitespace-padded values [D vanquish.atr].
    /// Keys absent from the text keep `game_defaults()`.
    pub fn parse_atr(text: &str) -> CarParams {
        let mut p = CarParams::game_defaults();
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('[') { continue; }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let key = key.split('.').next().unwrap_or(key).trim();
            let num: &str = value.trim()
                .split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')))
                .next().unwrap_or("");
            let Ok(v) = num.parse::<f32>() else { continue };
            match key {
                "MASS" => p.mass = v,
                "MAXACC" => p.maxacc = v,
                "MAXREVACC" => p.maxrevacc = v,
                "MAXBRAKE" => p.maxbrake = v,
                "MAXSTEERING" => p.maxsteering = v,
                "MODSTEERSPEED" => p.modsteer_speed = v,
                "MINSTEER" => p.min_steer = v,
                "GEAR_LIMITR" => p.gear_limit[0] = v,
                "GEAR_LIMIT1" => p.gear_limit[1] = v,
                "GEAR_LIMIT2" => p.gear_limit[2] = v,
                "GEAR_LIMIT3" => p.gear_limit[3] = v,
                "GEAR_LIMIT4" => p.gear_limit[4] = v,
                "GEAR_LIMIT5" => p.gear_limit[5] = v,
                "GEAR_RATIOR" => p.gear_ratio[0] = v,
                "GEAR_RATIO1" => p.gear_ratio[1] = v,
                "GEAR_RATIO2" => p.gear_ratio[2] = v,
                "GEAR_RATIO3" => p.gear_ratio[3] = v,
                "GEAR_RATIO4" => p.gear_ratio[4] = v,
                "GEAR_RATIO5" => p.gear_ratio[5] = v,
                "FRICTIONLIMITFRONT" => p.friction_limit_front = v,
                "FRICTIONLIMITREAR" => p.friction_limit_rear = v,
                "TYREGRIPFACTOR" => p.tyre_grip_factor = v,
                "WHEEL_RADIUS" => p.wheel_radius = v,
                "BRAKING_FRICTION" => p.braking_friction = v,
                "FRONT_SLIPPING_SCALE" => p.front_slipping_scale = v,
                "REAR_SLIPPING_SCALE" => p.rear_slipping_scale = v,
                "SLIPPING_RANGE" => p.slipping_range = v,
                "SPEED_CUTOFF_REVERSE" => p.speed_cutoff_reverse = v,
                "SPEED_CUTOFF_RATE_REVERSE" => p.speed_cutoff_rate_reverse = v,
                "HANDBRAKE_FORCE" => p.handbrake_force = v,
                "TOTAL_HANDBRAKE_FRICTION_SCALE" => p.total_handbrake_friction_scale = v,
                "BOOST_EXTRA_ACC" => p.boost_extra_acc = v,
                "CAR_WHEEL_ZF_OFFSET" => p.wheel_zf_offset = v,
                "CAR_WHEEL_ZR_OFFSET" => p.wheel_zr_offset = v,
                "CAR_WHEEL_X_OFFSET" => p.wheel_x_offset = v,
                "IS_4X4" => p.is_4x4 = v != 0.0,
                "IS_RALLY" => p.is_rally = v != 0.0,
                "WHEEL_FORCE_APP_SCALE" => p.wheel_force_app_scale = v,
                "FRIC_MOD_SPEED" => p.fric_mod_speed = v,
                "FRIC_MOD_RANGE" => p.fric_mod_range = v,
                "FRIC_MOD_MIN" => p.fric_mod_min = v,
                "YAW_STABILITY_FACTOR" => p.yaw_stability = v,
                "SLOPE_SCALE" => p.slope_scale = v,
                "SPRING_STIFFNESS_FRONT" => p.spring_stiffness[0] = v,
                "SPRING_STIFFNESS_REAR" => p.spring_stiffness[1] = v,
                "SPRING_DAMPING_FRONT" => p.spring_damping[0] = v,
                "SPRING_DAMPING_REAR" => p.spring_damping[1] = v,
                "SPRING_REST_LENGTH" => p.spring_rest = v,
                "SPRING_COMPRESSION_LIMIT" => p.spring_limit = v,
                "TYRE_RADIUS" => p.tyre_radius = v,
                "ACCEL_FRW_RATIO" => p.accel_frw = v,
                "COAST_FRW_RATIO" => p.coast_frw = v,
                "BRAKE_FRW_RATIO" => p.brake_frw = v,
                "REVERSE_FRW_RATIO" => p.reverse_frw = v,
                "HANDBRAKE_FRW_RATIO" => p.handbrake_frw = v,
                "REV_HANDBRAKE_FRW_RATIO" => p.rev_handbrake_frw = v,
                "HANDBRAKE_RW_FRIC_SCALE" => p.handbrake_rw_fric_scale = v,
                "WHEELSPINSCALE" => p.wheelspin_scale = v,
                "WS_ACCEL" => p.ws_accel = v,
                "COUNTER_STEER_SCALE" => p.counter_steer_scale = v,
                "COUNTER_STEER_SCALE_LIMIT" => p.counter_steer_limit = v,
                _ => {}
            }
        }
        p
    }

    /// The four wheel base points in the body frame (x right, y up, z forward), wheel order
    /// FL, FR, RL, RR [M FUN_001f72d8's model-hull branch: class+0x80.. =
    /// (−hx+X, −hy, hz+ZF), (hx−X, −hy, hz+ZF), (−hx+X, −hy, −hz+ZR), (hx−X, −hy, −hz+ZR)].
    pub fn wheel_points(&self) -> [[f32; 3]; 4] {
        let [hx, hy, hz] = self.half_extents;
        let x = self.wheel_x_offset;
        [
            [-hx + x, -hy, hz + self.wheel_zf_offset],
            [hx - x, -hy, hz + self.wheel_zf_offset],
            [-hx + x, -hy, -hz + self.wheel_zr_offset],
            [hx - x, -hy, -hz + self.wheel_zr_offset],
        ]
    }

    /// The per-unit-mass inverse inertia about the body x / y / z axes [M FUN_00210938: the
    /// `hz ≤ 4` branch `z: 0.5/(m·0.5·(hx²+hy²))`, `x = y: 0.5/(m·0.08333·(2hz)² + m·0.25·(hx²+hy²))`;
    /// the `hz > 4` branch `m·0.3333·(..)`], times the mass the force API multiplies in
    /// [M FUN_001f8888/FUN_001f88c8 scale torques by inst+0x4c].
    pub fn inv_inertia_per_mass(&self) -> [f32; 3] {
        let [hx, hy, hz] = self.half_extents;
        if hz > 4.0 {
            let k = 0.3333;
            [1.0 / (k * (hy * hy + hz * hz)), 1.0 / (k * (hx * hx + hz * hz)), 1.0 / (k * (hx * hx + hy * hy))]
        } else {
            let s = hx * hx + hy * hy;
            let xy = 0.5 / (0.08333 * (hz + hz) * (hz + hz) + 0.25 * s);
            [xy, xy, 0.5 / (0.5 * s)]
        }
    }
}

// ---------------------------------------------------------------------------
// The ground the car stands on
// ---------------------------------------------------------------------------

/// One car's input, the fields the controller reads [M actor+0x240 steer / +0x250 throttle /
/// +0x254 brake / +0x259 handbrake].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarInput {
    pub throttle: f32,
    pub brake: f32,
    /// −1..1, the pad's X [M +0x240]
    pub steer: f32,
    pub handbrake: bool,
}

impl Default for CarInput {
    fn default() -> Self { CarInput { throttle: 0.0, brake: 0.0, steer: 0.0, handbrake: false } }
}

/// The plane a wheel query returns: a point on the triangle, its unit normal and the surface
/// byte [M FUN_001f6ae0: `n = norm((v1 − v0) × (v0 − v2))`, flipped to `n.y ≥ 0`, `n.y` clamped
/// to 0.9999; the wheel's `w = (v0 − p)·n` is recomputed from the cached triangle each tick].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelPlane {
    pub v0: [f32; 3],
    pub normal: [f32; 3],
    /// The triangle's surface byte (+0x2c) — the Friction table row [M FUN_0018cef8].
    pub surface: u8,
}

/// A hull point's swept segment hitting a wall: the hit point and the wall normal facing the
/// segment's start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallHit {
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

/// What the land car asks of the world: the per-wheel ground plane (the track query
/// FUN_002322c8 behind FUN_001f6ae0) and the hull sweep (FUN_001fc2f0's segment tests).
pub trait Ground {
    /// The ground triangle under the world point `p`, as a plane; `None` = no triangle (the
    /// car keeps the wheel's last plane, as the game's per-wheel cache record does).
    fn wheel_plane(&self, p: [f32; 3]) -> Option<WheelPlane>;
    /// The first wall the segment `a → b` crosses.
    fn wall_hit(&self, _a: [f32; 3], _b: [f32; 3]) -> Option<WallHit> { None }
    /// True for the legacy no-ground stand-in (`FlatWorld`): `Car::step` then runs the flat
    /// `step_rail` model instead of the wheels.
    fn legacy_flat(&self) -> Option<&FlatWorld> { None }
}

/// The legacy no-ground stand-in: a caller without the track's collision (the AI rail driver,
/// a view that sets the height itself) passes this, and `Car::step` runs the flat `step_rail`
/// model — no wheels, no gravity, no height floor. A caller with the track passes a
/// `CollisionGround`; the flat test plane is `FlatGround`.
#[derive(Debug, Clone, Copy)]
pub struct FlatWorld {
    /// The `SURFACES` index the car drives on (2 = GRAVEL, 3 = GRASS, 9 = SNOW…).
    pub surface: usize,
}

impl FlatWorld {
    pub fn paved() -> FlatWorld { FlatWorld { surface: 1 } }
}

impl Ground for FlatWorld {
    fn wheel_plane(&self, _p: [f32; 3]) -> Option<WheelPlane> { None }
    fn legacy_flat(&self) -> Option<&FlatWorld> { Some(self) }
}

/// An endless flat plane at `height` of one surface kind — the wheels' test ground.
#[derive(Debug, Clone, Copy)]
pub struct FlatGround {
    pub height: f32,
    /// The surface byte (1 = PAVED).
    pub surface: u8,
}

impl FlatGround {
    pub fn paved(height: f32) -> FlatGround { FlatGround { height, surface: 1 } }
}

impl Ground for FlatGround {
    fn wheel_plane(&self, p: [f32; 3]) -> Option<WheelPlane> {
        // An up-facing triangle: n.y clamps to 0.9999 [M FUN_001f6ae0 `0.9999 <= n.y`].
        Some(WheelPlane { v0: [p[0], self.height, p[2]], normal: [0.0, 0.9999, 0.0], surface: self.surface })
    }
}

/// The track's collision triangles (`.crp` CDat, `Crp::extract_collision_tris`) on a 16 m XZ
/// grid. The wheel query skips class-bit-0x04 triangles [M 0x2138b0]; the wall sweep tests the
/// steep triangles of every class [S: which classes FUN_002127f0/FUN_002187f0 test is not traced].
pub struct CollisionGround {
    tris: Vec<nf_formats::crp_mesh::CollisionTri>,
    /// per-cell (ground tris, wall tris)
    cells: std::collections::HashMap<(i32, i32), (Vec<u32>, Vec<u32>)>,
}

impl CollisionGround {
    const CELL: f32 = 16.0;
    /// How far above the wheel point a triangle still counts as the ground under it [S: the
    /// track query's own window is not traced; one metre covers the 0.75 m compression limit].
    pub const UP_REACH: f32 = 1.0;
    /// |n̂.y| below this = a wall for the hull sweep [S].
    pub const WALL_NY: f32 = 0.7;

    pub fn new(tris: Vec<nf_formats::crp_mesh::CollisionTri>) -> CollisionGround {
        let mut cells: std::collections::HashMap<(i32, i32), (Vec<u32>, Vec<u32>)> = std::collections::HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            let n = tri_normal(&t.v);
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if !(len > 1e-8) { continue; }
            let ny = (n[1] / len).abs();
            let ground = ny > 0.05 && !t.ground_query_skips();
            let wall = ny < Self::WALL_NY;
            if !ground && !wall { continue; }
            let (x0, x1) = (t.v[0][0].min(t.v[1][0]).min(t.v[2][0]), t.v[0][0].max(t.v[1][0]).max(t.v[2][0]));
            let (z0, z1) = (t.v[0][2].min(t.v[1][2]).min(t.v[2][2]), t.v[0][2].max(t.v[1][2]).max(t.v[2][2]));
            for cx in (x0 / Self::CELL).floor() as i32..=(x1 / Self::CELL).floor() as i32 {
                for cz in (z0 / Self::CELL).floor() as i32..=(z1 / Self::CELL).floor() as i32 {
                    let e = cells.entry((cx, cz)).or_default();
                    if ground { e.0.push(i as u32); }
                    if wall { e.1.push(i as u32); }
                }
            }
        }
        CollisionGround { tris, cells }
    }

    pub fn tris(&self) -> &[nf_formats::crp_mesh::CollisionTri] { &self.tris }
}

fn tri_normal(v: &[[f32; 3]; 3]) -> [f32; 3] {
    // (v1 − v0) × (v0 − v2) [M FUN_001f6ae0: fStack_c0 = v1 − v0, fStack_d0 = v0 − v2]
    cross(sub(v[1], v[0]), sub(v[0], v[2]))
}

impl Ground for CollisionGround {
    fn wheel_plane(&self, p: [f32; 3]) -> Option<WheelPlane> {
        let key = ((p[0] / Self::CELL).floor() as i32, (p[2] / Self::CELL).floor() as i32);
        let (list, _) = self.cells.get(&key)?;
        let mut best: Option<(f32, usize)> = None;
        for &ti in list {
            let [a, b, c] = self.tris[ti as usize].v;
            let den = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if den.abs() < 1e-9 { continue; }
            let u = ((b[2] - c[2]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[2] - c[2])) / den;
            let v = ((c[2] - a[2]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[2] - c[2])) / den;
            let w = 1.0 - u - v;
            if u < -1e-5 || v < -1e-5 || w < -1e-5 { continue; }
            let y = u * a[1] + v * b[1] + w * c[1];
            if y > p[1] + Self::UP_REACH { continue; }
            if best.is_none_or(|(by, _)| y > by) { best = Some((y, ti as usize)); }
        }
        let (_, ti) = best?;
        let t = &self.tris[ti];
        let mut n = norm(tri_normal(&t.v));
        if n[1] < 0.0 { n = scale(n, -1.0); }
        if n[1] >= 0.9999 { n[1] = 0.9999; }
        Some(WheelPlane { v0: t.v[0], normal: n, surface: t.surface() })
    }

    fn wall_hit(&self, a: [f32; 3], b: [f32; 3]) -> Option<WallHit> {
        let d = sub(b, a);
        let (cx0, cx1) = ((a[0].min(b[0]) / Self::CELL).floor() as i32, (a[0].max(b[0]) / Self::CELL).floor() as i32);
        let (cz0, cz1) = ((a[2].min(b[2]) / Self::CELL).floor() as i32, (a[2].max(b[2]) / Self::CELL).floor() as i32);
        let mut best: Option<(f32, usize)> = None;
        for cx in cx0..=cx1 {
            for cz in cz0..=cz1 {
                let Some((_, list)) = self.cells.get(&(cx, cz)) else { continue };
                for &ti in list {
                    let [p0, p1, p2] = self.tris[ti as usize].v;
                    let e1 = sub(p1, p0);
                    let e2 = sub(p2, p0);
                    let h = cross(d, e2);
                    let det = dot(e1, h);
                    if det.abs() < 1e-9 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(a, p0);
                    let u = inv * dot(s, h);
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = inv * dot(d, q);
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let t = inv * dot(e2, q);
                    if (0.0..=1.0).contains(&t) && best.is_none_or(|(bt, _)| t < bt) { best = Some((t, ti as usize)); }
                }
            }
        }
        let (t, ti) = best?;
        let mut n = norm(tri_normal(&self.tris[ti as usize].v));
        if dot(n, d) > 0.0 { n = scale(n, -1.0); }
        Some(WallHit { point: add(a, scale(d, t)), normal: n })
    }
}

// ---------------------------------------------------------------------------
// Small vector helpers
// ---------------------------------------------------------------------------

#[inline] fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
#[inline] fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
#[inline] fn scale(a: [f32; 3], s: f32) -> [f32; 3] { [a[0] * s, a[1] * s, a[2] * s] }
#[inline] fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
#[inline] fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
#[inline] fn len(a: [f32; 3]) -> f32 { dot(a, a).sqrt() }
/// The VU0 rsqrt normalize: a zero vector stays zero (the rsqrt of 0 saturates and the multiply
/// by 0 gives 0 on the VU) [S: the PS2 saturation, not traced on this build].
#[inline] fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 1e-12 { scale(a, 1.0 / l) } else { [0.0; 3] }
}

// ---------------------------------------------------------------------------
// The car state and the per-tick step
// ---------------------------------------------------------------------------

/// The player car's state — the fields the port's step integrates, named for the game's
/// offsets (actor+0x… / instance+0x… per docs/systems/drive-physics.md).
#[derive(Debug, Clone)]
pub struct Car {
    pub params: CarParams,
    /// instance+0x10 (the sim position, the body centre; y = height)
    pub pos: [f32; 3],
    /// the heading, `atan2(fwd.x, fwd.z)` of the orientation's forward row. Writing it from
    /// outside (a rail snap, a respawn) re-levels the body at that heading on the next step.
    pub yaw: f32,
    /// instance+0x20 (the linear velocity)
    pub vel: [f32; 3],
    /// ω·y (instance+0x34, the world yaw rate)
    pub yaw_rate: f32,
    /// actor+0x2d8 (the auto gearbox's gear, 0 = reverse, 1..5 forward)
    pub gear: u8,
    /// actor+0x2d9 (last tick's gear — the anti-hunt hold [M 0x18aa88:53-56])
    pub last_gear: u8,
    /// actor+0x2d7 — the shift inhibit: 0xf on every gear change [M 0x18cef8:410-413],
    /// decremented once per tick [M 0x18cef8 tail]
    pub shift_inhibit: u8,
    /// actor+0x28c (the steer angle)
    pub steer_angle: f32,
    /// actor+0x288 (|velocity|)
    pub speed: f32,
    /// the engine rpm FUN_0018aa88 returns
    pub rpm: f32,
    /// actor+0x29c — the reverse state, re-derived every tick [M 0x18cef8: cleared, then set
    /// when the brake has been held > 29 ticks at a longitudinal speed ≤ 0]
    pub reversing: bool,
    /// any wheel in contact (actor+0x299 > 0)
    pub grounded: bool,
    /// the working gravity this car integrates under (−13 bodge / −34 named) [M 0x1f7c60:136-144]
    pub gravity: f32,
    /// the orientation rows: right (body +x), up (+y), forward (+z), world space (the class
    /// matrix rows, class+0x00..0x2f)
    pub rot: [[f32; 3]; 3],
    /// instance+0x30, the world angular velocity
    pub ang_vel: [f32; 3],
    /// instance+0x60 / +0x70: the accumulated force and torque, per unit mass (the force API
    /// multiplies by the mass and the integration divides it back [M FUN_001f87d8, S VU0 side])
    pub acc_accum: [f32; 3],
    pub torque_accum: [f32; 3],
    /// actor+0x210.. — each wheel's last compression [M FUN_0018c1b0 `prev[i] = x`]
    pub spring_prev: [f32; 4],
    /// actor+0x299 — the contacting wheel count FUN_0018c1b0 returns
    pub contacts: u8,
    /// instance+0x6c — the hull points below their ground plane this tick (pass C,
    /// `FUN_001f9500` 0x1f9804 `sb s6, 0x6c(s0)`); the righting reset counts on it
    /// (`driving::reset`)
    pub hull_contacts: u8,
    /// actor+0x29a — the brake-hold counter (30 when the brake is up)
    pub brake_ticks: u8,
    /// actor+0x274 — the signed wheel-spin rate
    pub wheel_spin: f32,
    /// actor+0x2da — counter-steering this tick
    pub counter_steering: bool,
    /// actor+0x1b0.. — the per-wheel slip ratio |f_lat|/limit
    pub slip: [f32; 4],
    /// actor+0x90 + 0x40·i — the per-wheel cached ground plane
    pub wheel_cache: [Option<WheelPlane>; 4],
    /// the yaw the last step left (to spot an outside write to `yaw`)
    last_yaw: f32,
}

/// The body rows for an upright car at `yaw`: right = (cos, 0, −sin), up = y, forward =
/// (sin, 0, cos) — the port's yaw convention (forward = (sin yaw, 0, cos yaw)).
pub fn rot_from_yaw(yaw: f32) -> [[f32; 3]; 3] {
    let (s, c) = yaw.sin_cos();
    [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]]
}

impl Car {
    pub fn new(params: CarParams, pos: [f32; 3], yaw: f32) -> Car {
        let gravity = Rigid::game_defaults().bodge_player_gravity;
        Car {
            params, pos, yaw, vel: [0.0; 3], yaw_rate: 0.0, gear: 1, last_gear: 1, shift_inhibit: 0,
            steer_angle: 0.0, speed: 0.0, rpm: 0.0, reversing: false, grounded: false, gravity,
            rot: rot_from_yaw(yaw), ang_vel: [0.0; 3], acc_accum: [0.0; 3], torque_accum: [0.0; 3],
            spring_prev: [0.0; 4], contacts: 0, hull_contacts: 0, brake_ticks: 0x1e, wheel_spin: 0.0,
            counter_steering: false, slip: [0.0; 4], wheel_cache: [None; 4], last_yaw: yaw,
        }
    }

    /// The body height above a flat ground at which the wheels just touch at rest length:
    /// wheel point y = −hy, compression = d + REST = 0 ⇒ ground at −hy − REST (n.y ≈ 1).
    pub fn rest_height(&self) -> f32 { self.params.half_extents[1] + self.params.spring_rest }

    /// A wheel's world position (class+0x180 + inst+0x10 [M FUN_0018cef8's wheel loop]).
    pub fn wheel_world(&self, i: usize) -> [f32; 3] {
        let l = self.params.wheel_points()[i];
        add(self.pos, self.body_to_world(l))
    }

    fn body_to_world(&self, l: [f32; 3]) -> [f32; 3] {
        add(add(scale(self.rot[0], l[0]), scale(self.rot[1], l[1])), scale(self.rot[2], l[2]))
    }

    fn world_to_body(&self, w: [f32; 3]) -> [f32; 3] {
        [dot(self.rot[0], w), dot(self.rot[1], w), dot(self.rot[2], w)]
    }

    /// `FUN_0018aa88` [M]: |velocity|, then the RPM/gearbox. `driving` = the game's +0x299 byte
    /// (the contact count; 0 → the neutral rev `throttle·7000`).
    pub fn update_rpm_gear(&mut self, throttle: f32, driving: bool) {
        let speed = len(self.vel);
        self.speed = speed;
        if !driving {
            self.rpm = throttle * 7000.0; // [M 0x18aa88:32]
            return;
        }
        // The stateless auto walk [M 0x18aa88:36-48].
        let mut gear: u8 = 1;
        let mut limit = self.params.gear_limit[1];
        if limit <= speed {
            loop {
                gear += 1;
                if gear > 4 { break; }
                limit = self.params.gear_limit[gear as usize];
                if speed < limit { break; }
            }
        }
        // The anti-hunt hold [M 0x18aa88:53-56].
        if gear < self.last_gear && self.shift_inhibit != 0 {
            gear = self.last_gear;
        }
        self.gear = gear;
        // The change detection and the inhibit reload [M 0x18cef8:410-413]; the per-tick
        // decrement [M 0x18cef8 tail].
        if gear != self.last_gear {
            self.last_gear = gear;
            self.shift_inhibit = 0xf;
        } else if self.shift_inhibit != 0 {
            self.shift_inhibit -= 1;
        }
        // rpm = |v|·6000/LIMIT[gear] [M 0x18aa88:62] (gear 5 uses LIMIT4 per the loop's break).
        let gl = if gear >= 5 { self.params.gear_limit[4] } else { self.params.gear_limit[gear as usize] };
        self.rpm = if gl > 0.0 { speed * 6000.0 / gl } else { 0.0 };
    }

    /// `FUN_0018cef8`'s steering block [M]: the speed falloff between MODSTEERSPEED and +25,
    /// floored at MINSTEER, scaled by MAXSTEERING.
    pub fn update_steering(&mut self, steer_input: f32) {
        let p = &self.params;
        let scale = if self.speed <= p.modsteer_speed {
            1.0
        } else {
            let t = (self.speed - p.modsteer_speed) / 25.0;
            (1.0 - t.min(1.0)).max(p.min_steer)
        };
        self.steer_angle = steer_input * p.maxsteering * scale;
    }

    /// The engine's drive acceleration actor+0x1dc [M FUN_0018cef8 :414-431]:
    /// `boost + throttle·MAXACC·RATIO[gear] − brake·MAXBRAKE`; in the reverse state the brake term
    /// uses the decayed reverse accel `max(0, MAXREVACC − max(0, |v| − SPEED_CUTOFF_REVERSE)·
    /// SPEED_CUTOFF_RATE_REVERSE)` instead of MAXBRAKE [M :401-407, :425-429]. The boost
    /// (+0x428, the pickup countdown at actor+0x294) is not ported: 0 here.
    pub fn drive_accel(&mut self, input: &CarInput) -> f32 {
        let p = &self.params;
        let engine = p.maxacc * p.gear_ratio[self.gear as usize];
        if self.reversing {
            let cut = self.speed - p.speed_cutoff_reverse;
            let rev = if cut > 0.0 { (p.maxrevacc - cut * p.speed_cutoff_rate_reverse).max(0.0) } else { p.maxrevacc };
            return input.throttle * engine - input.brake * rev;
        }
        input.throttle * engine - input.brake * p.maxbrake
    }

    /// The per-wheel friction-modifier actor+0x2e8 [M FUN_0018cef8 tail before FUN_0018c1b0]:
    /// 1 up to FRIC_MOD_SPEED, then `1 − min(speed − SPEED, RANGE)/RANGE`, floored at FRIC_MOD_MIN.
    pub fn fric_mod(&self, speed: f32) -> f32 {
        let p = &self.params;
        if p.fric_mod_speed < speed {
            let t = (speed - p.fric_mod_speed).min(p.fric_mod_range);
            (1.0 - t / p.fric_mod_range).max(p.fric_mod_min)
        } else {
            1.0
        }
    }

    /// The flat-road rail step for the AI driver (ai.rs), which snaps the car back onto the
    /// spline every tick, and for any caller passing `FlatWorld`: the longitudinal drive only,
    /// no wheels, no gravity, the height left as it is (no floor). ⚠ A stand-in [S]: the game's
    /// AI cars run `FUN_0018ecb8` (actor+0x2c8 != 0 [M FUN_00188200 0x188bf8]), not ported.
    pub fn step_rail(&mut self, input: &CarInput, world: &FlatWorld, dt: f32) {
        let driving = self.speed > 0.5 || input.throttle > 0.0 || self.reversing;
        self.update_rpm_gear(input.throttle, driving);
        self.update_steering(input.steer);
        let accel = self.drive_accel(input);
        let (sin, cos) = self.yaw.sin_cos();
        let boost = (1.0 - (self.speed / 15.0).min(1.0)) * 2.5 + 1.0;
        let surface_grip = Friction::game_defaults().friction[world.surface.min(12)];
        let lat_limit = 12.5 * boost * surface_grip;
        let wheelbase = self.params.wheel_zr_offset - self.params.wheel_zf_offset;
        let mut yaw_rate = self.speed * self.steer_angle.tan() / wheelbase;
        let lat = (yaw_rate * self.speed).abs();
        if lat > lat_limit && lat > 1e-6 { yaw_rate *= lat_limit / lat; }
        self.yaw_rate = yaw_rate;
        self.vel[0] += sin * accel * dt;
        self.vel[2] += cos * accel * dt;
        let speed = (self.vel[0] * self.vel[0] + self.vel[2] * self.vel[2]).sqrt();
        if speed > 1e-4 {
            let roll = (ROLLING_RESISTANCE * dt / speed).min(1.0);
            self.vel[0] -= self.vel[0] * roll;
            self.vel[2] -= self.vel[2] * roll;
        }
        self.vel[1] = 0.0;
        self.pos[0] += self.vel[0] * dt;
        self.pos[2] += self.vel[2] * dt;
        self.yaw += self.yaw_rate * dt;
        self.last_yaw = self.yaw;
        self.rot = rot_from_yaw(self.yaw);
        self.grounded = true;
    }

    /// One physics tick of the land car, in the game's order [M FUN_001ff3d8]:
    /// B — the hull sweep against the walls (FUN_001fc2f0); D — the integration of last tick's
    /// accumulated force/torque (FUN_001f89e0, VU0 microcode — semi-implicit Euler here [S]);
    /// E — zero the accumulators, add gravity (FUN_001f7c60); then the controller
    /// (FUN_00188200 → FUN_0018cef8 → FUN_0018c1b0) accumulates the wheel forces for the NEXT
    /// tick, and FUN_0018aa88 updates the gearbox.
    pub fn step<G: Ground + ?Sized>(&mut self, input: &CarInput, world: &G, rigid: &Rigid, dt: f32) {
        if let Some(flat) = world.legacy_flat() {
            self.step_rail(input, flat, dt);
            return;
        }
        if self.yaw != self.last_yaw {
            // An outside write to the heading: re-level the body at it.
            self.rot = rot_from_yaw(self.yaw);
            self.ang_vel = [0.0; 3];
        }
        self.walls(world, dt);
        self.hull_ground(world);
        self.integrate(rigid, dt);
        // E: the accumulators zero, the working gravity lands (0, g, 0) [M 0x1f7c60:55-60, :168].
        self.acc_accum = [0.0, self.gravity, 0.0];
        self.torque_accum = [0.0; 3];
        self.controller(input, world);
        self.update_rpm_gear(input.throttle, self.contacts > 0);
        self.yaw = self.rot[2][0].atan2(self.rot[2][2]);
        self.last_yaw = self.yaw;
        self.yaw_rate = self.ang_vel[1];
        self.grounded = self.contacts > 0;
    }

    /// Pass B [S form]: each of the 8 hull corners (the model-less branch of FUN_001f72d8 puts
    /// the box corners at class+0x80..0xfc [M]; which branch the vanquish takes is not traced)
    /// sweeps `p → p + vel·dt` [M FUN_001fc2f0:251-255]; on a wall hit the response pushes along
    /// the horizontal wall normal (the land car's response vector has y zeroed [M :316-325]) and
    /// removes the inward velocity. The response magnitude (FUN_001fb318) is not ported.
    fn walls<G: Ground + ?Sized>(&mut self, world: &G, dt: f32) {
        let [hx, hy, hz] = self.params.half_extents;
        for k in 0..8 {
            let l = [if k & 1 == 0 { -hx } else { hx }, if k & 4 == 0 { -hy } else { hy }, if k & 2 == 0 { hz } else { -hz }];
            // The port sweeps from the body's centre column out to the moved corner, so a corner
            // already through a wall still finds it [S].
            let a = add(self.pos, self.body_to_world([0.0, l[1], 0.0]));
            let b = add(add(self.pos, self.body_to_world(l)), scale(self.vel, dt));
            let Some(hit) = world.wall_hit(a, b) else { continue };
            let n = norm([hit.normal[0], 0.0, hit.normal[2]]);
            let depth = dot(sub(hit.point, b), n);
            if depth > 0.0 {
                self.pos = add(self.pos, scale(n, depth));
            }
            let vn = dot(self.vel, n);
            if vn < 0.0 {
                self.vel = sub(self.vel, scale(n, vn));
            }
        }
    }

    /// The hull points pass C tests, in the body frame [S: the game's points come from the car
    /// model's hull list (`FUN_001f72d8`'s hull branch, points 4.. after the four wheel points),
    /// which the port does not read]. Tilted (`up.y <= 0.8`) the game tests points 4..count;
    /// upright only points 8..11 [M 0x1f9670-0x1f96a8]. The port's stand-ins: tilted = the 8
    /// box corners; upright = the 4 bottom corners raised to where a wheel's spring reaches its
    /// compression limit (a bump stop: they meet the ground only once the springs are spent).
    pub fn hull_points(&self) -> Vec<[f32; 3]> {
        let [hx, hy, hz] = self.params.half_extents;
        let mut out = Vec::with_capacity(8);
        if self.rot[1][1] > Self::HULL_UPRIGHT_Y {
            let y = -hy + (self.params.spring_limit - self.params.spring_rest);
            for (sx, sz) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
                out.push([sx * hx, y, sz * hz]);
            }
        } else {
            for k in 0..8 {
                out.push([if k & 1 == 0 { -hx } else { hx }, if k & 4 == 0 { -hy } else { hy }, if k & 2 == 0 { hz } else { -hz }]);
            }
        }
        out
    }

    /// `FUN_001f9500`'s split: up.y above this tests only the upright points [M 0x1f9670: `0.8 < up.y`].
    pub const HULL_UPRIGHT_Y: f32 = 0.8;

    /// Pass C — the hull against the ground (`FUN_001f9500`, run between the wall sweep and the
    /// integration [M FUN_001ff3d8: the 0x1fc2f0 loop, then the 0x1f9500 loop, then 0x1f8ff0]).
    /// Each hull point queries the ground plane under it (`FUN_001f6ae0`); a point whose depth
    /// `w = (v0 − p)·n` is > 0 counts [M 0x1f9770 `c.olt.s 0, w`]; the count lands in +0x6c
    /// [M 0x1f9804]; the body moves out along the deepest point's plane normal by that depth
    /// [M 0x1f983c-0x1f9894: `pos += plane[fp].n · max w`]. The game's contact response
    /// (`FUN_001f91e8`, VU0 microcode 0x1a0) is not lifted: the port removes the velocity into
    /// that plane instead [S].
    fn hull_ground<G: Ground + ?Sized>(&mut self, world: &G) {
        let mut count = 0u8;
        let mut deepest: Option<(f32, [f32; 3], [f32; 3])> = None;
        for l in self.hull_points() {
            let r = self.body_to_world(l);
            let p = add(self.pos, r);
            let Some(pl) = world.wheel_plane(p) else { continue };
            let w = dot(sub(pl.v0, p), pl.normal);
            if w > 0.0 {
                count += 1;
                if deepest.is_none_or(|(d, _, _)| w > d) { deepest = Some((w, pl.normal, r)); }
            }
        }
        self.hull_contacts = count;
        if let Some((w, n, r)) = deepest {
            self.pos = add(self.pos, scale(n, w));
            // [S] stand-in for FUN_001f91e8: an inelastic impulse along n at that point, so the
            // point stops moving into the ground (linear and angular parts, I⁻¹ per mass).
            let vp = add(self.vel, cross(self.ang_vel, r));
            let vn = dot(vp, n);
            if vn < 0.0 {
                let ii = self.params.inv_inertia_per_mass();
                let k_world = |t: [f32; 3]| {
                    let b = self.world_to_body(t);
                    self.body_to_world([b[0] * ii[0], b[1] * ii[1], b[2] * ii[2]])
                };
                let krn = k_world(cross(r, n));
                let j = -vn / (1.0 + dot(n, cross(krn, r)));
                self.vel = add(self.vel, scale(n, j));
                self.ang_vel = add(self.ang_vel, scale(krn, j));
            }
        }
    }

    /// Put the body at `pos` with the orientation rows `rot`, at rest: `FUN_001f7880` zeroes the
    /// velocity (+0x20), the angular velocity (+0x30), the momenta (+0x40/+0x50) and the hull
    /// contact byte (+0x6c), and sets the position and matrix [M 0x1f7880].
    pub fn place(&mut self, pos: [f32; 3], rot: [[f32; 3]; 3]) {
        self.pos = pos;
        self.rot = rot;
        self.vel = [0.0; 3];
        self.ang_vel = [0.0; 3];
        self.acc_accum = [0.0; 3];
        self.torque_accum = [0.0; 3];
        self.hull_contacts = 0;
        self.speed = 0.0;
        self.yaw = rot[2][0].atan2(rot[2][2]);
        self.last_yaw = self.yaw;
        self.yaw_rate = 0.0;
    }

    /// Pass D [M setup FUN_001f89e0, S the VU0 body]: vel += acc·dt (|v| ≤ M_LIMIT), pos += vel·dt;
    /// ω += I⁻¹·τ·dt in the body frame (|ω| ≤ AM_LIMIT); the rows turn by ω and re-orthonormalise.
    /// No angular damping for cars: FUN_001f8088's ×NATURAL_ANGULAR_DAMPING is gated on
    /// inst+0x6d > 3, and cars are kinds 1-3 [M FUN_001f8088, FUN_001f6e98].
    fn integrate(&mut self, rigid: &Rigid, dt: f32) {
        self.vel = add(self.vel, scale(self.acc_accum, dt));
        let v = len(self.vel);
        if v > rigid.m_limit { self.vel = scale(self.vel, rigid.m_limit / v); }
        self.pos = add(self.pos, scale(self.vel, dt));
        let tb = self.world_to_body(self.torque_accum);
        let ii = self.params.inv_inertia_per_mass();
        let ab = [tb[0] * ii[0], tb[1] * ii[1], tb[2] * ii[2]];
        self.ang_vel = add(self.ang_vel, scale(self.body_to_world(ab), dt));
        let w = len(self.ang_vel);
        if w > rigid.am_limit { self.ang_vel = scale(self.ang_vel, rigid.am_limit / w); }
        let mut r = self.rot;
        for row in r.iter_mut() {
            *row = add(*row, scale(cross(self.ang_vel, *row), dt));
        }
        let f = norm(r[2]);
        let right = norm(cross(r[1], f));
        let up = cross(f, right);
        self.rot = [right, up, f];
    }

    /// The land controller: FUN_0018cef8 (pedal states, grip shares, drive, steering, per-wheel
    /// ground query) and FUN_0018c1b0 (the per-wheel spring, tyre and drive forces).
    fn controller<G: Ground + ?Sized>(&mut self, input: &CarInput, world: &G) {
        let p = self.params.clone();
        let speed = len(self.vel); // actor+0x288 [M 0x18cef8 top]
        self.speed = speed;
        let fwd = self.rot[2];
        let up = self.rot[1];
        let right = self.rot[0];
        let long_v = dot(self.vel, fwd); // 0x1cc
        let roll_vec = scale(self.vel, ROLLING_RESISTANCE); // 0x1d0 → 0x160 (param_7)
        let throttle = input.throttle;
        let mut brake = input.brake;

        // The brake / reverse states [M 0x18cef8 0x18d0e0..0x18d18c].
        let mut braking_fwd = false; // 0x298
        self.reversing = false; // 0x29c cleared each tick
        let mut front = 0.5f32; // 0x1fc
        let mut state: u8; // 0x2b1
        if brake == 0.0 {
            self.brake_ticks = 0x1e;
        } else if long_v >= 2.5 {
            self.brake_ticks = 0;
            braking_fwd = true;
        } else if self.brake_ticks > 0x1d {
            if long_v <= 0.0 {
                front = p.reverse_frw;
                self.reversing = true;
            }
        } else {
            self.brake_ticks += 1;
            if long_v < 0.5 { brake = 0.0; }
        }
        if self.reversing {
            state = 3;
        } else if brake > MIN_BUTTON_VALUE {
            front = p.brake_frw;
            state = 2;
        } else if throttle > MIN_BUTTON_VALUE {
            front = p.accel_frw;
            state = 1;
        } else {
            front = p.coast_frw;
            state = 0;
        }
        let mut rear = 1.0 - front; // 0x20c
        let mut hb_drag = 0.0f32; // 0x1ec
        let mut park = false; // 0x440
        if input.handbrake {
            if speed < 1.0 {
                if up[1] <= 0.707 {
                    hb_drag = speed + speed + 1.0;
                } else {
                    // inst+0x40 := (0, +0x44, 0): the momentum mirror keeps y only [M]; the
                    // port zeroes the horizontal velocity [S: how +0x40 feeds the VU0 step].
                    self.vel[0] = 0.0;
                    self.vel[2] = 0.0;
                }
            } else {
                hb_drag = p.handbrake_force;
                park = true;
            }
            // 0x29f (the reverse-handbrake byte) has no traced writer: the 0x29f == 0 arm [S].
            state = 4;
            let f = p.handbrake_frw;
            let r = (1.0 - f) * p.handbrake_rw_fric_scale;
            front = f * p.total_handbrake_friction_scale;
            rear = r * p.total_handbrake_friction_scale;
            if speed < 5.0 {
                front = 0.5;
                rear = 0.5;
            }
        }
        if state == 2 {
            front *= p.braking_friction;
            rear *= p.braking_friction;
        }
        // The wheel-spin flag 0x445: +0x29b && !handbrake (0x29b has no traced writer: false
        // [S]), or the accel state while the wheels spin backwards with throttle > 0.75.
        let mut spin = false;
        if state == 1 && self.wheel_spin < 0.0 {
            let k = (p.braking_friction + p.braking_friction).min(1.0);
            if p.is_rally { front *= k; }
            rear *= k;
            if throttle > 0.75 { spin = true; }
        }

        // Counter-steer 0x43c [M 0x18cef8 0x18d4xx].
        let mut cs = 1.0f32;
        self.counter_steering = false;
        if p.counter_steer_scale != 0.0 {
            let wy = self.ang_vel[1];
            let steer = input.steer;
            if (wy > 0.0 && steer < 0.0) || (wy < 0.0 && steer > 0.0) {
                self.counter_steering = true;
                let mut c = wy.abs() * p.counter_steer_scale * steer.abs();
                if state == 1 { c *= 1.2; }
                cs = c.min(p.counter_steer_limit).max(1.0);
            }
        }

        // The wheel roll per tick 0x424 [M]: |v_xz|/(WHEEL_RADIUS·6.32·fps), ≤ 1; 0 below 0.01
        // with the handbrake on.
        let mut roll = (self.vel[0] * self.vel[0] + self.vel[2] * self.vel[2]).sqrt() / (p.wheel_radius * 6.32 * FPS);
        if roll < 0.01 && input.handbrake { roll = 0.0; }
        let roll = roll.min(1.0);

        let drive = self.drive_accel(&CarInput { brake, ..*input });

        // The wheel spin 0x274 [M]: decays in the air (×0.95, ×0.25 braking) or tipped
        // (up.y ≤ 0.1); otherwise min(roll, MAX_WHEEL_SPIN_RATE), signed by the longitudinal speed.
        if self.contacts == 0 || up[1] <= 0.1 {
            self.wheel_spin *= if braking_fwd { 0.25 } else { 0.95 };
        } else {
            let s = roll.min(MAX_WHEEL_SPIN_RATE);
            self.wheel_spin = if long_v < 0.0 { -s } else { s };
        }

        self.update_steering(input.steer);
        let (sa, ca) = self.steer_angle.sin_cos();
        let front_dir = add(scale(right, sa), scale(fwd, ca)); // 0x1f0 (FUN_002cc590 + 0x2cbb88)

        // Per wheel: the ground query and the four tyre terms 0x170.. [M the 0x18db.. loop].
        let friction = Friction::game_defaults();
        let fric_mod = self.fric_mod(speed);
        let prev_contacts = self.contacts;
        let mut count = 0u8;
        let fl = p.friction_limit_front;
        let rl = p.friction_limit_rear * cs;
        for i in 0..4 {
            let wp = self.wheel_world(i);
            if let Some(pl) = world.wheel_plane(wp) { self.wheel_cache[i] = Some(pl); }
            let front_axle = i < 2;
            let mut grip = if front_axle { front * p.tyre_grip_factor } else { rear * p.tyre_grip_factor * cs };
            let mut wdrive = if front_axle {
                if drive >= 0.0 { if p.is_rally { drive } else { 0.0 } } else { drive }
            } else if spin {
                drive * p.ws_accel
            } else {
                drive.max(0.0)
            };
            wdrive *= grip;
            let mut limit = if front_axle { fl } else { rl };
            let mut hb = hb_drag;
            let surf = self.wheel_cache[i].map_or(1, |c| c.surface) as usize;
            // friction[surface] [M `DAT_003319d8 + byte·4`]; the table has 13 rows [S past it].
            let fk = friction.friction.get(surf).copied().unwrap_or(1.0);
            limit *= fk;
            wdrive *= fk;

            // FUN_0018c1b0 for this wheel.
            let Some(pl) = self.wheel_cache[i] else {
                self.spring_prev[i] = 0.0;
                self.slip[i] = 0.0;
                continue;
            };
            let n = pl.normal;
            let d = dot(sub(pl.v0, wp), n); // param_5.w [M FUN_001f6ae0]
            let x = (d + p.spring_rest).min(p.spring_limit).max(0.0);
            let prev = self.spring_prev[i];
            let mut process = x > 0.0 || prev > 0.0;
            if process {
                count += 1;
            } else {
                // 0x2b2 has no traced writer: the 0x2b2 == 0 arm [S].
                process = prev_contacts > 0;
            }
            let upright = UPRIGHT_GATE < up[1] * n[1];
            if process && upright {
                let f_spring = if x > 0.0 {
                    let (k, c) = if front_axle { (p.spring_stiffness[0], p.spring_damping[0]) } else { (p.spring_stiffness[1], p.spring_damping[1]) };
                    x * k + (x - prev) * c
                } else {
                    0.0
                };
                let axle = if front_axle { front_dir } else { fwd };
                let lat0 = norm(cross(axle, n));
                let longd = norm(cross(n, lat0)); // 0x2a0
                let mut lat = cross(n, longd);
                lat[1] = 0.0;
                let mut lat = norm(lat); // 0x260
                let mut r = sub(wp, self.pos); // 0x250
                let vpt = add(cross(self.ang_vel, r), self.vel); // 0x290
                if hb != 0.0 && !front_axle {
                    lat = scale(norm(vpt), -1.0);
                }
                if p.is_rally {
                    let m = (5.0 - speed).max(1.0);
                    limit *= m;
                    grip *= m;
                }
                let mut f_lat = dot(vpt, lat) * grip * fric_mod; // 0x320
                if pl.surface == 8 {
                    // kICE [M]: the drive/handbrake halve when braking or handbraking; limit ×0.6
                    if wdrive < 0.0 || hb != 0.0 {
                        wdrive *= 0.5;
                        hb *= 0.5;
                    }
                    limit *= 0.6;
                }
                let mut slipping = false;
                let mut excess = 0.0;
                if limit < f_lat.abs() {
                    excess = f_lat.abs() - limit;
                    f_lat = if f_lat < 0.0 { -limit } else { limit };
                    slipping = true;
                }
                let mut longd = longd;
                if spin && !front_axle {
                    f_lat *= p.wheelspin_scale;
                    longd = scale(longd, p.wheelspin_scale);
                }
                // The drive along the wheel's long axis (vtbl+0x214 gates it per wheel and its
                // axle partner; its meaning is not traced — taken as true [S]).
                let mut force = scale(longd, wdrive);
                // The application point's height scale (class 0x4fd/0x4fe flags taken 0 [S]).
                r[1] *= p.wheel_force_app_scale;
                if slipping && !spin {
                    let ratio = (excess / limit).min(1.0);
                    let s = if front_axle { p.front_slipping_scale } else { p.rear_slipping_scale };
                    f_lat *= 1.0 - (1.0 - s) * ratio;
                }
                let latv = scale(lat, f_lat);
                let mut spring = scale(n, f_spring); // 0x280
                // The slope term [M]: the spring's horizontal part × SLOPE_SCALE·|n̂_xz·f̂_xz|
                // (×0.2 at ≤ 0.707, ≥ 0.05).
                let mut sl = dot(norm([n[0], 0.0, n[2]]), norm([fwd[0], 0.0, fwd[2]])).abs();
                if sl <= 0.707 { sl *= 0.2; }
                if sl < 0.05 { sl = 0.05; }
                if !p.is_rally || speed >= 5.0 || hb == 0.0 {
                    spring[0] *= p.slope_scale * sl;
                    spring[2] *= p.slope_scale * sl;
                }
                if hb != 0.0 {
                    spring = sub(spring, scale(norm(vpt), hb));
                }
                if hb == 0.0 || speed > 1.0 {
                    spring = sub(spring, latv);
                }
                spring = sub(spring, roll_vec);
                force = add(force, spring);
                self.slip[i] = (f_lat.abs() / limit).min(1.0);
                // lateralLoss past 10 m/s [M]: the table is all 1.0 [M .data 0x331a18], a no-op.
                if speed > 10.0 {
                    let mut ll = friction.lateral_loss.get(pl.surface as usize).copied().unwrap_or(1.0);
                    if ll != 1.0 {
                        if front_axle { ll += (1.0 - ll) * 0.5; }
                        if ll * limit < f_lat.abs() {
                            let mut fb = self.world_to_body(force);
                            fb[0] *= ll * limit / f_lat.abs();
                            force = self.body_to_world(fb);
                        }
                    }
                }
                self.acc_accum = add(self.acc_accum, force);
                self.torque_accum = add(self.torque_accum, cross(r, force));
            } else if x == 0.0 {
                self.slip[i] = 0.0;
                // A lifted wheel on a grounded car [M the x == 0 arm]: when its point rises
                // (ω×r).y > 0, a (0, −50, 0) push at the wheel adds torque only (FUN_001f88c8).
                // 0x29d / the class 0x4fd/0x4fe flags have no traced writer: 0 [S].
                if prev_contacts != 0 && upright {
                    let r = sub(wp, self.pos);
                    if cross(self.ang_vel, r)[1] > 0.0 {
                        self.torque_accum = add(self.torque_accum, cross(r, [0.0, -50.0, 0.0]));
                    }
                }
            }
            self.spring_prev[i] = x;
        }
        // The yaw stability torque [M FUN_0018c1b0 tail]: −ω_body.y·YAW_STABILITY (×8 while
        // counter-steering) about the body up axis.
        if p.yaw_stability != 0.0 {
            let wb = self.world_to_body(self.ang_vel);
            let k = if self.counter_steering { 8.0 } else { 1.0 };
            let t = self.body_to_world([0.0, -wb[1] * p.yaw_stability * k, 0.0]);
            self.torque_accum = add(self.torque_accum, t);
        }
        self.contacts = count;
        // The handbrake park [M 0x18cef8 after FUN_0018c1b0]: all four down, handbrake, throttle
        // < 0.1 and |v| < 0.08 → the horizontal velocity, the horizontal force and the yaw
        // torque zero; the spin is reset (FUN_001f8588 with the zero vector 0x355b80 [S value]).
        if count == 4 && input.handbrake && throttle < 0.1 && len(self.vel) < 0.08 {
            self.vel[0] = 0.0;
            self.vel[2] = 0.0;
            self.acc_accum[0] = 0.0;
            self.acc_accum[2] = 0.0;
            self.torque_accum[1] = 0.0;
            self.ang_vel = [0.0; 3];
        }
        let _ = park;
    }
}
