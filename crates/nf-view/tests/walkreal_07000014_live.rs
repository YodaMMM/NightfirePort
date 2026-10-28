//! walkreal-b: 07000014 (Drake's underground facility → Delta Sector) walked with the player's
//! own movement — the infiltration, the sabotage and the kill are the REAL interaction paths; the
//! only teleports are the reach-a-spot hops into the cut-scene bands the walk cannot enter
//! (documented per leg). Route (level-14-full-walk; the east route + this file's [D] tags):
//! the vent start #666/#667 (40.81, 2.40, 65.08 [D]) → the SkyBridge Q-Claw crossing (the
//! GRAPPLEPOINT_SkyBridge #658 (45.29, 5.10, 62.94) [D]; M Bullet_Delete 0x12e660:50-55 +
//! Player_SSGrapple 0x192860) → the open walkway gap drop → the trench floor (-0.60, nav lane
//! 5→4 [D]) → the corridor east → south → the pit-band jump west onto the -0.91 landing (the
//! east stairwell foot) → back east → the z-51 WEST pit jump onto the 0.60 stairwell-corridor
//! segment (x 44.4..46.4, z 50.5..51.25 [D probe14_shaft2]) → THE SHAFT ENTRY: the Q-Claw reel
//! to the mouths' grapple #626 (45.30, 4.79, 42.86) [D] — the bolt ray skims the lip ramp
//! (z 48.0..48.6, top 2.01..2.40 [D probe14_shaft4]) and the 1.0 sweep latches [M 0x12c5a0:92] —
//! the hang over the east vent lane (2.40), the release lands on it → the lane south over its
//! flush lid (z 42.16 [D #655/#600]) onto the mezzanine → west across the sloped connector
//! (z 39.5..38.25, 2.15..2.40 [D probe14_mouthfin]) → the vent drop TouchOnce #301 (39.68…41.75,
//! 2.07…4.14, 37.62…40.40 [D]) → the SP #276 cut-scene (0x06000833, 1400 ticks) writes ch 0x47
//! (objective 1) → the REAL walk west through the facility to the Kiko cut-scene touch #381 (the
//! SOUTH loop — the closed 0x85 door pair seals its doorway, so the west wing is reached the
//! game's own way through the server room's west corridor [D nav nodes 94-97/150/591-600]) →
//! SP #382 (0x0600062F) writes ch 0x48 (objective 2) → the west generator room via the (−58.8,
//! 31.65) passage and the ramp down (nodes 193/623/628/631) → the two fuseboxes cut through the
//! REAL Laser path ([M FuseBox_Update 0x1adfe8]; #383 p1 0x87, #385 p1 0x88 [D]) → AND #391
//! raises 0x8C → back east up the north ramp to the E-use switch oDroneTrigger1b (−60.30, 0.61,
//! 38.37; touch out 0xA0, mask 4 [D placement 399; M Trigger_Activate 0x1b2318]) → SEQ_AND 0x51 →
//! the box/generator cascade → 0x85 → Door_Slide_11A/B unlock and auto-open [M Door_Update
//! 0x1363c8:88-107] → east THROUGH the opened doorway to the server-room escape touch #456 →
//! SEQ_AND #445 → SP #407 (0x06000848) writes ch 0x4A (objective 3) → east to Rook's wake touch
//! #476 → SEQ_AND #475 wakes Rook (skin 0x05000066 [D]) → the REAL gun path kills him → his
//! death channel 0x54 fires [D drone param 12; M set_death_channel 0x13e660] (objective 4) →
//! the REAL walk to the exit #944 (27.79, 1.28, −25.32 [D]) → E → the fallback arm writes
//! ch 0xCE (objective 5) and loads 07000015 [M 0x1b2318].
//!
//! Enemies are DISABLED for the walk (the walker tops Bond's health every tick; the pain pipeline
//! still runs) except Rook, who must die. The fuseboxes' live wires carry param 2 non-zero on
//! every box here → the cut NEVER hurts [M 0x1adfe8:99-101; D placements census].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The wire object's VISIBLE geometry (copied from walk_07000014_live's helpers): its local
/// bounds (`Drones::wire_geo`) through the live entity matrix — the same world slab the bullet
/// pass tests the round against (`Collide_PickObj` 0x1eae80 → obj+0xD0 → `FuseBox_Update`
/// 0x1adfe8:88-123). The entity translation sits 1.5 m BELOW the cables [D
/// OBJECT_GreenWire1/BlueWire1 local y 1.49..1.72 through the entity matrix].
fn wire_world_box(sim: &nf_view::scene::Sim, obj: usize) -> Option<([f32; 3], [f32; 3])> {
    match sim.drones.scripts.objects.list.get(obj)? {
        Some(nf_game::nis::Obj::Entity { hash, matrix, .. }) => {
            let (lo, hi) = sim.drones.wire_geo.get(hash)?;
            let (mut wlo, mut whi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for cx in 0..2 { for cy in 0..2 { for cz in 0..2 {
                let p = [if cx == 0 { lo[0] } else { hi[0] }, if cy == 0 { lo[1] } else { hi[1] }, if cz == 0 { lo[2] } else { hi[2] }];
                let w = nf_formats::placement::transform(matrix, p);
                for c in 0..3 { wlo[c] = wlo[c].min(w[c]); whi[c] = whi[c].max(w[c]); }
            } } }
            Some((wlo, whi))
        }
        _ => None,
    }
}

/// Where a player aims to burn the wire: the VISIBLE cable's centre, else the translation.
fn wire_aim_point(sim: &nf_view::scene::Sim, obj: usize) -> [f32; 3] {
    if let Some((lo, hi)) = wire_world_box(sim, obj) {
        return [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    }
    nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), obj).unwrap_or([0.0; 3])
}

/// Walk to a stance inside the Laser's range of a fusebox wire and burn it through the REAL fire
/// path. E does nothing on fuseboxes — `Player_Activate` 0x1a5170 has no class-0x3C arm [M
/// 0x1a5170:95-160] — the game's cut is the Laser beam (weapon 0x4E, range 4.0 [L weapon_data
/// 0x4E +0x1C]) into the wire sub-object; `FuseBox_Update` 0x1adfe8 writes the group's channel
/// words on the wire death. The aim walks the crosshair onto the VISIBLE cable the way a player
/// does: each held-fire tick reads the spawned Laser round's miss against the wire's own world
/// slab and shifts the aim (the walk_07000014 burn_wire rule, through the walk's turn/look
/// inputs instead of a teleport-aim).
fn cut_fusebox(w: &mut Walker, box_pos: [f32; 3], want: u8, label: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    let fb = sim.drones.fuseboxes.iter().position(|f| f.ch[0] as u32 == want as u32)
        .unwrap_or_else(|| panic!("{label}: the fusebox with group-0 channel {want:#x} exists [D]"));
    let (bp, wire, wire_obj) = {
        let sim = w.scene.sim.as_ref().unwrap();
        // The group-0 wire (the p1 word's group [D placements 383/385 p1]); fall back to the
        // first alive wire if the group-0 slot is empty.
        let obj = sim.drones.fuseboxes[fb].alive_wires().find(|(g, _, _)| *g == 0)
            .or_else(|| sim.drones.fuseboxes[fb].alive_wires().next())
            .map(|(_, _, o)| o)
            .expect("the script registered wires (groups 0 and 1 [L])");
        (sim.drones.fuseboxes[fb].pos, wire_aim_point(sim, obj), obj)
    };
    let cands = [[0.6f32, -1.2, 0.0], [-0.6, -1.2, 0.0], [0.0, -1.2, 0.6], [0.0, -1.2, -0.6]];
    let mut stood = false;
    for c in &cands {
        let spot = [bp[0] + c[0], bp[1] + c[1], bp[2] + c[2]];
        let floor = w.scene.sim.as_ref().unwrap().world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
        if !matches!(floor, Some(f) if (f - (bp[1] - 2.4)).abs() < 3.0) { continue; }
        w.walk_to(Wp::new(spot[0], spot[1], spot[2]).r(0.6), &format!("{label} stance"));
        let head = w.scene.sim.as_ref().unwrap().bond.head();
        let d = [wire[0] - head[0], wire[1] - head[1], wire[2] - head[2]];
        if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 3.4 { stood = true; break; }
    }
    assert!(stood, "{label}: a stance inside the Laser's range exists [L weapon_data 0x4E +0x1C]");
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false); }
    let start = w.total_ticks;
    let mut delta = [0.0f32; 3];
    loop {
        let (n0, want_on) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.player_bullets.len(), sim.drones.channels.on(want as u32))
        };
        if want_on { break; }
        assert!(w.total_ticks - start < 600, "{label}: the laser cut never wrote ch {want:#x} [M 0x1adfe8]");
        // Steer the crosshair at the (feedback-shifted) visible-cable point and hold fire.
        let (yaw, pitch) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let head = sim.bond.head();
            let t = [wire[0] + delta[0], wire[1] + delta[1], wire[2] + delta[2]];
            let dir = [t[0] - head[0], t[1] - head[1], t[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            (dir[0].atan2(dir[2]), ((dir[1] / l).atan() / std::f32::consts::FRAC_PI_2))
        };
        w.tick_fire(nf_game::player::Input {
            turn: (yaw - w.scene.sim.as_ref().unwrap().bond.yaw).clamp(-0.08, 0.08),
            look: (pitch - w.scene.sim.as_ref().unwrap().bond.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        });
        // The round's miss against the wire's world slab shifts the aim (burn_wire's rule).
        let round = w.scene.sim.as_ref().unwrap().drones.player_bullets.get(n0)
            .filter(|b| b.weapon == 0x4e).map(|b| (b.pos, b.dir, b.speed));
        if let Some((pos, dir, speed)) = round {
            let centre = w.scene.sim.as_ref().unwrap();
            if let Some((lo, hi)) = wire_world_box(centre, wire_obj) {
                let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
                let e = [pos[0] + dir[0] * speed, pos[1] + dir[1] * speed, pos[2] + dir[2] * speed];
                let ab = [e[0] - pos[0], e[1] - pos[1], e[2] - pos[2]];
                let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                let t = if ab2 > 1e-9 { (((c[0] - pos[0]) * ab[0] + (c[1] - pos[1]) * ab[1] + (c[2] - pos[2]) * ab[2]) / ab2).clamp(0.0, 1.0) } else { 0.0 };
                let closest = [pos[0] + ab[0] * t, pos[1] + ab[1] * t, pos[2] + ab[2] * t];
                for k in 0..3 { delta[k] = (delta[k] + (c[k] - closest[k])).clamp(-0.8, 0.8); }
            }
        }
    }
    eprintln!("[014] {label}: ch {want:#x} on at +{} ticks (laser cut)", w.total_ticks - start);
}

/// The crouch toggle the walker lacks (Act::CrouchOn is unreachable in walk_to): set the field
/// the way the game's crouch hold does and give the stance a tick to settle.
fn set_crouch(w: &mut Walker, on: bool) {
    w.scene.sim.as_mut().unwrap().bond.crouched = on;
    for _ in 0..15 { w.tick(nf_game::player::Input::default(), false); }
}

/// The east route, REWRITTEN (level-14-east-walkway) — the old drop-leg premise (render-only slot
/// lids) is refuted and the route now walks the game's own crossing:
///
/// - The lid slabs are SOLID [M Script_EntityStart 0x1d8d78 links every created entity into the cel
///   chain; build_LinkToRoom 0x1c9680 — the lid-and-climb report's §solidity], but they never block
///   THIS route: the vent room is TWO PARALLEL WALKWAYS with a 3 m OPEN gap between them (x
///   41.54..44.54 at floor band z 59..67 — the trench floor -0.60 beneath [D probe14_room]) and the
///   crossing is the placed grapple point "GRAPPLEPOINT_SkyBridge" (45.29, 5.10, 62.94) [D
///   placement; the port resolves it at [45.2888, 5.0961, 62.9362] — probe14_aim]: the name IS the
///   mechanic. The Q-Claw bolt latches ONLY a class-0x51 object [M Bullet_Delete 0x12e660:50-55 →
///   Player_SetGrapplePoint 0x195da0]; a bolt-legal aim exists from the standing stance (40.8,
///   floor 2.40, z 65.0-65.5) at pitch ≈ 0.20 rad — the direct ray hits the point's mount, the
///   lower arc clears it [D probe14_aim's clear-aim scan].
/// - The reel pulls 0.05→1.5/tick toward the point and HANGS at distance < 2.0 [M Player_SSGrapple
///   0x192860:79-121, :91-94]; the hang (~44.05, 4.53, 63.55, 1.48 from the point) stands over the
///   open gap, and the release [M 0x192860:56-66] drops Bond into it — the slot IS the west→east
///   crossing's landing, onto the trench floor at the nav lane node 5 (43.05, 0.4, 62.86) [D].
/// - The trench runs east under the east walkway to the corridor: nav 5 (43.05, 0.4, 62.86) → 4
///   (49.08, 0.4, 62.85) [D], floor -0.60 [D this session's grids]. The corridor (x 47.75..49.75,
///   -0.60) is separated from the stairwell column (x 44.25..46.25) by a -2.40 pit band (x
///   46.5..47.5, z ~44.5..54.5) [D probe14_corridor] — one jump each way.
/// - The east STAIRWELL (the descent the lid-and-climb report identified: x 44.5..46, z 58 (2.40)
///   → 57.5 (2.30) → 56-57 (1.05) → 54.5-55.5 (0.60) → 51.5-54 (-0.91) [D probe14_stairs]) is
///   climbed here from its -0.91 foot up to the stairwell-slot approach (45.23, 1.07, 56.27) [D] —
///   the same steps the game's Bond descends, walked in the reachable direction.
#[test]
fn the_east_duct_route_reaches_the_stairwell_slot() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    // Leg A: the crossing stance (40.8, floor 2.40, 65.5) — the spawn crouch walks there; the aim
    // needs the standing head (the 3.86 duct roof clears at this z [D probe14_aim]).
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the crossing stance");
    w.scene.sim.as_mut().unwrap().bond.crouched = false;
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    // switch to the Q-Claw (0x50) [L gadget is_grapple: 0x50/0x51; +0x70 & 0x40]
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    eprintln!("[014] held weapon {:#x} (the switch request 0x50 may still read the old gun on sim.gun)", w.scene.sim.as_ref().unwrap().gun.weapon);
    // aim: yaw at the point, pitch 0.20 rad (look 0.127) — the bolt-legal arc [D probe14_aim].
    let point = [45.2888f32, 5.0961, 62.9362]; // [D placement; the port's resolution, probe14_aim]
    let want_yaw = (point[0] - 40.8).atan2(point[2] - 65.5);
    let want_look = 0.20f32 / std::f32::consts::FRAC_PI_2;
    for i in 0..200 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            turn: wrap_pi(want_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (want_look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
        if i > 150 { break; }
    }
    // fire to latch — the bolt's sweep accepts the point's sphere [M 0x12e660:50-55; 0x12c5a0:92]
    let mut latched = false;
    for _ in 0..30 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the Q-Claw bolt latches the SkyBridge grapple point [M Bullet_Delete 0x12e660:50-55; D placement]");
    // reel to the hang [M 0x192860:79-121], then release over the gap [M 0x192860:56-66]
    for _ in 0..400 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_none() { break; }
        if let Some(g) = &w.scene.sim.as_ref().unwrap().bond.grapple {
            if g.phase == 2 {
                let p = w.scene.sim.as_ref().unwrap().bond.pos;
                let d = ((p[0] - g.point[0]).powi(2) + (p[1] - g.point[1]).powi(2) + (p[2] - g.point[2]).powi(2)).sqrt();
                assert!(d < 2.5, "the hang sits within 2.0 of the point [M 0x192860:91-94], got {d:.2}");
                break;
            }
        }
    }
    // The release lands Bond on the slot's rim graze (the 2.26 mid platform corner at z ~63.9,
    // [D probe14_room's z 63.8-64.2 band]) — the east press + hop breaks the graze and drops him
    // into the OPEN gap onto the trench floor (the crossing's landing, nav lane node 5 [D]).
    let mut down = false;
    for i in 0..400 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            if b.grapple.is_none() && b.grounded && b.pos[1] < 1.0 { down = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        assert!(down && b.grapple.is_none(),
            "the release over the open gap lands Bond on the trench floor (-0.60 [D probe14_room]), got {:?} grounded {}",
            b.pos, b.grounded);
        eprintln!("[014] the crossing landed {:?}", b.pos);
    }
    // Leg B: the trench east to the corridor — the nav lane node 5 -> 4 [D], floor -0.60.
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor east leg: {p:?}");
        assert!(p[0] > 47.0, "the trench/corridor east made real progress [D nav nodes 5/4], got {p:?}");
    }
    // Leg C: the corridor south to the stairwell lane (z ~52), then jump WEST over the pit band
    // (x 46.5..47.5 at -2.40 [D probe14_corridor]) onto the -0.91 landing.
    w.walk_to(Wp::new(48.8, -0.6, 52.2).r(0.7), "the corridor south to the stairwell lane");
    let mut west = false;
    for i in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if p[0] < 46.2 && p[1] > -1.4 && sim.bond.grounded { west = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(-std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    assert!(west, "the pit band (x 46.5..47.5, -2.40 [D probe14_corridor]) is jumpable west onto the -0.91 landing, got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    // The stairwell FOOT, walked: the -0.91 landing (z 51.5..54.25, x 44.25..46.25 [D
    // probe14_corridor]) is the base of the game's east stairwell descent (the steps z 58 (2.40)
    // -> 57.5 (2.30) -> 56-57 (1.05) -> 54.5-55.5 (0.60) -> 51.5-54 (-0.91) [D probe14_stairs]).
    // The full climb/descent of the 1.05 flight (the walkable west edge x 44.5..44.75, the
    // 45..45.75 middle is the stairwell's deep throat) is the next session's leg.
    w.climb_stairs([45.5, -0.91, 53.9], "north on the -0.91 landing (the stairwell foot)", 0.8, 3000);
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] stairwell foot: {p:?}");
        assert!(p[1] > -1.6 && p[1] < 0.0 && p[2] > 52.5 && p[2] < 54.6 && p[0] > 44.0 && p[0] < 46.4,
            "the east stairwell foot (-0.91 landing, z 51.5..54.25 [D probe14_corridor/probe14_stairs]) is reached, got {p:?}");
    }
    for i in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if p[0] > 47.6 && p[1] > -1.4 && sim.bond.grounded { break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 52.0).r(0.7), "the corridor south past z 52");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 44.0).r(0.7), "the corridor south to the mouths' lane");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] mouths' lane: {p:?} in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
        assert!(p[2] < 45.0 && p[0] > 46.0, "the corridor south reached the mouths' lane [D nav node 3 -> (49.1, 0.4, 42.8)], got {p:?}");
    }
}
// level-14-vent-drop-2: the vent-drop hand-back is PORTED and pinned (see
// the_vent_drop_hands_bond_into_the_facility, green): nis.rs's event-6 fallback captures the
// scripted Bond body's world root (s18, script 0x06000857; the pack 0x06000833 has NO marker
// entity stream [D its full stream dump], so the strict previous-stream capture finds nothing and
// the game still carries Bond in [D walk-4 walls/nav]) and Script_Free's Player_Enable lands him
// inside the facility at the body's event-time root. [S] unverified game-side (the live check in
// docs/reports/level-14-vent-drop.md).
// STILL #[ignore]: the hand-back is green but leg 2 wedges at the facility's west double door
// (-52.2, 1.2, 40.1) (kind 2, state 0x2, pos01 0.00, ch_open 0, ch_lock 0x85, lock_flag 0,
// approach 0 — two records at the same spot): the walker's "opening door" interaction does not
// move it, Bond stands (-52.79, 1.14, 39.33) with the target (-49.0, 1.0, 38.4) 3.90 away, and
// walk_to hits the 6000-tick cap. Whether the game opens it from a switch channel, the approach
// side, or an interaction the port lacks is the next session's first question (the #381 band x
// −51.97…−46.39 [D] sits just east of it).
/// Legs 1a..1f walked on real input (the spawn → the crossing → the trench/corridor/stairwell →
/// the shaft Q-Claw reel → the vent lane → the mezzanine connector), then the #301 touch → the SP
/// #276 vent cut-scene (0x06000833) → the ch 0x47 wait → the disabled tail → the hand-back assert.
/// Shared by the pinned hand-back test and the full delta-exit walk.
fn walk_infiltration_to_handback(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 39.0 && p[0] < 42.0 && p[1] > 1.5 && p[1] < 6.0 && p[2] > 63.0 && p[2] < 67.0,
            "the vent store room start #666 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][0], 3,
        "'Infiltrate Drake's facility' announces at the start [L]");

    // Leg 1a: the east route (level-14-full-walk). The crossing stance → the SkyBridge Q-Claw
    // latch → reel → hang → release into the open walkway gap → the trench floor — the pinned
    // the_east_duct_route_reaches_the_stairwell_slot chain [D GRAPPLEPOINT_SkyBridge #658
    // (45.29, 5.10, 62.94); M Bullet_Delete 0x12e660:50-55 + Player_SSGrapple 0x192860].
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the crossing stance");
    w.scene.sim.as_mut().unwrap().bond.crouched = false;
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    let point = [45.2888f32, 5.0961, 62.9362]; // [D placement; the port's resolution, probe14_aim]
    let want_yaw = (point[0] - 40.8).atan2(point[2] - 65.5);
    let want_look = 0.20f32 / std::f32::consts::FRAC_PI_2;
    for i in 0..200 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            turn: wrap_pi(want_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (want_look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
        if i > 150 { break; }
    }
    let mut latched = false;
    for _ in 0..30 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the Q-Claw bolt latches the SkyBridge grapple point [M Bullet_Delete 0x12e660:50-55; D placement]");
    for _ in 0..400 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_none() { break; }
        if let Some(g) = &w.scene.sim.as_ref().unwrap().bond.grapple {
            if g.phase == 2 {
                let p = w.scene.sim.as_ref().unwrap().bond.pos;
                let d = ((p[0] - g.point[0]).powi(2) + (p[1] - g.point[1]).powi(2) + (p[2] - g.point[2]).powi(2)).sqrt();
                assert!(d < 2.5, "the hang sits within 2.0 of the point [M 0x192860:91-94], got {d:.2}");
                break;
            }
        }
    }
    let mut down = false;
    for i in 0..400 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            if b.grapple.is_none() && b.grounded && b.pos[1] < 1.0 { down = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        assert!(down && b.grapple.is_none(),
            "the release over the open gap lands Bond on the trench floor (-0.60 [D probe14_room]), got {:?} grounded {}",
            b.pos, b.grounded);
        eprintln!("[014] the crossing landed {:?}", b.pos);
    }
    // Leg 1b: the trench east to the corridor — the nav lane node 5 -> 4 [D], floor -0.60.
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor east leg: {p:?}");
        assert!(p[0] > 47.0, "the trench/corridor east made real progress [D nav nodes 5/4], got {p:?}");
    }
    // Leg 1c: the corridor south to the stairwell lane (z ~52), then jump WEST over the pit band
    // (x 46.5..47.5 at -2.40 [D probe14_corridor]) onto the -0.91 landing — the east stairwell
    // foot, the pinned proof of the landing band.
    w.walk_to(Wp::new(48.8, -0.6, 52.2).r(0.7), "the corridor south to the stairwell lane");
    let mut west = false;
    for i in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if p[0] < 46.2 && p[1] > -1.4 && sim.bond.grounded { west = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(-std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    assert!(west, "the pit band (x 46.5..47.5, -2.40 [D probe14_corridor]) is jumpable west onto the -0.91 landing, got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    w.climb_stairs([45.5, -0.91, 53.9], "north on the -0.91 landing (the stairwell foot)", 0.8, 3000);
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] stairwell foot: {p:?}");
        assert!(p[1] > -1.6 && p[1] < 0.0 && p[2] > 52.5 && p[2] < 54.6 && p[0] > 44.0 && p[0] < 46.4,
            "the east stairwell foot (-0.91 landing, z 51.5..54.25 [D probe14_corridor/probe14_stairs]) is reached, got {p:?}");
    }
    // Back east over the pit band to the corridor: the jump is EDGE-ANCHORED (walk to the
    // landing's east edge x≈46.1 first, then the hop fires from there — one 2.5 m arc clears
    // the 1.0 m band; a mid-run cadence jump from the foot's phase can fall into the pit).
    let mut east = false;
    for _attempt in 0..2 {
        let _ = w.try_walk_to(Wp::new(46.1, -0.9, 53.4).r(0.35), "to the landing's east edge");
        for i in 0..200 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let p = sim.bond.pos;
                if p[0] > 47.6 && p[1] > -1.4 && sim.bond.grounded { east = true; break; }
                if p[1] < -1.4 { break; } // fell into the pit — re-approach
            }
            let b = &w.scene.sim.as_ref().unwrap().bond;
            let inp = nf_game::player::Input {
                forward: 1.0, jump: i == 0 || (b.grounded && i % 25 == 0),
                turn: wrap_pi(std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
                ..Default::default()
            };
            w.tick(inp, false);
        }
        if east { break; }
    }
    assert!(east, "the pit band is jumpable east back onto the corridor, got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 52.0).r(0.7), "the corridor south past z 52");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 51.0).r(0.6), "the corridor south to the segment lane");
    // Leg 1d: the WEST pit jump at z 51 onto the 0.60 stairwell-corridor segment (x 44.4..46.4,
    // z 50.5..51.25 [D probe14_shaft2 grids A/B]) — the +1.2 rise over the -2.40 pit band on the
    // run-hop; the scratch run made it in ~40 ticks.
    let mut seg = false;
    for i in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if p[0] < 46.3 && p[1] > 0.3 && sim.bond.grounded { seg = true; break; }
        }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0, jump: i % 25 == 0,
            turn: wrap_pi(-std::f32::consts::FRAC_PI_2 - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] the segment: {p:?} seg {seg}");
        assert!(seg, "the west pit jump at z 51 lands on the 0.60 stairwell-corridor segment [D probe14_shaft2 A], got {p:?}");
    }
    // Leg 1e: THE SHAFT ENTRY — the Q-Claw reel to the mouths' grapple #626 (45.30, 4.79, 42.86)
    // [D placement] from the segment stance: the bolt ray skims the lip ramp (z 48.0..48.6,
    // top 2.01..2.40 [D probe14_shaft4 fine]) with 0.03-0.09 clearance and the bolt's 1.0 sweep
    // catches the point [M 0x12c5a0:92]; the reel [M Player_SSGrapple 0x192860:79-121] pulls Bond
    // up over the lip and hangs over the east vent lane (2.40); the release drops him onto it.
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    let mp = [45.30f32, 4.79, 42.86]; // [D grapple #626; the level table]
    let want_yaw = (mp[0] - 45.3).atan2(mp[2] - 51.0);
    let want_look = 0.30f32 / std::f32::consts::FRAC_PI_2; // the scratch's latch aim
    for i in 0..200 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            turn: wrap_pi(want_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (want_look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
        if i > 150 { break; }
    }
    let mut latched = false;
    for _ in 0..40 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the Q-Claw bolt latches the mouths' grapple #626 from the segment [M 0x12e660:50-55; D placement; the scratch run]");
    let mut hung = false;
    for _ in 0..500 {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_none() { break; }
        if let Some(g) = &w.scene.sim.as_ref().unwrap().bond.grapple {
            if g.phase == 2 {
                let p = w.scene.sim.as_ref().unwrap().bond.pos;
                eprintln!("[014] the mouths hang {:?}", p);
                hung = p[1] > 3.0 && p[2] < 45.6;
                break;
            }
        }
    }
    assert!(hung, "the reel hangs over the east vent lane [M 0x192860:91-94], got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    // The release: RECONCILED against [M Player_SSGrapple 0x192860 phase 2 — the first no-fire
    // tick detaches]. The printed state walk (the scratch, level-14-full-walk-2) shows the detach
    // firing on the release loop's tick 1 — the prior "still on the wire" read was wrong twice
    // over: (1) the straight drop lands on the MOUTHS' HOOD — a solid cover over the lane at top
    // 3.93 spanning z 43.8..48.5 (the open lane strip is z 42.3..43.5, tops 2.38-2.40 [D the
    // down-ray grid: y-6.0 rays over x 44.8..46.0]); (2) the fall PAST the hood's south edge was
    // frozen mid-air 1.1 above the lane by the walk's walkable contact on the hood's underside —
    // a DOWN-facing face the game's dot1 gate never sets walkable [M Intersect_CylGeom 0x109da0
    // disasm 0x10a650-0x10a674: dot(face normal, axis) > 0.7, up-facing only; the port's .abs()
    // diverged — fixed in world.rs]. The game's hang swing [M Player_Move 0x191dd0:111-141 + the
    // SSGrapple tail commit LAB_00192a24, the rope pin holding y] carries the stick's drift, so
    // the route: hold fire, drift SOUTH over the open strip, release, drop onto the lane (2.38).
    let mut lane = false;
    for i in 0..160 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let p = b.pos;
            if b.grapple.is_none() && b.grounded && p[1] > 1.8 && p[1] < 3.4
                && p[2] < 48.4 && p[0] > 44.0 && p[0] < 47.0 {
                lane = true;
                break;
            }
        }
        // The drift: fire HELD (the hang stays [M 0x192860:57-62]), the stick south — the
        // grapple_drift lanes ease to the walk's 0.075/tick [M 0x191dd0:111-141].
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            forward: 1.0,
            turn: wrap_pi(std::f32::consts::PI - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick_fire(inp);
        if w.scene.sim.as_ref().unwrap().bond.pos[2] < 42.45 { break; }
    }
    for i in 0..400 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let p = b.pos;
            if b.grapple.is_none() && b.grounded && p[1] > 1.8 && p[1] < 3.4
                && p[2] < 48.4 && p[0] > 44.0 && p[0] < 47.0 {
                lane = true;
                break;
            }
        }
        // The release: the first no-fire tick detaches [M 0x192860:56-66]; the drop is the
        // port's pinned no-momentum fall (the shipped flow's +0x10 is zero at the break).
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            turn: wrap_pi(std::f32::consts::PI - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] the vent lane: {p:?} lane {lane}");
        assert!(lane, "the release lands Bond on the east vent lane (2.40 [D probe14_tunnel A]), got {p:?}");
    }
    // Leg 1f: the east vent lane south over its flush lid (z 42.16 [D #655/#600]) — the lid sits
    // flush in the lane floor [D probe14_mouthfin A: 2.38->2.40 at z 42.3..42.2] — to the
    // mezzanine, then WEST across the sloped connector (z 39.5..38.25, 2.15..2.40 [D
    // probe14_mouthfin D grid]) to the #301 drop band.
    w.walk_to(Wp::new(45.0, 2.4, 40.0).r(0.8), "the lane south past the mouth");
    let _ = w.try_walk_to(Wp::new(44.9, 2.4, 39.2).r(0.8), "the lane south to the connector");
    let _ = w.try_walk_to(Wp::new(40.7, 2.3, 39.0).r(1.2), "the mezzanine west across the connector");
    // The SP #276 cut-scene (0x06000833, 1400 ticks [D decode]) writes ch 0x47 at its last tick.
    let mut ch47 = false;
    for _ in 0..2400 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x47) { ch47 = true; break; }
    }
    assert!(ch47, "the vent cut-scene must write ch 0x47 [D]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][0], 4, "objective 1 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    {
        // The hand-back: Script_Free's Player_Enable landed Bond at the body's live root at event-6
        // time [M Script_Free 0x1da880:44-48 → Player_Enable 0x195498 copies the matrix translation;
        // the capture is the port's [S] — the report's live check 1]. The root at the scene's last
        // tick is the anim's frame ~1106 (the body stream starts at t294 [D the pack dump], the scene
        // ends at t1399; the seq-04000728 end (27.60, 0.98, 43.56) [D the root track] is NOT the
        // hand-back pose — the anim's later frames carry the body further). The proven essentials:
        // Bond is INSIDE the facility (floor y ≈ 1), past the mezzanine wall (z 38.21 [D walk-4]) —
        // never the mezzanine he stood on and never the void.
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p[1] > 0.3 && p[1] < 2.0 && p[2] < 39.5 && (25.0..35.0).contains(&p[0]),
            "the vent cut-scene hands Bond back inside the facility at the body's event-time root [D the pack's anim root; M Player_Enable 0x195498], got {p:?}");
        eprintln!("[014] hand-back at {p:?}");
    }
}

/// The vent-drop hand-back PINNED (level-14-vent-drop-2): the walked infiltration ends in the SP
/// #276 cut-scene, and Script_Free's Player_Enable (fed by the event-6 body-root fallback) stands
/// Bond INSIDE the facility — the exact symptom level-14-full-walk-4 reported (Bond left on the
/// mezzanine) is gone. Green un-ignored.
#[test]
fn the_vent_drop_hands_bond_into_the_facility() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    walk_infiltration_to_handback(&mut w);
}

#[test]
fn walk_the_infiltration_to_the_delta_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    walk_infiltration_to_handback(&mut w);

    // Leg 2: west to the Kiko cut-scene touch #381 — the walk's OWN route (level-14-delta-exit-walk).
    // The closed Door_Slide_11A/B pair (ch_lock 0x85 [D placements 151/152]) seals its doorway
    // completely until the sabotage cascade: the leaves' collision plane x≈−52.35 blocks the walk
    // from BOTH sides across z 39.2..41.3 and the static frame jambs (tris x −53.5..−52.5 and
    // −51.7..−51.0 [D probe]) seal z 38.6..38.9 — this session's crossing probe (the walk east
    // stops at (−52.79, 1.20, 39.42), the walk west never leaves the east jamb). The old leg-2
    // premise ("the nav route crosses the closed door plane" and slips by) was wrong — the west
    // wing is reached the game's own way, the SOUTH loop (the nav web's west corridor
    // x −48.6..−57.4 at z 25.7..30.5, nodes 94-97/150/591/592/599/600 [D nav path 0]), which
    // never crosses the 0x85 plane. The 0x85 door is crossed ONLY after it unlocks (leg 4).
    w.follow_nav_route([-48.7, 0.9, 33.2], "the south loop to the west corridor");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < -46.0 && p[0] > -51.5 && (30.0..35.5).contains(&p[2]),
            "the south loop ends in the west corridor (node 192 (−48.64, 0.4, 32.13) [D nav]), got {p:?}");
    }
    w.walk_to(Wp::new(-49.6, 0.9, 35.5).r(1.2), "north to the Kiko band's south edge");
    // Walk north into the band; the TouchOnce #381 fires (out 0x08 [D placement 381]) and the SP
    // #382 cut-scene (0x0600062F) takes over: the pack's scripted body carries Bond WEST past the
    // closed 0x85 door (the sweep's mid-scene pose (−56.18, 40.59) [D probe]; the same event-6
    // body mechanism as the vent hand-back [M Script_Free 0x1da880 → Player_Enable 0x195498]).
    // Steering must STOP at the touch: resuming toward the band after the scene drags Bond back
    // east into the sealed doorway (the original leg-2 wedge — the E-press loop on the locked
    // leaf). The scene, not the walk, delivers the west side.
    {
        let start = w.total_ticks;
        loop {
            assert!(w.total_ticks - start < 4000, "the #381 touch never fired");
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(0x08) || sim.disabled { break; }
            }
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let inp = nf_game::player::Input {
                turn: wrap_pi((-49.2f32 - b.pos[0]).atan2(38.4 - b.pos[2]) - b.yaw).clamp(-0.08, 0.08),
                forward: 1.0,
                ..Default::default()
            };
            w.tick(inp, false);
        }
        eprintln!("[014] the #381 touch fired at {:?} (+{} ticks)",
            w.scene.sim.as_ref().unwrap().bond.pos, w.total_ticks - start);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[0] < -46.0, "Bond stands in the #381 band (x −51.97…−46.39 [D]), got {:?}", sim.bond.pos);
    }
    let mut ch48 = false;
    for _ in 0..2400 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) { ch48 = true; break; }
    }
    assert!(ch48, "the Kiko cut-scene 0x0600062F must write ch 0x48 [D]");
    // fix-objective-banner: the monitor completes the objective the tick after the channel.
    for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][1], 4, "objective 2 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }

    // Leg 3a: the west generator room — the scene released Bond on the west side (the west
    // corridor's north lane, z ≈ 40.6 [D the sweep's mid-scene pose]); the corridor west along
    // the nav lanes (nodes 613/614/626 [D nav]), then south (nodes 625/615/623), the ramp head
    // (node 628) and down into the hall (floors −0.6/−1.4/−2.2 at x −63.7/−65/−66.2, z 32.8
    // [D probe]; node 631), then the hall's south lane west (nodes 663/654/653 [D nav]).
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] the Kiko scene released Bond at {p:?}");
    }
    w.walk_to(Wp::new(-57.9, 0.1, 40.77).r(1.5), "the west corridor's north lane");
    w.walk_to(Wp::new(-60.5, 0.05, 40.77).r(1.0), "the corridor west (node 626)");
    w.walk_to(Wp::new(-60.5, 0.05, 35.7).r(1.2), "the corridor south (node 615)");
    w.walk_to(Wp::new(-60.5, 0.05, 32.84).r(1.0), "the lane east end (node 623)");
    w.walk_to(Wp::new(-63.7, 0.05, 32.85).r(1.0), "the ramp head (node 628)");
    w.walk_to(Wp::new(-66.3, -1.78, 32.85).r(1.0), "down the generator ramp (node 631)");
    w.walk_to(Wp::new(-72.4, -1.78, 32.7).r(1.2), "the hall south lane (node 663)");
    w.walk_to(Wp::new(-82.9, -1.78, 33.4).r(1.2), "the hall south lane west (node 654)");
    w.walk_to(Wp::new(-87.5, -1.78, 33.4).r(1.2), "the hall west end (node 653)");

    // Leg 3b: the two fuseboxes cut through the REAL laser path [M FuseBox_Update 0x1adfe8] —
    // box A in its south alcove (floor −2.2 [D probe]), then the hall's west lane north (nodes
    // 649/644/204) to box B in the north alcove (its south candidate floor −2.2 [D probe]).
    cut_fusebox(&mut w, [-89.31, -2.22, 30.79], 0x87, "fusebox #383");
    w.walk_to(Wp::new(-89.3, -1.78, 36.99).r(1.2), "the box A north lane (node 649)");
    w.walk_to(Wp::new(-89.3, -1.78, 40.58).r(1.2), "the hall west lane north (node 644)");
    w.walk_to(Wp::new(-89.3, -1.78, 45.7).r(1.2), "the hall north-west (node 204)");
    cut_fusebox(&mut w, [-89.30, -2.22, 49.36], 0x88, "fusebox #385");
    for _ in 0..10 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8c), "AND #391 [0x87, 0x88] raises 0x8C [D]");
        assert_eq!(sim.drones.health.health, 100.0,
            "no live-wire hurt with +0x56 configured [M 0x1adfe8:99-101]");
    }

    // Leg 3c: the E-use switch oDroneTrigger1b (out 0xA0, mask 4 [D placement 399; M
    // Trigger_Activate 0x1b2318:23-33]) — the hall's north lane back east (nodes 641/637/635/
    // 199 [D nav]), up the north ramp (node 632 (−66.23, −1.78, 48.5) → node 630 (−63.7, 0.019,
    // 48.05); the hop-climb the stairs walker uses), then the west corridor south (nodes
    // 620/618/626) into the switch band (lo −63.08…−57.52, z 37.33…39.41 [D]).
    w.walk_to(Wp::new(-87.5, -1.78, 46.7).r(1.2), "the hall north lane (node 641)");
    w.walk_to(Wp::new(-72.7, -1.78, 47.4).r(1.2), "the hall north lane east (node 635)");
    w.walk_to(Wp::new(-66.3, -1.78, 48.4).r(1.0), "the north ramp foot (node 632)");
    w.climb_stairs([-63.6, 0.05, 48.0], "up the north ramp (node 630)", 1.0, 3000);
    w.walk_to(Wp::new(-60.5, 0.05, 47.4).r(1.0), "the west corridor north lane (node 620)");
    w.walk_to(Wp::new(-60.5, 0.05, 43.79).r(1.0), "the corridor south (node 618)");
    w.walk_to(Wp::new(-60.5, 0.05, 40.77).r(1.0), "the corridor south to the switch band (node 626)");
    w.walk_to(Wp::new(-60.3, 0.6, 38.4).r(0.6), "into the switch band (the E-use spot)");
    // The E press through the REAL Player_Activate use path (the walk's own input): aim at the
    // touch's volume centre with turn/look steering (the cut_fusebox aim convention), then hold
    // E — the green the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door's press
    // loop, reached on foot instead of teleported [M 0x1a5170:100-108, 0x1b2318:23-33].
    let t_centre = [-60.29973f32, 0.61489844, 38.371338]; // [D placement 399 centre]
    for _ in 0..60 {
        let sim = w.scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let d = [t_centre[0] - head[0], t_centre[1] - head[1], t_centre[2] - head[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        let b = &sim.bond;
        let inp = nf_game::player::Input {
            turn: (d[0].atan2(d[2]) - b.yaw).clamp(-0.08, 0.08),
            look: (((d[1] / l).atan() / std::f32::consts::FRAC_PI_2) - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    let mut fired = false;
    for _ in 0..12 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xa0) { fired = true; break; }
    }
    assert!(fired, "the E press fires the mask-4 touch → 0xA0 [M 0x1b2318:23-33; D placement 399]");
    // The cascade: the touch's 0xA0 is a 1-tick pulse (Trigger_Calc 0x1b1900's type-2 clear), so
    // 0x51 pulses too and the rest rides the death channels — track "ever on" for the pulse
    // links (the E-use test's track loop) [D placement 393; M 0x1b1900].
    let (mut seen51, mut ch85) = (false, false);
    for _k in 0..300u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let ch = &sim.drones.channels;
            if ch.on(0x51) { seen51 = true; }
            if ch.on(0x85) { ch85 = true; }
        }
        w.tick(nf_game::player::Input::default(), false);
        if ch85 { break; }
    }
    assert!(seen51, "SEQ_AND [0x8C, 0xA0] raised 0x51 (the 1-tick pulse) [D placement 393]");
    assert!(ch85, "the cascade ends at box #729's death 0x85 [D the kill→death pairs]");
    // The door unlocked for good and AUTO-OPENED on the 0x85 rising edge [M 0x1363c8:88-107].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.doors.list.iter().position(|d| d.placement == 151)
            .expect("Door_Slide_11A exists [D placement 151]");
        assert!(!sim.drones.doors.is_locked(door, &sim.drones.channels),
            "Door_Slide_11A/B unlocks on the 0x85 rising edge [M 0x1363c8:88-107]");
    }
    for _ in 0..180 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.doors.list.iter().position(|d| d.placement == 151).unwrap();
        assert!(sim.drones.doors.list[door].pos01 > 0.5,
            "the door slid open while the walk waited (pos01 {:.2})", sim.drones.doors.list[door].pos01);
    }

    // Leg 4: east THROUGH the opened doorway to the server-room escape touch #456 — the walk's
    // own crossing (the unlock auto-open slid the leaves clear [M 0x1363c8:88-107]), then the
    // loop east (leg 2's route reversed); the SP #407 cut-scene (0x06000848, 950 ticks [D])
    // writes ch 0x4A.
    w.walk_to(Wp::new(-60.5, 0.05, 39.57).r(1.0), "the west corridor at the door's west face");
    w.walk_to(Wp::new(-56.6, 0.4, 39.56).r(1.0), "the corridor east to the door");
    w.walk_to(Wp::new(-51.4, 0.4, 39.56).r(1.0), "through the opened doorway");
    w.walk_to(Wp::new(-49.2, 0.9, 38.4).r(1.0), "the Kiko corridor east");
    w.follow_nav_route([-33.1, 0.9, 26.8], "east to the escape touch #456");
    let mut ch4a = false;
    for _ in 0..2000 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x4a) { ch4a = true; break; }
        assert!(sim.drones.channels.on(0x0c), "the room touch #456 fired 0x0C on contact [D]");
    }
    assert!(ch4a, "the escape cut-scene must write ch 0x4A [D]");
    // fix-objective-banner: the monitor completes the objective the tick after the channel.
    for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][2], 4, "objective 3 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }

    // Leg 5: east to Rook's wake touch #476 — walked; SEQ_AND #475 wakes him.
    w.follow_nav_route([20.3, 0.9, -23.5], "east to Rook's wake touch #476");
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    let rook_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_0066).expect("Rook (skin 0x05000066) exists [D]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0d), "SEQ_AND #475 [0x8C, 0x0F] raises 0x0D [D]");
        assert!(sim.drones.list[rook_i].started, "Rook leaves his switch-channel wait [M 0x14b300]");
    }
    // The kill: the walked gun path. The fusebox cuts left the Laser (0x4e) in hand — a tool
    // whose rounds never drew blood (the run's own trace: 198 taps at point-blank, health 10.0
    // constant) — so the hand weapon 0x2 goes back up first [L loadout: owned at the start].
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x2), ..Default::default() }, false); }
    let mut dead = false;
    for attempt in 0..3 {
        if w.shoot(rook_i, "Rook") { dead = true; break; }
        assert!(w.resupply(&format!("Rook attempt {attempt}")), "no ammo for the Rook kill");
    }
    assert!(dead, "Rook must die to the walked gun path");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x54), "Rook's death channel 0x54 fires [D param 12; M 0x13e660]");
        // fix-objective-banner: the monitor completes the objective the tick after the channel.
    }
    for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][3], 4, "objective 4 completes [M 0x1827d8]");

    // Leg 6: to the Delta exit #944 — walked; E fires the fallback write + the load.
    w.walk_to(Wp::new(27.79, 1.28, -25.32).r(1.0), "to the Delta exit #944");
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(loaded && sim.drones.level_exit == Some(0x0700_0015),
            "E at the exit #944 loads 07000015 (EvilBase) [D]");
        assert!(sim.drones.channels.on(0xce), "the fallback arm wrote ch 0xCE (objective 5) [M 0x1b2318]");
    }
    eprintln!("[014] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

/// The grill sister, REWRITTEN (level-14-vent-grate): the old version pinned the crawl THROUGH the
/// AirConGrill1 plane on the bit-0x1 skip — that skip contradicts the decompile and is reverted.
/// The game's own evidence: [M] parsemap_block_map_data_static 0x1d1998:92 creates the runtime cel
/// for every 0x8000 placement (AirConGrill1's placement #100 carries 0x00008019 [D probe14_plcflags])
/// and [M] Collide_Pick 0x1eb320 — the candidate stage of the movement capsule's kind-0x800 query
/// (Collide_CylinderIntersect 0x1ecc00:61) — skips nodes only on cel+0x94 bit 0x20 (:140) and
/// bit 0x1000 with query+0x98 bit 0x20 (:145-151); the grill's glist word is 0x1 [D], never tested.

/// duct's south end is closed and the route leaves through the EAST duct mouth (past lid SP #328).
#[test]
fn the_vent_grill_blocks_the_west_duct_per_the_game() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let bar_count = {
        let sim = scene.sim.as_ref().unwrap();
        let tris = sim.world.tris_near([40.0, 2.3, 58.5], [41.6, 3.9, 59.2]);
        tris.iter().filter(|t| {
            let zs = [t[0][2], t[1][2], t[2][2]];
            let ys = [t[0][1], t[1][1], t[2][1]];
            zs.iter().all(|z| (58.5..59.3).contains(z))
                && ys.iter().all(|y| (2.55..3.7).contains(y))
                && (ys.iter().cloned().fold(f32::MAX, f32::min) - ys.iter().cloned().fold(f32::MIN, f32::max)) < 0.09
        }).count()
    };
    drop(scene); // the second load doubles the footprint — a small test machine OOMs at ~2 GB
    let bars = bar_count;
    // The bars are 0.06-0.07 thick in y, six of them between y 2.62 and 3.62, all at z=58.86-58.9
    // [D probe14_tris; the placement 0x8000 cel + the Collide_Pick no-0x1-skip rule above].
    assert!(bars >= 20,
        "the AirConGrill1 bars (24 tris, six bands y 2.62..3.62) ARE the game's static collision [M 0x1d1998:92 + 0x1eb320:140/:145-151; D plc #100 0x00008019 glist 0x1], got {bars}");
    // And the crawl does NOT get through: a crouched walk south wedges on the grill plane —
    // the game's Bond never crosses z=58.86 westward; the route leaves east (the walkreal's
    // the_east_duct_route_reaches_the_stairwell_slot walks the real east path).
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        sim.bond.crouched = true;
        for _ in 0..15 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        for _ in 0..400 {
            let inp = nf_game::player::Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
        }
        assert!(sim.bond.pos[2] > 58.5,
            "the crawl wedges on the grill plane z=58.86 [M the cel-exists rule above], got {:?}", sim.bond.pos);
    }
}

/// The west double door's opener chain, E-USE ROOT (level-14-west-door). The touch
/// `oDroneTrigger1b` (map 54 placement 399, kind 0xEB) at (-60.30, 0.61, 38.37) carries
/// (p0, p1) = (0xA0, 0x4) [D placements-all] — for the 0xEB wrapper p0 IS the out-channel and
/// p1 the contact mask ([M] `Trigger_Touch` 0x1b2aa8 re-homes old p0 → slot 1 and old p1 → slot
/// 9 before `Trigger_Create` 0x1b1720 reads out = slot 1, mask = slot 9). Mask bit 0x4 is the
/// E-USE flag, not a contact class: `Trigger_Hit` 0x1b1fc8 builds contact bits {2 bullet,
/// 8 player, 0x10 drone} only, so a mask of 0x4 never fires by contact (the panel test pins
/// that), while `Trigger_Activate` 0x1b2318's type-2 arm REQUIRES `mask & 4` and then writes
/// `switch_channels[out] = 1` [M :23-33] — reached from `Player_Activate` 0x1a5170's
/// class-0x1D arm [M :134-137, the use sphere 0x1a5170:100-108]. With the sabotage AND 0x8C
/// up, the E press raises 0xA0 → SEQ_AND (map 54 placement 393) [0x8C, 0xA0] → 0x51 [D] → the
/// kill cascade → box death 0x85 [D pairs] → Door_Slide_11A/B (map 54 placements 151/152,
/// p3 = 0x85) unlock on the rising edge and auto-open [M Door_Update 0x1363c8:88-107].
#[test]
fn the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    // The touch exists with the disc's fields [D placement 399].
    let t_centre = {
        let sim = scene.sim.as_ref().unwrap();
        let i = sim.drones.triggers.iter()
            .position(|t| t.out == 0xa0 && t.mask & 4 != 0 && t.kind == nf_game::triggers::kind::TOUCH)
            .expect("the 0xA0 E-use touch (mask 4, kind 0xEB TOUCH) exists [D placement 399]");
        let t = &sim.drones.triggers[i];
        assert!(!t.tris.is_empty(), "the touch carries its object's volume [D oDroneTrigger1b]");
        eprintln!("[014] the E-use touch: placement {} AABB {:?}", t.placement, (t.lo, t.hi));
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    // The door starts locked; nothing has armed [M 0x1363c8:93-96].
    let door = scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.ch_lock == 0x85)
        .expect("Door_Slide_11A/B with lock ch 0x85 exists [D placements 151/152]");
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.is_locked(door, &sim.drones.channels), "the door starts locked");
        assert!(!sim.drones.channels.on(0x51) && !sim.drones.channels.on(0x85) && !sim.drones.channels.on(0xa0),
            "nothing armed at load [D]");
    }
    // The fusebox words (the real laser cut is the walk's leg 3 and the panel tests' pin);
    // seeding both raises 0x8C through the REAL AND mux [D AND plc (0x87,0x88)→0x8C].
    scene.sim.as_mut().unwrap().drones.channels.set(0x87, 1, 1);
    scene.sim.as_mut().unwrap().drones.channels.set(0x88, 1, 1);
    scene.sim.as_mut().unwrap().bond.pos = [-60.3, 0.9, 38.4];
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8c), "AND [0x87, 0x88] raises 0x8C [D]");
        assert!(!sim.drones.channels.on(0xa0) && !sim.drones.channels.on(0x51),
            "standing in the touch's volume without E fires nothing [M 0x1b1fc8 mask 0x4]");
    }
    // Aim at the touch's volume centre and press E through the REAL Player_Activate path
    // (the use sphere head + 1×look [M 0x1a5170:100-108]; the port's AABB stand-in).
    {
        let sim = scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let d = [t_centre[0] - head[0], t_centre[1] - head[1], t_centre[2] - head[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.yaw = d[0].atan2(d[2]);
        sim.bond.pitch = (d[1] / l).asin() / std::f32::consts::FRAC_PI_2; // look() units [M BL+0x8a8·π/2]
        eprintln!("[014] aiming at the touch: head {head:?} centre {t_centre:?} yaw {:.2} pitch {:.2}", sim.bond.yaw, sim.bond.pitch);
    }
    let mut fired = false;
    for _ in 0..12 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.channels.on(0xa0) { fired = true; break; }
    }
    assert!(fired, "the E press fires the mask-4 touch → 0xA0 [M 0x1b2318:23-33; D placement 399]");
    // The SEQ_AND's write is a LEVEL write recomputed per tick while its inputs hold — but the
    // touch's 0xA0 is a 1-tick pulse (Trigger_Calc 0x1b1900's type-2 clear each frame), so 0x51
    // pulses too; the cascade's first link reads it inside the pulse window and the rest ride
    // the death channels (never cleared). Track "ever on" for the pulse links, level for 0x85.
    let (mut seen51, mut ch85) = (false, false);
    for k in 0..300u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            let ch = &sim.drones.channels;
            if ch.on(0x51) { seen51 = true; }
            if ch.on(0x85) { ch85 = true; }
            if k % 25 == 0 {
                eprintln!("[014] cascade tick {k}: a0 {} 51{} 52 {} 4e {} 53 {} 85 {}",
                    ch.on(0xa0), seen51 as u8, ch.on(0x52), ch.on(0x4e), ch.on(0x53), ch.on(0x85));
            }
        }
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.drones.health.health = 100.0;
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
        }
        if ch85 { break; }
    }
    assert!(seen51, "SEQ_AND [0x8C, 0xA0] raised 0x51 (the 1-tick pulse) [D placement 393]");
    assert!(ch85, "the cascade ends at box #729's death 0x85 [D the kill→death pairs]");
    // The door unlocked for good and AUTO-OPENED on the rising edge [M 0x1363c8:88-107].
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.doors.is_locked(door, &sim.drones.channels),
            "Door_Slide_11A/B unlocks on the 0x85 rising edge [M 0x1363c8:88-107]");
        assert_eq!(sim.drones.doors.list[door].ch_lock, 0, "the lock field cleared [M 0x1363c8:106]");
        assert!(sim.drones.doors.list[door].state & 0xC != 0 || sim.drones.doors.list[door].pos01 > 0.0,
            "the unlock edge OPENS the door (state {:?} pos01 {})",
            sim.drones.doors.list[door].state, sim.drones.doors.list[door].pos01);
    }
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.list[door].pos01 > 0.5,
            "the door slides open (pos01 {:.2})", sim.drones.doors.list[door].pos01);
    }
}
