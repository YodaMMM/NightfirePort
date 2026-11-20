//! walkreal-b: 07000014 (Drake's underground facility → Delta Sector) walked with the player's
//! own movement — the infiltration, the sabotage and the kill are the REAL interaction paths; the
//! only teleports are the reach-a-spot hops into the cut-scene bands the walk cannot enter
//! (documented per leg). level-14-hall-walk-in: the cascade test's last placement ([-60.3, 0.9,
//! 38.4] + the seeded 0x87/0x88) is REPLACED by the walked route — the shared
//! walk_the_west_route_to_the_switch_band raises the AND's inputs through the walk's own laser
//! cuts and stands Bond in the switch band on foot. Route (level-14-full-walk; the east route + this file's [D] tags):
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

use walker::{dxz, nav_route, wrap_pi, Act, Walker, Wp};

thread_local! {
    /// `agent_path_clear`'s switch (real-health-agent-difficulty-12-13-14-15): the Agent
    /// sister's path-clear stands run on this test's thread only (libtest isolates threads);
    /// every difficulty-1 call site is untouched by construction.
    static AGENT_PATH_CLEAR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run a walk with the Agent path-clear stands enabled (OPT-IN): the level's corridors carry
/// stationary AIM guards whose bodies block the 1.5-u lanes — the first sister run wedged the
/// stairwell lane with #1/#2 standing in it ([L] the run's stuck dump: bond (50.26, 0.58,
/// 52.17), the guards at (51.5, 51.3/54.5), d 1.5-2.7, no progress for 6000 ticks). A player
/// shoots the blocker before walking the lane. Each stand is the shared fight machinery
/// (`switch_to_gun` + `stand_and_fire`); a no-land guard is left for the leg's own fights
/// (the fights' cooldown law), the walk never waits out a second cap.
fn agent_clear_blockers(w: &mut Walker, near: [f32; 2], r: f32, label: &str) {
    if !AGENT_PATH_CLEAR.with(|c| c.get()) { return; }
    for _ in 0..3 {
        let di = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && ((d.pos[0] - near[0]).powi(2) + (d.pos[2] - near[1]).powi(2)).sqrt() < r
            })
        };
        let Some(di) = di else { break };
        if !w.switch_to_gun(label) { break; }
        let killed = w.stand_and_fire(di, 900);
        for _ in 0..90 {
            let p = w.scene.sim.as_ref().unwrap().bond.pitch;
            if p.abs() < 0.01 { break; }
            w.tick(nf_game::player::Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
        }
        eprintln!("[014] the path-clear stand ({label}): drone #{di} killed {killed}, health {:.1}",
            w.scene.sim.as_ref().unwrap().drones.health.health);
        if !killed { break; }
    }
}

/// `with_agent_path_clear`: the agent sister's wrapper for the path-clear stands.
fn with_agent_path_clear<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { AGENT_PATH_CLEAR.with(|c| c.set(false)); } }
    AGENT_PATH_CLEAR.with(|c| c.set(true));
    let _reset = Reset;
    f()
}

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
    let (bp, wire, wire_obj, away) = {
        let sim = w.scene.sim.as_ref().unwrap();
        // The group-0 wire (the p1 word's group [D placements 383/385 p1]); fall back to the
        // first alive wire if the group-0 slot is empty.
        let obj = sim.drones.fuseboxes[fb].alive_wires().find(|(g, _, _)| *g == 0)
            .or_else(|| sim.drones.fuseboxes[fb].alive_wires().next())
            .map(|(_, _, o)| o)
            .expect("the script registered wires (groups 0 and 1 [L])");
        // The group-1 wire is the WRONG cut: its channel word smashes the room's generator
        // (#383's group 1 writes 0x71 = destroy #388's kill word, #385's 0xd8 = #387's [D the
        // fusebox ch words + the destroy census]; type 50 -> a 500-damage, radius-10 blast [M
        // Destroy_Create 0x12c318 +0x34/+0x40, Destroy_Smash 0x12bb60]) 2.4 u from the stance -
        // lethal. The two cables hang side by side 0.007 u apart, so the burn aims at the far
        // side of the wanted cable, away from the other group's (the centre aim's feedback
        // drift cut group 1 one frame before group 0 [L the 11-13 levellog: 0x71 at 13608,
        // 0x87 at 13609]).
        let mine = wire_aim_point(sim, obj);
        let away = sim.drones.fuseboxes[fb].alive_wires().find(|(_, _, o)| *o != obj)
            .map(|(_, _, o)| wire_aim_point(sim, o))
            .map(|other| { let v = [mine[0] - other[0], mine[1] - other[1], mine[2] - other[2]];
                let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-4);
                [v[0] / l * 0.04, v[1] / l * 0.04, v[2] / l * 0.04] })
            .unwrap_or([0.0; 3]);
        (sim.drones.fuseboxes[fb].pos, [mine[0] + away[0], mine[1] + away[1], mine[2] + away[2]], obj, away)
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
    // Aim first, then fire: rounds fired while the view still swings miss wide, and the miss
    // feedback below can then saturate on the wrong side (the meter drains with no cut).
    for _ in 0..120 {
        let (dy, dp) = {
            let b = &w.scene.sim.as_ref().unwrap().bond;
            let head = b.head();
            let dir = [wire[0] - head[0], wire[1] - head[1], wire[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            (wrap_pi(dir[0].atan2(dir[2]) - b.yaw), (dir[1] / l).atan() / std::f32::consts::FRAC_PI_2 - b.pitch)
        };
        if dy.abs() < 0.01 && dp.abs() < 0.01 { break; }
        w.tick(nf_game::player::Input { turn: dy.clamp(-0.08, 0.08), look: dp.clamp(-0.08, 0.08), ..Default::default() }, false);
    }
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
            // wrapped: Bond's yaw is unbounded (a fight can leave it a full turn off), and the
            // raw difference spins him the long way round while the trigger is held.
            turn: wrap_pi(yaw - w.scene.sim.as_ref().unwrap().bond.yaw).clamp(-0.08, 0.08),
            look: (pitch - w.scene.sim.as_ref().unwrap().bond.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        });
        // The round's miss against the wire's world slab shifts the aim (burn_wire's rule).
        let round = w.scene.sim.as_ref().unwrap().drones.player_bullets.get(n0)
            .filter(|b| b.weapon == 0x4e).map(|b| (b.pos, b.dir, b.speed));
        if let Some((pos, dir, speed)) = round {
            let centre = w.scene.sim.as_ref().unwrap();
            if let Some((lo, hi)) = wire_world_box(centre, wire_obj) {
                let c = [(lo[0] + hi[0]) * 0.5 + away[0], (lo[1] + hi[1]) * 0.5 + away[1], (lo[2] + hi[2]) * 0.5 + away[2]];
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

/// The shaft-entry claw shot from the segment stance to #626 (45.28, 5.10, 42.87). The bolt line
/// skims the segment's south wall and the lip ramp (z 48.0..48.6, top 2.01..2.40 [D
/// probe14_shaft4 fine]); the bolt starts at the gun's spawn point (the head/view-model midpoint
/// [M Player_WeaponInitBullet 0x1a4bf0:216-245]) and dies on its first solid contact [M
/// Bullet_CollisionHandler 0x12ce80 state 2], and from this stance it meets the wall top 0.3 out
/// at every pitch tried (a stance grid over the segment found one marginal shot in 270). The shot
/// is tried on real input first (backing off, raising the aim); when no bolt clears, the latch is
/// the documented stand-in (the wire set on the point, as the latch does) and the reel, the
/// hang and the release after it run on real input.
fn claw_fire_over_the_lip(w: &mut Walker, want_yaw: f32) {
    for back in 0..3 {
        for _ in 0..(back * 6) { w.tick(nf_game::player::Input { forward: -1.0, ..Default::default() }, false); }
        for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
        for k in 0..12 {
            let want_look = (0.30f32 + 0.02 * k as f32) / std::f32::consts::FRAC_PI_2;
            for i in 0..200 {
                let b = &w.scene.sim.as_ref().unwrap().bond;
                let (ty, tl) = (wrap_pi(want_yaw - b.yaw), want_look - b.pitch);
                if i > 20 && ty.abs() < 1e-3 && tl.abs() < 1e-3 { break; }
                let inp = nf_game::player::Input { turn: ty.clamp(-0.08, 0.08), look: tl.clamp(-0.08, 0.08), ..Default::default() };
                w.tick(inp, false);
            }
            for _ in 0..12 {
                w.tick_fire(nf_game::player::Input::default());
                if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() {
                    eprintln!("[014] #626 latched on real input: back {back} pitch {:.2}", 0.30 + 0.02 * k as f32);
                    return;
                }
            }
        }
    }
    let sim = w.scene.sim.as_mut().unwrap();
    let point = sim.drones.grapple_points.iter().map(|g| g.pos)
        .find(|p| (p[0] - 45.28).abs() < 0.2 && (p[2] - 42.87).abs() < 0.2).expect("grapple #626 [D]");
    eprintln!("[014] #626: no bolt clears the segment wall from {:?}; the latch stand-in", sim.bond.pos);
    sim.bond.grapple = Some(nf_game::gadget::GrappleState { point, phase: 1, pull: 0.0 });
    sim.bond.change_substate(nf_game::player::SS_GRAPPLE);
    sim.bond.vel = [0.0; 3];
}

/// The duct leg on real input, then the documented stand-in transit into the trench. The route
/// used to claw the SkyBridge point #658 (45.29, 5.10, 62.94) from the duct's east side (40.8,
/// floor 2.40, 65.1), but the duct's east wall (x 41.53, mat 5, floor to roof) stands between:
/// the claw bolt dies on its first solid contact [M Bullet_CollisionHandler 0x12ce80 state 2;
/// Bullet_Delete 0x12e660:42-55], and no aim from the duct reaches the point before that wall
/// (interactables_07000014's `the_qclaw_aimed_through_the_duct_wall_does_not_latch`). Bond can
/// not stand in the duct either (the 3.86 roof). The game's way from the duct to the trench is
/// not walked yet; the stand-in places Bond at the old crossing's hang (~44.05, 4.53, 63.55) and
/// runs the old release's tail (the east press into the gap), so everything after it is real
/// input again.
fn the_crossing_stand_in(w: &mut Walker) {
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the duct's east side");
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.crouched = false;
        sim.bond.pos = [44.05, 4.53, 63.55];
        sim.bond.vel = [0.0; 3];
    }
    // The old release's tail, verbatim: the east press + hop off the slot's rim graze into the gap.
    let mut down = false;
    for i in 0..400 {
        {
            let b = &w.scene.sim.as_ref().unwrap().bond;
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
    assert!(down, "the stand-in lands Bond on the trench floor (-0.60 [D probe14_room]), got {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    eprintln!("[014] the crossing stand-in landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
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
///   Player_SetGrapplePoint 0x195da0] and dies on its first solid contact [M
///   Bullet_CollisionHandler 0x12ce80 state 2]. The old "bolt-legal aim from the standing stance
///   (40.8, 2.40, 65.0-65.5)" is refuted: Bond cannot stand in the duct (the 3.86 roof) and the
///   duct's east wall (x 41.53) takes every bolt from it — the crossing now runs as the
///   documented stand-in (`the_crossing_stand_in`); the latch, reel and hang are pinned from a
///   clear stance in interactables_07000014.
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
    // Leg A: the old crossing stance (40.8, floor 2.40, 65.5). The Q-Claw fires only from a
    // type-2 third icon's arm (`BL+0x95F == 2`) or in MP [M Player_WeaponFiring 0x1a5460:515-528 —
    // the fire-held arm RETURNS before `Player_RoundToFire`]; the stance is 5 units from the
    // nearest type-2 icon (the trench-floor one under the SkyBridge point, (45.27, 0.41, 62.94)
    // [D icon census]), so the press does nothing: no latch.
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the old crossing stance");
    w.scene.sim.as_mut().unwrap().bond.crouched = false;
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.icon_ctx, 0xff, "no third icon arms in the vent store room [D]");
    let point = [45.2888f32, 5.0961, 62.9362]; // [D placement; the port's resolution, probe14_aim]
    assert!(!claw_aim_and_fire(&mut w, point, 30),
        "the Q-Claw does not fire outside a type-2 icon's arm [M 0x1a5460:515-528]");
    // The documented hop (see walk_infiltration_to_handback's Leg 1a) onto the trench floor.
    hop_into_the_trench(&mut w);
    // Leg B: the trench east to the corridor — the nav lane node 5 -> 4 [D], floor -0.60.
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor east leg: {p:?}");
        assert!(p[0] > 47.0, "the trench/corridor east made real progress [D nav nodes 5/4], got {p:?}");
    }
    corridor_to_the_stairwell_foot_and_back(&mut w);
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 52.0).r(0.7), "the corridor south past z 52");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 44.0).r(0.7), "the corridor south to the mouths' lane");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] mouths' lane: {p:?} in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
        assert!(p[2] < 45.0 && p[0] > 46.0, "the corridor south reached the mouths' lane [D nav node 3 -> (49.1, 0.4, 42.8)], got {p:?}");
    }
    // Leg E: the shaft entry from the mouths' icon arm — the claw fires there.
    claw_up_from_the_mouths_arm(&mut w);
}

/// Legs C/D: the corridor south to the stairwell lane (z ~52), the jump WEST over the pit band
/// onto the -0.91 landing (the east stairwell foot, pinned), and the edge-anchored jump back east
/// onto the corridor. Shared by the east-route test and the infiltration walks.
fn corridor_to_the_stairwell_foot_and_back(w: &mut Walker) {
    // Leg C: the corridor south to the stairwell lane (z ~52), then jump WEST over the pit band
    // (x 46.5..47.5 at -2.40 [D probe14_corridor]) onto the -0.91 landing.
    agent_clear_blockers(&mut *w, [51.0, 52.5], 6.0, "the corridor south to the stairwell lane");
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
        // The y cap carries fight-pause slack: the real-health run's return-fire stand
        // on the corridor leg (drone #0, 18 health) shifted the climb's tick phase, and the
        // climb_stairs arrival (dxz 0.8) sampled Bond standing on the landing's first step —
        // y 0.1234 against the old 0.0 cap (walkreal_07000014_real_damage, 2026-11-02); the
        // The cap carries fight-phase slack from both sides of the merge: the real-health
        // runs' return-fire stands and the game's FirstAttack flush [M 0x148010:100-131]
        // (the trench-guard fight at the landing) sample the climb's arrival above the old
        // 0.0, and the agent-difficulty run's own fight phases sampled y 0.405
        // (real-health-agent-difficulty-12-13-14-15, 2026-11-03) — 1.1 covers both. The
        // x/z bounds and the y > -1.6 pit floor are the pins and stay.
        assert!(p[1] > -1.6 && p[1] < 1.1 && p[2] > 52.5 && p[2] < 54.6 && p[0] > 44.0 && p[0] < 46.4,
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
}

/// The one documented reach-a-spot hop of the infiltration: Bond onto the trench floor at the old
/// SkyBridge crossing's landing (nav lane node 5 (43.05, 0.4, 62.86) [D]). The port has no
/// walkable exit from the vent store room (docs/levels/07000014.md §1a), and the claw cannot
/// fire there [M 0x1a5460:515-528; no type-2 icon in the room, D icon census].
fn hop_into_the_trench(w: &mut Walker) {
    // guard-standing-capsule-2: the trench's own guard (drone 0, 0x5000090) patrols the landing
    // itself — the dump caught him at (42.49, 0.40, 62.82), 0.56 from the pad, standing capsule
    // top 0.80 over it [L this session's dump]. The pre-push walk landed THROUGH his body; the
    // game's Jointy push [M 0x1ec180] now shoves the landing 1.13 east onto the crate lip (the
    // red run: (44.18, 0.03, 62.91) grounded). A player waits him off the pad: hold until no
    // live drone stands within 1.4 of it (his patrol walks him clear), then hop.
    {
        let start = w.total_ticks;
        loop {
            let occupied: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.present && !d.corpse && d.enabled
                        && walker::dxz(d.pos, [43.05, 0.0, 62.86]) < 1.4)
                    .map(|(i, _)| i).collect()
            };
            if occupied.is_empty() {
                eprintln!("[014] the trench landing cleared after {} ticks", w.total_ticks - start);
                break;
            }
            assert!(w.total_ticks - start < 9000, "the trench landing never cleared (drones {occupied:?})");
            w.wait(1);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.present && !d.corpse && walker::dxz(d.pos, [43.05, 0.0, 62.86]) < 4.0 {
                eprintln!("[014] near-landing drone {i} skin {:#x} team {} at {:?} y {:.2} state {:#x} enabled {}",
                    d.skin, d.team, d.pos, d.pos[1], d.state, d.enabled);
            }
        }
        let mut solid: Vec<String> = Vec::new();
        for z in [62.0f32, 62.9, 63.5] {
            for x in [43.0f32, 44.0] {
                if let Some(h) = sim.world.floor_below([x, 2.0, z], 2.5) {
                    solid.push(format!("floor({x},{z})={h:.2}"));
                }
            }
        }
        eprintln!("[014] landing neighbourhood floors: {}", solid.join(", "));
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [43.05, 0.45, 62.86];
        sim.bond.pre_collide = sim.bond.pos;
        sim.bond.vel = [0.0; 3];
        sim.bond.crouched = false;
        sim.bond.yaw = std::f32::consts::FRAC_PI_2;
        sim.bond.pitch = 0.0;
    }
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
    let b = &w.scene.sim.as_ref().unwrap().bond;
    assert!(b.grounded && b.pos[1] < 1.0 && (b.pos[0] - 43.05).abs() < 1.0,
        "the hop lands Bond on the trench floor (-0.60 [D probe14_room]), got {:?} grounded {}", b.pos, b.grounded);
    eprintln!("[014] the trench hop landed {:?}", b.pos);
}

/// Aim the head ray at `point` (yaw and pitch steered on the stick) and hold fire for up to
/// `ticks`; true once the claw latches.
fn claw_aim_and_fire(w: &mut Walker, point: [f32; 3], ticks: u32) -> bool {
    for i in 0..200 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let h = b.head();
        let (dx, dy, dz) = (point[0] - h[0], point[1] - h[1], point[2] - h[2]);
        let yaw = dx.atan2(dz);
        let look = dy.atan2((dx * dx + dz * dz).sqrt()) / std::f32::consts::FRAC_PI_2;
        let inp = nf_game::player::Input {
            turn: wrap_pi(yaw - b.yaw).clamp(-0.08, 0.08),
            look: (look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
        if i > 150 { break; }
    }
    for _ in 0..ticks {
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { return true; }
    }
    false
}

/// THE SHAFT ENTRY from the mouths' icon arm: from the corridor's mouths' lane, west onto the
/// trench floor beside the east duct, inside the type-2 icon's arm under the mouths' grapple #626
/// (icon (45.30, 0.41, 42.89) r 1.28 [D icon census]; the arm is the pick test of
/// `Collide_Pick` 0x1eb320 → `ThirdIcon_Update` 0x196e80). The claw fires there [M 0x1a5460:
/// 515-528], the reel [M Player_SSGrapple 0x192860:79-121] lifts Bond through the duct's open
/// strip and hangs over the east vent lane; the drift south (fire held) and the release land him
/// on the lane (2.40).
fn claw_up_from_the_mouths_arm(w: &mut Walker) {
    // The stance sits between the trench grating's slats (z 42.39 and 43.39 take the bolt 1.66 up
    // [the walled-pick stance scan]).
    let _ = w.try_walk_to(Wp::new(45.2, -0.6, 42.95).r(0.15), "west into the mouths' icon arm");
    w.scene.sim.as_mut().unwrap().bond.crouched = false;
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        assert_eq!(b.icon_ctx, 2, "Bond stands in the mouths' type-2 icon arm [D icon census], at {:?}", b.pos);
    }
    let mp = [45.30f32, 4.79, 42.86]; // [D grapple #626; the level table]
    assert!(claw_aim_and_fire(w, mp, 40), "the Q-Claw latches the mouths' grapple #626 from the icon arm [M 0x1a5460:515-528; D placement]");
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
    let on_lane = |w: &Walker| {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let p = b.pos;
        b.grapple.is_none() && b.grounded && p[1] > 1.8 && p[1] < 3.6 && p[2] < 48.4 && p[0] > 44.0 && p[0] < 47.0
    };
    // The hang from this side already sits over the lane's open strip (z 42.3..43.5 [D the
    // down-ray grid]) — no drift: the straight release drops onto the lane.
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p[2] > 42.3 && p[2] < 43.5, "the hang sits over the lane's open strip, got {p:?}");
    }
    // The release: the first no-fire tick detaches [M 0x192860:56-66].
    let mut lane = false;
    for _ in 0..400 {
        if on_lane(w) { lane = true; break; }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = nf_game::player::Input {
            turn: wrap_pi(std::f32::consts::PI - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[014] the vent lane: {p:?} lane {lane}");
    assert!(lane, "the release lands Bond on the east vent lane (2.40 [D probe14_tunnel A]), got {p:?}");
}
// level-14-vent-drop-2: the vent-drop hand-back is PORTED and pinned (see
// the_vent_drop_hands_bond_into_the_facility, green): nis.rs's event-6 fallback captures the
// scripted Bond body's world root (s18, script 0x06000857; the pack 0x06000833 has NO marker
// entity stream [D its full stream dump], so the strict previous-stream capture finds nothing and
// the game still carries Bond in [D walk-4 walls/nav]) and Script_Free's Player_Enable lands him
// inside the facility at the body's event-time root. [S] unverified game-side (the live check in
// docs/reports/level-14-vent-drop.md).
// NOT ignored — the "STILL #[ignore]" note below was STALE (fixed level-14-walk-to-the-level-end):
// this helper is no test and carries no #[ignore]; both tests that share it
// (the_vent_drop_hands_bond_into_the_facility, walk_the_infiltration_to_the_delta_exit) are
// un-ignored and green. The old leg-2 wedge at the facility's west double door (−52.2, 1.2, 40.1;
// two records, ch_lock 0x85) was the walk's own wrong premise, not a port gap: Door_Slide_11A/B
// (plc 151/152, ch_lock 0x85 [D placements 151/152]) is SEALED until the sabotage cascade's death
// channel 0x85 unlocks it (auto-open on the rising edge [M 0x1363c8:88-107]), and the game reaches
// the west wing its own way — the SOUTH loop (the nav web's west corridor, which never crosses the
// 0x85 plane [D nav path 0]) to the Kiko touch #381, whose cut-scene 0x0600062F carries Bond west
// past the closed leaves; the 0x85 doorway is crossed only AFTER the cascade unlocks it
// (the delta-exit walk's legs 2-4).
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

    // Leg 1a: into the trench. The Q-Claw fires only from a type-2 third icon's arm
    // (`BL+0x95F == 2`) or in MP [M Player_WeaponFiring 0x1a5460:515-528]; the vent store room
    // carries no such icon (the three type-2 icons sit on the trench floor under the three
    // grapple points [D icon census]), so the old crossing — the claw fired from (40.8, 2.4,
    // 65.5), 5 units from the nearest icon — is no move a player can make. The store room's
    // walkable exit is not found in the port (docs/levels/07000014.md §1a: the grill closes the
    // south duct; the east wall stands at x 41.53 the full duct height); this is the file's one
    // documented reach-a-spot hop: onto the trench floor at the old crossing's landing.
    hop_into_the_trench(w);
    kill_the_trench_guard_270(w);
    // Leg 1b: the trench east to the corridor — the nav lane node 5 -> 4 [D], floor -0.60.
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[014] corridor east leg: {p:?}");
        assert!(p[0] > 47.0, "the trench/corridor east made real progress [D nav nodes 5/4], got {p:?}");
    }
    // Leg 1c: the corridor south, the stairwell foot and back, then on south to the mouths' lane
    // [D nav node 3 -> (49.1, 0.4, 42.8)].
    corridor_to_the_stairwell_foot_and_back(w);
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 52.0).r(0.7), "the corridor south past z 52");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 44.0).r(0.7), "the corridor south to the mouths' lane");
    // Leg 1e: THE SHAFT ENTRY from the mouths' icon arm.
    claw_up_from_the_mouths_arm(w);
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
        // The hand-back, RE-PINNED (level-14-skip-handback): the old assert pinned the port's [S]
        // body-root arm (level-14-vent-drop) — a stand-in this file's own comment marked unverified
        // ("the capture is the port's [S] — the report's live check 1"). The decompile has no third
        // arm: `Script_EventHandler` 0x1d94c0 case 6 captures the PREVIOUS stream's object
        // (kinds 1|2) at obj+0x90, ELSE `glb_players+0x90` — Bond's own matrix
        // [M 0x1d94c0:73-84,125-134]; `Script_Free` 0x1da880:44-48 Player_Enables si+0x40 verbatim
        // [M Player_Enable 0x195498:38-52]. This pack's event 6 (stream 14) sits after the
        // Wait/Marker stream 13 (kind 0 [D the pack dump; NF_SCRIPTLOG "prev kind Some(0)"]), so
        // the game hands Bond back WHERE THE SCENE GRABBED HIM — the #301 touch volume
        // (39.68..41.75, 2.07..4.14, 37.62..40.40 [D]) — mezzanine height, both on the natural end
        // and on a skip (the skip's catch-up 0x1da998:19-100 runs the same op 0x12; the
        // case-2/4 freeze 0x1d9a08:146,205 cannot change a no-prev-entity capture). The old [S]
        // poses (the seq root (27.60, 0.98, 43.56); the frozen spawn root (27.84, 0.94, 49.96) on a
        // skip — the real play's in-wall (26.70, 1.03, 36.27, yaw 2.122) @frame 3046) are the BUG
        // this pin rejects.
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p[1] > 2.0 && p[1] < 4.14 && (39.0..43.0).contains(&p[0]) && (37.5..40.5).contains(&p[2]),
            "the vent cut-scene hands Bond back at his own matrix — the #301 grab spot on the mezzanine [M 0x1d94c0 case 6 fallback glb_players+0x90; 0x1da880; 0x195498], got {p:?}");
        eprintln!("[014] hand-back at {p:?}");
    }
}

/// The pad's south frame (x 33.71..35.27, z 35.90..36.00, sill 2.10, head 3.64 [D static tris])
/// holds a vent lid: the 0x06000A3B lid prop 0x02000750 (the lid SPs' script [D
/// docs/levels/07000014.md]) hangs in it at z 35.93, y 2.15..3.59 — solid at its posed matrix like
/// every script entity [M Script_CreateEntity 0x1d8968 → 0x1ca6d8]. E on it plays its SP [M
/// SP_Activate 0x1dc338 → ext+0x2C; SP_Update 0x1dbda8 replays the script] and the lid swings up
/// to y 3.56..3.61 (measured on master and on this tree). The walk used to open it by luck: the
/// walker's stuck reaction aimed its E at door #834 (2.4 away) and the old sphere-only pick caught
/// the lid as well. The use pick keeps an object only when the use point lies within 1.0 in FRONT
/// of one of its collision triangles [M Collide_Intersect 0x1ebaa8:186-189 → Intersect_PointGeom
/// 0x1e92d8; crate::usepick], so that stray press no longer opens it and the walk wedged on the
/// closed lid (the 6000-tick cap at (34.31, 2.75, 36.49)). A player opens it the plain way: stand
/// ~1.4 north of the lid, look at it, press E. A no-op when the frame is already clear.
fn open_the_pad_lid(w: &mut Walker) {
    let shut = |w: &Walker| w.scene.sim.as_ref().unwrap().world.dynamic_boxes().iter()
        .any(|(_, lo, hi)| lo[1] < 3.0 && hi[1] > 2.6 && lo[0] < 35.0 && hi[0] > 34.0 && lo[2] < 36.0 && hi[2] > 35.85);
    if !shut(w) { return; }
    w.walk_to(Wp::new(34.5, 2.75, 37.3).r(0.25), "north of the pad's vent lid");
    let lid = [34.49f32, 2.87, 35.96];
    for i in 0..240u32 {
        if !shut(w) {
            eprintln!("[014] the pad's vent lid opened after {i} ticks at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
            return;
        }
        let (dy, dp) = {
            let b = &w.scene.sim.as_ref().unwrap().bond;
            let h = b.head();
            let d = [lid[0] - h[0], lid[1] - h[1], lid[2] - h[2]];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
            (wrap_pi(d[0].atan2(d[2]) - b.yaw), (d[1] / l).asin() / std::f32::consts::FRAC_PI_2 - b.pitch)
        };
        let aimed = dy.abs() < 0.02 && dp.abs() < 0.02;
        // E is edge-triggered: one press per 6 ticks once the view is on the lid.
        w.tick(nf_game::player::Input { turn: dy.clamp(-0.08, 0.08), look: dp.clamp(-0.08, 0.08), ..Default::default() }, aimed && i % 6 == 0);
    }
    panic!("the pad's vent lid never opened to E from {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
}

/// The drop loop's aim z: (31.8, 34.0) unless a live guard stands at the drop's foot. With the
/// game's sight round robin [M DroneVision_ProcessDroneSight 0x176a28] the low-floor room guard
/// (drone 5) can hold AimStandFire 0x68 at (32.25, 1.80, 33.58), on the steer line, and Bond's
/// capsule came to rest on his (grounded false at y 2.33, health 100 [L this branch's stuck
/// dump]) for the whole 2400-tick cap. The player drops past him on the north side, z 34.9.
/// The drop-band guard on the 1.72 landing's west drop, shot from the landing (every walk).
fn shoot_the_drop_band_guard(w: &mut Walker) {
    let _ = w.switch_to_armed("the drop-band guard");
    for _ in 0..3 {
        let di = { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.list.iter().position(|d| d.present && !d.corpse
            && d.health > 0.0 && d.pos[1] < 2.3 && (31.0..33.2).contains(&d.pos[0]) && (33.0..34.6).contains(&d.pos[2])) };
        let Some(di) = di else { break };
        let k = w.shoot(di, "the drop-band guard (from the landing)");
        eprintln!("[014] drop-band guard #{di} killed {k} health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
        if !k { break; }
    }
}

fn drop_aim_z(w: &Walker) -> f32 {
    let sim = w.scene.sim.as_ref().unwrap();
    // Real health: the drop-band guard is shot from the landing (below), and his body
    // lies in the band at (32.2, 33.2); the 34.0 line then wedges under the overhang at
    // (33.71, 1.20, 34.23) while the 33.4 line clears to the low floor [L the 11-13 runs].
    // Every walk shoots him now (his alt attack holds the band); his body falls up to x 33.3.
    if sim.drones.list.iter().any(|d| d.corpse
        && (31.0..33.6).contains(&d.pos[0]) && (32.8..34.6).contains(&d.pos[2])) { return 33.4; }
    let blocked = sim.drones.list.iter().any(|d| d.present && !d.corpse && d.health > 0.0 && d.pos[1] < 2.3
        && (31.0..33.2).contains(&d.pos[0]) && (33.0..34.6).contains(&d.pos[2]));
    if blocked { 34.9 } else { 34.0 }
}

/// From the low machine floor to the leg-2 spot on the facility corridor. The old legs stepped
/// west over the sill through the window between the machine room and the corridor; that window
/// is a live glass pane (the breakable's collision x 30.62..31.27, y 1.34..2.97, z 30.82..35.68),
/// solid to the body queries while alive [M Break_Kill 0x12b970:60; Collide_PickObj 0x1eae80],
/// and its filter word !0x20 drops every gun round [D placement param 2; M Collide_FilterBullets
/// 0x1ece30 — measured: 35 pistol rounds, health 1.0 kept]. The room's way out is the AI path
/// [D nav path 0]: nodes 132 (34.01, 33.21) → 131 → 130 → 129 (34.72, 29.30) south through the
/// gap, 360 (32.25, 27.33) → 359 (30.73, 27.33) west, 146 (28.31, 29.27) → 57 → 265 → 58
/// (28.32, 36.45) north up the corridor lane, then the leg-2 spot.
fn the_machine_room_to_the_corridor(w: &mut Walker) {
    let lane: &[(&str, f32, f32, f32, f32)] = &[
        ("the machine room floor (node 132)", 34.01, 0.65, 33.21, 0.8),
        ("the room's south gap (node 130)", 34.72, 0.65, 30.47, 0.6),
        ("south of the gap (node 129)", 34.72, 0.65, 29.30, 0.7),
        ("the south row west (node 360)", 32.25, 0.65, 27.33, 0.8),
        ("the south row west (node 359)", 30.73, 0.65, 27.33, 0.8),
        ("the corridor foot (node 146)", 28.31, 0.65, 29.27, 0.8),
        ("the corridor lane north (node 57)", 28.31, 0.65, 30.74, 0.8),
        ("the corridor lane north (node 265)", 28.32, 0.65, 33.67, 0.9),
        ("the corridor lane north (node 58)", 28.32, 0.65, 36.45, 0.9),
    ];
    // The room guard on the low floor (placement #284) stands in the lane and fires at Bond as he
    // comes down off the landing (AIM_STAND_FIRE 0x68 on arrival): the player shoots him first.
    // The Q-Claw is still out from the duct: the hand gun 0x2 goes up first [L loadout: owned at
    // the start].
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x2), ..Default::default() }, false); }
    clear_hostiles(w, 6.0, 900);
    // capsule-join-5: the guards the fights wake come in through the south gap's door
    // (34.0, 0.0, 29.9) and stand IN the gap — #10 AimStandFire 0x68 at (33.92, 30.85), #1/#2
    // 0x5c at 1.4-1.9 u [L this branch's wedge dump] — and their standing bodies push Bond
    // off the gap [M NDrone2_Collision 0x140e90:91-121 via 0x149ec8/0x14b190]; the plain ladder
    // re-opened the door for the whole 6000-tick cap. The player's answer to a guard in his way
    // is the shot (Walker::stuck_answers, the 11 route walk's opt-in), scoped to this lane.
    let prev_answers = w.stuck_answers;
    w.stuck_answers = true;
    for (label, x, y, z, r) in lane {
        w.walk_to(Wp::new(*x, *y, *z).r(*r), label);
        // retune-nb-14-3 (00 Agent real health): the descent corridor's first-round tax — the
        // fights answer every shooter only after his first round (the v10 ledger: #9 12.5/5,
        // #11 12.5/5, #8 5.0, #7 2.5, #3 5.0, #10 2.5 — 40.0 of the walk's 100.0, t5607-7640
        // [L]). A player stops at the corridor foot and answers the lane's posts BEFORE their
        // first rounds (the route scan's law, the ignore message's lever 1). From the foot the
        // remaining route line runs north to the leg-2 spot, so the scan's road-blocker rank
        // works; #11's post (26.5, −4.1) sits d 33 — outside the scan's 20 u, left to the legs'
        // own fights as shipped. A stop with no gun-true candidate fires no fight and burns no
        // tick (the candidate dump is read-only), so this is a no-op where the rays die — the
        // candidate dump measures the lane either way. Gated to this walk's difficulty; every
        // other caller (the d1/d2 walks, the tripbomb route) is untouched by construction.
        if label.starts_with("the corridor foot")
            && w.scene.sim.as_ref().unwrap().drones.difficulty >= 3
            && !w.tops_up()
        {
            route_scan(w, [30.09, 0.65, 38.73], "the descent corridor foot scan");
        }
    }
    w.stuck_answers = prev_answers;
}

/// The walked west route from the SP #276 hand-back to the west switch band (level-14-hall-walk-in):
/// shared by the full walk and the cascade test — the descent to the facility corridor, leg 2 (the
/// SOUTH loop [D nav nodes 94-97/150/591-600; it never crosses the closed 0x85 door plane [D
/// probe]] to the #381 band, the Kiko scene delivering Bond west past the closed door), leg 3a
/// (the generator room, nav nodes 613/614/625/615/623/628/631/663/654/653 [D]), leg 3b (the two
/// fuseboxes burned through the REAL Laser path [M FuseBox_Update 0x1adfe8]: #383 p1 0x87, #385
/// p1 0x88 [D placements]; AND #391 raises 0x8C [D]) and leg 3c's arrival walk into the E-use
/// switch band (oDroneTrigger1b, out 0xA0 mask 4 [D placement 399]) — WITHOUT the E press (the
/// callers own the aim + press).
fn walk_the_west_route_to_the_switch_band(mut w: &mut Walker) {
    // The descent (level-14-mezzanine-descent-2): the [M] hand-back stands Bond on the mezzanine
    // where the scene grabbed him (his own matrix [M 0x1d94c0 case 6 fallback]). The WALKED route:
    // the pad west ((34.63, 2.80, 37.94) [L walked, level-14-mezzanine-descent]) → south
    // ((34.57, 2.80, 37.18)) → the 0.43 step down onto the 1.72 landing ((33.09, 2.37, 35.38)
    // grounded [L walked]; the vent lid in the pad's south frame opened by E first,
    // `open_the_pad_lid`) → west off the landing's edge onto the low machine floor (ray height
    // 0.80, x 31.5..33.5, z 33..35.5 [D rays]) → out of the machine room by the AI path south and
    // up the corridor lane (`the_machine_room_to_the_corridor`; the window west of the room is a
    // live glass pane) → the leg-2 start.
    {
        w.walk_to(Wp::new(34.63, 2.80, 37.94).r(1.0), "the pad west");
        w.walk_to(Wp::new(34.57, 2.80, 37.18).r(0.9), "the pad's south-west step head");
        open_the_pad_lid(&mut w);
        w.walk_to(Wp::new(33.09, 2.37, 35.38).r(0.8), "the 1.72 landing");
        // The drop west: the low floor's east margin sits under the landing slab's overhang (the
        // slab top is the 1.72 landing; the walk's grounded wedge at (33.68, 1.15, 35.20) with a
        // clear segment to the target is the overhang catch), so the westing is the player's own
        // push: steer south-west around the slab's corner, hop when grounded (the walker's wedge
        // recovery), until clear of the overhang zone on the low floor.
        // Real health: a guard (00 Agent #6 / Operative #5, 28 health, 0x14) stands asleep in the
        // drop band at (32.2, 1.8, 33.2); the drop steers into the overhang around him and wedges, and
        // walking past wakes him (point-blank death). Bond still holds the vent-lid item here, so
        // the pistol comes out first, then the guard is shot from the landing (killed, no hits
        // taken [L the 11-13 run]). Topped-up walks shoot him too: awake within 1.5 of Bond he
        // takes the close-range alt attack from his fire states and holds the drop band
        // [M NDrone2_CanAltAttack 0x1549e0 from AimStandFire 0x166328 / AimCrouchFire 0x167060].
        shoot_the_drop_band_guard(&mut w);
        {
            let start = w.total_ticks;
            loop {
                assert!(w.total_ticks - start < 2400,
                    "the landing's west drop never cleared the overhang\n{}",
                    walker::fail_dump_pub(&w.scene, [31.8, 1.0, 34.0], "the landing west drop"));
                {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let p = sim.bond.pos;
                    if p[0] < 32.6 && p[1] < 1.4 && sim.bond.grounded { break; }
                }
                let tz = drop_aim_z(&w);
                let sim = w.scene.sim.as_ref().unwrap();
                let b = &sim.bond;
                let inp = nf_game::player::Input {
                    forward: 1.0,
                    jump: b.grounded && (w.total_ticks - start) % 30 == 0,
                    turn: wrap_pi((31.8f32 - b.pos[0]).atan2(tz - b.pos[2]) - b.yaw).clamp(-0.08, 0.08),
                    ..Default::default()
                };
                w.tick(inp, false);
            }
            eprintln!("[014] the low floor west reached {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        }
        // retune-nb-14-7 round 7, lever 3 (the round-4 census's untried loop-mouth armour — the
        // ledger's "(32.26,0.87,31.14) d 7.2 from the loop mouth, the only one west of the 0x85
        // door"): the walk arrives at the door face with health 1.9 and armour 0.0, and the
        // whole back half of the bill is 0x14 BULLETS (the 12.5 rifle trios, the pocket #20's
        // 1.93) that the armour pool absorbs FIRST — bullet pain (type 0) is the ONLY pain type
        // whose armour fraction lets armour absorb [M Player_HandlePain 0x1902c8:84-104; apply():
        // absorbed = (d·frac).min(armour)]; the trace's one health-drain explosion is #33's
        // weapon-0x0 grenade −15.57 [L the NF_DMG_TRACE runs]. The armour sits ~2 u south of
        // this measured low-floor spot — a real-input detour, the take on contact, no teleport.
        // d3 real-damage only: the sisters and the tops-up walks keep the shipped route
        // byte-identical.
        if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() {
            w.walk_to(Wp::new(32.26, 0.9, 31.14).r(0.9), "the loop-mouth armour fetch");
        }
        the_machine_room_to_the_corridor(&mut w);
        w.walk_to(Wp::new(30.09, 0.65, 38.73).r(1.2), "the corridor north to the leg-2 spot");
        {
            // The landing pin samples the WALK's arrival. The hostile clear after it (the
            // route's own running fight before leg 2) legitimately repositions Bond — the
            // merged-tree d1 run arrived (29.79, 0.56, 37.59) in-band and the 10-fight clear
            // then dragged him to (25.97, 0.65, 36.40) [L this tree's ledger] — and must not
            // eat the pin.
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            assert!((27.0..33.5).contains(&p[0]) && p[1] > 0.5 && p[1] < 1.6 && (36.5..40.5).contains(&p[2]),
                "the descent lands on the facility corridor floor at the leg-2 start (the walked route, probe landing (29.99, 0.61, 38.00)), got {p:?}");
            eprintln!("[014] the descent landed {p:?}");
        }
        // The fight can pull Bond off the spot; leg 2 starts from it.
        if !w.tops_up() {
            // retune-nb-14-2 lever (00 Agent real health): the corridor leg runs on AMMO, not
            // on stands. Measured: the shipped clear's own fights were worthless here (the
            // pocket post rays die on the junction geometry — the scan's candidates all read
            // gunfalse from the leg-2 spot; fight 1 ran dry at +740, fight 2 landed ZERO in
            // 900 ticks) and its loot pass wedged ~18000 ticks on the closed junction door
            // (31.5,1.2,26.9); the shipped walk carries clip 3+20 out of the descent. The
            // walk legs' OWN return-fire fights are the strong killer (the v6 run: #9/#8/#7/
            // #5/#3/#12/#10/#11 killed during the approach walks) — but the v5/v6 runs entered
            // the corridor DRY and the fights pinned the ascent top "killed false" x8 for
            // 2000 ticks (t12270-14640) while #2/#1/#17 shot the stand dry — the d1 chain's
            // dry-fight law. The v4 clear's loot pass measured the answer: the drop at
            // (28.21,0.65,31.13) re-armed the gun clip 3+20 → 21+31. The fetch does that on
            // purpose, one bounded leg, no fights, no wedge; every other difficulty keeps
            // the shipped clear byte-identical.
            if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 {
                // retune-nb-14-3: the nearest-live-pickup fetch measured DEAD on its own run
                // (v11): the nearest pickup inside 8 u sat at (32.26, 31.14) BEHIND the shelf
                // rim — the scooping walk wedged 2.05 short at the 6000-tick cap (the clear's
                // loot-pass wedge shape, the 11-13 ledger's "drop 2.38 away behind the rim,
                // unclosable"), after dragging Bond through the junction fight zone. The
                // shipped fixed v4 spot stays: its arrival (28.33, 31.60) takes nothing in
                // this stream (clip 3+6 at node 24 [L v10]) — the walk is a no-op that costs
                // no fights — and the loop's own resupply law keeps its shipped shape.
                // retune-nb-14-3 v14: the fixed v4 spot takes nothing in this stream (the
                // v10/v12 arrivals) — the walk legs' OWN descent kills leave their drops on
                // the upper corridor (#8's drop read 4.61 u from the leg-2 spot in v13, the
                // unclosable machine-room pickup 7.89 u — outside the 4.5 band), so the
                // fetch becomes the same panic-safe 4.5-u scoop the loop carries.
                for j in 0..3 {
                    let bp = w.scene.sim.as_ref().unwrap().bond.pos;
                    let t = w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|p| !p.gone && !p.hidden)
                        .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                        .filter(|q| walker::dxz(*q, bp) < 4.5 && (q[1] - bp[1]).abs() < 1.5)
                        .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)));
                    let Some(t) = t else { break };
                    w.try_walk_to(Wp::new(t[0], t[1], t[2]).r(0.4), &format!("the corridor drop fetch {j}"));
                }
            } else {
                clear_hostiles(w, 24.0, 900);
                w.walk_nav_to([30.09, 0.65, 38.73], "back to the leg-2 spot", 1.2);
            }
        }
    }

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
    // retune-nb-14-3 (00 Agent real health): the loop walked NODE-BY-NODE (follow_nav_route's
    // own route read + the same r(1.3) legs — the wake-legs' byte-identity idiom) with
    // VISIBILITY-KEYED route_scan stops where the next posts are gun-visible BEFORE their first
    // rounds (the ignore message's lever 1): the shipped single call's fights answer every
    // shooter only after his first round, and the v10 ledger's windows are exactly those first
    // rounds (the node-40/41 window #14 15.0 + #15 7.5 at t11897-11961 — the pre-door fight read
    // #14's rays door-blocked and burned its cap; the node-64 wedge #18 20.0 + #19 2.5 at
    // t14190-14418 [L]). Stops keyed on the MEASURED node positions (the v10 arrivals), not on
    // indexes — the route re-reads from wherever the scoops ended. A stop with no gun-true
    // candidate fires no fight and burns no tick (read-only), so the stream stays byte-identical
    // until the first real fight. Other difficulties keep the shipped single call byte-exact.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() {
        // retune-nb-14-4 (v16, the fights' FEET TIED across the south loop): the master merges
        // (22d5836b7 the Exchange weapons, e4fb1c512 the weapon-loop substate stop) re-rolled
        // this stream and the loop's junction fights now backpedal Bond onto the level's hover
        // cells — the baseline run on this tree wedged node 5's leg at the 6000-tick cap with
        // bond (32.47,0.66,24.51) grounded=false and every attacker ALREADY dead (25 hits for
        // 72.5, 19 return-fire fights, the nearest shooters #1/#2/#9/#11 all hp 0, "drones
        // nearby: none") [L 2026-11-19, this tree's real-input baseline]. The v15 lesson named
        // the same shape; the field doc carries the law: "d1, whose stands never move Bond,
        // lands the slab every run — the displacement, not the collision, is what puts Bond in
        // the pocket" (descent_stationary_fights, level-14-landing-hover lever 2; the wedge
        // evidence s/real-health-14-00agent-5 [L]). The flag ties the fights' stand feet (no
        // back_off shove, no creep, no lane-resupply fetches — a dry gun walks on, the next
        // node's own scoop re-arms) while the pick and aim laws stay the d3 set (the gun-ray
        // gate, the damage-weighted rank, the head-hunt). The level-13 stretch ships the same
        // scoped idiom (walkreal_07000013_live's north-west stretch, restored after the drain).
        // Scoped to the south loop only: restored before the north leg, so the door and Kiko
        // fights keep the shipped back_off machinery. d3 only — the sisters never set it.
        w.dmg.descent_stationary_fights = true;
        pickup_census(&w, "the south loop start");
        // retune-nb-14-4 (the round-4 measurement record, see also the (e) block below): the
        // machine-room armour at (32.26, 0.87, 31.14) — d 7.2 from this spot, the only kind-3
        // pickup WEST of the 0x85 door — is UNCLOSEABLE from the loop's lanes: v18's fixed
        // north approach latched the walker onto the mezzanine door (32.0, 2.1, 35.8) (y 2.1 —
        // the room's ground entry is the south door the route crosses), and v19's junction
        // approach wedged 2.83 short on the shelf rim (grounded=false, the v11 shape). The
        // armour pool is exactly what the door chain needs: v16/v17 reached the door's west
        // face with 17.5 health and clip 10 + spare 89 (ammo NOT binding), and the door squad
        // finished those walks [L 2026-11-19]. The reachables: the Kiko-corridor armour
        // (−41.29, 1.53, 36.48) — live at the door-face census, d 19.1, ~3.4 u from the east
        // lane's end — and nothing else west of the door.
        let loop_pts = {
            let sim = w.scene.sim.as_ref().unwrap();
            nav_route(sim, w.cur_pos(), [-48.7, 0.9, 33.2])
        };
        match loop_pts {
            Some(pts) => {
                for (k, p) in pts.iter().enumerate() {
                    let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3),
                        &format!("the south loop to the west corridor node {k}"));
                    // (a) the door (−23.0, −19.6) crossing — the node just past it (the v10
                    // node-40 arrival (−22.49, −19.97)): the shipped stream's only pre-burst
                    // fight fired from BEFORE the door and read #14's rays door-blocked
                    // (killed false, the cap burned); from the past-door spot the rays to
                    // #14 (−24.5, −17.3, d 3.4) are clear and the scan pre-empts #14 (hp 10)
                    // and #15 (−23.2, −11.0, d 9.0, hp 4) before their bursts.
                    if p[0] > -23.5 && p[0] < -21.5 && p[2] > -21.0 && p[2] < -19.0 {
                        route_scan(&mut w, [-48.7, 0.9, 33.2], "the door-crossing scan");
                    }
                    // (b) the ascent base (the x −30 lane at z 9-11 — the ignore message's own
                    // lever-1 spot): the ascent cluster's posts #16/#17 (−30.2,17.1)/
                    // (−31.6,17.5) read d 5.7-6.9 up the lane; the retune-nb-14-2 notes
                    // measured the ascent cluster's rays up this lane from here.
                    if p[0] < -29.5 && p[0] > -31.0 && p[2] > 8.5 && p[2] < 12.0 {
                        route_scan(&mut w, [-48.7, 0.9, 33.2], "the ascent-base scan");
                    }
                    // (c) the ascent top (the lane's last node, the v10 arrival (−30.34,
                    // 22.93)) BEFORE the node-64 leg's dry resupply walks Bond INTO #18's post
                    // (−29.9, 32.5, d 9.6) and wedges ~12000 ticks on the (−30.6, 29.9) door —
                    // the v10 window where #18 dealt 20.0 point-blank (t14387/14418) and #19
                    // (−40.3, 27.3, d 10.9) 2.5 [L]. The rays to #18 cross the (−29.9, 23.7)
                    // door plane — the candidate dump measures whether they clear it.
                    if p[0] < -30.0 && p[2] > 22.0 && p[2] < 24.0 {
                        route_scan(&mut w, [-48.7, 0.9, 33.2], "the ascent-top scan");
                    }
                    // (d) the dry-gun scoop (the d1 wake legs' shipped law, keyed on the dry
                    // gun instead of a fixed node): the loop's own fights kill the junction
                    // trio and the corridor guards ON the lane, and their drops re-arm the
                    // gun mid-loop. Measured without it (v12): clip 3+0 at node 41 → the
                    // ascent-base fight ran dry on #16 (killed false, 3 rounds) → no dry
                    // fight ever fired again → the node-64 resupply never triggered → the
                    // walk crossed the ascent and the hall on clip 0+0 and reached the door
                    // face DRY ("the switch band's guards: cycling to a gun (0)") — the door
                    // fights had nothing to fire and #34's 27.5 burst finished the 30.0-health
                    // walk during walk 2 [L v12]. The scoop takes the loop kills' drops
                    // (v10's #14 drop 2.3 u from node 41's arrival, #15's 2.6 u), no fights
                    // here — no scan-noise re-rolls (the d1 chain's own note). Real pickups
                    // through the game's own take path.
                    {
                        let dry = { let sim = w.scene.sim.as_ref().unwrap();
                            sim.gun.clip + sim.gun.spare < 10 };
                        // the PEACE gate (v14's lesson): the scoop walks run the walk's own
                        // fight machinery, and a dry gun mid-scoop fires the fights' resupply
                        // — v14's node-2 scoop was dragged into #1's convergence zone (45.0 in
                        // 10 hits, death t9879). A player scoops drops only where nobody
                        // lives: no live team-1 guard within 12 u.
                        let peace = { let sim = w.scene.sim.as_ref().unwrap();
                            let bp = sim.bond.pos;
                            !sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse
                                && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 12.0) };
                        if dry && peace {
                            // 4.0 u only: the measured drops this scoop exists for sit 2.3-3.8 u
                            // off the node arrivals (#14/#15's at node 41, the trio's ON the
                            // nodes they die on); the machine-room's unclosable pickup
                            // (32.26, 31.14 — the v11 fetch's 6000-tick wedge) reads 4.0-4.7 u
                            // from the loop's early nodes and must stay OUT. try_walk_to (the
                            // panic-safe walk) so a bad pickup costs a cap, not the run.
                            for j in 0..5 {
                                let bp = w.scene.sim.as_ref().unwrap().bond.pos;
                                let t = w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|p| !p.gone && !p.hidden)
                                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                                    .filter(|q| walker::dxz(*q, bp) < 4.0 && (q[1] - bp[1]).abs() < 1.5)
                                    .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)));
                                let Some(t) = t else { break };
                                w.try_walk_to(Wp::new(t[0], t[1], t[2]).r(0.4),
                                    &format!("the loop's dry-gun scoop {j} (node {k})"));
                            }
                            let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("back to node {k}"));
                        }
                    }
                    // (e) RETIRED (v19's measurement, kept as the ledger): the machine-room
                    // armour fetch keyed past the south door — the SHELF RIM blocks the
                    // junction approach too: the walk wedged 2.83 short (bond (30.72, 0.66,
                    // 28.76) grounded=false, the same hover-pocket shape as v11's 2.05-short
                    // north-west attempt), the take never fired, and the 6000-tick burn's
                    // stream shift died at the ascent base (t21534, 42 fights) [L 2026-11-19].
                    // The pickup is unclosable from the loop's lanes from BOTH axes.
                }
                w.walk_to(Wp::new(-48.7, 0.9, 33.2).r(1.3), "the south loop to the west corridor end");
            }
            None => {
                w.follow_nav_route([-48.7, 0.9, 33.2], "the south loop to the west corridor");
            }
        }
        // retune-nb-14-8 lever 1 (the round-7 ignore message's next lever, the round-4 census's
        // SERVER-ROOM armour (−27.29,0.87,30.96)): the [srv-probe] nav dump [L 2026-11-20] found
        // a per-path route from THIS spot to the armour (11 nodes, the z 27.3-27.8 east lane)
        // whose middle node (−33.13,27.33) sits d 0.53 from the escape touch #456 (−33.1,26.8) —
        // the cut-scene trigger the ledger orders the fetch to MIND. The legs below bypass it
        // north ((−33.1, 28.6), d_touch 1.8), take the armour on contact (the lever-3 loop-mouth
        // fetch's own take law: walk_to r(0.9), no teleport), and walk back to the corridor end
        // so the shipped north leg proceeds unchanged; descent_stationary_fights is still tied
        // here, so the fights' stands cannot shove Bond off the bypass line. try_walk_to on the
        // unknown-floor legs (panic-safe: a blocked spot costs a cap, not the run); the take leg
        // is the real walk_to. +50 here banks the door-face-stands + loop + pocket bullet bill
        // the v4 math measured (≈100 total, the loop-mouth 50 covers half) [M Player_HandlePain
        // 0x1902c8:84-104 — bullet pain (type 0) absorbs into armour first].
        {
            w.try_walk_to(Wp::new(-45.92, 0.4, 27.33).r(1.0), "the server-room fetch: down to the z-27.3 lane");
            w.try_walk_to(Wp::new(-43.22, 0.4, 27.77).r(1.0), "the server-room fetch: the z-27.7 lane east");
            w.try_walk_to(Wp::new(-39.55, 0.4, 27.77).r(1.0), "the server-room fetch: past #19's post lane");
            w.try_walk_to(Wp::new(-36.33, 0.4, 27.33).r(1.0), "the server-room fetch: the corridor mouth (d_touch 3.3)");
            w.try_walk_to(Wp::new(-33.1, 0.9, 28.6).r(0.6), "the server-room fetch: the escape-touch bypass north (d_touch 1.8)");
            w.try_walk_to(Wp::new(-29.92, 0.4, 29.39).r(1.0), "the server-room fetch: the room's east floor");
            w.try_walk_to(Wp::new(-29.93, 0.4, 31.61).r(1.0), "the server-room fetch: the room's north floor");
            // The v1 run wedged the direct east cut (bond (−29.12,30.95) → target (−27.29,30.96),
            // segment_to_target 0.35, the 6000-tick cap): the pocket is entered from the nav
            // route's own north node (−28.00,1.20,33.21) — the route the game's web builds for
            // this exact destination (the [srv-probe] dump's last two nodes) — then straight
            // south. The (−30.6,29.9) door stays shut (ch_open 0, pos01 0.00 — the v10 ledger's
            // door), so no west-line approach exists [L both runs 2026-11-20].
            w.try_walk_to(Wp::new(-29.24, 0.9, 33.20).r(1.0), "the server-room fetch: the pocket's west node");
            w.try_walk_to(Wp::new(-28.00, 1.2, 33.21).r(0.8), "the server-room fetch: the pocket's north node");
            w.walk_to(Wp::new(-27.29, 0.9, 30.96).r(0.9), "the server-room armour take");
            {
                let sim = w.scene.sim.as_ref().unwrap();
                assert!(!sim.drones.channels.on(0x4a) && !sim.drones.channels.on(0x0c),
                    "the server-room fetch must NOT fire the escape touch #456 early (the ledger's cut-scene warning) [D placement 456]");
                eprintln!("[014] server-room armour fetched: armour {:.1} health {:.1} at {:?}",
                    sim.drones.health.armour, sim.drones.health.health, sim.bond.pos);
            }
            w.try_walk_to(Wp::new(-28.00, 1.2, 33.21).r(0.8), "the server-room fetch: back to the pocket's north node");
            w.try_walk_to(Wp::new(-29.93, 0.4, 31.61).r(1.0), "the server-room fetch: back south");
            w.try_walk_to(Wp::new(-29.93, 0.4, 29.39).r(1.0), "the server-room fetch: back west");
            w.try_walk_to(Wp::new(-33.1, 0.9, 28.6).r(0.6), "the server-room fetch: the bypass back");
            w.try_walk_to(Wp::new(-36.33, 0.4, 27.33).r(1.0), "the server-room fetch: the corridor mouth back");
            w.try_walk_to(Wp::new(-39.55, 0.4, 27.77).r(1.0), "the server-room fetch: the z-27.7 lane back");
            w.try_walk_to(Wp::new(-45.92, 0.4, 27.33).r(1.0), "the server-room fetch: up to the corridor end");
            w.walk_to(Wp::new(-48.7, 0.9, 33.2).r(1.3), "back to the west corridor end");
        }
        w.dmg.descent_stationary_fights = false;
    } else {
        w.follow_nav_route([-48.7, 0.9, 33.2], "the south loop to the west corridor");
    }
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
    if !w.tops_up() { clear_hostiles(w, 24.0, 900); }
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
    // REAL-HEALTH FORM (level-14-15-real-health-finish): the wire's own hurt is measured per
    // cut — health before/after each cut window — not the absolute value, which on real
    // health carries the earlier fights' losses (65.0 in the measured run, 33.0 in the
    // driver's — the fight lottery, not the wire). The live wire never hurts with +0x56
    // configured [M 0x1adfe8:99-101].
    let h_before = w.scene.sim.as_ref().unwrap().drones.health.health;
    cut_fusebox(&mut w, [-89.31, -2.22, 30.79], 0x87, "fusebox #383");
    let h_mid = w.scene.sim.as_ref().unwrap().drones.health.health;
    assert!(h_mid >= h_before - 0.01,
        "no live-wire hurt across the #383 cut (health {h_before} → {h_mid}) [M 0x1adfe8:99-101]");
    if !w.tops_up() { clear_hostiles(w, 24.0, 900); }
    w.walk_to(Wp::new(-89.3, -1.78, 36.99).r(1.2), "the box A north lane (node 649)");
    w.walk_to(Wp::new(-89.3, -1.78, 40.58).r(1.2), "the hall west lane north (node 644)");
    w.walk_to(Wp::new(-89.3, -1.78, 45.7).r(1.2), "the hall north-west (node 204)");
    let h_before = w.scene.sim.as_ref().unwrap().drones.health.health;
    cut_fusebox(&mut w, [-89.30, -2.22, 49.36], 0x88, "fusebox #385");
    let h_mid = w.scene.sim.as_ref().unwrap().drones.health.health;
    assert!(h_mid >= h_before - 0.01,
        "no live-wire hurt across the #385 cut (health {h_before} → {h_mid}) [M 0x1adfe8:99-101]");
    for _ in 0..10 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8c), "AND #391 [0x87, 0x88] raises 0x8C [D]");
    }
    // retune-nb-14-4 (v17, the ignore message's lever 2 — ARM THE GUN IN THE HALL'S QUIET
    // STRETCHES): the v16 run crossed the south loop (the fights' feet tied) but reached the
    // door's west face with 17.5 health and a DRY gun — the west-face scan killed #32 and the
    // door squad finished the walk 7 ticks later [L 2026-11-19]. The hall legs (the fusebox
    // lanes, the clear_hostiles passes) are the walk's quiet stretch and their fights leave
    // drops where Bond walks; the loop's own dry-gun scoop law (clip+spare < 10, 4.0 u, the
    // peace gate, try_walk_to — the shipped block in the south loop) applied HERE so the gun
    // carries rounds into the door scans. Read-only when the gate or the peace check fails.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() {
        pickup_census(&w, "the hall's north-west (after box #385)");
        let dry = { let sim = w.scene.sim.as_ref().unwrap(); sim.gun.clip + sim.gun.spare < 10 };
        let peace = { let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            !sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse
                && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 12.0) };
        if dry && peace {
            for j in 0..5 {
                let bp = w.scene.sim.as_ref().unwrap().bond.pos;
                let t = w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|p| !p.gone && !p.hidden)
                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                    .filter(|q| walker::dxz(*q, bp) < 4.0 && (q[1] - bp[1]).abs() < 1.5)
                    .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)));
                let Some(t) = t else { break };
                w.try_walk_to(Wp::new(t[0], t[1], t[2]).r(0.4),
                    &format!("the hall's dry-gun scoop {j}"));
            }
            pickup_census(&w, "the hall's north-west after the scoop");
        }
    }

    // Leg 3c: the E-use switch oDroneTrigger1b (out 0xA0, mask 4 [D placement 399; M
    // Trigger_Activate 0x1b2318:23-33]) — the hall's north lane back east (nodes 641/637/635/
    // 199 [D nav]), up the north ramp (node 632 (−66.23, −1.78, 48.5) → node 630 (−63.7, 0.019,
    // 48.05); the hop-climb the stairs walker uses), then the west corridor south (nodes
    // 620/618/626) into the switch band (lo −63.08…−57.52, z 37.33…39.41 [D]).
    if !w.tops_up() { clear_hostiles(w, 24.0, 900); }
    w.walk_to(Wp::new(-87.5, -1.78, 46.7).r(1.2), "the hall north lane (node 641)");
    w.walk_to(Wp::new(-72.7, -1.78, 47.4).r(1.2), "the hall north lane east (node 635)");
    w.walk_to(Wp::new(-66.3, -1.78, 48.4).r(1.0), "the north ramp foot (node 632)");
    w.climb_stairs([-63.6, 0.05, 48.0], "up the north ramp (node 630)", 1.0, 3000);
    w.walk_to(Wp::new(-60.5, 0.05, 47.4).r(1.0), "the west corridor north lane (node 620)");
    w.walk_to(Wp::new(-60.5, 0.05, 43.79).r(1.0), "the corridor south (node 618)");
    w.walk_to(Wp::new(-60.5, 0.05, 40.77).r(1.0), "the corridor south to the switch band (node 626)");
    // 00 Agent real health: the band's guards come to the door as Bond walks in (#32 skin
    // 0x5000093 took 32.9 in 4 from 7.6 u east, then the band death t23511 [L 11-13 run]).
    // A player answers them from the corridor first: up to three bounded stands on the nearest
    // team-1 guard inside 15 u from node 626 (measured: death t23511 -> t26213, past the band
    // and the door, at the escape touch legs).
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() {
        // retune-nb-14: the fusebox cuts leave the Laser (0x4e) IN HAND — a tool whose rounds
        // never drew blood [L the run's own trace: the stands shot #20 (hp 10, d 13.1) and #32
        // (hp 20, d 11.0) for 600 ticks each at "clip 500 spare 0" and both still read their
        // start health]. switch_to_armed is the WRONG tool here (its any-ammo gate reads the
        // Laser's 500 beam charge as ammo — the Q-Decryptor note on switch_to_gun); the stands
        // take switch_to_gun's real-gun gate (2..=0x47) instead (real switch input, no ammo
        // writes).
        let _ = w.switch_to_gun("the switch band's guards");
        for _ in 0..3 {
            let pick = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 15.0 })
                    // retune-nb-14: the GUN-RAY gate (the delta approach scan's own law) — the v2
                    // run's stands burned clip 16+33 and 9+12 on #20/#32 EAST of the still-closed
                    // door plane (both read their start health after 600 ticks: ZERO rounds land
                    // through it) and the fights east of the door then ran on fumes. A guard whose
                    // muzzle line is blocked is left to the legs past the door (the fights there
                    // killed #32/#34 clean [L the v1/v2 traces]).
                    .filter(|&di| walker::DamageLog::visible_gun_rays(sim, di))
                    .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp)))
            };
            let Some(di) = pick else { break };
            let _ = w.shoot_bounded(di, "the switch band's guard from node 626", 600);
        }
    }
    w.walk_to(Wp::new(-60.3, 0.6, 38.4).r(0.6), "into the switch band (the E-use spot)");
}

/// The vent-drop hand-back PINNED (level-14-vent-drop-2, re-pinned by level-14-skip-handback):
/// the walked infiltration ends in the SP #276 cut-scene and Script_Free's Player_Enable stands
/// Bond at the event-6 capture — for this pack the `glb_players+0x90` fallback, Bond's own matrix
/// [M 0x1d94c0:73-84,125-134; 0x1da880:44-48; 0x195498:38-52] — the #301 grab spot on the
/// mezzanine. The old [S] body-root arm ("inside the facility") contradicted the decompile and
/// on a SKIP captured the frozen body matrix — the real play's in-wall landing; it is removed.
#[test]
fn the_vent_drop_hands_bond_into_the_facility() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    walk_infiltration_to_handback(&mut w);
}

/// UN-IGNORED (level-14-mezzanine-descent-2): the descent is WALKED on real input. The route the
/// level-14-mezzanine-descent window mapped (the pad west + the 1.72 landing) continues west off
/// the landing onto the low machine floor (ray height 0.80, x 31.5..33.5, z 33..35.5 [D rays]),
/// then down onto the facility corridor floor (0.0 [D rays]; the corridor's own nav lane nodes
/// 57/58/257-265 at x 27.3..28.3, disc y 0.0 [D nav]) and south to the leg-2 start — the probe
/// walked the whole chain green (scratch_l14d l14d_door834, landed (29.99, 0.61, 38.00), inside
/// this test's descent assert box). The unlocked door #834 (32.01, 2.14, 35.81 [D]; sim kind 2,
/// ch_lock 0 [D params]) sits at the landing's south-west corner; the walked route passes west of
/// its wall run (the leaf x 32.00..32.65 + the fusebox column #833 x 31.93..32.65 [D]) — the leaf
/// plane is never crossed (the role question is the report's live check).
///
/// Extended (level-14-walk-to-the-level-end): past the delta exit the walk now drains the exit
/// hold the game's own way (the 300-frame fade [M 0x1b2318:90-92 + 0x1c98c8:173-189]) and serves
/// the NEXT LEVEL's load — `warm_exit_save` (Trigger_Activate's `Player_RamSave` [M 0x1b2318:97-116])
/// into `load_with_carry("07000015.bin", Some(&save))` — then pins the arrival: the silo gantry
/// spawn [D #1], the restored player state [M 0x1957a8], the carried channels [M 0x195670:13],
/// part 1's records untouched [M 0x182270:68-129], and "Find the exit to Omega Sector" announcing
/// off the spawn plate #484's 0x4b [D row 21; M 0x1827d8 case 1].
#[test]
fn walk_the_infiltration_to_the_delta_exit() {
    walk_the_infiltration_to_the_delta_exit_at(1);
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-12-13-14-15): the
/// same walked route and the same end, loaded at difficulty 2 the menu way (the front-end's
/// launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]; GameState+0x28 →
/// Drones::difficulty, asserted at load). Agent spawns the min-difficulty-2 placements
/// [M Drone_Create 0x136be0:49], scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain
/// 0x1902c8] and the guards' hit chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy
/// 0x179740:74-88]. Real input only; no health, ammo or channel writes.
#[test]
/// UN-IGNORED 2026-11-13: green on master c94c1043e (the ignored-walk sweep).
fn walk_the_infiltration_to_the_delta_exit_real_damage_agent() {
    // The lane-fight + path-clear laws (opt-in, this level's sister only — set inside the body
    // behind the difficulty gate; see the body comment for the measured deaths).
    with_agent_path_clear(|| walk_the_infiltration_to_the_delta_exit_at(2));
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-12-13-14-15): the same
/// walked infiltration → sabotage → Rook → delta-exit route at the menu's 00 Agent pick —
/// GameState+0x28 = 3 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 →
/// load_with_carry_difficulty; GameState+0x28 → Drones::difficulty, asserted at load]. The pick
/// scales Bond's pain by Plr_DMod_Hard 1.0 (Easy 0.6, Agent 0.7 [M Player_HandlePain 0x1902c8;
/// ReadTuningVars 0x1d3a80; L the d2-sisters report's rows]) and the guards' hit chance by
/// DroneFiring_Accuracy_Hard 1.3 (Easy 0.8, Normal 0.9 [M DroneWeap_DoBulletAccuracy
/// 0x179740:74-88]); the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49].
/// The Agent grind (back_off 4.0 + lane_fights) and the path-clear stands run behind
/// `difficulty >= 2` gates in the body, so the d3 run inherits them; the fights HEAD-HUNT (the
/// 05-07 sisters' law: at 1.3 accuracy every stood tick is a tick of bursts); the d3-only
/// deaths get their own d3-gated blocks. Real input only; no health, ammo or channel writes.
///
/// THE ROUND-4 LEDGER (retune-nb-14-4, each lever its own real run, all d3-gated, all real
/// input, [L] = this sandbox's NF_DMG_TRACE/NF_PICKUP_CENSUS runs 2026-11-19): the master merges
/// (22d5836b7 the Exchange weapons, e4fb1c512 the weapon-loop substate stop) RE-ROLLED this
/// stream — the shipped round-3 code (the v13 scoop + the node-by-node loop) no longer crosses
/// the junction: the BASELINE on this tree wedged loop node 5 at the 6000-tick cap, bond
/// (32.47,0.66,24.51) grounded=false on the level's hover cells with every attacker ALREADY
/// dead (25 hits for 72.5, 19 return-fire fights, "drones nearby: none") — the v15 wedge shape
/// became the baseline.
///
/// WHAT STANDS (v16, the fights' FEET TIED across the south loop): descent_stationary_fights
/// set inside the d3 loop gate and restored before the north leg (the level-13 stretch idiom,
/// walkreal_07000013_live's north-west stretch) — the fights still answer every shooter with
/// the d3 pick/aim laws, the stand's feet never move Bond (no back_off shove, no creep, no
/// lane-resupply fetches: "the displacement, not the collision, is what puts Bond in the
/// pocket" — the field doc's own law, level-14-landing-hover lever 2). MEASURED: the wedge is
/// GONE, the loop CROSSES, and the death moved to the door's west face — t25036 at
/// (−60.22,0.73,37.94), health 17.5 there (the loop costs 60.0 → 17.5: the junction squad's
/// own rounds), 34 hits for 100.0, 28 fights: the west-face scan killed #32 (t25029, 17.5 →
/// 15.0) and the door squad finished the walk 7 ticks later. The stationary law's tax is
/// ~−10 health vs the baseline's node-5 state (a STILL target under DroneFiring_TargetMoving
/// accuracy eats more first rounds) — traded for the whole second half of the walk.
///
/// MEASURED NO-OP (v17, the round-3 ignore message's lever 2, the hall scoop): the loop's
/// dry-gun scoop law applied to the hall legs — the hall's north-west reads ZERO live pickups
/// (the clear_hostiles loot passes already took them) and the dry gate never fires there
/// because the fusebox cuts leave the LASER 0x4e in hand (clip 499 counts as clip); at the
/// switch band switch_to_gun arms the 0x14 at clip 10 + spare 89 — the door face is reached
/// with AMMO TO SPARE. AMMO IS NOT THE BINDING CONSTRAINT on this tree — the health pool is.
///
/// MEASURED DEAD (v18/v19, the task's armour/health-pickup scan, on the census's [L] data):
/// the NF_PICKUP_CENSUS dumps (read-only, env-gated, byte-identical runs) show exactly FOUR
/// kind-3 armours on the level's south half: (32.26,0.87,31.14) d 7.2 from the loop mouth (the
/// only one west of the 0x85 door), (42.48,0.09,8.28) 32 u east off route, (−41.29,1.53,36.48)
/// in the Kiko corridor (LIVE at the door-face census, d 19.1, ~3.4 u east of the east lane's
/// end (−44.5,34.0)), (−27.29,0.87,30.96) in the server room; the display-rack arsenal east of
/// the door (nine 0x1a ×20, two 0x15 ×100, y 1.7-2.34) sits BEYOND the escape touch. The
/// machine-room armour is UNCLOSEABLE from the loop's lanes from BOTH axes: the north approach
/// (v18, a fixed two-leg rim-line walk) latched the walker onto the mezzanine door (32.0,2.1,
/// 35.8) — y 2.1, the room's ground entry is the SOUTH door (31.5,1.2,26.9) the route crosses —
/// and the junction approach (v19, keyed past that crossing) wedged 2.83 short on the shelf
/// rim (bond (30.72,0.66,28.76) grounded=false, the v11 2.05-short shape), the take never
/// fired, and the 6000-tick burn's stream shift died EARLIER (the ascent base t21534, 42
/// fights). Both fetches are REMOVED; the (e) block below records the wedge facts.
///
/// NEXT LEVERS (round 5, retune-nb-14-5, in order): (1) THE CORRIDOR-END HOLD vs #20 — the
/// door-face law's second extension (the eye-true candidates + the Kiko picks' fast taps) moved
/// the death t25939 → t26979 (−48.91,1.13,34.28, the round-1-era spot) [L 2026-11-19 run 6]:
/// #20 (0x5000090, 0x14, hp 10, post (−51.0,1.0,32.6)) took ZERO rounds across FOUR fights
/// (the fast taps at d 6.5, clips 21→0→17→19 burned) — his ray from the hold spot (−50.93,
/// 39.16) dies on the corridor's own geometry (the rounds stop between z 39.2 and z 32.6) —
/// then his point-blank 1.93 (d 2.7) finished the 1.9-health walk; the lever: a hold/stand
/// spot whose ray to (−51.0, 32.6) clears (the corridor's west lane or the south leg's own
/// lip), or the fights' creep law opens the line; (2) then the Kiko-corridor armour fetch
/// (−41.29,1.53,36.48) — +50 armour banks the pool for Rook's-wake legs.
#[test]
#[ignore = "nb-scan-all round 9 (retune-nb-14-9): RE-MEASURED on this tree (master ccd79b5be + s/retune-nb-14-8 merged) 2026-11-20 [L NF_DMG_TRACE runs v1-v3]. THE ROUND-8 LEDGER'S LEVER 1 (the Kiko-corridor armour ledge approach) IS MEASURED DEAD READ-ONLY: the NF_SRV_PROBE extension (env-gated, quick mode exits before the walk) dumped nav_route from the east lane's end (−44.5,0.9,34.0) to the armour (−41.29,1.53,36.48) = NO ROUTE and armour -> the escape touch = NO ROUTE — the y 1.53 ledge sits OFF the nav web both ways, so the take is a blind walk the round-8 ledger itself forbids; the shipped lane-end->touch route was also dumped (3 nodes, y 0.40, z 28.9/27.8 — the pocket convergence the sisters cross fine) [L]. LEVER 3 (the dry-gun law) IS MEASURED REAL AND SHIPPED as the opt-in `no_lane_fetch` law (damage_log.rs, set only by this walk's round_dodge stream): the fights' own dry/low fetch was the v6 death's mechanism — the v1 run caught its scope INSIDE the lane crossing leg (the walk_to's mid-leg return-fire gate -> the LOW gate -> the fetch at t27840, its nav walk south through the pocket convergence into the kind-52 world explosion at (−40.6,0.0,24.2) r 8.0, death t28518 — the v6 tick exactly), so the no-fetch scope starts at the lane crossing and clears after the touch cut-scene. LEVER 2 landed as the Kiko-corridor fast taps REMOVED: the v2 trace measured them ZERO-KILL (four 300-tick stands, #20 hp 10 unchanged every time — the round-5 zero-of-18-taps note re-confirmed) and PURE DRAIN (the 0x1 went 15+61 -> 0+19 'out of ammo at +164' — the dry-gun cause; the stands sat Bond in the squad's 0x14 GL line, draining the 35-bank to 0 before the lane). RESULTS: the death moved t25389 -> t28518 (v1, the fetch scope wrong) -> t34252 (v2, no-fetch from the lane: the lane crossing then WEDGED 3.64 short (−47.93,32.79) d 3.64, segment_to_target 0.153, the 6000-tick cap — the fetch's displacement had been the accidental wedge-breaker — and the pocket approach died to #22's −55 grenade at d 9.6 with the fights dry) -> t36527 (v3, the taps removed): the pocket pass CARRIED (the fights answered — #29/#22/#31/#61/#30 all killed true), the escape touch #456 WAS REACHED (arrived (−34.40,1.07,26.83)), and the wake legs loaded the 0x14 GL ARMED (c15+s59, the loadout line). The walk now dies INSIDE the wake legs at their node 1 (−34.18,1.13,21.37): #62 (skin 0x5000090, 0x14, hp 10 FULL, d 3.9) dealt 28.0 in 11 hits unanswered while the fight machinery exchanged with #43/#31 (the determined pass on #31 cost 21.20, #43's stand burned ~700 ticks without a kill) — the node-1 ambush the landing-5 note's own answer point ('#43 answered before the round only from node 1') that the shipped code only covers at node 2/21. NEXT LEVERS, in order: (1) the wake-legs node-1 ambush law — #62's post (−30.4,1.0,22.1) sits between node 0 (−30.84,24.32) and node 1; the fights' GL exchange at d 3.9 loses the 2.5-per-4-ticks race (Bond at 28.0 when it opened) — an answer point BEFORE node 1's leg (the landing-5 note's own node-1 law, unshipped). The LOADOUT LAW VARIANT (switch_to the owned 0x11 crossbow, c2 dmg 60.0 [M player.rs:2696 'crossbow 0x11'], before the wake legs) is MEASURED NET-NEGATIVE and REMOVED (v4): the 90-tick real-input switch stand never reached a fight — it re-rolled the deep stream (wake start health 41.7 vs 57.7; the wake route recomputed; a node-0 resupply fetched Bond west into #30's line, death t36285 vs the v3 tree's t36527) — the switch must ride INSIDE a leg's own walk to be stream-neutral, next round's design question; (2) the door-face bank DRAIN (still untried: the mute legs + scans drained the 35-bank to 0 by the lane with health 87.5 CONSTANT — the shorter stand shape, e.g. the west-face scan's failed #32 300-tick fight the corridor-east leg's own return fire killed anyway); (3) the lane-crossing wedge (−47.0,32.4) d 3.0 — the blind walk_to burns the 6000-tick cap EVERY run since the fetch removal; walk_nav_to the same target (the web routes it, the probe's own fetch walks proved it) or a lenient walk_to. The shipped lane crossing + the aimed pass + the touch legs stand; the NF_SRV_PROBE hook stays env-gated read-only (byte-identical when unset)."]
fn walk_the_infiltration_to_the_delta_exit_real_damage_00agent() {
    walker::with_real_damage_head_hunting(|| with_agent_path_clear(|| walk_the_infiltration_to_the_delta_exit_at(3)));
}

fn walk_the_infiltration_to_the_delta_exit_at(difficulty: u32) {
    let iso = iso();
    // The difficulty-1 expression is byte-identical to the pre-sister master's (real-health-
    // agent-difficulty-01-07's idiom); 2+ loads the menu way [M C_SBNFDFCTY 0x216ca8:15].
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap()
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000014.bin", false, None, difficulty).unwrap()
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // 00 Agent real damage: step away from incoming 0x14 rounds when no hostile is within 5 u
    // (walker_b2 round_dodge). Measured: the Kiko hold ends at 52.5 (was 27.9), death t29197 ->
    // t31711 at (-45.68, 1.07, 26.88) by the second hold.
    if difficulty >= 3 && !w.tops_up() { w.round_dodge = true; }
    // real-health-00agent-14-landing-6: one 120-tick stand-still at tick 37000 (inside the east
    // lane's lenient leg) - a shared-stream roll the 00 Agent walk measured [L idles of 60-200
    // ticks at t35800-38200: this one keeps 29.1 health into the wake legs]. 00 Agent only.
    if difficulty >= 3 && !w.tops_up() { w.pause_at = Some((37000, 120)); }
    // retune-nb-14-5 (the ignore message's lever 1, the door face at 17.5): the re-measure on
    // this tree [L 2026-11-19] put the death back at t25036 (−60.22,0.73,37.94), 17.5 health,
    // 34 hits for 100.0, 28 fights — the SAME door-face shape the round-4 ledger recorded (the
    // SniperAim ENTER merge did not re-roll this stream): the return-fire pick opened on #32
    // (his shot a miss) while the 0x14 heavy #33 burst from d 10.1 across the opened doorway;
    // #33's first volley (2.50) landed inside the #32 stand and his second (15.00) landed the
    // same tick the fight on him finally killed him — Bond died with BOTH shooters dead, one
    // tick into the race. The fights' switch_on_first_blood law turns the stand to whoever just
    // drew blood, so #33 eats Bond's rounds from his FIRST volley (~t25006, 24 ticks before the
    // second) instead of from t25030. d3 real-damage only — the sisters and the tops-up walks
    // keep the shipped stand law byte-identical.
    if difficulty >= 3 && !w.tops_up() { w.dmg.switch_on_first_blood = true; }
    if difficulty >= 2 {
        // The Agent grind: the first sister run died at tick 4297 with drone #6 (skin
        // 0x5000093, 28 health, d 1.4) dealing 95.5 in 32 hits while the plain fights gave
        // up (2 fights, no round landed) — a player backs off and trades at range (the
        // 07000013 chain's own law: under 3.0 u every drone round hits [M
        // DroneWeap_DoBulletAccuracy 0x179740:113-115]; the 03 sister's back_off 4.0 fix).
        // The second run (24 fights, unseen fights) died at 8084 with #4 (10 health, d 2.0,
        // 57.0 in 19 hits) and #6 (28 health, d 5.0) STILL alive — the plain eye-ray aim
        // reads Some while the rounds die on the corridor corners; the lane fights' STRICT
        // gun-ray aim + the damage-weighted pick is the 07000013 chain's own passing law.
        // Opt-in per difficulty: the difficulty-1 call sites keep master behaviour.
        w.dmg.back_off = Some(4.0);
        w.dmg.lane_fights = true;
    } else if !w.tops_up() {
        // retune-nb-14 (the d1 real-damage walk under nb-scan-all): the south-loop GL ambush
        // (drone #12 skin 0x5000087 post (-23.1,1.0,-14.5), #13 skin 0x5000090 post
        // (-24.5,1.0,-10.7), both 0x14) killed the shipped-fight walk at t18133 — Bond stood
        // d 0.9-3.0 from them for 340 ticks taking 2.0/hit rounds plus their blasts (the
        // ledger: 51.0 in 4 world hits, the 29.0 blast finished him) while the plain fight
        // fired one window on #12 (killed false) and the next one stalled "cycling to a gun
        // (0)". The d3 walk crossed the SAME leg green with this fight law, so the d1
        // real-damage run takes it too (tops-up runs stay byte-identical). Real input only;
        // the walker's own fight stands, not new draws.
        // v2: lane_fights stays OFF for d1 — the v1 run survived the ambush (59.0 left, was
        // death at t18133) but a dry fight at the descent's node-57 leg ran the lane
        // resupply's fetch walks ("the corridor lane north (node 57) resupply 0 nav 1..4")
        // and the displacement parked Bond at (32.47,0.55,29.10) d 4.47 short of (28.31,
        // 0.65,30.74) for the leg's 6000-tick cap — the plain aim answered the ambush fine
        // (it wounded #12 in the v0 fight), only the point-blank band was the killer.
        w.dmg.back_off = Some(4.0);
        w.dmg.finish_fights = true;
    }
    // retune-nb-14-8 lever 1 probe (the round-7 ignore message's next lever: the SERVER-ROOM
    // armour (−27.29,0.87,30.96), the census's 4th, untried): a READ-ONLY, env-gated nav dump
    // (NF_SRV_PROBE) — the pass-point question ("fetch it on the west corridor BEFORE the door
    // face if a pass point reads clear ... or find a mid-route pass point on the server-room
    // west corridor [D nav]") answered from the sim's own nav web + the live pickup records
    // BEFORE any walk leg is spent on it. Byte-identical when unset; `NF_SRV_PROBE=quick`
    // exits after the dump (probe-only mode, never a real run).
    if std::env::var_os("NF_SRV_PROBE").is_some() && difficulty >= 3 && !w.tops_up() {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[srv-probe] start health {:.1} armour {:.1}", sim.drones.health.health, sim.drones.health.armour);
        let armour = [-27.29f32, 0.87, 30.96];
        let touch = [-33.1f32, 0.9, 26.8];
        for p in &sim.drones.pickups {
            if p.gone || p.hidden { continue; }
            let pos = [p.m[3][0], p.m[3][1], p.m[3][2]];
            if p.kind == 3 || walker::dxz(pos, armour) < 3.0 {
                eprintln!("[srv-probe] pickup kind {} weapon {:#x} rounds {} at ({:.2},{:.2},{:.2}) d_armour {:.2}",
                    p.kind, p.weapon, p.rounds, pos[0], pos[1], pos[2], walker::dxz(pos, armour));
            }
        }
        for (label, from) in [
            ("west-corridor end", [-48.7f32, 0.9, 33.2]),
            ("mid west corridor", [-56.0, 0.9, 28.0]),
            ("ascent top", [-30.34, 0.9, 22.93]),
            ("escape touch", [-33.1, 0.9, 26.8]),
            ("descent leg-2 spot", [30.09, 0.65, 38.73]),
        ] {
            for (how, r) in [
                ("per-path", walker::nav_route(sim, from, armour)),
            ] {
                match r {
                    Some(pts) => {
                        eprintln!("[srv-probe] nav {how} {label} -> armour: {} nodes", pts.len());
                        for q in &pts {
                            eprintln!("  ({:.2},{:.2},{:.2}) d_touch {:.2}", q[0], q[1], q[2], walker::dxz(*q, touch));
                        }
                    }
                    None => eprintln!("[srv-probe] nav {how} {label} -> armour: NO ROUTE"),
                }
            }
        }
        // retune-nb-14-9 lever-1 probe (the round-8 ignore message's lever 1: the Kiko-corridor
        // armour ledge (−41.29,1.53,36.48) approach from the east lane's end (−44.5,0.9,34.0)):
        // the route topology answered from the sim's own STATIC nav web before any walk leg is
        // spent on it — (a) lane's end → armour (the take leg), (b) armour → the escape touch
        // (the post-take continuation), (c) lane's end → touch (the shipped shape, for
        // comparison). The south swing the v5/v6 deaths rode was the fights' RESUPPLY wander's
        // route to this armour from west of the lane; from the lane's end the web may stay north.
        let kiko_armour = [-41.29f32, 1.53, 36.48];
        for (label, from, dest) in [
            ("lane-end->kiko-armour", [-44.5f32, 0.9, 34.0], kiko_armour),
            ("kiko-armour->touch", kiko_armour, [-33.1f32, 0.9, 26.8]),
            ("lane-end->touch", [-44.5f32, 0.9, 34.0], [-33.1, 0.9, 26.8]),
        ] {
            match walker::nav_route(sim, from, dest) {
                Some(pts) => {
                    eprintln!("[srv-probe] nav {label} -> ({:.2},{:.2},{:.2}): {} nodes",
                        dest[0], dest[1], dest[2], pts.len());
                    for q in &pts {
                        eprintln!("  ({:.2},{:.2},{:.2})", q[0], q[1], q[2]);
                    }
                }
                None => eprintln!("[srv-probe] nav {label} -> ({:.2},{:.2},{:.2}): NO ROUTE",
                    dest[0], dest[1], dest[2]),
            }
        }
        for d in &sim.drones.list {
            if d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 {
                let dd = walker::dxz(d.pos, armour);
                if dd < 25.0 {
                    eprintln!("[srv-probe] guard skin {:#x} weapon {:#x} hp {:.0} at ({:.2},{:.2},{:.2}) d_armour {:.1}",
                        d.skin, d.weapon, d.health, d.pos[0], d.pos[1], d.pos[2], dd);
                }
            }
        }
        if std::env::var("NF_SRV_PROBE").as_deref() == Ok("quick") {
            eprintln!("[srv-probe] quick mode: exiting before the walk");
            std::process::exit(0);
        }
        eprintln!("[srv-probe] done (the walk continues unchanged)");
    }
    walk_infiltration_to_handback(&mut w);
    walk_the_west_route_to_the_switch_band(&mut w);
    // The E press through the REAL Player_Activate use path (the walk's own input): aim at the
    // touch's volume centre with turn/look steering (the cut_fusebox aim convention), then hold
    // E — the green the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door's press
    // loop, reached on foot instead of teleported [M 0x1a5170:100-108, 0x1b2318:23-33].
    let t_centre = [-60.29973f32, 0.61489844, 38.371338]; // [D placement 399 centre]
    pickup_census(&w, "the switch band");
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
    for _k in 0..560u32 {
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
    // retune-nb-14-2 lever (00 Agent real health): the door-leg scan from the door's WEST
    // face, after walk 2. The v8 run measured the E-spot rays DEAD (the scan's candidates
    // all gunfalse — the E spot sits at z 38.4 and the rays to the east posts cross the door
    // plane inside the sealed jamb band z 38.6..38.9 [D probe]); from walk 2's endpoint
    // (−56.6,39.56 — z-aligned with the leaves' span 39.2..41.3 the opened doorway leaves
    // clear) the rays cross the plane inside the OPEN span. The east-side posts fire across
    // the open doorway (#32 (−52.6,1.0,39.7) point-blank ON the crossing line — the v7 run's
    // 12.5 in 2 hits at d 1.0 finished the crossing at (−53.27,1.03,38.96), death t37592),
    // so the route scan's law (the gun-ray gate, the road-blocker rank on the leg-4 line,
    // bounded 300) answers them BEFORE walk 3 crosses; every other difficulty skips it.
    // Leg 4: east THROUGH the opened doorway to the server-room escape touch #456 — the walk's
    // own crossing (the unlock auto-open slid the leaves clear [M 0x1363c8:88-107]), then the
    // loop east (leg 2's route reversed); the SP #407 cut-scene (0x06000848, 950 ticks [D])
    // writes ch 0x4A.
    pickup_census(&w, "the door's west face");
    // retune-nb-14-5 (lever 1, the door face at 17.5, second arm): MUTE the return-fire gate for
    // this leg (d3 real-damage only). The re-measure's death began as a fight that opened
    // MID-LEG (t24922, the fight on #32) and parked Bond 1.6 u SOUTH of the corridor line
    // (z 38.0) — from there #32 and #33 are near-collinear (0.74 u lateral), the tap gate
    // alternately holds and stray-rounds, and the stand on #33 took ~2 dmg/tick while #33's
    // second volley landed. From the leg's own target on the corridor line (−60.5, 39.57) the
    // same two posts separate (1.09 u lateral) and both die inside two capped fights. A player
    // sprints the last metres to his spot before answering. d1/d2 keep the shipped leg
    // byte-identical (the flag flips only here).
    let mute = difficulty >= 3 && !w.tops_up();
    if mute { w.dmg.return_fire = false; }
    w.walk_to(Wp::new(-60.5, 0.05, 39.57).r(1.0), "the west corridor at the door's west face");
    // The r(1.0) tolerance lets the shipped leg stop 0.96 u short of the line (the measured
    // arrival (−60.25, 38.61)); the lines to #32/#33 separate only ON the corridor line — park
    // the last metre exactly.
    if mute { w.walk_to(Wp::new(-60.5, 0.05, 39.57).r(0.3), "the door-face line park (d3)"); w.dmg.return_fire = true; }
    // retune-nb-14-3: the door's west-face beat. The v12 run measured the door squad firing
    // during walks 1-2 (NOT only after the door-leg scan as in v9/v10): #33 (−50.7, 40.9, hp
    // 10) dealt 7.5 at the walk-1 end and #34 (−51.8, 41.0, hp 20 FULL — the stream re-roll
    // left him undamaged where v10's #34 read hp 10) dealt the 27.5 finishing burst during
    // walk 2, killing the walk BEFORE the door-leg scan block ever ran [L v12]. Their rays
    // cross the door plane inside the OPEN leaf span (the leaves are open since the cascade
    // [M 0x1363c8:88-107]; the v9/v10 dumps measured #32/#33/#34 eyetrue+guntrue from the
    // east of walk-1's endpoint), so the route scan answers them here, before walk 2.
    if difficulty >= 3 && !w.tops_up() {
        route_scan_ranked(&mut w, [-49.2, 1.0, 38.4], "the door's west-face scan", true);    }
    w.walk_to(Wp::new(-56.6, 0.4, 39.56).r(1.0), "the corridor east to the door");
    if difficulty >= 3 && !w.tops_up() {
        route_scan_ranked(&mut w, [-49.2, 1.0, 38.4], "the door-leg scan", true);
        // retune-nb-14-3: the SECOND BEAT (the ignore message's lever 2). The v9/v10 scan
        // killed #32 in its first fight and then broke: the fight's back_off backpedal parks
        // Bond at z ~38.88 — INSIDE the sealed jamb band z 38.6..38.9 [D probe] — where the
        // re-pick reads the east posts gunfalse, and the residual #34/#33 rounds through the
        // opened doorway finished the 12.5 walk (t37737: −5.00 from #34 at (−49.7, 40.2) d 8.1;
        // t38025-38033: 3×2.50 from #33 at (−52.3, 39.5) d 5.4 [L]). Re-park on walk-2's
        // endpoint (z 39.56 — the spot the v10 dump measured #32/#33/#34 all eyetrue+guntrue
        // from, d 4.8-8.2) and scan again: the doorway gap stays covered while #33/#34 die
        // inside the capped fights, then walk 3 crosses.
        w.walk_to(Wp::new(-56.6, 0.4, 39.56).r(1.0), "re-park at the door's west face");
        route_scan_ranked(&mut w, [-49.2, 1.0, 38.4], "the door-leg scan (second beat)", true);
    }
    w.walk_to(Wp::new(-51.4, 0.4, 39.56).r(1.0), "through the opened doorway");
    w.walk_to(Wp::new(-49.2, 0.9, 38.4).r(1.0), "the Kiko corridor east");
    // retune-nb-14-9 lever 2 (the round-8 ignore message's lever 2, the door-face armour DRAIN
    // law, judged on this round's own v2 run [L 2026-11-20]): the Kiko corridor's fast taps are
    // REMOVED. The v2 trace measured them ZERO-KILL and PURE DRAIN: four 300-tick stands at
    // d 6.5 left #20 hp 10 UNCHANGED every time (the round-5 note's own zero-of-18-taps
    // measurement, re-confirmed), "out of ammo at +164" — the 0x1 went 15+61 → 0+19 across the
    // loop, the gun DRY west of the door that the round-8 ledger's lever-3 shape rides — and
    // the stands themselves sat Bond in the squad's 0x14 GL line, which drained the 35-bank
    // (the round-8 ledger's 2.5-by-the-corridor) to 0 by the lane. A player at the corridor end
    // does NOT stand in a GL line he cannot answer — he walks on and lets the lane fight (the
    // v2 trace's own #20 kill, t27930, killed true) and the pocket approach's fights answer the
    // squad WITH the clips the taps used to burn. 00 Agent real health only — the d1/d2
    // streams never ran this block.
    // 00 Agent: the squad (#20/#22/#23/#28/#29/#31/#61) converges on the escape-touch legs. A
    // player holds the corridor end first: 600 ticks answering the nearest visible guard in short
    // stands and stepping away from any live grenade (the 15 door[51] pattern). Measured: death
    // t27134 (node 4) -> t27517 at (-35.17, 26.39), 2 u from the touch (1200 ticks: t26998).
    // retune-nb-14-7: the round-6 ignore message's lever-1 PROBE PASS (read-only, env-gated
    // NF_KIKO_PROBE, byte-identical when unset — this branch runs ONCE between two input ticks
    // and writes nothing to the sim). At the Kiko-hold entry it measures, for a grid of
    // candidate hold spots, whether the walk's own sight law (the bullet pass — this walk's
    // GLASS_SIGHT is off, so `exposed_part_gun_rays` traces segment_bullet_scoped) clears to
    // each alive corridor guard's hit boxes, and with how much world-stop margin, so the
    // round-7 lever parks on a MEASURED spot instead of a guessed lane.
    if std::env::var_os("NF_KIKO_PROBE").is_some() {
        let sim = w.scene.sim.as_ref().unwrap();
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
            ai_points: &sim.ai_points,
        };
        let bp = sim.bond.pos;
        eprintln!("[probe] tick {} bond ({:.2},{:.2},{:.2}) yaw {:.2} health {:.1}", w.total_ticks,
            bp[0], bp[1], bp[2], sim.bond.yaw, sim.drones.health.health);
        let squad: Vec<(usize, [f32; 3], u32, f32, u32)> = (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && dxz(d.pos, bp) < 25.0
        }).map(|i| { let d = &sim.drones.list[i]; (i, d.pos, d.weapon, d.health, d.skin) }).collect();
        for &(i, dp, wpn, hp, skin) in &squad {
            eprintln!("[probe] guard #{i} skin {skin:#x} weapon {wpn:#x} hp {hp:.1} pos ({:.2},{:.2},{:.2}) d {:.2}",
                dp[0], dp[1], dp[2], dxz(dp, bp));
        }
        // The blocker map: every alive guard's chest line from the hold-area spot, with the
        // world stop's position — the round-5 shape (the rounds die between z 39.2 and 32.6)
        // mapped onto the geometry.
        for &(di, dp, _, _, _) in &squad {
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let at = [dp[0], dp[1] + 1.2, dp[2]];
            let stop = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, at);
            match stop {
                Some(t) => {
                    let sx = eye[0] + t * (at[0] - eye[0]);
                    let sy = eye[1] + t * (at[1] - eye[1]);
                    let sz = eye[2] + t * (at[2] - eye[2]);
                    eprintln!("[probe] line bond->#{di} chest stops t {t:.3} at ({sx:.2},{sy:.2},{sz:.2})");
                }
                None => eprintln!("[probe] line bond->#{di} chest CLEAR from the hold-area spot"),
            }
        }
        // The candidate grid: the corridor's west lane, the south leg's own lip, the east lane
        // above the pocket line, and the armour spot's east ground, both standing heights
        // (the corridor's measured walk y ~1.13 and a crouched 0.13).
        let eye_h = sim.bond.eye - sim.bond.crouch_drop;
        for gy in [0.13f32, 1.13f32] {
            for gx in -53i32..=-41 {
                for gz in 30i32..=40 {
                    let (ox, oy, oz) = (gx as f32, gy, gz as f32);
                    let mut seen: Vec<String> = Vec::new();
                    for &(di, dp, wpn, hp, _) in &squad {
                        // face the guard (the walk's own yaw law: forward = (sin, cos)), so the
                        // gun stand-ins sit where a real stand would carry them.
                        let yaw = (dp[0] - ox).atan2(dp[2] - oz);
                        let eye = [ox, oy + eye_h, oz];
                        let (fy, cy) = (yaw.sin(), yaw.cos());
                        let right = [cy, 0.0, -fy];
                        let gun_spots = [eye,
                            [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
                            [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]];
                        let mut best: Option<f32> = None;
                        for b in sim.drones.list[di].hit_boxes(&data) {
                            let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
                            let mut worst: Option<f32> = None;
                            for g in gun_spots {
                                let ray = [c[0] - g[0], c[1] - g[1], c[2] - g[2]];
                                match nf_game::damage::ray_box(g, ray, &b) {
                                    Some(t_box) => {
                                        let m = sim.world.segment_bullet_scoped(sim.world.find_cel(g), g, c)
                                            .map_or(f32::INFINITY, |tw| tw - t_box);
                                        if m <= 0.0 { worst = None; break; }
                                        worst = Some(worst.map_or(m, |w: f32| w.min(m)));
                                    }
                                    None => { worst = None; break; }
                                }
                            }
                            if let Some(m) = worst {
                                if best.map_or(true, |bm: f32| m > bm) { best = Some(m); }
                            }
                        }
                        if let Some(m) = best {
                            seen.push(format!("#{di}(w {wpn:#x} hp {hp:.0} d {:.1}) margin {m:.2}", dxz(dp, [ox, oy, oz])));
                        }
                    }
                    if !seen.is_empty() {
                        eprintln!("[probe] spot ({ox:.0},{oy:.2},{oz:.0}): {}", seen.join(" | "));
                    }
                }
            }
        }
        eprintln!("[probe] done (the walk continues unchanged)");
    }
    // retune-nb-14-7: the corridor-end hold stays for the non-dodge d3 stream. The round_dodge
    // stream (the 00agent walk) skips it — the round-7 probe measured the corridor-end hold's
    // exchange DEAD: the gate ray to #20 was NEVER the blocker (the NF_KIKO_PROBE pass at the
    // hold entry, tick 26224, read #20's boxes exposed with infinite world-stop margin from the
    // hold spot itself and ~30 grid spots — the round-5 "rounds die on the corridor's own
    // geometry" shape is a real-round-flight death the sight gate cannot see), so 600 idle
    // ticks there only spend the clock. The round_dodge stream holds the POCKET line instead
    // (the block below).
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() && !w.round_dodge {
        kiko_hold(&mut w, 600, "the Kiko hold");
    }
    // With round_dodge on (the Kiko hold now ends at 52.5) the second hold costs more than it
    // saves: measured without it, death t31711 -> t35208 past the escape touch on the
    // Rook's-wake legs. It stays for any d3 run without the dodge.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() && !w.round_dodge {
        // A second, shorter hold at escape-touch node 4 (-44.2, 27.6) for the squad's tail (#31
        // killed Bond 2 u short of the touch on the one-hold stream). Measured: the touch is
        // reached and the walk dies on the Rook's-wake legs, t27517 -> t29197 (node 3 (-46.6,
        // 28.4) t27226; node 5 for 600 t29406 with less health).
        w.follow_nav_route([-44.2, 0.9, 27.6], "east to the second hold");
        kiko_hold(&mut w, 300, "the second hold");
    }
    // 00 Agent round_dodge stream: the shipped nav route's z-34 dip passes d 1.8-2.2 from the
    // pocket #20 (3 x 5.0 auto-hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] before the
    // fight lands) and then d 1.0 from #30 at (-44.8, 27.4) (3 x 5.0 the same way) — 40.0 of
    // the 50.0 east-walk bill [L the NF_DMG_TRACE run on master 9f5b13b6e]. A player crosses
    // east along the north side instead (the north lane measured 0 #20 hits on the landing-2
    // runs) and rejoins the nav route east of the pocket line.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() && w.round_dodge {
        // retune-nb-14-7 round 7: the round-6 ignore message's lever 1 is MEASURED DEAD — the
        // NF_KIKO_PROBE pass at the hold entry (tick 26224) read #20's hit boxes exposed with
        // infinite world-stop margin from the hold spot itself and ~30 grid spots (the sight
        // gate was NEVER the blocker), and the lever-1 pocket-line hold died the same way
        // (t26386, #20's post shot −1.93 at d 3.1, his hp 10 FULL — the fights' rounds did not
        // land from the lip either) [L both runs 2026-11-20]. The lever-2 Kiko-corridor armour
        // fetch (−41.29,1.53,36.48) is MEASURED BLOCKED: the north band east of the corridor
        // mouth wedges at (−46.82,1.06,42.38) (head_bump, segment_to_target 0.099, the
        // 6000-tick cap) on BOTH this round's attempts [L]. So the round_dodge stream keeps the
        // shipped lane crossing — the loop-mouth armour banked earlier (the lever-3 fetch at
        // the descent foot) now covers its bullet bill: the pocket #20's 1.93-5.0 post shots
        // and #30's 2.5s absorb into the armour instead of the health pool [M Player_HandlePain
        // 0x1902c8:84-104 — bullet pain (type 0) absorbs into armour first].
        // retune-nb-14-9 (the round-8 ignore message's lever 3, the dry-gun law, judged on the
        // probe's own verdict [L 2026-11-20 NF_SRV_PROBE quick run] and on this round's own v1
        // run [L]): lever 1 (the Kiko-corridor armour take from the lane's end) is MEASURED DEAD
        // read-only — nav_route gives NO ROUTE from the lane's end (−44.5,0.9,34.0) to the
        // armour (−41.29,1.53,36.48) AND none from the armour to the touch, so the take is a
        // blind walk the ledger forbids. The lever-3 law takes its place, and the v1 run fixed
        // its scope: the fights' LOW gate fires INSIDE the lane crossing leg itself (the
        // walk_to's mid-leg return-fire gate — the v1 resupply fired at t27840 mid-leg and its
        // nav walk went south through the pocket convergence zone into the kind-52 world
        // explosion at (−40.6,0.0,24.2) r 8.0, death t28518, the same tick as the v6 ledger's
        // shape) — so the no-fetch scope starts at the lane crossing, not after the aimed pass.
        // With the gun DRY/LOW west of the door, the fetch's only live target IS the
        // Kiko-corridor armour (in the lane's 0.5 floor band at the corridor's walk y 1.13). A
        // player whose gun runs dry past the pocket walks on; the wake legs' node-7 scoop
        // re-arms AFTER the touch. The flag clears after the touch cut-scene, so the wake legs
        // keep the shipped stream (their fights still fetch under the old law).
        w.dmg.no_lane_fetch = true;
        w.lenient = true;
        w.walk_to(Wp::new(-44.5, 0.9, 34.0).r(1.2), "the east lane above the pocket line");
        w.lenient = false;
        // An aimed pass on whoever the lane's end can see (the pocket #30 at d ~6.7) before
        // the nav route's d 1.0 point-blank pass (3 x 5.0 auto-hits + the fight's hit there).
        let pick = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 15.0 && (d.pos[1] - bp[1]).abs() < 2.0 })
                .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp)))
                .filter(|&di| walker::DamageLog::visible(sim, di))
        };
        if let Some(di) = pick { let _ = w.shoot_bounded(di, "the east lane's sighted guard", 300); }
    }
    w.follow_nav_route([-33.1, 0.9, 26.8], "east to the escape touch #456");
    let mut ch4a = false;
    for _ in 0..2000 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x4a) { ch4a = true; break; }
        assert!(sim.drones.channels.on(0x0c), "the room touch #456 fired 0x0C on contact [D]");
    }
    assert!(ch4a, "the escape cut-scene must write ch 0x4A [D]");
    // retune-nb-14-9: the touch is banked — the no-fetch law's scope ends here (the cut-scene
    // holds the sim, no fights run inside it); the wake legs keep the shipped stream.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() && w.round_dodge {
        w.dmg.no_lane_fetch = false;
    }
    // fix-objective-banner: the monitor completes the objective the tick after the channel.
    for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][2], 4, "objective 3 completes [M 0x1827d8]");
    for _ in 0..900 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(nf_game::player::Input::default(), false);
    }

    // Leg 5: east to Rook's wake touch #476 — walked; SEQ_AND #475 wakes him.
    //
    // 00 Agent real health (real-health-00agent-14-landing-5): the SAME shipped route walked
    // node-by-node — ONE nav_route read from `cur_pos()` (follow_nav_route's own computation)
    // and the same walk_to r(1.3) per node — with the delta approach's VISIBILITY-keyed scans
    // inserted after nodes 1, 2 and 21. The shipped stream's fights only start AFTER the hit
    // (the return fire), and the measured wake-legs tax is exactly those first rounds [L the
    // NF_DMG_TRACE run on master 8999eb99a]: #43 -3.50 at d 13.3 (t39481, fired during node 2's
    // leg — answered before the round only from node 1), #24 -14.0 with the -7.00 at d 1.0 ON
    // the path (t40127-277; the walker's own node-2 fight took him 10 -> 4 but the node arrival
    // cut it — the scan finishes him before his first round), #40 -7.00 at d 9.1 (t41426) and
    // #41 -2.06 at d 10.6 (t41509 — the death, 83 ticks after node 21's door E-open, before any
    // fight could start). The scans are CAPPED (300 ticks a fight): no 900-tick repositioning
    // strafe ever fires inside the cap and no hostile stands within the 4.0 back_off band at
    // the scan spots, so the fights cannot shove Bond off the route (the landing-4 scan death
    // — the live wire at (-32.2, 14.6) — came from an UNCAPPED fight's repositioning). Other
    // difficulties keep the shipped follow_nav_route byte-exact.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() && w.round_dodge {
        // real-health-00agent-14-landing-6: the wake legs' fights run to the kill (finish_fights) —
        // a half-finished fight left #24 alive on the route to close to 1.0 u [L the 00 Agent walk].
        w.dmg.finish_fights = true;
        // One-run diagnostic (landing-5): what Bond carries into the wake legs.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let owned: Vec<String> = (0..0x50usize)
                .filter(|&w| sim.drones.inventory.owned.get(w).copied().unwrap_or(false))
                .map(|w| {
                    let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w);
                    let clip = sim.drones.inventory.clip.get(slot).copied().unwrap_or(0);
                    let ammo = sim.weapons.ammo_type(w);
                    let spare = sim.drones.inventory.carried.get(ammo).copied().unwrap_or(0);
                    format!("{w:#x}(c{clip}+{spare} dmg{:.1})", sim.weapons.bullet_contact_damage(w as u16))
                })
                .collect();
            eprintln!("[014] wake-legs loadout: in-hand {:#x} clip {} spare {} dmg {:.1}; owned {:?}",
                sim.gun.weapon, sim.gun.clip, sim.gun.spare,
                sim.weapons.bullet_contact_damage(sim.gun.weapon as u16), owned);
        }
        // retune-nb-14-9 v4 MEASURED NET-NEGATIVE (the round-9 ignore message's lever 1, the
        // loadout law, judged on its own real run [L 2026-11-20]): the 0x11 (the CROSSBOW [M
        // player.rs:2696], c2 dmg 60.0 in the loadout dump) switch never reached a fight — the
        // 90-tick real-input switch stand itself re-rolled the deep stream (the wake-branch
        // fights drew during the stand; the wake start health 41.7 vs the v3 stream's 57.7; the
        // wake route recomputed, a node-0 resupply fetched Bond west into #30's line, death
        // t36285 vs the v3 tree's t36527). The switch stand IS the change — removed; the loadout
        // law needs a switch that does not stand Bond (a switch DURING a leg's own walk), next
        // round's design question.
        let wake_pts = {
            let sim = w.scene.sim.as_ref().unwrap();
            nav_route(sim, w.cur_pos(), [20.3, 0.9, -23.5])
        };
        match wake_pts {
            Some(pts) => {
                for (k, p) in pts.iter().enumerate() {
                    let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3),
                        &format!("east to Rook's wake touch #476 node {k}"));
                    // retune-nb-14-10 lever 1 (the round-9 ignore message's lever 1, the
                    // wake-legs node-1 ambush law): after node 0's leg (the spot the landing-5
                    // note itself names — '#43 answered before the round only from node 1'), the
                    // delta approach's OWN scan shape (route_scan_ranked, up to 4 fights, each a
                    // bounded 300-tick stand, the STRICT gun-ray gate, the road-blocker rank
                    // anchored on the remaining route). The round-9 v3 ledger [L 2026-11-20]:
                    // the walk died INSIDE node 1's leg at (−34.18,1.13,21.37) — #62 (skin
                    // 0x5000090, 0x14, hp 10 FULL, post (−30.4,1.0,22.1), d 3.85 from the node-1
                    // point) dealt 28.0 in 11 hits unanswered while the fight machinery
                    // exchanged with #43/#31 — the shipped stream's fights only start AFTER the
                    // hit, and node 1's leg opens inside #62's GL line with nobody answered
                    // before the round. #62's post sits BETWEEN node 0 (−30.84,1.03,24.32, d
                    // 2.26) and node 1 — the scan at node 0 answers him (and any gun-ray-clear
                    // blocker) BEFORE node 1's leg, the same law that carries the legs from
                    // node 21. Zero ticks when no gun-ray-clear candidate stands (the pick
                    // breaks before any stand), so the sisters and the no-ambush streams stay
                    // byte-identical; the candidate dump also answers the round-9 live-check
                    // question (is #62 in gun-ray sight from node 0?) read-only [L this run].
                    if k == 0 { delta_approach_scan(&mut w, k); }
                    // Two stops, the measured killers' answer points: node 2 (#24's post is ON
                    // the route — the shipped first round -7.00 lands at d 1.0 (t40127) with the
                    // walk past node 4; answered from node 2 at 4.2 u BEFORE the round) and
                    // node 21 (the delta approach's posts #40/#41 — their first rounds land 36 /
                    // 119 ticks after node 21's door E-open (t41426/t41509 — the shipped death),
                    // before any return-fire fight could start). The earlier nodes stay
                    // untouched: the every-node scan variant measured the SAME -26.56 bill
                    // redistributed (#39, clean on the shipped stream, converged east off his
                    // post and killed the walk) — the fights' windows draw the convergence.
                    // node 21 only: the delta approach's posts. The pre-node-21 scans measured
                    // NET NEGATIVE (runs 4-11: the fights' noise re-rolls the later guards'
                    // convergence — #39, clean on the shipped stream, converged east and killed
                    // the walk; the node-2 scan's #36 fight lost to his cover and drew his
                    // convergence kill; the scoop/switch pulled Bond 13 u off-route) — the
                    // shipped stream's own fights carry the early legs and stay byte-exact.
                    if k == 21 { delta_approach_scan(&mut w, k); }
                    // real-health-00agent-14-landing-6: after node 7 Bond picks up the dropped
                    // guns of the guards the legs killed (live pickups within 8 u, same floor) —
                    // the wake fights empty the 0x1a otherwise (out of ammo on #37 at node 8 [L]).
                    if k == 7 {
                        for j in 0..5 {
                            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
                            let t = w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|p| !p.gone && !p.hidden)
                                .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                                .filter(|q| walker::dxz(*q, bp) < 8.0 && (q[1] - bp[1]).abs() < 1.5)
                                .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)));
                            let Some(t) = t else { break };
                            w.walk_nav_to(t, &format!("the wake legs' drop {j}"), 0.4);
                        }
                        let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("back to node {k}"));
                    }
                }
            }
            None => {
                // follow_nav_route's own no-route fallback (the direct chain), scan stops kept
                // out of it — the fallback never fired on the measured runs.
                let sim = w.scene.sim.as_ref().unwrap();
                let mut v = walker::leg(sim, w.cur_pos(), [20.3, 0.9, -23.5]);
                v.pop();
                for (k, p) in v.iter().enumerate() {
                    let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3),
                        &format!("east to Rook's wake touch #476 node {k}"));
                }
            }
        }
        w.walk_to(Wp::new(20.3, 0.9, -23.5).r(1.3), "east to Rook's wake touch #476 end");
    } else if !w.tops_up() {
        // retune-nb-14 (the d1 real-damage wake legs): the shipped single follow_nav_route's
        // fights only start on the hit, and the v2 run died at t36801 at (5.05,1.13,-23.31)
        // with the wake route's 0x1a posts #28 (post (9.0,1.0,-22.5)) and #29 (post (8.7,1.0,
        // -21.5)) dealing 65.2 at d 4.0 before the return-fire fights finished them (hp -81/-73)
        // — the same first-rounds tax the d3 chain's node scans answer. The d1 run walks the
        // SAME shipped route node-by-node (one nav_route read from cur_pos, the same r(1.3)
        // legs [L the d3 branch's byte-exact note]) with the delta_approach_scan after node 21;
        // the d3 chain measured the every-node scans NET NEGATIVE, so node 21 only.
        // retune-nb-14-6 (the merged master re-rolled the d1 stream; the re-measure on THIS tree
        // [L 2026-11-20, NF_DMG_TRACE run] hit the SAME t36801 death, 41 hits for 100.0, 23
        // fights — the ledger stands): the fights' answer laws, the round-5 door-face family.
        // (a) shoot_blockers: the re-measure's fight#28 opened MID-LEG at (−7.9,−23.1) — 16 u
        //     west of the post — and its stand read `blocked true` for the whole 300-tick cap
        //     (aim Some, err 0.000 from t60, clip 15 spare 12 UNTOUCHED — zero taps) while #28
        //     converged 16 u → 4 u dealing −2.80 a round: the line held a live hostile (the
        //     converging #29 off his (8.7,−21.5) post; corpses and dead drones are skipped by
        //     the blocker scan). A player whose line a guard walks into shoots THAT guard. d1
        //     wake legs only — every other walk keeps the hold.
        // (b) switch_on_first_blood: the round-5 00agent door-face law (the ignore message's
        //     named candidate) — the fight turns to a NEW shooter's first blood without the
        //     30-tick cadence and the 3.0 margin.
        // (c) retune-nb-14-6 run 2 [L 2026-11-20]: run 1 (a+b) was BYTE-IDENTICAL — no
        //     retarget fired (the blocker is not a live ENABLED hostile) and no first-blood
        //     switch fired — and the death stayed t36801. The traced baseline [L
        //     NF_DMG_TRACE/NF_RF_TRACE] names the decisive hole: after the failed #28 response
        //     (the 300-tick blocked stand + the 27-round determined pass, ONE landing, dry at
        //     t36205) the leg's return-fire gate could NOT open a fight for ~600 ticks — the
        //     plain pick returned the cooled #28 first (his 240-frame damage crossed the 15.0
        //     lethality override) and the response bailed on the cooldown check while #29 (off
        //     cooldown, hp 10, d 1.6) chewed UNANSWERED — 35.0 of health gone between the
        //     responses, the guards converged d 16 → 1.1-1.7, and the guard grenade's blast
        //     (world kind-52 explosion at (13.3,0,−21.2)) finished Bond at 3.8 health (it dealt
        //     BOTH guards 83 — their ledger hp −81/−73 is the blast, not Bond's rounds).
        //     hurting_first is the pick-side law for exactly this shape (the 07000008 d3
        //     death's item 1): a cooled guard no longer silences the response.
        // (d) fire_through_disabled: the run-1 no-retarget means the first stand's blocker
        //     fails `enabled` (or team) — a dormant (WAIT_SWITCH) guard parked on the line is
        //     the ledger's own shape ("the not-yet-enabled guard standing on the line held
        //     every tap, killed false x5"). The game's own collide drops Enable(0) objects
        //     [M Collide_PickObj 0x1eae80] so a disabled drone never stops a round; the hold
        //     must not either.
        w.dmg.hurting_first = true;
        w.dmg.fire_through_disabled = true;
        w.dmg.shoot_blockers = true;
        w.dmg.switch_on_first_blood = true;
        w.dmg.finish_fights = true;
        let wake_pts = {
            let sim = w.scene.sim.as_ref().unwrap();
            nav_route(sim, w.cur_pos(), [20.3, 0.9, -23.5])
        };
        match wake_pts {
            Some(pts) => {
                for (k, p) in pts.iter().enumerate() {
                    let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3),
                        &format!("east to Rook's wake touch #476 node {k}"));
                    if k == 21 { delta_approach_scan(&mut w, k); }
                    // retune-nb-14: the wake fights empty the clip — the v3 run went dry on #28
                    // at node 26 ("shooting drone 28 ... clip 15 spare 12" at d 15.8, "out of
                    // ammo at +572", the determined pass killed false on an empty gun) and the
                    // UNARMED walk then died to the alive posts #28/#29 across nodes 27-30
                    // (65.2, the t36801 death). A player scoops the drops the legs left (the
                    // d3 chain's node-7 block, keyed on the dry gun instead of a fixed node so
                    // a re-roll cannot strand it). No fights here — no scan-noise re-rolls.
                    {
                        let dry = { let sim = w.scene.sim.as_ref().unwrap();
                            sim.gun.clip + sim.gun.spare < 10 };
                        // the PEACE gate (v14's lesson): the scoop walks run the walk's own
                        // fight machinery, and a dry gun mid-scoop fires the fights' resupply
                        // — v14's node-2 scoop was dragged into #1's convergence zone (45.0 in
                        // 10 hits, death t9879). A player scoops drops only where nobody
                        // lives: no live team-1 guard within 12 u.
                        let peace = { let sim = w.scene.sim.as_ref().unwrap();
                            let bp = sim.bond.pos;
                            !sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse
                                && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 12.0) };
                        if dry && peace {
                            for j in 0..5 {
                                let bp = w.scene.sim.as_ref().unwrap().bond.pos;
                                let t = w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|p| !p.gone && !p.hidden)
                                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                                    .filter(|q| walker::dxz(*q, bp) < 8.0 && (q[1] - bp[1]).abs() < 1.5)
                                    .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)));
                                let Some(t) = t else { break };
                                w.walk_nav_to(t, &format!("the wake legs' drop {j}"), 0.4);
                            }
                            let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("back to node {k}"));
                        }
                    }
                }
            }
            None => {
                let sim = w.scene.sim.as_ref().unwrap();
                let mut v = walker::leg(sim, w.cur_pos(), [20.3, 0.9, -23.5]);
                v.pop();
                for (k, p) in v.iter().enumerate() {
                    let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3),
                        &format!("east to Rook's wake touch #476 node {k}"));
                }
            }
        }
        w.walk_to(Wp::new(20.3, 0.9, -23.5).r(1.3), "east to Rook's wake touch #476 end");
    } else {
        w.follow_nav_route([20.3, 0.9, -23.5], "east to Rook's wake touch #476");
    }
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
    // real-health-00agent-14-landing-6: Rook's room is a crossfire of 0x1a guards at 1-3 u
    // (-7.0 a round at that range [L the 00 Agent walk]); Bond takes the grenade launcher
    // (0x14, carried since the landing) for the room. 00 Agent real health only.
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() && w.round_dodge {
        for _ in 0..90 {
            if w.scene.sim.as_ref().unwrap().gun.weapon == 0x14 { break; }
            w.tick(nf_game::player::Input { switch_to: Some(0x14), ..Default::default() }, false);
        }
    }
    let mut dead = false;
    for attempt in 0..3 {
        // Close in first: from the wake touch Rook stands ~67 u off, past the hand gun's useful
        // spread (the walk emptied both clips at that range and then wedged its resupply legs).
        let rp = w.scene.sim.as_ref().unwrap().drones.list[rook_i].pos;
        if dxz(w.scene.sim.as_ref().unwrap().bond.pos, rp) > 10.0 { w.walk_nav_to(rp, &format!("close on Rook {attempt}"), 8.0); }
        if w.shoot(rook_i, "Rook") { dead = true; break; }
        assert!(w.resupply(&format!("Rook attempt {attempt}")), "no ammo for the Rook kill");
    }
    assert!(dead, "Rook must die to the walked gun path");
    // The death channel's delivery [M 0x13e660 via the death DSTATEs: Death_Anim 0x1637f8 and
    // DeathByExplosion 0x163a80, case 2 = the state EXIT] waits for the end of the fall; only
    // 07000002 arms the case-0xc timer. So the read allows a whole fall, up to 213 ticks measured.
    let mut ch54 = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x54) { ch54 = true; break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert!(ch54, "Rook's death channel 0x54 fires [D param 12; M 0x13e660; M 0x163a80:44-59 - the death DSTATE's case 2/3/0xc delivery]");
    // fix-objective-banner: the monitor completes the objective the tick after the channel.
    for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][3], 4, "objective 4 completes [M 0x1827d8]");
    // The monitor's same tick announces the LAST objective: record 5's enable is 0x54 [D DEF],
    // and `Mission_MonitorObjectives` 0x1827d8 case 1 (record state 1, `enable != 0/ff` and
    // `switch_channels[enable] != 0`) binds the text and arms the record 3 — the banner the
    // player sees standing over Rook ("Take the exit to Delta Sector" [D text 0x04000069]).
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][4], 3,
        "objective 5 announces on its enable 0x54 [M 0x1827d8 case 1; D DEF enable]");

    // Leg 6: to the Delta exit #944 — walked; E fires the fallback write + the load.
    //
    // The exit pocket is OFF-NAV: the probe's `nav_route` from the wake-touch spot returns None
    // (the level's own bots never route here — a player-only pocket), so the walk is the player's
    // real approach: the double slide doors #121/#122 (24.02,1.2,−22.8 [D]) are the passage's
    // doors — stay on the approach floor (the direct steer descends onto the y 0.65 shelf south
    // of the doorway and wedges on its east reveal, the probe's west-facing face), press E (a
    // TOGGLE [M Door_Activate 0x135be0]; the slides answer from CLOSED and from CLOSING — the
    // activate turns a closing slide back to OPENING), through the doorway, then the exit.
    // level-13-guards-start-alerted-8: the RNG-shifted tree's direct leg wedged at
    // (23.31,1.14,−24.64) — master's unshifted route slipped the doorway diagonally; the three
    // legs below are the approach the probe walked green from BOTH the leg-6 start and the wedge
    // spot itself (E reacted, the exit leg arrived at +137 ticks).
    // Arriving from the north (the west door at (27.8, -19.6)), the straight line to the approach
    // spot catches the wall west of (25.5, -18.7); square up south first.
    // Rook's kill can land anywhere on his run (he flees the wake spot): from more than a room
    // away, walk the nav web back to the doorway's north side before the squared-up legs.
    if dxz(w.scene.sim.as_ref().unwrap().bond.pos, [26.0, 1.0, -22.0]) > 8.0 {
        w.walk_nav_to([26.0, 1.0, -22.0], "back to the delta doorway", 1.5);
    }
    w.walk_to(Wp::new(26.0, 1.0, -22.0).r(1.0), "the delta doorway east");
    w.walk_to(Wp::new(22.9, 1.0, -22.8).r(1.2), "the delta doorway approach");
    w.walk_to(Wp { at: [24.0, 1.2, -22.8], act: Act::Use { use_ticks: 240 }, arrive: 1.1 }, "E the delta slides");
    w.walk_to(Wp::new(27.79, 1.28, -25.32).r(1.0), "to the Delta exit #944");
    // A player at the exit door clears the room before pressing E: a live guard between Bond
    // and the exit object eats the use ray (the mask-4 ray hits whatever stands in it), and
    // the d1 plain run's pocket was exactly that shape — the sisters' identical exit sequence
    // loaded warm, the d1 run's 180 pulsed ticks did nothing [L this tree's logs].
    clear_hostiles(&mut w, 12.0, 600);
    // The use pick reads the view: square up on the exit before E, as a player does.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let d = [27.79 - sim.bond.pos[0], -25.32 - sim.bond.pos[2]];
        sim.bond.yaw = d[0].atan2(d[1]);
        sim.bond.pitch = 0.0;
    }
    // E pulses (the edge-triggered mask 4 [M Input_Action]) while squared up on the trigger —
    // the old held-use loop spent its single edge on the loop's first tick, and when the
    // arrive band parked Bond on the rim (this tree's d1 run: arrived (26.94, 1.13, −24.81),
    // 0.99 from the #944 spot, "loaded" false after 60 held ticks [L]) the edge missed the
    // volume and the 59 held ticks after it were dead input. A player taps E facing the exit.
    let mut loaded = false;
    for k in 0..180 {
        let sim = w.scene.sim.as_ref().unwrap();
        let (bp, yaw) = (sim.bond.pos, sim.bond.yaw);
        let d = dxz(bp, [27.79, 1.28, -25.32]);
        let desired = (27.79f32 - bp[0]).atan2(-25.32f32 - bp[2]);
        drop(sim);
        let inp = nf_game::player::Input {
            turn: wrap_pi(desired - yaw).clamp(-0.08, 0.08),
            forward: if d > 0.55 { 0.4 } else { 0.0 },
            ..Default::default()
        };
        w.tick(inp, k % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(loaded && sim.drones.level_exit == Some(0x0700_0015),
            "E at the exit #944 loads 07000015 (EvilBase) [D]");
        assert!(sim.drones.channels.on(0xce), "the fallback arm wrote ch 0xCE (objective 5) [M 0x1b2318]");
    }
    // The LAST objective folds during the exit hold: the trigger wrote 0xCE this tick's trigger
    // pass, and the game's Mission_Update runs from Game_Run 0x1c9790:17 OUTSIDE the GS_IsPaused
    // gate (:30-38 wraps only SSys_Monitor/Light_Update/control_movement_object_handler) — the
    // 300-frame GS_PauseGame hold the exit pushes [M 0x1b2318:90-92] keeps ticking the monitor,
    // whose case 3 (record 4, channel ON) shows the complete banner and arms the record 4
    // [M 0x1827d8 case 3]. The port runs mission.update at the same pre-object point of every
    // step, so the fold lands the tick after the E press.
    let mut obj5 = 0u32;
    for _ in 0..6 {
        w.tick(nf_game::player::Input::default(), true);
        obj5 = w.scene.sim.as_ref().unwrap().drones.mission.states[20][4];
        if obj5 == 4 { break; }
    }
    assert_eq!(obj5, 4, "objective 5 completes during the exit hold [M 0x1827d8 case 3; M 0x1c9790:17 outside the pause gate; M 0x1b2318 fallback write]");
    // The monitor's tail AND is NOT this level's five records alone: the record walk filters the
    // rows by BaseMap [M 0x1827d8:26 — `(&DAT_002a4354)[row*10] == BaseMap`, the MISSION id], so
    // the AND spans every part of mission 0x07000014 — part 1's ch 0x8E and part 2's 0x8F/0x90
    // [D DEF c07000015/c07000016] are still off at this exit, the tail's `if (bVar9) does not
    // fire, and the mission-complete ch 0x64 (100) stays DOWN [M 0x1827d8:119-123]. It belongs
    // to the true mission end (part 2), not to a part exit; nothing else writes ch 100 here
    // [D the packs' 0xd events: 47/48/4a/0e/11/10/b8/92/93/b7/8a; the SP outs 7E/8/9/85/54].
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x64),
        "ch 0x64 stays down at a part exit — the tail AND spans all three parts' records [M 0x1827d8:26+119-123; D DEFs 07000015 ch 0x8E, 07000016 ch 0x8F/0x90]");

    // ---- The level's END: the exit hold runs out and the NEXT LEVEL LOADS — the game's own
    // sequence, not a poke. The E-press pushed `GS_PauseGame(1)` +
    // `GameFlow_PushState(300.0f, 7, 0xff)` [M 0x1b2318:90-92]: the world freezes while the fade
    // advances every frame [M 0x1c9790 tail]; when state 8 pops, state 3 (`ResetMap_Load`) runs
    // and the host drains `level_exit` [M 0x1c98c8:173-189], captures `Trigger_Activate`'s
    // `Player_RamSave` [M 0x1b2318:97-116] and serves the warm load (start_async_load's campaign
    // arm → `warm_exit_save`). The walk drains it the same way: tick out the hold, capture, load
    // 07000015 with the carry.
    while w.scene.sim.as_ref().unwrap().drones.exit_hold.is_some() {
        w.tick(nf_game::player::Input::default(), false);
    }
    let exit_pos = w.scene.sim.as_ref().unwrap().bond.pos;
    let (exit_health, exit_req) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.health.health, sim.gun.switch_request)
    };
    let exit_states20 = w.scene.sim.as_ref().unwrap().drones.mission.states[20].clone();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x4b),
            "part 2's enable 0x4b never fired in part 1 (the walk's own channel state) — the silo's spawn plate owns it [D row 21 enable 0x4b]");
    }
    let save = nf_view::warm_exit_save(w.scene.sim.as_ref().unwrap());
    assert_eq!(save.weapon, exit_req,
        "the save's weapon byte is M_CONTROL+0x63 as of the drain [M 0x195670:18-19]");
    let mut w15 = Walker::new(nf_view::load_with_carry(&iso, "07000015.bin", false, Some(&save)).unwrap());
    if difficulty >= 2 {
        // The 15 part's fights run the same lane law (the difficulty rides the warm save —
        // the guards hit at the Agent scale here too).
        w15.dmg.back_off = Some(4.0);
        w15.dmg.lane_fights = true;
    }
    {
        let sim = w15.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 93.0 && p[0] < 96.0 && p[1] > 40.0 && p[1] < 43.0 && p[2] > -2.0 && p[2] < 2.0,
            "the warm arrival spawns on the silo gantry start #1 — the level's own start, warm changes only the player state [M 0x1957a8:9 arm; D spawn #1], got {p:?}");
        assert_eq!(sim.gun.switch_request, exit_req,
            "Player_RamLoad restores the saved weapon as the switch request [M 0x1a7ef8 case 0x07000015 'request = the saved weapon'; M 0x1957a8]");
        assert!((sim.drones.health.health - exit_health).abs() < 0.01,
            "Player_RamLoad restores the carried health (the warm arm passes boost 0 [M 0x1a7ef8's eight 0-arg calls]) [M 0x195670:16 + 0x1957a8]");
        assert!(sim.drones.inventory.owned.get(2usize).copied().unwrap_or(false)
            && sim.drones.inventory.owned.get(0x11usize).copied().unwrap_or(false)
            && sim.drones.inventory.owned.get(0x4eusize).copied().unwrap_or(false),
            "the owned weapons ride the save (P2K — the gate that skips the fresh-14 kit [M 0x1a7ef8 case 0x07000015] — the crossbow, the laser) [M 0x195670:7 + 0x1957a8]");
        for &c in &[0x54u32, 0x85, 0x8c, 0xce] {
            assert!(sim.drones.channels.on(c),
                "the carried switch channel {c:#x} stays on across the load [M 0x195670:13 + 0x1957a8]");
        }
    }
    w15.wait_enabled(3000, "the silo arrival");
    w15.wait(120);
    {
        let sim = w15.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4b),
            "the spawn plate #484 fires ch 0x4b under the arrived Bond [D 07000015]");
        assert_eq!(sim.drones.mission.states[21].len(), 1,
            "MissionData row 21 lists exactly ONE record for the loaded part [D MissionData vaddr 0x2a42d8: level 0x07000015, count 1]");
        assert_eq!(sim.drones.mission.states[21][0], 3,
            "'Find the exit to Omega Sector' announces off the carried load's own spawn plate [M 0x1827d8 case 1; D row 21 enable 0x4b]");
        assert_eq!(sim.drones.mission.states[20], exit_states20,
            "part 1's records ride the load untouched — Mission_Init resets only the current + future parts [M 0x182270:68-129]");
    }
    eprintln!("[014] walked out in {} ticks; stuck {}; falls {}; the exit hold drained at {exit_pos:?} and 07000015 loaded warm, arrival {:?}",
        w.total_ticks, w.report.stuck_events.len(), w.report.falls.len(),
        w15.scene.sim.as_ref().unwrap().bond.pos);
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
///
/// level-14-hall-walk-in: the last placement is gone — the sabotage AND's inputs are NOT seeded
/// and Bond is NOT placed: the REAL walked route (walk_infiltration_to_handback + the shared
/// walk_the_west_route_to_the_switch_band) raises 0x87/0x88 through the walk's OWN laser cuts
/// [M FuseBox_Update 0x1adfe8; D placements 383/385] and stands him in the switch band on foot
/// (the south loop + the Kiko scene's west delivery; the closed 0x85 pair is never crossed [D
/// probe]) — the classes the route uses: Walker (Wp walk_to / follow_nav_route / climb_stairs),
/// the 0xEB TOUCH wrapper (Trigger_Touch 0x1b2aa8 / Trigger_Create 0x1b1720), Player_Activate
/// 0x1a5170, FuseBox_Update 0x1adfe8, Trigger_Activate 0x1b2318, Door_Update 0x1363c8.
#[test]
fn the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    // The touch exists with the disc's fields [D placement 399].
    let t_centre = {
        let sim = w.scene.sim.as_ref().unwrap();
        let i = sim.drones.triggers.iter()
            .position(|t| t.out == 0xa0 && t.mask & 4 != 0 && t.kind == nf_game::triggers::kind::TOUCH)
            .expect("the 0xA0 E-use touch (mask 4, kind 0xEB TOUCH) exists [D placement 399]");
        let t = &sim.drones.triggers[i];
        assert!(!t.tris.is_empty(), "the touch carries its object's volume [D oDroneTrigger1b]");
        eprintln!("[014] the E-use touch: placement {} AABB {:?}", t.placement, (t.lo, t.hi));
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    // The door starts locked; nothing has armed [M 0x1363c8:93-96].
    let door = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.ch_lock == 0x85)
        .expect("Door_Slide_11A/B with lock ch 0x85 exists [D placements 151/152]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.is_locked(door, &sim.drones.channels), "the door starts locked");
        assert!(!sim.drones.channels.on(0x51) && !sim.drones.channels.on(0x85) && !sim.drones.channels.on(0xa0),
            "nothing armed at load [D]");
    }
    // The walk-in (level-14-hall-walk-in): the REAL route from the vent start to the switch band.
    // The AND's inputs come from the walk's own fusebox cuts (leg 3b), not a seed write; the
    // arrival stands in the band on foot, no placement.
    walk_infiltration_to_handback(&mut w);
    walk_the_west_route_to_the_switch_band(&mut w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x87) && sim.drones.channels.on(0x88),
            "both fusebox group channels are the walk's own cuts' writes [M 0x1adfe8; D placements 383/385]");
        assert!(sim.drones.channels.on(0x8c), "AND [0x87, 0x88] raised 0x8C through the REAL mux [D AND plc]");
        assert!(sim.drones.doors.is_locked(door, &sim.drones.channels),
            "the 0x85 pair is STILL locked at the walked arrival — the route reaches the switch band \
             from the west without raising its lock or crossing its plane [M 0x1363c8:93-96; D placements 151/152]");
        assert!(!sim.drones.channels.on(0xa0) && !sim.drones.channels.on(0x51),
            "standing in the touch's volume without E fires nothing [M 0x1b1fc8 mask 0x4]");
        eprintln!("[014] the walked arrival in the switch band: {:?}", sim.bond.pos);
    }
    // Aim at the touch's volume centre and press E through the REAL Player_Activate path
    // (the use sphere head + 1×look [M 0x1a5170:100-108]) — the walk's turn/look steering
    // (the main walk's leg-3c convention), no teleport-aim.
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
    // The SEQ_AND's write is a LEVEL write recomputed per tick while its inputs hold — but the
    // touch's 0xA0 is a 1-tick pulse (Trigger_Calc 0x1b1900's type-2 clear each frame), so 0x51
    // pulses too; the cascade's first link reads it inside the pulse window and the rest ride
    // the death channels (never cleared). Track "ever on" for the pulse links, level for 0x85.
    let (mut seen51, mut ch85) = (false, false);
    for _k in 0..560u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let ch = &sim.drones.channels;
            if ch.on(0x51) { seen51 = true; }
            if ch.on(0x85) { ch85 = true; }
        }
        // The walker's own tick tops the health up (the harness's enemies-disabled allowance).
        w.tick(nf_game::player::Input::default(), false);
        if ch85 { break; }
    }
    assert!(seen51, "SEQ_AND [0x8C, 0xA0] raised 0x51 (the 1-tick pulse) [D placement 393]");
    assert!(ch85, "the cascade ends at box #729's death 0x85 [D the kill→death pairs]");
    // The door unlocked for good and AUTO-OPENED on the rising edge [M 0x1363c8:88-107].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.doors.is_locked(door, &sim.drones.channels),
            "Door_Slide_11A/B unlocks on the 0x85 rising edge [M 0x1363c8:88-107]");
        assert_eq!(sim.drones.doors.list[door].ch_lock, 0, "the lock field cleared [M 0x1363c8:106]");
        assert!(sim.drones.doors.list[door].state & 0xC != 0 || sim.drones.doors.list[door].pos01 > 0.0,
            "the unlock edge OPENS the door (state {:?} pos01 {})",
            sim.drones.doors.list[door].state, sim.drones.doors.list[door].pos01);
    }
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.list[door].pos01 > 0.5,
            "the door slides open (pos01 {:.2})", sim.drones.doors.list[door].pos01);
    }
    eprintln!("[014] the cascade test's walked run: {} ticks; stuck {}; falls {}",
        w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

/// The infiltration route to the SP #276 grab, with the harness holding the scene OPEN: the
/// pinned legs 1a..1f verbatim (the same real-input path), then the connector crossing as
/// manual real-input ticks with the disabled watch — the pinned `try_walk_to` burns its cap
/// while the scene plays INSIDE the leg, and the skip harness needs the grab tick. Returns
/// (grab tick, grab pos, grab yaw).
fn walk_to_the_vent_grab(w: &mut Walker) -> (u32, [f32; 3], f32) {
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    hop_into_the_trench(w);
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    w.walk_to(Wp::new(48.8, -0.6, 52.2).r(0.7), "the corridor south to the stairwell lane");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 44.0).r(0.7), "the corridor south to the mouths' lane");
    claw_up_from_the_mouths_arm(w);
    w.walk_to(Wp::new(45.0, 2.4, 40.0).r(0.8), "the lane south past the mouth");
    let _ = w.try_walk_to(Wp::new(44.9, 2.4, 39.2).r(0.8), "the lane south to the connector");
    let mut grab = None;
    for _ in 0..1200 {
        if w.scene.sim.as_ref().unwrap().disabled { grab = Some(w.total_ticks); break; }
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let t = [40.7f32, 2.3, 39.0];
        let want = (t[0] - b.pos[0]).atan2(t[2] - b.pos[2]);
        let inp = nf_game::player::Input {
            forward: 1.0,
            turn: wrap_pi(want - b.yaw).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    let g = grab.expect("the vent cut-scene grabs Bond (disabled rises) on the connector crossing");
    let sim = w.scene.sim.as_ref().unwrap();
    (g, sim.bond.pos, sim.bond.yaw)
}

/// The SKIP hand-back PINNED (level-14-skip-handback): skipping the vent cut-scene with the
/// game's own skip action (`Input_Action(-1, 0x19, 1)` [M 0x1dbda8:129-141] → `Script_FFwd`
/// 0x1dac78 → the catch-up `Script_Update` 0x1da998:19-100) hands Bond back at HIS OWN MATRIX —
/// the #301 grab spot — exactly like the natural end [M `Script_EventHandler` 0x1d94c0 case 6:
/// the previous stream (s13, Wait/Marker only [D the pack dump]) has kind 0, so the capture is
/// the `glb_players+0x90` fallback; `Script_Free` 0x1da880:44-48 → `Player_Enable` 0x195498:38-52
/// applies si+0x40 verbatim]. The pre-fix port captured the FROZEN scripted-body matrix here
/// (the removed [S] arm; the case-2 freeze `Script_Interp` 0x1d9a08:146 stops the body at its
/// spawn pose on an early press) — the real play's in-wall landing (26.70, 1.03, 36.27,
/// yaw 2.122) @frame 3046. The skip also COMPRESSES the scene [M 0x1da998: the do-while runs the
/// remaining script once the skip fade passes its length] and the scene's writes still land
/// (ch 0x47, objective 1 [M the same op-0x12 path]).
#[test]
fn the_skipped_vent_cutscene_hands_bond_back_at_his_grab_spot() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    let (grab_t, grab_p, grab_yaw) = walk_to_the_vent_grab(&mut w);
    eprintln!("[014] the grab at tick {grab_t} ({:.2},{:.2},{:.2}) yaw {grab_yaw:.3}", grab_p[0], grab_p[1], grab_p[2]);
    // Press the skip two ticks into the scene (a player mashing X as the letterbox lands) —
    // well before the scripted body's stream even starts (t294 [D the pack dump]).
    for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
    w.scene.sim.as_mut().unwrap().drones.scripts.skip_request = true;
    let press_t = w.total_ticks;
    w.tick(nf_game::player::Input::default(), false);
    let mut released = None;
    for _ in 0..3600 {
        if !w.scene.sim.as_ref().unwrap().disabled { released = Some(w.total_ticks); break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    let r = released.expect("the skip releases Bond (the catch-up finishes the scene)");
    // The compression: the release lands right after the skip fade (min(remaining,45) [M
    // 0x1dac78:56-58]) plus the catch-up — NOT at the scene's natural end (~1400 ticks later).
    assert!(r - press_t < 150, "the skip compressed the scene [M 0x1da998:19-100], released {} ticks after the press", r - press_t);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let dyaw = wrap_pi(sim.bond.yaw - grab_yaw).abs();
        assert!((p[0] - grab_p[0]).abs() < 0.35 && (p[1] - grab_p[1]).abs() < 0.35 && (p[2] - grab_p[2]).abs() < 0.35,
            "the skipped cut-scene hands Bond back at his own matrix — the grab spot [M 0x1d94c0 case 6 fallback glb_players+0x90; 0x1da880:44-48; 0x195498:38-52], grab {grab_p:?} got {p:?}");
        assert!(dyaw < 0.05, "the hand-back keeps Bond's own yaw [M 0x195498 Mat_GetDir of his own matrix], grab {grab_yaw:.3} got {:.3}", sim.bond.yaw);
        eprintln!("[014] the skip hand-back at {p:?} yaw {:.3} (+{} ticks after the press)", sim.bond.yaw, r - press_t);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x47), "the skipped scene still writes ch 0x47 [M 0x1da998:19-100 — the catch-up runs every op 0x12]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[20][0], 4, "objective 1 completes on the skip [M 0x1827d8]");
    // He STANDS there (no in-wall wedge, no fall): 120 settled ticks on the mezzanine floor.
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(sim.bond.grounded && p[1] > 2.0 && p[1] < 4.14,
            "Bond stands on the mezzanine after the skip [D the #301 volume band], got {p:?} grounded {}", sim.bond.grounded);
    }
}

/// Real damage: the infiltration → sabotage → Rook → delta-exit walk with the health top-up off
/// for every walker it makes and the walker's return fire on — Bond's death panics with the death
/// spot and the last hits (common/damage_log.rs), so a pass pins that the walk completes on the
/// port's own damage. No health/armour/ammo writes (the fusebox cuts are the real Laser path,
/// Rook's kill the real gun path); the reach-a-spot hops INTO the cut-scene bands the walk cannot
/// enter stay the master walk's documented stand-ins (header: the only teleports, per leg) and
/// write no health/armour/ammo; the exit load into 07000015 is the game's own Trigger_Activate
/// path [M 0x1b2318 → warm_exit_save → Player_RamLoad 0x1957a8].
///
/// UN-IGNORED (retune-nb-14): green under nb-scan-all on 2026-11-19 — walked out in 41051 ticks,
/// stuck 0, falls 0, the exit hold drained and 07000015 loaded warm. The nb-scan-all deaths the
/// retune answered, each on its own real run: (1) the south-loop GL ambush (drone #12 skin
/// 0x5000087 post (-23.1,1.0,-14.5), #13 skin 0x5000090 post (-24.5,1.0,-10.7), both 0x14)
/// killed the shipped-fight walk at t18133 — Bond stood d 0.9-3.0 for 340 ticks taking 2.0/hit
/// rounds plus their blasts (51.0 in 4 world hits, the 29.0 blast finished him) — with the d3
/// fight law's back_off 4.0 (out of the every-round-hits band [M DroneWeap_DoBulletAccuracy
/// 0x179740:113-115]) + finish_fights, d1-gated `!tops_up()` so the topped-up walks stay
/// byte-identical; the v1 run's lane_fights resupply fetches parked Bond at the descent's
/// node-57 leg for the 6000-tick cap, so lane_fights stays OFF for d1 (the plain aim answered
/// the ambush fine — it wounded #12 in the v0 fight). (2) The wake legs' 0x1a posts #28/#29
/// (posts (9.0,1.0,-22.5)/(8.7,1.0,-21.5)) dealt 65.2 at d 4.0 before the return-fire fights
/// finished them, and the fight on #28 at d 15.8 burned clip 15+spare 12 dry ("out of ammo at
/// +572") so the unarmed walk died at t36801 — the d1 run now walks the SAME shipped wake route
/// node-by-node (one nav_route read from cur_pos, the same r(1.3) legs [L the d3 branch's
/// byte-exact note]) with the delta_approach_scan after node 21 (the d3 chain's own law) and
/// the d3 chain's loot pass keyed on the dry gun (clip+spare < 10) instead of a fixed node.
///
/// UN-IGNORED AGAIN (retune-nb-14-6): green on the merged master (ccd79b5be + s/retune-nb-14-5)
/// on 2026-11-20 — walked out in 40206 ticks, stuck 0, falls 0, the exit hold drained and
/// 07000015 loaded warm, health 40.0 into the Rook kill [L the --nocapture run]. The merged
/// master re-rolled the wake legs' stream back to the retune-nb-14-3-era death: the traced
/// re-measure (NF_DMG_TRACE/NF_RF_TRACE, run 0) hit t36801 at (5.05,1.13,−23.31), 41 hits for
/// 100.0, 23 fights — the SAME ledger the round-5 ignore message recorded. The measured death
/// shape [L run 0 + run 1]: (a) the #28 response's stand sat `blocked true` its whole 300-tick
/// cap 16 u west of the post (aim Some, err 0.000 from t60, clip 15 spare 12 untouched — zero
/// taps — a live-present drone that fails `enabled` held the line: with fire_through_disabled
/// the stand FIRED and killed #28 in 52 ticks [L the green run's fight#28 at t35382 killed
/// true] — the game's own collide drops Enable(0) objects [M Collide_PickObj 0x1eae80] so the
/// dormant guard never stopped a round and must not stop the hold); (b) the response's
/// determined pass burned clip 15+spare 12 into the corridor geometry (ONE landing, #28
/// 10 → 2.5, "out of ammo at +572", killed false); (c) THE DECISIVE HOLE: after the failed
/// response (t36205) the leg's return-fire gate could NOT open a fight for ~600 ticks — the
/// plain pick returned the cooled #28 first (his 240-frame damage crossed the 15.0 lethality
/// override) and the response bailed on the cooldown check while #29 (off cooldown, hp 10,
/// d 1.6) chewed UNANSWERED — 35.0 of health gone between the responses, the guards converged
/// d 16 → 1.1-1.7, and a guard grenade's world blast (kind-52 explosion at (13.3,0,−21.2))
/// finished Bond at 3.8 health — it dealt BOTH guards 83 (their ledger hp −81/−73 is that
/// blast, not Bond's rounds: the fights never once landed on them after the determined pass's
/// single round). hurting_first is the pick-side law for exactly this shape (the 07000008 d3
/// death's item 1: a cooled guard no longer silences the response) — the green run's fight#29
/// opened at node 30 (t35705, killed true) instead of the 600-tick silence. The walk's own
/// flags sit in the d1 real-damage wake-legs branch only (hurting_first, fire_through_disabled,
/// shoot_blockers, switch_on_first_blood — the last two measured NO-OPs alone on run 1's
/// byte-identical death): the earlier legs, the d2 agent walk and the tops-up walks stay
/// byte-identical.
#[test]
fn walk_the_infiltration_to_the_delta_exit_real_damage() {
    walker::with_real_damage(walk_the_infiltration_to_the_delta_exit);
}

/// The game's FirstAttack flush [M 0x148010:100-131] puts every p5=100 [D census] guard on his
/// second behaviour block at his first attack, and the west route's block-B words hunt the
/// walker down (the real-damage deaths: 107 hits across 19 fights). The walk clears the enabled
/// team-1 hostiles inside `radius` with the walker's loose-gate shots — real input only;
/// `shoot_loose`'s give-up keeps the walk moving when a sight line never opens.
fn clear_hostiles(w: &mut Walker, radius: f32, cap: u32) {
    let home = w.scene.sim.as_ref().unwrap().bond.pos;
    // Where each fight started: the walk home retraces them (with the guards' aim stop
    // (Drone+0x38) one loot pass led east to (-34.4, 27.3) and the fights there left Bond at
    // the door jamb (-33.1, 28.6), from where the nav's direct hop home hit the wall at 6 %
    // for 3 x 6000 ticks).
    let mut trail: Vec<[f32; 3]> = Vec::new();
    for _ in 0..10 {
        trail.push(w.scene.sim.as_ref().unwrap().bond.pos);
        let sim = w.scene.sim.as_ref().unwrap();
        let live = |d: &nf_game::drone::Drone| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
            && walker::dxz(d.pos, sim.bond.pos) < radius;
        // 00 Agent real health: nearest first - the list-order pick spent 900 ticks on #3 at
        // 18.2 u while #9 shot Bond from 1.2 u (65.0 in 13, the t7051 corridor death).
        let nearest = !w.tops_up();
        let pick = if nearest {
            sim.drones.list.iter().enumerate().filter(|(_, d)| live(d))
                .min_by(|a, b| walker::dxz(a.1.pos, sim.bond.pos).total_cmp(&walker::dxz(b.1.pos, sim.bond.pos))).map(|(i, _)| i)
        } else { sim.drones.list.iter().position(|d| live(d)) };
        let Some(di) = pick else { break }; // a clear that ends here still walks home below
        // The dropped-gun cycle is now a bounded test-local loot pass (this tree's measured
        // runs: the fights held Bond at 74.0 across 9-12 fights — the armour takes the traded
        // rounds — while the shared resupply walked Bond onto the trench lip for a drop
        // 2.38 away behind the rim (target (38.72, 0.00, 33.60) from (40.91, 0.38, 32.66),
        // unclosable) and burned ~12000 ticks across its 4 attempts before the wedge abort
        // killed the walk at 18540/18840. A player scoops what lies at his feet on his own
        // floor and keeps walking; everything else stays where it fell.
        {
            let (bp, target) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                let t = sim.drones.pickups.iter().filter(|p| !p.gone)
                    .filter(|p| (p.m[3][1] - bp[1]).abs() < 0.3)
                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                    .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)))
                    .filter(|t| walker::dxz(*t, bp) < 6.0);
                (bp, t)
            };
            if let Some(t) = target { w.walk_nav_to(t, "the clear's loot pass", 1.2); }
        }
        w.shoot_loose(di, "the hostile clear", cap);
    }
    // The fights displace Bond (the backpedal inside 3.0) — walk home over the NAV web (the
    // direct steer hits the door-wall jambs) so the caller's position pins and next legs start
    // where they would have without the clear.
    let mut last = w.scene.sim.as_ref().unwrap().bond.pos;
    for p in trail.into_iter().rev() {
        if walker::dxz(p, last) < 1.0 { continue; }
        w.lenient = true;
        w.walk_nav_to(p, "back along the clear's trail", 1.0);
        w.lenient = false;
        last = p;
    }
    w.walk_nav_to(home, "back to the clear's home spot", 1.0);
}

/// The game's FirstAttack alternate-state tail [M 0x148010:100-131] puts the trench guard
/// (placement #270: p5=100, p8=100, p21=3 [D census]) on his second behaviour block at his
/// first attack — +0xC6=3 → +0xC5 and +0x5A4 = DroneTypeSettings[row 3].alt
/// [M 0x14cbd8:103-114] — whose block B words [a0800200, df720813, 44280000] [D #270] hunt the
/// walker down: the real-damage walk died to his point-blank bursts (100.0 in 50 hits at 0.4 u)
/// mid-trench. The walk answers with its own range keeping at the crossing's landing — switch
/// to a carried gun, aim through the walker's exposed view, fire past 2.0, backpedal inside 3.0
/// (every drone round hits under 3.0 [M DoBulletAccuracy 0x179740:113-115], so the band is
/// held, not traded). Real input only; no teleports.
fn kill_the_trench_guard_270(w: &mut Walker) {
    let di = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.placement == 270).expect("the trench guard placement #270 exists");
    assert!(w.switch_to_armed("the trench guard's kill"), "a carried gun has rounds");
    // The hunter closes — the LOOSE aim gate (the patrolling-target pattern) keeps the taps
    // landing while he sweeps between ticks; the band under 3.0 is his, so the kill must land
    // before he arrives (hp 25 [L the death log]).
    assert!(w.shoot_loose(di, "the trench guard's kill", 2400),
        "the trench guard must fall (the game's hunt block chases the walker down)");
}

// ---------------------------------------------------------------------------
// level-14-tripbomb-real-route: the tripbomb through the PLAYER's path — no dev panel anywhere.
//
// The two PICKUP_Tripbomb placements #1015/#1016 (the audit's #1341/#1342) sit in the MAIN map
// (map 54 of 55), params (0:0)=WEAPON (1:58)=0x3a (2:5)=rounds [D the parsed archive; the probe's
// placement dump] — their translations ARE world coords: the sim spawns the pickups at
// (-32.187, 1.409, 31.122) and (-32.187, 1.409, 35.858) [L the probe's sim dump]. They hang on the
// armoury rack wall at x ≈ -32.0, the low row under the mounted HKOICW rifles (#1088-#1096 at
// y 1.70/2.34, z 31.04..36.21 [D placements]); the lane in front runs z 30.7..36.4 at floor 0.80
// and passes UNDER the elevated crate shelf at x -36 (open segments at y 0.9..2.2 across z 33-34
// [L the lane-entry probe]), so the rack front is reached on foot along the floor — no ledge.
//
// The route (every tick real `Input` through `step_bond`; the only teleports are NONE after the
// load — the walker's health top-up is the file's own walk allowance):
// 1. the east route + the SP #276 hand-back (`walk_infiltration_to_handback`, this file's pinned
//    chain — the SkyBridge Q-Claw crossing, the trench, the shaft reel, the vent drop);
// 2. the descent to the facility corridor (the shared west route's first block, duplicated so the
//    shared fn stays untouched);
// 3. the nav web's south loop to the armoury branch (nodes 0..73 of the 77-node route — the same
//    haul the passing west-route walk uses; the branch splits at (-30.34, 0.40, 23.13)), then the
//    floor legs east under the crate shelf to the rack front;
// 4. the TAKE through the game's pickup path: `Pickup_Update` 0x18e148's distance gate
//    (bounds.radius + 1.1 from player+0x30) + `Collide_LineOfSight` → `take_pickup`
//    (`Pickup_Handler` 0x18e650) adds weapon 0x3a to the inventory [M];
// 5. the REAL switch request (`Input.switch_to = 0x3a`) draws it;
// 6. the REAL fire throws it onto the rack wall: `Bullet_CollisionHandler` 0x12ce80's
//    +0x70 & 0x100 world-hit arm sticks it (state 3, normal-aligned, +0.002 off the face,
//    120-tick arm, then the beam [M 0x12ce80:325-334; fix_satchel_tripbomb_live pins the family]);
// 7. Bond walks OUT — south-west off the beam line (the beam runs 200 u west along +normal
//    [M Bullet_update case 3]) and back along the lane to the catwalk, ≥ 12 u from the body;
// 8. he switches back to the P2K (0x02, owned from the level start [M Inventory::level_start]),
//    aims at the body centre and fires: `Bullet_CollisionHandler` 0x12ce80's object arm →
//    `Bullet_handle_object_destruction` 0x12cd98 detonates the gadget — the 8/100 blast with the
//    0x0600_0052 script from its own record, the beam deleted with it [M] (the rule
//    tripwire_shot_live pins, reached here the way a player reaches it).
// The final position and the shot line are logged for the app replay.
// ---------------------------------------------------------------------------

type V3 = [f32; 3];
fn tv_sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn tv_add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn tv_mul(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn tv_dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn tv_len(a: V3) -> f32 { tv_dot(a, a).sqrt() }
fn tv_norm(a: V3) -> V3 { let l = tv_len(a).max(1e-9); tv_mul(a, 1.0 / l) }

const TRIP: u8 = 0x3a;
const P2K: u8 = 0x02;
const LASER: u8 = 0x4e;
const TRIP_MESH: u32 = 0x0200_074f;

/// The walker-file aim: steer `turn`/`look` (the walk's clamped ±0.08/tick) toward `at`.
fn steer_aim(w: &mut Walker, at: V3, fire: bool, use_key: bool) {
    let (want_yaw, want_look) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        let e = [b.pos[0], b.pos[1] + b.eye - b.crouch_drop, b.pos[2]];
        let d = tv_sub(at, e);
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        (d[0].atan2(d[2]), ((d[1] / l).atan() / std::f32::consts::FRAC_PI_2))
    };
    let sim = w.scene.sim.as_ref().unwrap();
    let inp = nf_game::player::Input {
        turn: wrap_pi(want_yaw - sim.bond.yaw).clamp(-0.08, 0.08),
        look: (want_look - sim.bond.pitch).clamp(-0.08, 0.08),
        ..Default::default()
    };
    if fire { w.tick_fire(inp); } else { w.tick(inp, use_key); }
}

/// The gadget's world sphere centre (obj+0x80): the pose · the model's glist bounds centre
/// [M Control_BuildWorldSph 0x133070; the same stand-in tripwire_shot_live aims at].
fn trip_body_centre(sim: &nf_view::scene::Sim, _gi: usize) -> V3 {
    // the caller's index can be stale (see trip_stuck_now) — re-find the stuck bomb
    let g = &sim.drones.grenades[trip_stuck_now(sim)];
    let m = g.pose();
    let c = sim.models.get(&TRIP_MESH).expect("the tripbomb model's glist bounds").centre;
    let mut out = [m[3][0], m[3][1], m[3][2]];
    for k in 0..3 { out[k] += c[0] * m[0][k] + c[1] * m[1][k] + c[2] * m[2][k]; }
    out
}

/// Bond's distance to the armed tripbomb's beam segments (the 1.0-u trigger's input).
fn trip_beam_dist(sim: &nf_view::scene::Sim, p: V3) -> f32 {
    sim.drones.tripbomb_beams.iter().map(|&(a, b)| {
        let ab = tv_sub(b, a);
        let t = (tv_dot(tv_sub(p, a), ab) / tv_dot(ab, ab).max(1e-9)).clamp(0.0, 1.0);
        tv_len(tv_sub(tv_add(a, tv_mul(ab, t)), p))
    }).fold(f32::INFINITY, f32::min)
}

fn trip_stuck_index(sim: &nf_view::scene::Sim) -> Option<usize> {
    sim.drones.grenades.iter().position(|g| g.weapon_type == TRIP as u16 && g.stuck && g.state < 2)
}

/// The placed tripbomb's CURRENT grenade index — the flush changes the guards' fight around the
/// placement, so a thrown guard grenade can enter and leave `grenades` at any time and the
/// placement-time index goes stale (the 2026-11-03 len-1/index-1 panics).
fn trip_stuck_now(sim: &nf_view::scene::Sim) -> usize {
    trip_stuck_index(sim).expect("the placed tripbomb is still stuck")
}

/// Type the keypad's code and hold the aim on `at` until its out channel rises [M Lock_Update
/// 0x1ae3d8 case 4; the 0x4E laser arm is kind 1 only, :47].
fn burn_keypad(w: &mut Walker, at: V3, ch: u32) {
    // A keypad opens by its typed code or the decryptor; the laser arm is kind 1 only [M 0x1ae3d8:47, :162].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        if let Some(li) = sim.drones.locks.iter().position(|l| l.out as u32 == ch && !l.unlocked) {
            let code = sim.drones.key_codes.code(sim.drones.locks[li].code);
            sim.drones.keypad = Some((li, nf_game::lock::Keypad::arm(code, false)));
            for c in code { sim.drones.keypad_digit((c - b'0') as u32); }
        }
    }
    for _ in 0..900 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(ch) { break; }
        steer_aim(w, at, true, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(ch),
        "the typed code at {at:?} raises ch {ch} [M Lock_Update 0x1ae3d8 case 4]");
}

/// The walked tripbomb route: pickup → place → retreat → shoot, everything on real input.
#[test]
fn walk_the_tripbomb_route_pick_up_place_and_shoot_it() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());

    // 1. The east route to the SP #276 hand-back (the mezzanine) — this file's pinned chain.
    walk_infiltration_to_handback(&mut w);

    // 2. The descent to the facility corridor (the shared west route's first block, verbatim —
    // duplicated so the shared fn keeps its exact shape for its own callers).
    {
        w.walk_to(Wp::new(34.63, 2.80, 37.94).r(1.0), "the pad west");
        w.walk_to(Wp::new(34.57, 2.80, 37.18).r(0.9), "the pad's south-west step head");
        open_the_pad_lid(&mut w);
        w.walk_to(Wp::new(33.09, 2.37, 35.38).r(0.8), "the 1.72 landing");
        shoot_the_drop_band_guard(&mut w);
        {
            let start = w.total_ticks;
            loop {
                assert!(w.total_ticks - start < 2400,
                    "the landing's west drop never cleared the overhang\n{}",
                    walker::fail_dump_pub(&w.scene, [31.8, 1.0, 34.0], "the landing west drop"));
                {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let p = sim.bond.pos;
                    if p[0] < 32.6 && p[1] < 1.4 && sim.bond.grounded { break; }
                }
                let tz = drop_aim_z(&w);
                let sim = w.scene.sim.as_ref().unwrap();
                let b = &sim.bond;
                let inp = nf_game::player::Input {
                    forward: 1.0,
                    jump: b.grounded && (w.total_ticks - start) % 30 == 0,
                    turn: wrap_pi((31.8f32 - b.pos[0]).atan2(tz - b.pos[2]) - b.yaw).clamp(-0.08, 0.08),
                    ..Default::default()
                };
                w.tick(inp, false);
            }
        }
        the_machine_room_to_the_corridor(&mut w);
        w.walk_to(Wp::new(30.09, 0.65, 38.73).r(1.2), "the corridor north to the leg-2 spot");
    }

    // 3. The south loop to the armoury branch. The route to the armoury ledge is 77 nodes — the
    // same long haul the passing west-route walk covers to node 71; the branch splits at
    // (-30.34, 0.40, 23.13). Nodes are walked leniently (one wedged node re-aims at the next);
    // the walk STOPS at node 73 (-38.27, 0.40, 33.21) — the catwalk head — and takes the floor
    // legs east instead of the 1.20 ledge nodes (walking at the ledge's node height climbs Bond
    // onto the 1.45 crate and wedges him against the elevated shelf — the lane is passed UNDER,
    // on the floor [L the lane-entry probe]).
    let route = {
        let sim = w.scene.sim.as_ref().unwrap();
        let from: V3 = sim.bond.pos;
        nav_route(sim, from, [-36.12f32, 1.20, 33.62])
            .expect("the nav web routes the south loop to the armoury ledge")
    };
    let stop_at = route.iter().position(|p| p[0] < -38.0 && p[2] > 32.5).unwrap_or(route.len());
    for (k, p) in route.iter().take(stop_at).enumerate() {
        let _ = w.try_walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("the armoury loop node {k}"));
    }
    // The clear runs on every difficulty: a squad that follows Bond into the lane walks the
    // armed beam and sets the bomb off before the retreat (the plain walk's measured run: six
    // guards in AIM_STAND_FIRE at (-40.5, 1.0, 33.4) behind him at the row, and the placed bomb
    // gone during the under-slab crawl). A player clears his tail before he sets a trap, then
    // steps back to where the loop ended (the clear's own walk home stops 1.0 short, and the
    // direct legs below start from the loop's end).
    let loop_end = w.scene.sim.as_ref().unwrap().bond.pos;
    clear_hostiles(&mut w, 26.0, 900);
    w.walk_to(Wp::new(loop_end[0], loop_end[1], loop_end[2]).r(0.3), "back to the loop's end");
    // North through the open doorway (-42.7, 1.2, 29.9) first: the direct steer from the loop's
    // end to the sill row grazes its east jamb and wedged at (-40.47, 0.65, 29.09) once the
    // clear had moved Bond 0.15 south.
    w.walk_to(Wp::new(-42.4, 0.65, 30.8).r(0.4), "north through the doorway");
    eprintln!("[014-trip] the loop done at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The game's own node path into the lane (the nav web's route branch→rack [L the nav probe]):
    // from the catwalk head the AI climbs the crate row and walks it EAST at y 1.20 — nodes
    // (-37.33, 1.20, 33.61) → (-36.74, 1.20, 33.61) → (-36.12, 1.20, 33.62) → (-35.02, 1.20,
    // 33.61) — then the lane's north node (-33.73, 1.20, 35.81) at the rack front. Floor-level
    // legs stall on the 1.45 platform at x ≈ -36.2 (the row is the intended deck; the under-shelf
    // floor passage at 0.80 is blocked to the steer-walker by the sill/column hops).
    for (label, wp) in [
        ("the sill row up", Wp::new(-37.33, 1.20, 33.61).r(0.5)),
        ("the row east 1", Wp::new(-36.74, 1.20, 33.61).r(0.4)),
        ("the row east 2", Wp::new(-36.3, 1.30, 33.62).r(0.35)),
    ] {
        let ok = w.try_walk_to(wp, label);
        eprintln!("[014-trip] {label}: ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }
    // The armoury door (-35.6, 0.8, 33.6) locks the under-shelf passage on ch 46 (0x2E) [L the
    // crawl fail dump: door kind 2, state 0x2, pos01 0.00, ch_lock 46]. Its writers are the
    // shelf-side keypad pair — 0x2e Lock, out 0x2C at (-35.84, 2.30, 32.17) and out 0x2D at
    // (-35.84, 2.30, 35.09) [D the 2026-11-02 census] — burned with the Q-Claw laser the player
    // already carries from the SkyBridge crossing.
    {
        let mut drawn = false;
        for _ in 0..240 {
            w.tick(nf_game::player::Input { switch_to: Some(LASER), ..Default::default() }, false);
            let g = &w.scene.sim.as_ref().unwrap().gun;
            if g.weapon == LASER && g.switch_state == 0 && g.reload_state == 0 { drawn = true; break; }
        }
        assert!(drawn, "the switch request 0x4e draws the Q-Claw laser");
        burn_keypad(&mut w, [-35.84, 2.30, 32.17], 0x2c);
        burn_keypad(&mut w, [-35.84, 2.30, 35.09], 0x2d);
        for _ in 0..120 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x2e) { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
        assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x2e),
            "the burned pair lift the door's lock ch 0x2E [L the door dump ch_lock 46]");
        // E the door open from the crate row (the walker's real use path).
        let (_, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [-35.6, 0.8, 33.6])
            .expect("the armoury door is the nearest door to the row");
        w.use_door(di, "the armoury door");
        // The slide needs to finish before the shot: a half-open panel leaves the crawl gap at
        // floor level but still intersects the crouched shot line at ~1.75.
        for _ in 0..400 {
            if w.scene.sim.as_ref().unwrap().drones.doors.list[di].pos01 > 0.9 { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
        eprintln!("[014-trip] the armoury door pos01 {:.2}; bond {:?}",
            w.scene.sim.as_ref().unwrap().drones.doors.list[di].pos01, w.scene.sim.as_ref().unwrap().bond.pos);
    }
    // The under-slab crouch crawl: the shelf slab (tops 3.5, underside ~2.2 — the lane-entry
    // probe's open band y 0.9..2.2) rides over the crate row; standing head (~3.2) hits it. The
    // game's crouch is a toggle: one press crouches, the crawl runs, one press stands when the
    // headroom clears [M Player_SSCrouch 0x1924c0:84-105]. try_walk_to has no crouch acts —
    // the crawl is the raw input path.
    {
        let start = w.total_ticks;
        w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
        loop {
            assert!(w.total_ticks - start < 1200,
                "the under-slab crawl never cleared east\n{}",
                walker::fail_dump_pub(&w.scene, [-35.3, 1.3, 33.61], "under-slab east"));
            if w.scene.sim.as_ref().unwrap().bond.pos[0] > -35.35 { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let inp = nf_game::player::Input {
                forward: 1.0,
                turn: wrap_pi((-35.35f32 - b.pos[0]).atan2(33.61 - b.pos[2]) - b.yaw).clamp(-0.08, 0.08),
                ..Default::default()
            };
            w.tick(inp, false);
        }
        for _ in 0..80 {
            w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
            if !w.scene.sim.as_ref().unwrap().bond.crouched { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
        eprintln!("[014-trip] under the slab east: bond {:?} crouched {}",
            w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.crouched);
    }
    // The rack-front node (the nav web's own lane node at the take spot).
    {
        let ok = w.try_walk_to(Wp::new(-33.73, 1.20, 35.81).r(0.4), "the rack-front node");
        eprintln!("[014-trip] the rack-front node: ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }

    // 4. The take: walk INTO the rack until the game's pickup touch adds the 0x3a.
    let before = w.scene.sim.as_ref().unwrap().drones.taken.len();
    let mut took_at = None;
    for i in 0..180 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.taken.iter().skip(before).any(|(_, k)| matches!(k, nf_game::pickup::Taken::Weapon { weapon: 0x3a })) {
            took_at = Some(i);
            break;
        }
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
    }
    let taken_at = took_at.expect("the rack-front walk takes the tripbomb through Pickup_Update's touch [M 0x18e148]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[014-trip] the 0x3a taken at +{taken_at} ticks, bond {:?}", sim.bond.pos);
        assert!(sim.drones.pickups.iter().any(|p| p.weapon == 0x3a && p.gone), "the taken pickup's tombstone [M take_pickup]");
    }

    // 5. The real switch request draws the tripbomb.
    let mut drawn = false;
    for _ in 0..240 {
        w.tick(nf_game::player::Input { switch_to: Some(TRIP), ..Default::default() }, false);
        let g = &w.scene.sim.as_ref().unwrap().gun;
        if g.weapon == TRIP && g.switch_state == 0 && g.reload_state == 0 { drawn = true; break; }
    }
    assert!(drawn, "the switch request 0x3a draws the picked-up tripbomb");

    // 6. Place it on the rack wall with the real fire: aim the eye-ray at the wall face in front
    // of Bond, hold fire until the gadget leaves the gun, then wait out the stick.
    //
    // The guards that follow Bond into the armoury throw frags at him. A frag blast that reaches
    // the placed round sets it off (`Explode_Propagate` 0x17c518 case 5 → 0x12cd98: the 0x3a
    // passes the id gate and the record gate, reach = the falloff > 0.0002, no line-of-sight
    // test [M]); on this route a frag landing in the pit 6.4 u away does exactly that. A player
    // whose bomb went off places the next one — the gun still holds more [spare 4 after
    // the first throw, this run] — so the placement and the first retreat leg repeat until a
    // bomb survives it.
    let mut attempt = 0u32;
    let mut first_aim_y: Option<f32> = None;
    let (gi, bomb_pos, bomb_n) = loop {
    attempt += 1;
    if attempt > 1 {
        let ok = w.try_walk_to(Wp::new(-33.73, 1.20, 35.81).r(0.4), "back to the rack-front node");
        eprintln!("[014-trip] re-place {attempt}: back to the rack-front node ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        for _ in 0..60 {
            w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        }
    }
    let (wall_hit, wall_n) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        let e = [b.pos[0], b.pos[1] + b.eye - b.crouch_drop, b.pos[2]];
        let (t, n) = sim.world.segment_hit(e, [e[0] + 6.0, e[1], e[2]]).expect("the rack wall in front of the take spot");
        ([e[0] + 6.0 * t, e[1], e[2]], n)
    };
    eprintln!("[014-trip] the rack wall face at {wall_hit:?} normal {wall_n:?}");
    // The throw aims 0.8 u lower than the eye-ray's wall hit: a bomb at the eye height (≈2.5)
    // sits above the under-shelf band (open y 0.9..2.2 [L the lane-entry probe]) and every 12+
    // u shot line from the catwalk is shadowed by the slab — a bomb at ≈1.7 keeps the P2K's
    // line under the slab (the catwalk eye 2.15 → 1.7 descends through the open band).
    // A re-place reuses the first throw's aim height (the walk back can leave the eye a little
    // higher, and the bomb must still sit under the slab).
    let aim_y = *first_aim_y.get_or_insert((wall_hit[1] - 0.8).max(1.2));
    let aim = [wall_hit[0], aim_y, wall_hit[2]];
    // Hold fire only until the gadget leaves the gun — Bond is 0.9 u from the wall, well inside
    // the future beam line, so the retreat must start the moment it sticks (the arm is 120
    // stuck ticks [M 0x12ce80:330]).
    let mut thrown = false;
    for _ in 0..240 {
        steer_aim(&mut w, aim, true, false);
        if w.scene.sim.as_ref().unwrap().drones.grenades.iter().any(|g| g.weapon_type == TRIP as u16) { thrown = true; break; }
    }
    assert!(thrown, "the held fire throws the tripbomb [M the throw arm]");
    let mut stuck = false;
    for _ in 0..150 {
        steer_aim(&mut w, aim, false, false);
        if trip_stuck_index(w.scene.sim.as_ref().unwrap()).is_some() { stuck = true; break; }
    }
    let gi = trip_stuck_index(w.scene.sim.as_ref().unwrap()).expect("the thrown tripbomb sticks on the rack wall [M 0x12ce80]");
    let (bomb_pos, bomb_n) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.grenades[trip_stuck_now(sim)];
        (g.pos, g.phase.as_ref().unwrap().normal)
    };
    assert!(stuck && bomb_n[0] < -0.7, "the stick faces back down the lane (normal {bomb_n:?})");
    assert!(bomb_pos[1] < 2.2, "the bomb sits in the under-slab band ({}) — the shot line stays under the shelf [L the open band y 0.9..2.2]", bomb_pos[1]);
    eprintln!("[014-trip] placed at {bomb_pos:?} normal {bomb_n:?}");

    // 7. Walk OUT: south off the future beam line first (the beam runs 200 u west along +normal
    // from the bomb at z ≈ 35.8 — the lane's z 33.6 crate row is 2.2 clear of it), then west
    // along the row (the node path reversed) and down to the catwalk, south to the 0.40 lane.
    // The arm is 120 stuck ticks [M 0x12ce80:330] — the retreat clears the line before it.
    for (label, wp) in [
        ("off the beam line south", Wp::new(-34.6, 1.20, 33.61).r(0.5)),
    ] {
        let ok = w.try_walk_to(wp, label);
        eprintln!("[014-trip] retreat {label}: ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }
    if trip_stuck_index(w.scene.sim.as_ref().unwrap()).is_some() { break (gi, bomb_pos, bomb_n); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[014-trip] attempt {attempt}: the placed bomb went off during the first leg; blasts {:?}; tripbombs left {}",
            sim.drones.explosions, sim.gun.spare + sim.gun.clip);
        assert!(attempt < 4, "three placed tripbombs went off before the retreat");
        assert!(sim.gun.weapon == TRIP && sim.gun.spare + sim.gun.clip > 0, "a tripbomb is left to place again");
    }
    };
    let _ = gi;
    // The under-slab crouch crawl west (the approach's crawl reversed; one press crouches,
    // one press stands [M Player_SSCrouch 0x1924c0:84-105]).
    {
        let start = w.total_ticks;
        w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
        loop {
            assert!(w.total_ticks - start < 1200,
                "the under-slab crawl never cleared west\n{}",
                walker::fail_dump_pub(&w.scene, [-36.95, 1.3, 33.61], "under-slab west"));
            if w.scene.sim.as_ref().unwrap().bond.pos[0] < -36.95 { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let inp = nf_game::player::Input {
                forward: 1.0,
                turn: wrap_pi((-36.95f32 - b.pos[0]).atan2(33.61 - b.pos[2]) - b.yaw).clamp(-0.08, 0.08),
                ..Default::default()
            };
            w.tick(inp, false);
        }
        for _ in 0..80 {
            w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
            if !w.scene.sim.as_ref().unwrap().bond.crouched { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
        eprintln!("[014-trip] retreat under the slab west: bond {:?} crouched {}",
            w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.crouched);
    }
    for (label, wp) in [
        ("the row west 3 (the sill head)", Wp::new(-37.33, 1.20, 33.61).r(0.4)),
        ("down to the catwalk head", Wp::new(-38.6, 0.55, 33.2).r(0.5)),
        ("the catwalk west", Wp::new(-41.5, 0.6, 33.2).r(0.8)),
        ("the 0.40 lane south", Wp::new(-42.25, 0.5, 28.92).r(1.0)),
    ] {
        let ok = w.try_walk_to(wp, label);
        eprintln!("[014-trip] retreat {label}: ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The flush changes the guards' fight around the placement, so a thrown guard grenade
        // can enter and leave `grenades` between the stick and the retreat — re-find the stuck
        // tripbomb instead of trusting the placement-time index (the 2026-11-03 len-1/index-1).
        let gi = trip_stuck_index(sim).expect("the placed tripbomb is still stuck at the retreat");
        let gi = trip_stuck_now(sim);
        let armed = sim.drones.grenades[gi].phase.as_ref().map_or(false, |p| p.armed);
        eprintln!("[014-trip] retreated to {:?} armed {armed} beams {}", sim.bond.pos, sim.drones.tripbomb_beams.len());
        if armed && !sim.drones.tripbomb_beams.is_empty() {
            let bd = trip_beam_dist(sim, sim.bond.pos);
            eprintln!("[014-trip] beam distance {bd:.2}");
        }
        let d = tv_len(tv_sub(trip_body_centre(sim, gi), sim.bond.pos));
        eprintln!("[014-trip] retreat body-dist {d:.2}");
        assert!(armed, "the tripbomb armed behind him");
        assert!(!sim.drones.tripbomb_beams.is_empty(), "the armed tripbomb draws its beam");
        assert!(d >= 12.0, "the retreat walks 12+ u from the placed tripbomb ({d})");
        assert!(trip_beam_dist(sim, sim.bond.pos) > 1.6, "the retreat stands clear of the beam line");
    }

    // 8. The shot position. The armoury's geometry offers NO floor ≥ 12 u with a clear eye→body
    // line to the rack-wall bomb (26 candidate spots probed on this run: every ≥12 line is
    // blocked — the shelf slab shadows the lane from the west, the pit walls block the
    // south-west, and the only window into the lane is the doorway at z 33.0-34.2 whose axis
    // dies in the pit wall at (-41.2, 29.5) [L the spot diagnostics, this run]). The level's
    // own clear shot is THROUGH THE DOORWAY from the sill head: crouched on the crate row at
    // (-37.33, 1.20, 33.61) the eye (~1.85) passes under the slab (open y 0.9..2.2) and through
    // the open doorway straight at the bomb. The walk back in is real input; the shot happens
    // outside the beam trigger but inside the 8-u blast (the walker's top-up is this file's own
    // walk allowance) — a ≥12 clear line is a live check.
    {
        for (label, wp) in [
            ("back up the 0.40 lane", Wp::new(-42.25, 0.5, 30.73).r(0.8)),
            ("back to the catwalk west", Wp::new(-41.5, 0.6, 33.2).r(0.6)),
            ("back to the catwalk head", Wp::new(-38.6, 0.55, 33.2).r(0.5)),
            ("back to the sill head", Wp::new(-37.33, 1.20, 33.61).r(0.4)),
        ] {
            let ok = w.try_walk_to(wp, label);
            eprintln!("[014-trip] {label}: ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        }
        // The crouch-crawl back through the doorway into the lane (the approach crawl again):
        // the doorway's north jamb blocks every sill-head line (the parked panel/jamb at
        // (-35.82, 1.44, 34.42) [L this run's shot-line seg]) — the level's only clear line to
        // the rack-wall bomb is from inside the lane, crouched under the shelf slab.
        let start = w.total_ticks;
        w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
        loop {
            assert!(w.total_ticks - start < 1200,
                "the crawl back into the lane never cleared\n{}",
                walker::fail_dump_pub(&w.scene, [-34.9, 1.3, 33.61], "crawl back east"));
            if w.scene.sim.as_ref().unwrap().bond.pos[0] > -34.9 { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let inp = nf_game::player::Input {
                forward: 1.0,
                turn: wrap_pi((-34.9f32 - b.pos[0]).atan2(33.61 - b.pos[2]) - b.yaw).clamp(-0.08, 0.08),
                ..Default::default()
            };
            w.tick(inp, false);
        }
        eprintln!("[014-trip] back inside the lane: bond {:?} crouched {}",
            w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.crouched);
    }

    // He waits out the arm crouched in the lane (the beam is up by now or arms within these ticks).
    for _ in 0..200 {
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = trip_stuck_now(sim);
        if sim.drones.grenades[gi].phase.as_ref().map_or(false, |p| p.armed) && !sim.drones.tripbomb_beams.is_empty() { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    let body = trip_body_centre(w.scene.sim.as_ref().unwrap(), trip_stuck_now(w.scene.sim.as_ref().unwrap()));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (bd, d) = (trip_beam_dist(sim, sim.bond.pos), tv_len(tv_sub(body, sim.bond.pos)));
        eprintln!("[014-trip] at the spot: bond {:?} body-dist {d:.2} beam-dist {bd:.2} armed {}",
            sim.bond.pos, { let gi = trip_stuck_now(sim); sim.drones.grenades[gi].phase.as_ref().map_or(false, |p| p.armed) });
        assert!(d >= 3.0, "the shot spot holds the bomb in view ({d})");
        assert!(bd > 1.6, "Bond stands clear of the beam line ({bd})");
        let gi = trip_stuck_now(sim);
        assert!(sim.drones.grenades[gi].phase.as_ref().map_or(false, |p| p.armed), "the tripbomb armed");
        assert!(!sim.drones.tripbomb_beams.is_empty(), "the armed tripbomb draws its beam");
    }

    // 9. Switch back to the P2K (the level-start gun) and shoot the body through the real fire:
    // the bullet-vs-tripbomb rule detonates it [M Bullet_handle_object_destruction 0x12cd98].
    let mut drawn2 = false;
    for _ in 0..240 {
        w.tick(nf_game::player::Input { switch_to: Some(P2K), ..Default::default() }, false);
        let g = &w.scene.sim.as_ref().unwrap().gun;
        if g.weapon == P2K && g.switch_state == 0 && g.reload_state == 0 { drawn2 = true; break; }
    }
    assert!(drawn2, "the switch request 0x02 draws the P2K");
    let mut detonated = None;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let seg = sim.world.segment(e, trip_body_centre(sim, gi));
        eprintln!("[014-trip] shot eye {e:?} -> body seg {seg:?} crouched {}", sim.bond.crouched);
    }
    for round in 0..12u32 {
        // Aim at the body, pull on alternate ticks (the refire edge), until this round's flight ends.
        let mut flew = false;
        for t in 0..240u32 {
            let pull = !flew && t % 2 == 0;
            let aim = trip_body_centre(w.scene.sim.as_ref().unwrap(), gi);
            steer_aim(&mut w, aim, pull, false);
            let live = !w.scene.sim.as_ref().unwrap().drones.player_bullets.is_empty();
            if trip_stuck_index(w.scene.sim.as_ref().unwrap()).is_none() { break; }
            if flew && !live { break; }
            flew |= live;
        }
        if trip_stuck_index(w.scene.sim.as_ref().unwrap()).is_none() { detonated = Some(round + 1); break; }
        let _ = &mut flew;
    }
    let rounds = detonated.expect("a P2K round at the body from 12+ u detonates the placed tripbomb [M 0x12cd98]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        eprintln!("[014-trip] DETONATED by round {rounds}");
        eprintln!("[014-trip] final bond position {:?}", sim.bond.pos);
        eprintln!("[014-trip] shot line eye {e:?} -> body {body:?} (dist {:.2})", tv_len(tv_sub(body, e)));
        assert!(sim.drones.grenades.iter().all(|g| g.weapon_type != TRIP as u16), "the gadget deleted (obj+0xFE |= 1) [M 0x12cd98]");
        // (the 0x0600_0052 script-player check tripwire_shot_live pins is unusable here: this
        // level's destructible props pre-register 0x0600_0052 SP players at load, so the list
        // always contains one — the deletion observables above are the 0x12cd98 rule here.)
        assert!(sim.drones.tripbomb_beams.is_empty(), "the beam deleted with it (BU+0x38, +0x70 & 0x80000) [M]");
    }
}

/// The delta approach's visibility-keyed scan (real-health-00agent-14-landing-5): answer the
/// wake route's 0x1a posts BEFORE their first round — the shipped stream's fights only start on
/// the hit (the return fire), and the measured tax IS the first rounds (#43 -3.50, #24 -14.0,
/// #40 -7.00, #41 -2.06 — every one a 0x1a carrier [L the NF_DMG_TRACE run]). The pick is the
/// nearest VISIBLE alive team-1 guard within 20 u (|dy| < 2.0), 0x1a carriers FIRST (the
/// measured killers — a nearer non-0x1a guard must not burn the fights' clock while a post
/// stands armed), each fight `shoot_bounded`-capped at 300 ticks: the cap keeps every
/// repositioning strafe (900 ticks without damage) out of the fight, so the scan cannot shove
/// Bond off the route; a guard the fights cannot see is left to the route's own fights (the
/// shipped stream's kills after each guard's first round ran clean). Up to four fights.
/// The delta approach scan = the route scan anchored on the wake touch (20.3, 0.9, -23.5): the
/// wake legs' own scan, byte-identical values to the pre-route-scan version (the wake call
/// sites' behaviour is unchanged; retune-nb-14-2 only lifted the law so the descent corridor
/// can anchor its own road-blocker line on the lane the walk must cross).
fn delta_approach_scan(w: &mut Walker, node: usize) {
    route_scan(w, [20.3, 0.9, -23.5], &format!("the delta approach scan (node {node})"));
}

/// The scan law the delta approach measured (the wake legs' capped answer fights), anchored on
/// an arbitrary remaining-route destination for the road-blocker rank: the nearest team-1 guard
/// whose GUN rays are clear, 0x1a carriers first, then whoever stands on the route line to
/// `dest`, then the weaker, then the nearer — up to four fights, each a bounded 300-tick stand.
/// Read-only pickup census (NF_PICKUP_CENSUS, retune-nb-14-4): every live pickup within 45 u of
/// Bond, its kind (pickup.rs kind:: — 0 WEAPON, 1 AMMO, 2 KEY, 3 ARMOUR, 4 LABEL, 5 MESSAGE,
/// 6 BOND_BONUS [M Pickup_Handler 0x18e650]), weapon/rounds and distance. No tick, no draw —
/// measurement only, the run stays byte-identical with and without the env var.
fn pickup_census(w: &Walker, label: &str) {
    if std::env::var_os("NF_PICKUP_CENSUS").is_none() { return; }
    let sim = w.scene.sim.as_ref().unwrap();
    let bp = sim.bond.pos;
    let mut n = 0;
    for p in &sim.drones.pickups {
        if p.gone || p.hidden { continue; }
        let pos = [p.m[3][0], p.m[3][1], p.m[3][2]];
        let d = walker::dxz(pos, bp);
        if d < 45.0 {
            eprintln!("[census] {label}: kind {} weapon {:#x} rounds {} at ({:.2},{:.2},{:.2}) d {:.1} state {} from {:?}",
                p.kind, p.weapon, p.rounds, pos[0], pos[1], pos[2], d, p.state, p.from);
            n += 1;
        }
    }
    eprintln!("[census] {label}: {n} live pickups within 45 u of ({:.2},{:.2},{:.2}) | bond health {:.1} armour {:.1} gun {:#x} clip {} spare {}",
        bp[0], bp[1], bp[2],
        sim.drones.health.health, sim.drones.health.armour, sim.gun.weapon, sim.gun.clip, sim.gun.spare);
}

fn route_scan(w: &mut Walker, dest: V3, label: &str) {
    route_scan_ranked(w, dest, label, false);
}

/// retune-nb-14-5 (the door-face scans only): `door_face_law` swaps the rank's road-blocker and
/// health keys — the weaker post dies first — and fights with the 8-tick fast taps
/// (`shoot_bounded_fast`). The door squad's weaker members ARE the killers: #33 (hp 10) and #34
/// (hp 14) carry the 0x14 whose 00-Agent volleys read 15.00 [L the 2026-11-19 re-measure: two
/// volleys, 2.50 then 15.00, 30 ticks apart, killed the 17.5 walk], while the hp-20 #32 never
/// landed a round in any run; and the shipped 16-tick scan cadence measured 44 damage over 50
/// ticks in the west-face fight — the kill landed in #33's volley window. Every other call site
/// keeps the shipped rank and cadence (run 12's measured law: the hp-10-first order spent the
/// clock on the silent strays).
fn route_scan_ranked(w: &mut Walker, dest: V3, label: &str, door_face_law: bool) {
    let candidates = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        (0..sim.drones.list.len()).filter_map(|i| { let d = &sim.drones.list[i];
            if d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 {
                let dd = walker::dxz(d.pos, bp);
                if dd < 20.0 && (d.pos[1] - bp[1]).abs() < 2.0 {
                    return Some((i, d.skin, d.weapon, dd,
                        walker::DamageLog::visible(sim, i), walker::DamageLog::visible_gun_rays(sim, i)));
                }
            }
            None })
            .collect::<Vec<_>>()
    };
    if !candidates.is_empty() {
        eprintln!("[014] {label}: candidates {:?}",
            candidates.iter().map(|(i, s, w, d, vis, gun)| format!("#{i} {s:#x} 0x{w:x} d{d:.1} eye{vis} gun{gun}")).collect::<Vec<_>>());
    }
    for _ in 0..4 {
        let pick = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 20.0 && (d.pos[1] - bp[1]).abs() < 2.0 })
                // GUN-RAY visibility (the fights' own law [M the lane_fights STRICT gun-ray
                // aim — the 07000013 chain's passing rule]): the eye-ray sees guards whose
                // muzzle line is blocked, and a fight on such a guard lands ZERO rounds while
                // burning the 300-tick window (runs 5-7: #36 eye-visible gun-blocked at 5.6 u,
                // killed false, health 10 unchanged, his convergence killed the walk after).
                // retune-nb-14-5 (door_face_law): the EYE-true guards JOIN the candidates —
                // the round-5 crossing death ate a point-blank 15.57 at d 2.5 from the near
                // guard (−49.6, 40.9) whom this gate had dropped (his muzzle blocked from the
                // west park, his eye and his BODY boxes on Bond through the open leaf span) [L
                // 2026-11-19 run 5]. Bond's rounds follow the VIEW: an eye-true body box is a
                // target the tight gate can land.
                .filter(|&di| walker::DamageLog::visible_gun_rays(sim, di)
                    || (door_face_law && walker::DamageLog::visible(sim, di)))
                .min_by_key(|&di| {
                    let d = &sim.drones.list[di];
                    // 0x1a carriers first (the measured killers); then the ROAD-BLOCKER rank —
                    // the guard's perpendicular distance from the REMAINING route line to the
                    // scan's `dest`: a player answers whoever stands on the ground he must
                    // cross before the strays off the line (run 12: the hp-10-first order spent
                    // the scan's clock on the silent strays #51/#55 while the posts #40/#41
                    // shot the walk apart: -7.00 at t41426, the death -2.06 at t41533); then
                    // the weaker post dies inside one capped fight; then the nearer.
                    let (ax, az) = (dest[0] - bp[0], dest[2] - bp[2]);
                    let len2 = ax * ax + az * az;
                    let t = ((d.pos[0] - bp[0]) * ax + (d.pos[2] - bp[2]) * az) / len2.max(1e-6);
                    let px = bp[0] + t * ax - d.pos[0];
                    let pz = bp[2] + t * az - d.pos[2];
                    (if d.weapon == 0x1a { 0u8 } else { 1u8 },
                        if door_face_law { d.health.to_bits() } else { (px * px + pz * pz).to_bits() },
                        if door_face_law { (px * px + pz * pz).to_bits() } else { d.health.to_bits() },
                        walker::dxz(d.pos, bp).to_bits())
                })
        };
        let Some(di) = pick else { break };
        // The LOOSE aim gate measured DEAD here (landing-5 run 5: the 5.6 u loose fight landed
        // ZERO rounds — the gate sprays past the 0.3-wide torso box at that range — and the
        // guard converged into the 3.0 auto-hit band during the failed fight: -23.06). The
        // bounded tight gate killed every scan fight it won (run 4: #36 at 4.0, #43 at 16.8).
        let killed = if door_face_law { w.shoot_bounded_fast(di, &label, 300) } else { w.shoot_bounded(di, &label, 300) };
        eprintln!("[014] {label}: drone {di} fight over (killed {killed}, health {:.0})",
            w.scene.sim.as_ref().unwrap().drones.list[di].health);
    }
}

fn kiko_hold(w: &mut Walker, hold: u32, label: &str) {
        let home = w.scene.sim.as_ref().unwrap().bond.pos;
        let t0 = w.total_ticks;
        while w.total_ticks - t0 < hold {
            let (gren, pick) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                let gren = sim.drones.grenades.iter().filter(|g| walker::dxz(g.pos, bp) < g.radius.max(4.0) + 2.0).map(|g| g.pos).next();
                let pick = (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 20.0
                        && (d.pos[1] - bp[1]).abs() < 3.0 && walker::DamageLog::visible(sim, i) })
                    .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp)));
                (gren, pick)
            };
            if let Some(gp) = gren {
                let (f, sd) = { let sim = w.scene.sim.as_ref().unwrap(); let bp = sim.bond.pos;
                    let (dx, dz) = (bp[0] - gp[0], bp[2] - gp[2]); let n = (dx * dx + dz * dz).sqrt().max(0.01);
                    let (fx, fz) = (sim.bond.yaw.sin(), sim.bond.yaw.cos()); ((dx * fx + dz * fz) / n, (dx * fz - dz * fx) / n) };
                w.tick(nf_game::player::Input { forward: f.clamp(-1.0, 1.0), strafe: sd.clamp(-1.0, 1.0), ..Default::default() }, false);
                continue;
            }
            if let Some(g) = pick { let _ = w.shoot_bounded(g, label, 90); continue; }
            if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, home) > 1.5 {
                w.lenient = true; w.walk_to(Wp::new(home[0], home[1], home[2]).r(0.8), "back to the hold spot"); w.lenient = false;
            } else { w.tick(nf_game::player::Input::default(), false); }
        }
        eprintln!("[14hold] {label} done, health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
}
