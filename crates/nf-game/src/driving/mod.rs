//! The driving missions' model: the vehicle physics (`physics.rs`, task `drive-physics`), the
//! road network (`road.rs`, the `RNgp` article) and the AI car step (`ai.rs`, task
//! `drive-ai-traffic`). docs/systems/drive-physics.md and docs/systems/driving-ai.md are the
//! write-ups; `ai::SimCar` is the physics seam (506's `Car` plugs in — see `ai::AiDriver`).

pub mod ai;
// 647 drive-mission-scripts: the mission state machine (objectives, fails, timers, the PTP end).
pub mod mission;
pub mod physics;
pub mod road;

// The car physics lives in `physics.rs` (a single `driving.rs` beside this `driving/`
// directory would collide, rustc E0761). The re-export keeps every public path the tests and
// callers use: `nf_game::driving::{Car, CarInput, CarParams,
// FlatWorld, Rigid, Tuning, DT, Friction, SURFACES}`.
pub use physics::{Car, CarInput, CarParams, DT, FlatWorld, Friction, Rigid, SURFACES, Tuning};
