//! 993-cutscene-camera-07000001: the intro scene's camera against the disc keys.
//!
//! Evidence base:
//! - SP #277 "Script Player Wedge" (kind 0xd9, script 0x06000087, pack 07800001.bin, mode 1 = plays
//!   at load) runs the level-start cut-scene [D placements #277; docs/levels/07000001.md]. The
//!   script is camera=1, 15 streams, 2120 ticks; stream 0 is `Camera{first:0, last:1160}`
//!   [D decode].
//! - The camera's world matrix is the keys ALONE: `Script_CameraStart` 0x1d9140 opens with
//!   `Mat_IdentityT()` on the ScriptPlayer's base si+0x00, so every interp's base concat runs on the
//!   identity [M; the port fix merged from s/level-14-cutscene-door]. SP #277's placement is
//!   (0, 0, 0) [D], so this level's intro cannot distinguish a compose bug by position alone — the
//!   pin below still fixes the key values and the fov formula on the real path.
//! - The key decode is `Script_GetInterp` 0x1da158 (key+0x00 vec4 position, +0x10 quaternion,
//!   +0x20 fov scale, +0x2c time, stride 0x30) [M]; fov = key scale × 4/3 [M Camera_CalcViewAngles
//!   0x1baf70]. Disc keys [D 07800001.bin!06000087, host-float]: t0 pos (-6.8660, 6.7988, -9.1352)
//!   scale 0.305986; t1 scale 0.157729; the dolly drifts to (-10.2547, 7.0901, -10.7304) by t2108.
//! - Stream 8's event 0x0D writes ch 0xB = 1 at the tail marker (~t2119 [D]; walk_07000001_live
//!   pins the hand) — the intro's hand-back to gameplay, and stream 9's marker entity
//!   (0x02000617, type 0x19) re-enables Bond at its key [M 0x1d94c0 event 4/6 arms].
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The disc key values [D 07800001.bin!06000087 stream 0].
const T0_POS: [f32; 3] = [-6.8660, 6.7988, -9.1352];
const FOV_T0: f32 = 0.305986 * 4.0 / 3.0;
const FOV_T1: f32 = 0.157729 * 4.0 / 3.0;
const T2108_POS: [f32; 3] = [-10.2547, 7.0901, -10.7304];
const FOV_T2108: f32 = 0.305170 * 4.0 / 3.0; // key[1159]'s own scale [D]
#[test]
fn the_intro_camera_matches_the_disc_keys() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso().unwrap(), "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.level, 0x0700_0001);
    // mode 1: the scene plays at load — no trigger, no input, no pos writes.
    let mut cam_tick: u32 = 0;
    let mut shots: Vec<(u32, [f32; 3], f32)> = Vec::new();
    for _ in 0..2200u32 {
        sim.step_bond(Input::default(), false, false, false);
        if let Some((pos, _yaw, _pitch, fov)) = sim.script_camera() {
            // assert the t0 hold (keys 0..200 hold the t0 pos, t=213 at key 200) and the t2108
            // endpoint; the mid-dolly ticks are logged only (the spline samples between keys).
            if matches!(cam_tick, 0 | 1 | 100 | 2108) {
                shots.push((cam_tick, pos, fov));
            }
            cam_tick += 1;
            if cam_tick > 2118 { break; }
        }
    }
    assert!(shots.iter().any(|&(t, _, _)| t == 2108), "the intro camera must reach the t2108 key (got ticks {:?})", shots.iter().map(|s| s.0).collect::<Vec<_>>());
    let close = |a: f32, b: f32| (a - b).abs() < 5e-3;
    for (t, pos, fov) in &shots {
        let (want_pos, want_fov): ([f32; 3], f32) = match *t {
            0 => (T0_POS, FOV_T0),
            1 | 100 => (T0_POS, FOV_T1),
            2108 => (T2108_POS, FOV_T2108),
            _ => continue,
        };
        assert!(close(pos[0], want_pos[0]) && close(pos[1], want_pos[1]) && close(pos[2], want_pos[2]),
            "camera tick {t}: pos {pos:?} vs the disc keys {want_pos:?} [D 06000087 stream 0 keys]");
        assert!(close(*fov, want_fov),
            "camera tick {t}: fov {fov:.6} vs the disc key scale × 4/3 {want_fov:.6} [M 0x1baf70]");
    }
    for (t, pos, fov) in &shots { eprintln!("trace: camera tick {t} pos {pos:?} fov {fov:.6}"); }
}
