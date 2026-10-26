//! fix-0a-rain-indoors: the 0700000a rain confinement. The game's rain exists only inside the
//! level's `RainBox_Create` volumes (placement 0x32): `RainBox_Clip` 0x18d0d0 cube-clips each box
//! to the viewer ±15 and `AddDrop` 0x18c278 accepts a drop only inside an active box's footprint;
//! `UpdateDrops` 0x18c580 folds `DAT_003194f4` (−0.5 for rain, `InitDrops` 0x18bf68) into every
//! per-tick y step, so a drop drifts ≤ 0.86 m from its box over a full 22.5 m fall. The port
//! missed the −0.5: rain fell at the table's snow speed (0.025–0.1/tick), drifted up to ~6 m, and
//! sprayed into the dark office lounge at Bond (−37.50, 1.11, −16.95, yaw 2.827, pitch −0.092),
//! frame 2719 — a room no 0a box covers (the office cel's west face −42.44 sits east of rainbox5's
//! east face −42.27 by nothing; the spot is 4.77 m inside). [D] the seven boxes below are the disc
//! parse (FILES.BIN 0700000a.bin, main map placements #791–#797 `Effect_rainbox1..7`, glist bounds
//! + translation; the Env placement #530 param 0 = 1 = rain [M `Env_Create` 0x17b968 reads
//! level_tag+0x2C into `InitDrops`]).
//!
//! The office state is a state pin (the reported play spot), the same pattern as
//! abseil_damage_live.rs; the ticks run the real `Sim::step_bond` path.

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

/// The office lounge interior, east of rainbox5's east face (−42.265) by more than the [M] drift
/// bound (0.86 m): a drop here cannot have come from any box.
const OFFICE_X: f32 = -41.0;

#[test]
fn no_rain_draws_inside_the_office_lounge() {
    let path = iso();
    let mut scene = nf_view::load(&path, "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    // The reported play state (state pin; abseil_damage_live.rs pattern).
    sim.bond.pos = [-37.50, 1.11, -16.95];
    sim.bond.yaw = 2.827;
    sim.bond.pitch = -0.092;
    let input = nf_game::player::Input::default();
    sim.step_bond(input, false, false, false); // the first tick consumes env_pending (Game_Run order)
    // [D] the weather setup: one Env placement (param 0 = 1, rain) over seven RainBox volumes.
    let env = sim.drones.env.as_ref().expect("the 0a rain env");
    assert_eq!(env.kind, 1, "drop type 1 = rain [D Env placement #530 param 0; M Env_Create 0x17b968]");
    assert_eq!(env.boxes.len(), 7, "the seven Effect_rainbox placements [D #791-#797]");
    assert_eq!(env.fall, -0.5, "rain's DAT_003194f4 [M 0x18bf68]");
    for _ in 0..300 { sim.step_bond(input, false, false, false); }
    let env = sim.drones.env.as_ref().unwrap();
    assert!(env.alive.iter().filter(|&&a| a).count() > 0, "the rain is running outside");
    // No drop inside the office lounge: every alive drop stays east of OFFICE_X out of it.
    for (p, &a) in env.drops.iter().zip(&env.alive) {
        if !a { continue; }
        assert!(p[0] <= OFFICE_X, "rain drop {:?} draws inside the office lounge (x > {OFFICE_X})", p);
    }
}

/// Sister: the rain still draws OUTSIDE — at the same office state the active rainbox5 strip west
/// of the wall keeps populated drops (rain shows through the west windows), and at the
/// 0a spawn the overhead rainbox1 fills.
#[test]
fn rain_still_draws_outside_its_boxes() {
    let path = iso();
    let mut scene = nf_view::load(&path, "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [-37.50, 1.11, -16.95];
    sim.bond.yaw = 2.827;
    sim.bond.pitch = -0.092;
    let input = nf_game::player::Input::default();
    for _ in 0..300 { sim.step_bond(input, false, false, false); }
    {
        let env = sim.drones.env.as_ref().unwrap();
        // rainbox5 [D]: x −81.67..−42.27, z −57.89..−20.53 — the strip the office's west windows face.
        let in_box5 = env.drops.iter().zip(&env.alive).filter(|(p, &a)| a &&
            -81.67 < p[0] && p[0] < -42.27 && -57.89 < p[2] && p[2] < -20.53).count();
        assert!(in_box5 > 10, "rainbox5's strip holds only {in_box5} drops — the outside rain died");
    }
    // And at the spawn: rainbox1 [D] x −6.61..5.98, z 25.0..30.1 over the courtyard.
    let mut scene2 = nf_view::load(&path, "0700000a.bin", false).expect("reload");
    let sim2 = scene2.sim.as_mut().unwrap();
    for _ in 0..120 { sim2.step_bond(input, false, false, false); }
    let env2 = sim2.drones.env.as_ref().unwrap();
    let in_box1 = env2.drops.iter().zip(&env2.alive).filter(|(p, &a)| a &&
        -6.61 < p[0] && p[0] < 5.98 && 25.0 < p[2] && p[2] < 30.1).count();
    assert!(in_box1 > 10, "rainbox1's courtyard strip holds only {in_box1} drops at the spawn");
}
