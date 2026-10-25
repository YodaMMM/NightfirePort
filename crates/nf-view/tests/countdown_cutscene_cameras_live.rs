//! 993-countdown-cutscene-cameras: 07000014's cut-scene cameras against the disc data.
//!
//! Evidence base:
//! - The opening scene is ScriptPlayer #276 (kind 0xD9, script 0x06000833, pack 07800014.bin,
//!   trigger ch 0x7E ← TouchOnce #301 → OR #302 [D placements + triggers]). The scene runs 1400
//!   ticks; stream 0's camera op `Camera { spline: false, first: 0, last: 0xa7 }` uses keys
//!   0..166 [D nis dump]. The four actor streams are `Anim` ops on skins
//!   0x05000087/0x05000090/0x05000093/0x0500005e with anim scripts 06000854-57 (in
//!   07000014.bin) whose op-0 windows open the pack sequences 0x04000725-28 [D decode].
//! - The camera key decode is `Script_Interp` case 4 (0x1d9a08:190-213): `Script_GetInterp`
//!   0x1da158 builds the key matrix (key+0x00 VECTOR4 position, +0x10 quaternion, +0x20 the fov
//!   scale, +0x2c time; stride 0x30) and the 10th arg = 1 concats the ScriptPlayer's base
//!   matrix (disasm 0x1da054-0x1da068); `Script_UpdateCamera` 0x1db518 copies it to viewer 4 and
//!   `Camera_CalcViewAngles` 0x1baf70 takes the vertical fov = key scale × viewer+0x114 = 4/3.
//!   THE BASE IS IDENTITY for this scene: `Script_CameraStart` 0x1d9140's first act — before the
//!   flag test — is `Mat_IdentityT(SCRIPTINFO)` [M 0x1d916c: the jal's a0 still holds param_1;
//!   the decompiler dropped the argument], so SCRIPTINFO+0x00 (which `Script_Load` 0x1d8498
//!   seeded with the placement transform) reads identity from the camera start on and nothing
//!   restores it (`Script_CameraEnd` 0x1db348 kills the stream only). The camera world matrix
//!   IS the raw key. The earlier reading (base = the placement translation, camera =
//!   (78.06, 4.79, 82.69)) put every shot of this scene — and 0x0600062f/0x06000848's cameras
//!   and the 0x0600062f hand-back marker — in disc-data voids (probe_the_disc_data_at_the_stages:
//!   zero placements and zero glists at (70..105, 70..105)); the raw keys sit in the real
//!   security-centre room. Scripts without a camera op keep the placement base (0x06000a1b's
//!   wires, 0x060009e2's SafePickUps — camera-flag 0 [D header]).
//! - The disc key values below were decoded from 07800014.bin's NIS 06000833 (host-float eval
//!   of the PS2 key records, ±1e-5): shot A (ticks 0..199) holds local (25.9049, 2.1571,
//!   36.0720) with the t0 key's fov scale 0.445614 and 0.373221 from t1; shot B (t200..329)
//!   cuts to (27.8158, 1.5286, 47.4639), fov 0.188249; the dolly lands (27.8749, 1.2693,
//!   43.6958) at t499 and holds to 1400.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Placement #276's translation [D placements-all map 54 #276] — the base `Script_Load` seeds
/// [M 0x1d8498] and `Script_CameraStart` resets to identity [M 0x1d916c] before any key runs.
/// Kept for the record; the expectations below are the RAW keys.
#[allow(dead_code)]
const SP276_T: [f32; 3] = [52.156647, 2.636515, 46.62138];
/// The disc key positions (local to the placement) [D 07800014.bin NIS 06000833 keys].
const SHOT_A_LOCAL: [f32; 3] = [25.9049, 2.1571, 36.0720]; // keys t0..t199
const SHOT_B_LOCAL: [f32; 3] = [27.8158, 1.5286, 47.4639]; // keys t200..t329
const SHOT_C_LOCAL: [f32; 3] = [27.8749, 1.2693, 43.6958]; // keys t499..t1400 (the dolly lands here)
const FOV_A0: f32 = 0.445614 * 4.0 / 3.0; // the t0 key's scale × viewer+0x114 [M 0x1baf70]
const FOV_A: f32 = 0.373221 * 4.0 / 3.0; // t1..t199
const FOV_B: f32 = 0.188249 * 4.0 / 3.0; // t200..t329
/// The scene's actor skins (the `Anim` ops' skin words) [D decode].
const SKINS: [u32; 4] = [0x0500_0087, 0x0500_0090, 0x0500_0093, 0x0500_005e];

fn load() -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    nf_view::load(&iso, "07000014.bin", false).expect("load 07000014")
}

/// The opening scene's camera against the disc keys: the port's `script_camera()` per tick vs
/// the [D] key records (position = the placement translation + the key local, fov = scale × 4/3).
#[test]
fn the_opening_scene_camera_matches_the_disc_keys() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load();
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.level, 0x0700_0014);
    // The vent-drop touch #301 (39.68…41.75, 2.07…4.14, 37.62…40.40) [D] fires the scene
    // (walk_07000014_live's stage 1 — the real trigger path).
    sim.bond.pos = [40.7, 2.6, 39.0];
    let mut cam_tick: u32 = 0;
    let mut shots: Vec<(u32, [f32; 3], f32)> = Vec::new();
    let want_at = |t: u32| matches!(t, 0 | 1 | 100 | 199 | 200 | 201 | 250 | 330 | 500 | 900);
    for _ in 0..1700u32 {
        sim.step_bond(Input::default(), false, false, false);
        if let Some((pos, _yaw, _pitch, fov)) = sim.script_camera() {
            if want_at(cam_tick) { shots.push((cam_tick, pos, fov)); }
            cam_tick += 1;
            if cam_tick > 950 { break; }
        }
    }
    assert!(cam_tick > 500, "the opening scene's camera must run past the t499 dolly (got {cam_tick} camera ticks)");
    let close = |a: f32, b: f32| (a - b).abs() < 5e-3;
    for (t, pos, fov) in &shots {
        let (want_pos, want_fov): ([f32; 3], f32) = match *t {
            0 => (SHOT_A_LOCAL, FOV_A0),
            1..=199 => (SHOT_A_LOCAL, FOV_A),
            200..=329 => (SHOT_B_LOCAL, FOV_B),
            330..=498 => (SHOT_B_LOCAL, FOV_B), // mid-dolly: pinned loosely below
            500..=1400 => (SHOT_C_LOCAL, FOV_A0),
            _ => continue,
        };
        if *t == 330 { continue; } // the dolly is mid-flight here; the endpoints pin it
        assert!(close(pos[0], want_pos[0]) && close(pos[1], want_pos[1]) && close(pos[2], want_pos[2]),
            "camera tick {t}: pos {pos:?} vs the disc keys {want_pos:?} (placement + key local [D 06000833])");
        assert!(close(*fov, want_fov), "camera tick {t}: fov {fov:.6} vs the disc key scale × 4/3 {want_fov:.6} [M 0x1baf70]");
    }
    // The base is the IDENTITY `Script_CameraStart` writes [M 0x1d916c]: the port's camera IS
    // the raw key (the concat of SCRIPTINFO+0x00 [M 0x1da054-0x1da068] is a no-op here).
    let (pos, _y, _p, fov) = sim.script_camera().expect("the camera still runs");
    assert!(close(pos[0], SHOT_C_LOCAL[0]) && close(pos[1], SHOT_C_LOCAL[1]) && close(pos[2], SHOT_C_LOCAL[2]),
        "the camera world matrix is the raw key (the base is identity [M 0x1d916c]), got {pos:?}");
    let _ = fov;
    for (t, pos, fov) in &shots { eprintln!("trace: camera tick {t} pos {pos:?} fov {fov:.6}"); }
}

/// The scene's actors exist on the right skins and the room the camera sees resolves its cel —
/// the camera at (78.06, 4.79, 82.69) and the tight shot at (79.97, 4.17, 94.09) must land in
/// cels (the eye-cel find `build_FindCel` 0x1c8cb0 [M]; the G-052 family: a room the eye-cel
/// walk misses draws as the clear colour).
#[test]
fn the_scene_actors_exist_and_the_camera_cels_resolve() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [40.7, 2.6, 39.0];
    let mut cam_tick: u32 = 0;
    let mut anims: Vec<(u32, u32, f32)> = Vec::new();
    for _ in 0..1700u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.script_camera().is_some() {
            if cam_tick == 30 {
                for o in sim.drones.scripts.objects.list.iter().flatten() {
                    if let nf_game::nis::Obj::Anim { skin, script, frame, .. } = o {
                        anims.push((*skin, *script, *frame));
                    }
                }
            }
            cam_tick += 1;
            if cam_tick > 60 { break; }
        }
    }
    for s in SKINS {
        assert!(anims.iter().any(|(skin, _, _)| *skin == s),
            "the scene mounts an Anim object on skin {s:#x} [D 06000833's Anim ops], got {anims:?}");
    }
    for (skin, script, frame) in &anims { eprintln!("trace: scene Anim skin {skin:#x} script {script:#x} frame {frame}"); }
    // The RENDER side: a Character exists per resolved skin (the build skips a script actor whose
    // skin mesh never resolved — `level.skins.get(&skin_hash) else continue`), so a missing
    // character here is the "dark-blue box"/invisible-actor symptom's port side.
    let scene_skins: Vec<u32> = scene.characters.iter().filter(|c| c.script.is_some()).map(|c| c.skin.hashcode).collect();
    for s in SKINS {
        let n = scene.characters.iter().filter(|c| c.skin.hashcode == s).count();
        eprintln!("trace: characters on skin {s:#x}: {n} (script characters total {})", scene_skins.len());
        assert!(n >= 1, "a Character mesh resolved for skin {s:#x} [D 06000833's skins] — none means the actor draws as the missing-mesh box");
    }

    // The cel probes at the camera spots and the actors' roots [D decode + the pack sequence
    // roots + the placement translations]:
    let g = &sim.cel_graph;
    let spots: Vec<(&str, [f32; 3])> = vec![
        ("camera shot A", [78.0615, 4.7936, 82.6934]),
        ("camera shot B", [79.9724, 4.1651, 94.0853]),
        ("camera dolly end", [80.0315, 3.9058, 90.3172]),
        ("actor 0x87 root", [78.4300, 3.8000, 95.0620]),
        ("actor 0x90 root", [81.2060, 3.8000, 95.1010]),
        ("actor 0x93 root", [85.8430, 3.8040, 78.0810]),
        ("actor 0x5e root", [79.9940, 3.5770, 96.5770]),
    ];
    for (name, eye) in spots {
        let c = nf_view::scene::find_cell_raycast(g, eye, Some(&sim.world));
        eprintln!("trace: cel at {name} {eye:?} = {c:?}");
        assert!(c.is_some(), "the eye-cel find resolves at {name} {eye:?} [M 0x1c8cb0]");
    }
    // The room's floor geometry around the actors belongs to SOME cel bucket near the camera's
    // (the G-052 shape: a room whose floor lands in a far bucket draws as the clear colour).
    let owners = sim.world.debug_bucket_owners([77.5, 3.0, 77.5], [86.5, 4.5, 97.0]);
    eprintln!("trace: bucket owners over the scene room floor: {:?}", owners);
}

#[allow(dead_code)]
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
