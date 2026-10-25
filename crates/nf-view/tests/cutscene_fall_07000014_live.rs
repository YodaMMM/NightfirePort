//! level-14-cutscene-fall: the real-input run from the 07000014 cut-scene triggers to the
//! hand-backs. The two reported symptoms:
//! - the opening scene (trigger TouchOnce #301, SP #276, script 0x06000833 [D placements]):
//!   the port's camera used to sit at the composed spot (placement #276 + key) = (78.06, 4.79,
//!   82.69) — a disc-data void (probe_the_disc_data_at_the_stages: zero placements and zero
//!   glists in (70..105, −10..15, 70..105)); the frame rendered the clear colour around the eye.
//! - the fall (trigger TouchOnce #381, SP #382, script 0x0600062f [D]): the port's hand-back
//!   enable used the COMPOSED marker matrix (−107.07, 1.55, 73.28) — also a disc-data void —
//!   and Bond fell below the map (real play: (−107.07, −70.14, 73.28) at frame 21799).
//! Root cause [M 0x1d916c]: `Script_CameraStart`'s first act — before the flag test — is
//! `Mat_IdentityT(SCRIPTINFO)` (the decompiler dropped the argument; the jal's a0 still holds
//! param_1 = SCRIPTINFO), so a script WITH a camera op keys every matrix in WORLD space from
//! the camera start on (nothing restores the base — `Script_CameraEnd` 0x1db348 kills the
//! stream only). The raw keys sit in the real rooms; `Script_Load`'s placement base [M
//! 0x1d8498] only governs scripts WITHOUT a camera op (the fusebox wires 0x06000a1b, the
//! SafePickUps 0x060009e2, the truck 0x06000a22 — camera-flag 0 [D header]).
//! Run: `NF_ISO=… cargo test -p nf-view --test cutscene_fall_07000014_live -- --test-threads=1`.

#[path = "common/walker.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The vent cut-scene (the broken-camera report): from the REAL trigger path (Bond walked into
/// TouchOnce #301's volume (39.68…41.75, 2.07…4.14, 37.62…40.40) [D]) the camera runs the whole
/// scene inside the security-centre room — every sampled tick within the room band WITH floor
/// below (the composed void had none) — and the scene completes (ch 0x47 [D]).
#[test]
fn the_vent_cutscene_camera_stays_in_the_room() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014"));
    // Watch the cut-scenes, never skip them (the walker's default presses the game's own skip
    // action while Bond is held [M Input_Action(−1, 0x19, 1)] — this test pins the scene itself).
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [40.7, 2.6, 39.0];
    }
    w.wait(30);
    let mut cam_ticks = 0u32;
    let mut samples: Vec<[f32; 3]> = Vec::new();
    for _ in 0..1800u32 {
        if let Some((pos, _yaw, _pitch, _fov)) = w.scene.sim.as_ref().unwrap().script_camera() {
            cam_ticks += 1;
            if cam_ticks % 150 == 1 { samples.push(pos); }
            if cam_ticks > 1400 { break; }
        }
        w.tick(Default::default(), false);
    }
    // The camera mounts when the script's first tick runs the op (~28 ticks after the staging
    // contact here), so the Some() count = the script's 1400 minus the pre-mount ticks.
    assert!(cam_ticks >= 1370, "the scene's camera runs the 1400-tick script [D length], got {cam_ticks}");
    for (k, p) in samples.iter().enumerate() {
        // The disc keys [D 07800014.bin NIS 06000833]: shot A (25.90, 2.16, 36.07) holds
        // t0..199, shot B (27.82, 1.53, 47.46) t200..329, the dolly lands (27.87, 1.27, 43.70)
        // t499..1400 — all inside the security-centre room band, none near the composed void.
        assert!(p[0] > 23.0 && p[0] < 31.0 && p[1] > 0.0 && p[1] < 4.5 && p[2] > 33.0 && p[2] < 50.0,
            "camera sample {k} at {p:?} leaves the security-centre room [D raw keys]");
        let floor = w.scene.sim.as_ref().unwrap().world.segment(*p, [p[0], p[1] - 30.0, p[2]]);
        assert!(floor.is_some(), "camera sample {k} at {p:?}: no floor within 30 m below (the composed void symptom)");
    }
    // The scene completes through the real path: ch 0x47 (objective 1) writes at its last tick.
    let mut ch47 = false;
    for _ in 0..300u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x47) { ch47 = true; break; }
    }
    assert!(ch47, "the vent cut-scene completes (ch 0x47 [D]) after the camera run");
    // Bond resumes where he was (event 6 finds no previous entity → the enable keeps his own
    // matrix [M 0x1d94c0:73-84]) — no teleport, still in the vent route.
    for _ in 0..600u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if !sim.disabled { break; }
        }
        w.tick(Default::default(), false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] > 38.0 && p[0] < 44.0 && p[2] > 36.0 && p[2] < 43.0,
        "Bond resumes near the trigger spot (the event-6 fallback keeps his matrix), got {p:?}");
}

/// The fall: from the REAL trigger path (TouchOnce #381's volume (−51.97…−46.39, 0…2.07,
/// 37.28…39.46) [D]) the Kiko cut-scene plays with its camera in the touch room, then the
/// hand-back puts Bond ON THE FLOOR at the marker's raw key (−55.49, 1.55, 42.02 [D
/// 0600062f key 471; the base is identity [M 0x1d916c]]) — the composed spot (−107.07, 1.55,
/// 73.28) was a disc-data void and Bond fell to y −70 in real play — and the walk continues.
#[test]
fn the_kiko_cutscene_hands_bond_back_on_the_floor_and_the_walk_continues() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014"));
    // Watch the cut-scenes, never skip them (the walker's default presses the game's own skip
    // action while Bond is held [M Input_Action(−1, 0x19, 1)] — this test pins the scene itself).
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-49.2, 0.9, 38.4];
    }
    w.wait(30);
    // The cut-scene: the camera mounts (event 4 disabled Bond) and runs the 1200-tick script.
    let mut cam_ticks = 0u32;
    let mut samples: Vec<[f32; 3]> = Vec::new();
    for _ in 0..1500u32 {
        if let Some((pos, _yaw, _pitch, _fov)) = w.scene.sim.as_ref().unwrap().script_camera() {
            cam_ticks += 1;
            if cam_ticks % 200 == 1 { samples.push(pos); }
        }
        w.tick(Default::default(), false);
        if cam_ticks > 1200 { break; }
    }
    assert!(cam_ticks > 1100, "the Kiko scene's camera runs [D 0600062f length 1200], got {cam_ticks}");
    for (k, p) in samples.iter().enumerate() {
        // The touch room band: the disc keys [D 0600062f] hold the camera at (−51.27, 0.57,
        // 39.70) (t0..514) then (−55.75, 0.89, 40.69) (t515..1200) — floor below throughout.
        assert!(p[0] > -60.0 && p[0] < -44.0 && p[1] > -1.0 && p[1] < 4.0 && p[2] > 34.0 && p[2] < 46.0,
            "camera sample {k} at {p:?} leaves the touch room [D raw keys]");
        let floor = w.scene.sim.as_ref().unwrap().world.segment(*p, [p[0], p[1] - 30.0, p[2]]);
        assert!(floor.is_some(), "camera sample {k} at {p:?}: no floor within 30 m below");
    }
    // The hand-back: Bond re-enables AT the marker's raw key, standing.
    let mut back: Option<[f32; 3]> = None;
    for _ in 0..400u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if !sim.disabled { back = Some(sim.bond.pos); break; }
        }
        w.tick(Default::default(), false);
    }
    let p = back.expect("the hand-back re-enables Bond [M Script_Free 0x1da880:46-49]");
    assert!((p[0] - -55.49).abs() < 0.4 && (p[2] - 42.02).abs() < 0.4,
        "the hand-back pose is the marker's raw key [D 0600062f key 471; M 0x1d916c base reset], got {p:?}");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let under = sim.world.segment(p, [p[0], p[1] - 5.0, p[2]]);
        let floor = under.map(|t| p[1] - 5.0 * t);
        // The floor is under the spot within the root-to-feet offset (the root rides ~1.1 above
        // the standing floor — the game's enable plants the matrix origin directly [M
        // 0x195498:57-68] and gravity settles the capsule; the port's stand_at does the same).
        assert!(matches!(floor, Some(f) if (f - p[1]).abs() < 1.3),
            "the floor is under the hand-back spot, got t {under:?} -> floor {floor:?} at {p:?}");
    }
    // Bond STAYS standing (the fall symptom: y −70 by 159 ticks after the enable).
    w.wait(180);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[1] > 0.4 && !sim.disabled,
            "Bond is still on the floor 180 ticks after the hand-back, got {:?}", sim.bond.pos);
    }
    // The walk continues — a real-input leg west toward the sabotage route (the objective's
    // direction: the west generator room, x −89 [D fuseboxes #383/#385]).
    w.try_walk_to(Wp::new(-59.5, 1.0, 41.5).r(1.0), "the walk continues west after the cut-scene");
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] < -56.5, "Bond walked west from the hand-back spot, got {p:?}");
}
