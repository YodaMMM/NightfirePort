//! The 07000001 start guard's first rounds vs the contact — the game-parity pin.
//! Opt-in via NF_ISO. Real input only (Bond idles with zero input after the intro; no teleports).
//!
//! Task `level-01-start-guard-contact`. At the load the contact (Mayhew, skin 0x4F, team 2) stands
//! 0.04 u off the line from the start-room guard #0 (placement #158) to Bond [L
//! docs/levels/07000001.md:28-31 cross-product against the spawn], and a team-1 guard's round that
//! meets a team-2/3 body lands on it [M NDrone2_DealWithObjHit 0x13fff0:181-232]. The question this
//! pin holds: does the port's guard ever fire a round down that line while the contact still stands
//! on it — a hit the game's own round would also be committed to?
//!
//! The game's answer, from its own recording [L tests/traces/01-doors.trace f+0..f+300, intro
//! skipped at f+0]: the contact sits at his spawn only until f+75 (state 28 = 0x1c AllyLeadInit
//! [L DroneTypeSettings 0x29aee8 mode 12 start]), walks west through states 32/29/40, and reaches
//! the hide spot (−10.76, −6.73) at f+137-150 (state 31 = 0x1f AllyLeadHide, cower anim 0x19
//! [M NDrone2_DSTATE_AllyLeadHide 0x15ec08 case 1 — in place, no teleport]) — while the guard is
//! still in his start script (state 3, the alertness-1.0 hold [M DroneVision_EnemyLookForOpponent
//! 0x177ed0:12-14]). The guard's script ends ~f+150 (state 68) and his first shot fires at f+289
//! (state 71) — about 150 ticks AFTER the contact reached the hide spot. The contact's hide path
//! does cross the moved guard→Bond line (~f+100-137), so the game's own margin, not a port rule,
//! is what keeps the crossing clear.
//!
//! The port must show the same order: the contact reaches the game's hide spot before the guard's
//! first shot, and an idle Bond at the start spot takes a 900-tick engagement with the contact
//! unhurt. The guard's fire model itself is pinned elsewhere (the wobble while
//! +0x270 < NewSighting_TimeToHit·60, dead-on at +0x1f0 on a hit roll, the yaw-only wobble
//! rotation [M DroneWeap_DoBulletAccuracy 0x179740:103-134; Drone_GetOpponentInfo 0x138f68:70-75];
//! the cross-side landing [M 0x13fff0:181-232] — ally_bullets_live).
//!
//! Run: NF_ISO=… cargo test -p nf-view --test first_rounds_contact_probe -- --nocapture

use nf_game::player::Input;
use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Perpendicular distance from p to the ray (from, dir), and the ray param at the foot.
fn seg_point_dist(from: [f32; 3], dir: [f32; 3], p: [f32; 3]) -> (f32, f32) {
    let d = [dir[0], dir[1], dir[2]];
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let (dx, dy, dz) = (d[0] / dl, d[1] / dl, d[2] / dl);
    let w = [p[0] - from[0], p[1] - from[1], p[2] - from[2]];
    let t = w[0] * dx + w[1] * dy + w[2] * dz;
    let c = [w[0] - dx * t, w[1] - dy * t, w[2] - dz * t];
    ((c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt(), t)
}

#[test]
fn the_contact_reaches_the_games_hide_spot_before_the_guards_first_shot() {
    let mut scene: Scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    nf_view::settle_start_draw(scene.sim.as_mut().unwrap());
    // The intro: wait for ch 0xB (the contact's enable channel), then the script tail.
    let mut b_at = None;
    for t in 0..4000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0xb) { b_at = Some(t); break; }
    }
    b_at.expect("intro must write ch 0xB");
    for _ in 0..6000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    let (gi, ci) = {
        let sim = scene.sim.as_ref().unwrap();
        let gi = sim.drones.list.iter().position(|d| d.placement == 158).expect("guard #0 (placement 158)");
        let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
        (gi, ci)
    };
    {
        let sim = scene.sim.as_ref().unwrap();
        let (g, c) = (&sim.drones.list[gi], &sim.drones.list[ci]);
        assert_eq!(g.team, 1, "guard #0 is team 1");
        assert_eq!(c.team, 2, "the contact is team 2 [M dtype 0xc -> team 2]");
        assert_eq!(c.health, 10.0, "the contact's 10 health at the start [L ee_start_07000001]");
        eprintln!("[pin] contact at {:.3?} state {} — guard #0 at {:.3?} state {}", c.pos, c.state, g.pos, g.state);
    }
    // Idle 900 ticks — the game's own first-shot window (the trace's first shot at f+289 with the
    // intro skipped; the port's settled stream fires in the same window). Track when the guard's
    // first shot happens relative to the contact's arrival at the hide spot.
    let mut first_shot_tick: Option<u32> = None;
    let mut hide_arrival_tick: Option<u32> = None;
    let mut first_shot_seen = 0u32;
    let mut first_shot_perp = 0.0f32;
    let mut last_aims = 0usize;
    let mut last_round_hits = 0usize;
    let mut bond_took_hit = false;
    for tick in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[gi];
        let c = &sim.drones.list[ci];
        if hide_arrival_tick.is_none()
            && (c.pos[0] - -10.76).abs() < 0.25 && (c.pos[2] - -6.73).abs() < 0.35 {
            hide_arrival_tick = Some(tick);
            eprintln!("[pin] tick {tick}: contact reached the game's hide spot {:.3?} (state {})", c.pos, c.state);
        }
        if g.shot_aims.len() != last_aims {
            for i in last_aims..g.shot_aims.len() {
                let (f, m, a) = g.shot_aims[i];
                let (perp, _t) = seg_point_dist(m, [a[0] - m[0], a[1] - m[1], a[2] - m[2]], c.pos);
                eprintln!("[pin] tick {tick} shot {} frame {f} seen {} dist {:.2} muzzle {:.3?} aim {:.3?} | line->contact perp {perp:.3}",
                    i, g.seen_ticks, g.dist, m, a);
                if first_shot_tick.is_none() {
                    first_shot_tick = Some(tick);
                    first_shot_seen = g.seen_ticks;
                    first_shot_perp = perp;
                }
            }
            last_aims = g.shot_aims.len();
        }
        if sim.drones.drone_round_hits.len() != last_round_hits {
            for h in &sim.drones.drone_round_hits[last_round_hits..] {
                eprintln!("[pin] tick {tick} ROUND HIT frame {} shooter {} victim {}", h.0, h.1, h.2);
                if h.2 == ci { panic!("the guard's round landed on the contact at tick {tick} — the game's trace gives the contact the hide spot before the first shot; investigate before touching this pin"); }
            }
            last_round_hits = sim.drones.drone_round_hits.len();
        }
        if sim.drones.health.health < 100.0 { bond_took_hit = true; }
        if tick % 30 == 0 { eprintln!("[pin] t{tick}: contact ({:.2},{:.2}) st{}, guard ({:.2},{:.2}) st{}", c.pos[0], c.pos[2], c.state, g.pos[0], g.pos[2], g.state); }
    }
    let sim = scene.sim.as_ref().unwrap();
    let (g, c) = (&sim.drones.list[gi], &sim.drones.list[ci]);
    let fst = first_shot_tick.expect("the start guard engages an idle Bond within the window");
    let hat = hide_arrival_tick.expect("the contact must reach the game's hide spot (−10.76,−6.73) [L 01-doors.trace f+137-150]");
    eprintln!("[pin] END hide at tick {hat}, first shot at tick {fst} (seen {first_shot_seen}, line->contact perp {first_shot_perp:.3}), contact {:.3?} health {:.1} hits {}, state {} anim {}",
        c.pos, c.health, c.hits_taken, c.state, c.anim.state);
    // The game's order [L 01-doors.trace]: the contact hides (f+137-150) before the guard's first
    // shot (f+289). The port must keep that order — it is the game's own margin, not luck.
    assert!(hat < fst, "the contact must reach the hide spot (tick {hat}) before the guard's first shot (tick {fst}) [L 01-doors.trace f+137-150 vs f+289]");
    assert_eq!(c.hits_taken, 0, "an idle-Bond engagement must not land a round on the contact — the game's contact is at the hide spot when the guard fires");
    assert!(c.health > 0.0, "the contact survives the start window (mission-fail 0x4000034 otherwise)");
    assert!(c.pos[0] < -10.2, "the contact ends cowering at the game's hide spot, got {:.3?}", c.pos);
    assert!(bond_took_hit || first_shot_perp > 0.3,
        "the engagement is live (Bond took {bond_took_hit:?}) — otherwise this pin proves nothing");
    let _ = &g.shot_rolls; // the roll sequence is stream-dependent; the order above is the invariant
}
