//! light-sun: the view model's sun-doubled ambient. `Env_Create` 0x17b968:116-121 builds `SunVector`
//! from the env placement (param 9 enables; `Vec_Spherical_2_Cartesian(1000, p10·π/180, −p11·π/180)`);
//! `HUD_UpdateLensFlarePane` 0x19d770:64-90 sets BLData+0x962 when `Collide_LineOfSight(head,
//! head − SunVector, …, 8)` is clear; `Lights_CalcAmbientLight` 0x1cb178:20-35 doubles (clamped to
//! 0xff) the cel ambient of objects whose flags carry 0x10000 while it is set. Measured at the
//! 0700000c start [L PCSX2, 2026-10-17]: BLData+0x962 = 1, Bond's obj+0x100 = (60, 60, 60), the view
//! model (obj+0xF0 = 0x1a8a0) = (120, 120, 120), both linked to the same cel.
//! Opt-in: NF_ISO.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn run(level: &str, ticks: u32) -> nf_view::scene::Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(iso().as_deref().unwrap(), level, false).expect("load");
    for _ in 0..ticks { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    scene
}

#[test]
fn the_0c_start_sees_the_sun_and_the_view_model_doubles_its_ambient() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    let scene = run("0700000c.bin", 340);
    let sim = scene.sim.as_ref().unwrap();
    // [D] the main map's env placement: params 9 = 1, 10 = 182, 11 = 25.
    let sv = sim.drones.sun_vector;
    assert!(sv != [0.0; 3], "0700000c's env enables the sun");
    let len = (sv[0] * sv[0] + sv[1] * sv[1] + sv[2] * sv[2]).sqrt();
    assert!((len - 1000.0).abs() < 0.5, "Vec_Spherical_2_Cartesian radius 1000, got {len}");
    assert!(sim.sun_seen, "BLData+0x962 = 1 at the 0700000c start [L]");
    let vm = scene.view_models.get(&sim.gun.weapon).copied().expect("the drawn view model");
    assert_eq!(sim.char_ambient[vm], [120, 120, 120], "the view model's doubled ambient [L PCSX2 obj+0x100]");
}

#[test]
fn sister_a_level_without_a_sun_keeps_the_plain_ambient() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    // [D] 07000005's env params 9..11 are 0 — no SunVector, no doubling.
    let scene = run("07000005.bin", 340);
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.sun_vector, [0.0; 3]);
    assert!(!sim.sun_seen);
    let vm = scene.view_models.get(&sim.gun.weapon).copied().expect("the drawn view model");
    let a = sim.char_ambient[vm];
    assert!(a.iter().all(|&b| b < 0xff) && a != [120, 120, 120], "no doubling without the sun: {a:?}");
}
