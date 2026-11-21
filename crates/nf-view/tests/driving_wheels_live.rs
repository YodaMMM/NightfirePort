//! The player car's wheels are posed each tick by the game's wheel rig (FUN_001d2cb0 via
//! nf_formats::carpose; docs/formats/car-model.md §11): rolling the car turns its wheel parts'
//! palette matrices, while the body's own parts stay baked. Before, every part was baked at rest.
//! Opt-in: NF_ISO.

#[test]
fn the_vanquish_wheels_roll_as_the_car_drives() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (_, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let rig = d.car_rig.as_ref().expect("the vanquish's wheel rig");
    assert!(rig.parts.len() >= 4, "the four wheels (and hubs) are posed parts: {:?}", rig.parts);
    let n = rig.parts.iter().map(|&(_, s)| s as usize).max().unwrap() + 1;
    for _ in 0..90 { d.step(1.0, 0.0, 0.0, false); }
    let grab = |d: &nf_view::driving::DrivingState| { let mut p = vec![[[0.0f32; 4]; 4]; n]; d.apply_to_palette(0, &mut p); p };
    let a = grab(&d);
    d.step(1.0, 0.0, 0.0, false);
    let b = grab(&d);
    let rel = |p: &Vec<[[f32; 4]; 4]>, s: u32| { let m = p[s as usize]; [m[0][0], m[0][1], m[0][2], m[1][0], m[1][1], m[1][2]] };
    let turned = d.car_rig.as_ref().unwrap().parts.iter().filter(|&&(_, s)| {
        let (x, y) = (rel(&a, s), rel(&b, s));
        // the body's own rotation also changes a little; a rolling wheel's axes change more
        x.iter().zip(y).map(|(p, q)| (p - q).abs()).fold(0.0, f32::max) > 0.01
    }).count();
    assert!(turned >= 2, "rolling at speed {:?} turns the wheel matrices ({turned} parts changed)", d.car.vel);
}
