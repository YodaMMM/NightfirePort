//! The SUB_PHYSICS player vehicles in the driving view: MIS11's vanquishsub and MIS13B's
//! ultralight (the mission set's spawn vehicle [D]) run the game's sub pass with the hull against
//! the track (nf_game::driving::sub_physics / sub_collide) instead of the flat stand-in, and
//! the car mirrors the body. Opt-in: NF_ISO.

#[test]
fn the_sub_and_the_ultralight_fly_their_own_pass() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    for viv in ["MIS11", "MIS13B"] {
        let (_, mut d) = nf_view::driving::load_driving_scene(&iso, viv).expect("load");
        assert!(d.sub.is_some(), "{viv}: the SUB_PHYSICS spawn vehicle");
        assert!(d.land.is_none(), "{viv}: not the land pass");
        let p0 = d.car.pos;
        for t in 0..600 { d.step(1.0, 0.0, if t > 300 { 0.5 } else { 0.0 }, false); }
        let p = d.car.pos;
        let moved = ((p[0] - p0[0]).powi(2) + (p[2] - p0[2]).powi(2)).sqrt();
        assert!(p.iter().all(|x| x.is_finite()), "{viv}: {p:?}");
        assert!(moved > 20.0, "{viv}: 10 s of throttle moves the craft: {moved}");
        let b = d.sub.as_ref().unwrap().0.body.pos;
        assert_eq!(b, d.car.pos, "{viv}: the car mirrors the body");
    }
}
