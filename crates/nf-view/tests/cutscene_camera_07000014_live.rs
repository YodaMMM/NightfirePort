//! 07000014 cut-scene camera: the camera must NOT take the SP's placement matrix as its base.
//!
//! [M `Script_CameraStart` 0x1d9140] opens with `Mat_IdentityT()` — the camera's base concat runs on the
//! identity, so its world matrix is the key matrix alone. The port composed `self.matrix` (the SP's
//! placement) in `Script_Interp` case 4, which pushed the first shot from the room out to
//! (78.06, 4.79, 82.69) — 56 units from the scene's own spot, inside rock: the flat-blue void.
//!
//! Fail-first: at the spot the first-shot camera sits ~56 units away on pristine; with the fix it is
//! ~16 units away, in the control room (the frames: out/cap14_scene_before vs _after).
//!
//! (The earlier "fov ~0.0001" reading was my probe printing the tuple's second element as the fov — it is
//! the YAW; the camera's fov is a healthy 0.25-0.50 throughout. This test pins the position.)

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn the_cutscene_camera_sits_in_the_room_not_outside_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let spot = [40.87, 2.98, 40.85];
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..100 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    sim.bond.pos = spot;
    let mut first: Option<([f32; 3], f32)> = None;
    for _ in 0..200 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if first.is_none() {
            if let Some((p, _yaw, _pitch, fov)) = sim.script_camera() { first = Some((p, fov)); }
        }
    }
    let (p, fov) = first.expect("the scene starts a script camera at the spot");
    assert!((0.15..1.2).contains(&fov), "the camera's fov is a sane key value, got {fov}");
    let d = dist(p, spot);
    assert!(d < 25.0,
        "the first shot must sit in the room around the spot — with the camera's base reset to the \
         identity [M Script_CameraStart 0x1d9140's Mat_IdentityT] the port put it {d:.1} units away \
         ({p:?}); the plate-matrix compose put it 56 units out, inside rock (the flat-blue void)");
}
