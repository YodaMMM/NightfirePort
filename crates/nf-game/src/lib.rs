//! Game simulation for 007: Nightfire (PS2), ported from ACTION.ELF: the fixed-tick world the viewer (and later the
//! game) runs. Rendering lives elsewhere; this crate only advances state.
//!
//! The host-OS seams (misc-9): the game's newlib/kernel facilities have no port counterpart
//! because the host does the job. `sbrk` 0x23c298 (172 B — grows the newlib heap for
//! `psiMem_Init` 0x11f470's boot `malloc`s through `malloc_extend_top`/`_malloc_trim_r`) bounds
//! itself with `EndOfHeap` 0x23be10 (syscall 0x3e stub) — the port allocates with the host
//! allocator. `write` 0x23c190 (fd 1/2 → `sceTtyInit`/`sceTtyWrite`), `isatty` 0x23c348
//! (always 1) and `fstat` 0x23c350 (character-device bit) serve the C standard streams when a
//! printf-family call fires (the SFXUpdate 0x1156c0:13 overflow warning, `_ErrMessage`'s error
//! path — 29 printf call sites total [M caller scan]); the port logs through the host's
//! stderr. `Deci2Call` 0x23c020 (syscall 0x7c) is the PS2 debug console — `PS2FreshBoot`
//! 0x120ab0:6 (from `main`) scePrintfs unconditionally at every fresh boot → `deci2Putchar`
//! → `kputs` → `Deci2Call` [M caller scan]; the port's boot log is the host console.
//! Never reached in the shipped game (docs/port-map/unreached.md): `GetMemorySize` 0x23c030
//! (its chain `Exit` → `TerminateLibrary` → `InitTLB` runs only when main returns — the frame
//! loop at 0x109080 never does), `read` 0x23c210 (stdin-only; no read-side stdio user exists)
//! and `lseek` 0x23c290 (no seekable/append stream is ever used).
//! Evidence per function: docs/systems/kernel-stubs-misc9.md.

// Two match arms for the same (state, message) leave the second dead: a merge once renamed the guard's
// GoToGoalPosition (0x63) arms to the ally's 0x28, freezing guards on their way to a goal, and only this lint saw it.
#![deny(unreachable_patterns)]

pub mod aipoint;
pub mod aivolume;
pub mod prof;
pub mod ally;
pub mod anim;
pub mod bullet;
pub mod breakable;
pub mod car;
pub mod corona;
pub mod cover;
pub mod creepwall;
pub mod damage;
pub mod door;
pub mod drone;
pub mod target_registry;
pub mod env;
pub mod explode;
pub mod gun;
pub mod hud_pane;
pub mod grenade;
pub mod hurt;
pub mod ladder;
pub mod kiko;
pub mod input;
pub mod light;
pub mod mp;
pub mod fusebox;
pub mod gadget;
pub mod gas;
pub mod gt;
pub mod body_glow;
pub mod rigid_body;
pub mod space_laser;
pub mod copter;
pub mod shadow;
pub mod watch;
pub mod casing;
pub mod effect;
pub mod lock;
pub mod ss;
pub mod sensor;
pub mod mission;
pub mod monitor;
pub mod ptp;
pub mod driving;
pub mod driving_hud;
pub mod music;
pub mod death;
pub mod nav;
pub mod nis;
pub mod props;
pub mod psi_tex;
pub mod shatter;
pub mod gunimp;
pub mod pickup;
pub mod player;
pub mod progress;
pub mod secret_codes;
pub mod profile;
pub mod stats;
pub mod ps2math;
pub mod searchlight;
pub mod sky;
pub mod sound;
pub mod spawners;
pub mod space_missile;
pub mod trig;
pub mod triggers;
pub mod tuning;
pub mod useobj;
pub mod view;
pub mod wire;
pub mod world;
pub mod worldobj;
pub mod world_fx;
pub mod subtitle;
