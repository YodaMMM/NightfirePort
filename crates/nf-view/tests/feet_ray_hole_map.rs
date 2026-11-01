//! level-03-fall-through-floor — the down-ray hole map (the feet-ray path, no walking).
//!
//! The wall probe showed Bond hovering at (12.61, 6.50, −19.69) with grounded=false on real
//! input (walking west from mid courtyard), and the raw-tri floor map showed cells with no
//! floor in y 0..9 west of the courtyard. This probe casts the game's own feet ray
//! (`segment_hit_scoped`, the port of Intersect_RayGeom under Player_FeetOnPoint 0x194188)
//! straight down on a 0.5 grid over the courtyard's west edge and reports where it finds
//! NOTHING — the through-world holes a walker falls into.

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn map_the_feet_ray_holes_west_of_the_courtyard() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    let cel = sim.bond.cel;
    eprintln!("bond at {:?} cel {:?}", sim.bond.pos, sim.bond.cel);

    // Grid x 2..14, z −24..−10, ray from y 8 down to y −120 (through everything).
    // FLAT rays: the honest geometry map — a scoped ray only sees Bond's own cel set, which a
    // static grid does not model. X here = no floor face at all under the column.
    eprintln!("--- flat feet-ray map (x 2..14 step 0.5, z −24..−10 step 0.5; from y 8 down to −120) ---");
    eprintln!("     (digit = floor y*10 mod 10; X = the ray finds NOTHING; d = floor only below y −20)");
    let mut z = -24.0f32;
    while z <= -10.0 {
        let mut row = format!("z {z:+06.1} ");
        let mut x = 2.0f32;
        while x <= 14.0 {
            let from = [x, 8.0, z];
            let to = [x, -120.0, z];
            let hit = sim.world.segment_hit(from, to);
            row.push(match &hit {
                None => 'X',                                // the ray finds NOTHING: a hole
                Some((t, _)) => {
                    let y = 8.0 + t * (-128.0);
                    if y < -20.0 { 'd' }                    // floor only far below (deep shaft)
                    else { char::from_digit((y * 10.0) as u32 % 10, 10).unwrap_or('?') }
                }
            });
            x += 0.5;
        }
        eprintln!("{row}");
        z += 0.5;
    }
    // The hover spot and the wall spot, exact rays + the flat (unscoped) comparison.
    for (label, at) in [
        ("hover spot", [12.61f32, 8.0, -19.69]),
        ("wall spot", [6.4, 8.0, -17.7]),
        ("wall north", [6.4, 8.0, -15.7]),
        ("mid courtyard", [13.0, 8.0, -18.0]),
    ] {
        let to = [at[0], -120.0, at[2]];
        let scoped = sim.world.segment_hit_scoped(cel, at, to);
        let flat = sim.world.segment_hit(at, to);
        eprintln!("{label} at ({:.2},{:.2}): scoped {:?} flat {:?}", at[0], at[2],
            scoped.map(|(t, _)| 8.0 + t * -128.0), flat.map(|(t, _)| 8.0 + t * -128.0));
    }
}
