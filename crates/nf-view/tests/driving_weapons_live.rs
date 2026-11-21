//! The driving view's weapons: the default loadout (FUN_0020f2b8 table, Rocket selected), a fire
//! press spends the selected weapon's ammo through the game's firing rules (FUN_0020de20) and
//! launches projectiles from the car's nose; Q cycles the selection; the HUD reads the
//! selected weapon and its ammo. Opt-in: NF_ISO.

#[test]
fn a_fire_press_spends_a_rocket_and_launches_it() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (_, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    for _ in 0..60 { d.step(0.0, 0.0, 0.0, false); }
    let w = d.weapons.as_ref().expect("weapons");
    let sel = w.sys.secondary as usize;
    let before = w.sys.slots[sel].ammo;
    d.weapons.as_mut().unwrap().fire = true;
    d.step(0.0, 0.0, 0.0, false);
    let w = d.weapons.as_ref().unwrap();
    assert!(w.sys.slots[sel].ammo < before, "ammo {before} -> {}", w.sys.slots[sel].ammo);
    assert!(!w.shots.is_empty(), "a projectile in flight");
    let p0 = w.shots[0].pos;
    for _ in 0..10 { d.step(0.0, 0.0, 0.0, false); }
    if let Some(s) = d.weapons.as_ref().unwrap().shots.first() {
        let m = ((s.pos[0] - p0[0]).powi(2) + (s.pos[2] - p0[2]).powi(2)).sqrt();
        assert!(m > 1.0, "the shot flies: {m}");
    }
    for _ in 0..30 { d.step(0.0, 0.0, 0.0, false); }
    d.weapons.as_mut().unwrap().cycle = true;
    d.step(0.0, 0.0, 0.0, false);
    assert_ne!(d.weapons.as_ref().unwrap().sys.secondary as usize, sel, "Q cycles the weapon");
}
