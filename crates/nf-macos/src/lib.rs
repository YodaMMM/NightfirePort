//! Native macOS Metal engine wrapper and CLI verification harness for `NightfirePort`.
//!
//! Runs 100% native Rust (wgpu Metal + CoreAudio via `nf-audio`) with feature parity
//! against `nf-view` and `nf-android` across FrontEnd, On-Foot, Multiplayer, and Driving modes.

pub mod controls;
pub mod engine;
pub mod inspect;
