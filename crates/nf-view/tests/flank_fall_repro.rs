//! level-03-fall-through-floor — the courtyard-flank fall, reproduced on real input.
//!
//! The symptom (the task): on 07000003, while a walk flanked servant 1's guard in the courtyard,
//! Bond fell through the world (y reached −5577) [origin/s/stealth-los-2's first suite run — that
//! run executed the pre-00310262 route blob, which is byte-identical to master's file today:
//! blob 61bb92a2 = 00310262^ = origin/master]. The flank is the single nav leg
//! `walk_nav_to([13.0, 5.9, −18.0], "back across the courtyard", 1.6)` from the gate-plate lawn
//! (11, 5.9, −35); the later commits pin the problem area to the wall at (6.35..6.4,
//! −15.7..−17.7) ("Bond climbs to y 7.3, airborne" — c118ed46; "parks Bond west of a wall at
//! (6.4, −17.7)" — 00310262).
//!
//! This test walks master's own legs to the gate-plate lawn, then the flank leg node by node
//! (the same `leg()` chain walk_nav_to walks), checking after each node whether a fall opened
//! (Walker records a fall when Bond drops > 4 u from his last grounded y [walker.rs tick_inner]),
//! so the gap localizes to one nav segment. The end assert is the pin: Bond walking that line
//! stays on the floor.

#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// FIXED: the airborne discharge no longer happens, because the push no longer waits for
/// airborne. The port's Jointy push now runs EVERY tick [M Collide_Jointy 0x1ec180:229-281 — no
/// ground gate], per bot gated by the game's own hittest arming [M NDrone2_Collision 0x140e90 +
/// NDrone2_DoCollision 0x149ec8: kind 0x13 out, kind 0xc always in, else +0x538 / near-2.0 /
/// obj+0xFC / +0x570&2; the dead (+0x4f8 & 0x200) arm a bone capsule unconditionally], so the
/// overlap discharges the tick it opens (a fraction of a unit per tick, like the game) and Bond
/// walks the wall's end-edge past the two guards without being carried across the plane. The
/// older notes: on master the walk rode the gatehouse wall's south end-edge (the edge contact
/// lifts him ~0.04 u/tick, t27140-27142) until the feet band missed; on that first airborne tick
/// the old airborne-only push discharged the whole overlap he walked into while grounded (two
/// guards, ~0.74 u in one tick, t27142→27143) and carried the spine across the wall plane
/// x 10.915 (z −33.3..−30.3).
#[test]
fn the_old_courtyard_flank_stays_on_the_floor() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    // Master's own legs up to the flank's start (blob 61bb92a2): the courtyard crossing, then the
    // west-lane detour to the gate-plate lawn.
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("mid courtyard", [13.0, 5.9, -18.0]),
        ("west lawn", [0.0f32, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("along the moat", [8.0, 5.9, -36.0]),
        ("gate plate lawn", [11.0, 5.9, -35.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    assert!(w.report.falls.is_empty(), "no fall before the flank: {:?}", w.report.falls);

    // The flank leg, node by node (walk_nav_to's own structure: the `leg()` chain, each node a
    // 1.3-arrival waypoint, wedged nodes reported). A fall opening on a node localizes the gap.
    let pts = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::leg(sim, [11.0, 5.9, -35.0], [13.0, 5.9, -18.0])
    };
    eprintln!("flank: {} nav nodes from the gate-plate lawn to mid courtyard", pts.len() - 1);
    for (k, p) in pts.iter().enumerate().skip(1) {
        let before = w.report.falls.len();
        let y_before = w.scene.sim.as_ref().unwrap().bond.pos[1];
        let r = {
            w.lenient = true;
            let r = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("flank nav {k}"));
            w.lenient = false;
            r
        };
        let p_now = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("flank nav {k} {:?} -> r {r} bond ({:.2},{:.2},{:.2}) falls {}",
            p, p_now[0], p_now[1], p_now[2], w.report.falls.len() - before);
        if w.report.falls.len() > before {
            panic!(
                "the flank leg opens a fall on nav node {k} (from {y_before:.2} to {:.2}, node {p:?}): {:?}",
                p_now[1], w.report.falls
            );
        }
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("flank done at ({:.2},{:.2},{:.2}); falls {:?}", p[0], p[1], p[2], w.report.falls);
    assert!(w.report.falls.is_empty(), "Bond walking the old flank must stay on the floor: {:?}", w.report.falls);
}

/// Sister seam probe: pressed against the courtyard's west lip (walk west from mid courtyard on
/// real input), Bond must not HANG. The game's ground flag is the capsule pick bit OR the ray band
/// [M Player_FeetOnPoint 0x194188:176-185]. An earlier tree hung ungrounded at (12.61, 6.50,
/// −19.69) — held by the lip push while the band missed by 0.025 u (floor 1.16 below the root;
/// cc+0.1 = 1.1347); on master's `walkable || band` rule the press stays grounded at the lip every
/// tick. The pin bounds the UNGROUNDED RUNS instead of every tick: the courtyard guard idles at
/// (12.74, −19.23) [D 07000003 placement, the walker's own log], the press line passes inside the
/// Jointy reach (r 0.4 + 0.55 < 2.0 band, closest-segment distance 0.65), and the game's
/// actor-actor push has no ground preservation [M Collide_Jointy 0x1ec180:229-281 — geometry
/// only] — brushing him lifts Bond for a few ticks (measured: 5, t13090-13095, apex y 6.49), so
/// `ungrounded == 0` contradicted the decompile and was relaxed to the hang signature: no
/// ungrounded run over 20 ticks (the old hang was unbounded), the press still reaches the lip,
/// and Bond lands back on the floor.
#[test]
fn pressed_against_the_west_lip_bond_stays_grounded() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    w.walk_nav_to([13.0f32, 5.9, -18.0], "mid courtyard", 1.6);
    // Press west into the lip: real input, forward held, sampling grounded every tick.
    let start = w.total_ticks;
    let mut ungrounded = 0u32;
    let (mut run, mut longest) = (0u32, 0u32);
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if p[0] < 12.0 || w.total_ticks - start > 420 { break; }
        let desired = (12.0f32 - p[0]).atan2(-18.0 - p[2]);
        let err = desired - sim.bond.yaw;
        let err = err - (err / std::f32::consts::PI).round() * std::f32::consts::PI;
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if !sim.bond.grounded {
            ungrounded += 1;
            run += 1;
            longest = longest.max(run);
            eprintln!("  ungrounded t{} bond ({:.2},{:.2},{:.2}) ny {:.2}",
                w.total_ticks, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.floor_ny);
        } else {
            run = 0;
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    eprintln!("lip press ended at ({:.2},{:.2},{:.2}) ungrounded ticks {ungrounded} longest run {longest}", p[0], p[1], p[2]);
    assert!(p[0] < 12.8, "the press must reach the lip (west of x 12.8), got {p:?}");
    assert!(longest <= 20, "pressed against the lip Bond must not hang ungrounded (the old band hang was unbounded [M 0x194188:176-185]), longest run {longest}");
    assert!(sim.bond.grounded || p[1] < 6.2, "the press must not end held airborne at the lip height (the old hang sat at y 6.50), got {p:?}");
}
