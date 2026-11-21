//! The driving view runs the scripted control layer (nf_game::driving::intro from the mission
//! script's events): on RACE the car is held through the countdown (GO and release at frame 243,
//! docs/systems/driving-intros.md) and moves once released; MIS4 is driven from the first frame.
//! Opt-in: NF_ISO.

fn travelled(viv: &str, ticks: u32) -> f32 {
    let iso = std::env::var("NF_ISO").unwrap();
    let (_, mut d) = nf_view::driving::load_driving_scene(&iso, viv).expect("load");
    for _ in 0..60 { d.step(0.0, 0.0, 0.0, false); } // settle on the springs
    let p0 = d.car.pos;
    for _ in 0..ticks { d.step(1.0, 0.0, 0.0, false); }
    let p = d.car.pos;
    ((p[0] - p0[0]).powi(2) + (p[2] - p0[2]).powi(2)).sqrt()
}

#[test]
fn the_race_countdown_holds_the_car_and_mis4_drives_at_once() {
    if std::env::var("NF_ISO").is_err() { eprintln!("NF_ISO not set — skipping"); return }
    let race_held = travelled("RACE", 120);
    assert!(race_held < 0.5, "RACE: frames 60..180 are inside the countdown, the car stays: moved {race_held}");
    let race_go = travelled("RACE", 400);
    assert!(race_go > 5.0, "RACE: released after GO: moved {race_go}");
    let mis4 = travelled("MIS4", 120);
    assert!(mis4 > 5.0, "MIS4: the player drives from the first frame: moved {mis4}");
}
