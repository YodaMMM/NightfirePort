//! The driving view's traffic cars run the AI land pass (FUN_0018ecb8 / FUN_0018e548 [M]) on
//! their own model hull and the track's 2-D wall list (FUN_001fb318's land branch [M]); see
//! docs/systems/drive-physics.md §12. Over a minute of play they stay upright and on the ground.

#[test]
fn traffic_cars_stay_upright_and_above_the_road_for_a_minute() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    for viv in ["MIS01", "MIS4"] {
        let (_, mut d) = nf_view::driving::load_driving_scene(&iso, viv).unwrap_or_else(|e| panic!("load {viv}: {e}"));
        if d.ai_cars.is_empty() { eprintln!("{viv}: no traffic cars spawned"); continue; }
        for (drv, _) in &d.ai_cars {
            assert_eq!(drv.car.kind, nf_game::driving::physics::DriverKind::Traffic, "{viv}: traffic driver kind [M FUN_0013fae0 passes 3]");
            assert!(!drv.car.params.hull.is_empty(), "{viv}: the traffic car carries its model's hull points [D]");
        }
        let start: Vec<[f32; 3]> = d.ai_cars.iter().map(|(c, _)| c.car.pos).collect();
        for _ in 0..3600 { d.step(0.0, 0.0, 0.0, false); }
        for (i, (drv, _)) in d.ai_cars.iter().enumerate() {
            let c = &drv.car;
            assert!(c.rot[1][1] > 0.5, "{viv} car {i}: upright after 60 s (up.y {:.2}) at {:?}", c.rot[1][1], c.pos);
            let moved = ((c.pos[0] - start[i][0]).powi(2) + (c.pos[2] - start[i][2]).powi(2)).sqrt();
            assert!(moved > 20.0, "{viv} car {i}: drives ({moved:.1} u in 60 s)");
            if let Some(land) = &d.land {
                use nf_game::driving::physics::Ground;
                if let Some(pl) = land.wheel_plane([c.pos[0], c.pos[1] + 2.0, c.pos[2]]) {
                    // The plane's height under the car (n·(p − v0) = 0 solved for y).
                    let n = pl.normal;
                    if n[1].abs() > 0.3 {
                        let gy = pl.v0[1] - (n[0] * (c.pos[0] - pl.v0[0]) + n[2] * (c.pos[2] - pl.v0[2])) / n[1];
                        assert!(c.pos[1] > gy - 0.2, "{viv} car {i}: body centre {:.2} not below the road {gy:.2}", c.pos[1]);
                    }
                }
            }
        }
    }
}
