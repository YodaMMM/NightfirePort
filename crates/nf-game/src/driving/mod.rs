//! The driving missions' model: the vehicle physics (`physics.rs`, task `drive-physics`), the
//! road network (`road.rs`, the `RNgp` article) and the AI car step (`ai.rs`, task
//! `drive-ai-traffic`). docs/systems/drive-physics.md and docs/systems/driving-ai.md are the
//! write-ups; `ai::SimCar` is the physics seam (506's `Car` plugs in — see `ai::AiDriver`).

pub mod ai;
// The sound logic (banks, mix groups, engine/skid/road-noise voices) — DRIVING.ELF audio classes.
pub mod audio;
// The hostile AI and the AI element lifecycle (AIEl/AICo, FUN_0012bec8 and the chase arms).
pub mod hostile;
// The hostile AI and the AI element lifecycle (AIEl/AICo, FUN_0012bec8 and the chase arms).
// The per-pattern chase steering, avoidance, turret, world cap, helicopter leaf rules (§11).
pub mod patterns;
// G-056 world-bounds: the driving maps' invisible boundary sheets (DRIVING.ELF 0x331a60).
pub mod bounds;
// The player chase camera (camera.ini HeliCam + FUN_001b2b48).
pub mod camera;
// Effects: the particle pool, emitters, wheel smoke/spray/sparks, glare pulse (FUN_001f29d8 family).
pub mod effects;
// The MIS01 sniper phase's flight path, triggers, timers, view limits and shot test.
pub mod flight;
// G-057 suspension: the per-wheel spring–damper (sub_18FC00 + kind siblings).
pub mod suspension;
// 647 drive-mission-scripts: the mission state machine (objectives, fails, timers, the PTP end).
pub mod mission;
// The game's mission rules: the trigger/action/event machine and the mission manager (FUN_00206b88).
pub mod script;
pub mod physics;
// Track scene objects: drawn/hidden, in the sim, knocked loose (trackobj.md).
pub mod props;
// The POV weapon mode (the MIS01 sniper phase): weapon table, loadout, gunner, mode flag.
pub mod pov;
pub mod road;
// The SUB_PHYSICS pass (FUN_00190f88: the sub + the ultralights), the VU0 rigid integrator and
// the per-vehicle pass dispatcher (FUN_00188200 0x188bf8).
pub mod sub_physics;
// The player car's recovery: the engine's righting reset (FUN_00188200 → FUN_00187b00) and the
// scripted placements (FUN_00187eb8).
pub mod reset;
// The weapon table, ammo/reload, projectiles and hit damage (FUN_0020f2b8 + the WeaponMan).
pub mod weapons;
// The 3-D listener, pan law, impacts, landings, horns, traffic engine (DRIVING.ELF audio).
pub mod sound_events;

// The car physics lives in `physics.rs` (a single `driving.rs` beside this `driving/`
// directory would collide, rustc E0761). The re-export keeps every public path the tests and
// callers use: `nf_game::driving::{Car, CarInput, CarParams,
// FlatWorld, Rigid, Tuning, DT, Friction, SURFACES}`.
pub use physics::{
    Car, CarInput, CarParams, CollisionGround, DT, FlatGround, FlatWorld, Friction, Ground, Rigid, SURFACES,
    Tuning, WallHit, WheelPlane,
};
