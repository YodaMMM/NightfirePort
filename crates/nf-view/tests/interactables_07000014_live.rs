//! interactables-14: one live test per KIND of player-touchable placement on 07000014 (Drake's
//! underground facility → Delta Sector), driven the way a player drives it (the walk, the use
//! key, the trigger pull, the walk-over, the gadget), each asserting the game's effect with a
//! sister. The census lives in docs/levels/07000014.md §1b (the [D] evidence per kind); the
//! end-to-end route is walkreal_07000014_live.rs.
//!
//! Kinds on this level [D census_07000014's dump + the doc table]: doors (0xdb), switches
//! (0x29), fuseboxes (0x31), pickups (0xf0), breakables (0x20), touch plates (0xEA/0xEB),
//! monitors (0x30), hints (0x33), grapple points (0x3a). **No lift kind exists on 07000014**
//! — the census lists no lift placement (the level's vertical moves are the stairs, the ducts
//! and the Q-Claw). Sensors (0x28, the security cameras) are the detection system, not a
//! use/touch kind; their out-channel trip is the alarm chain's input and the monitors' feed.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes. A teleport to reach a spot is
//! used only where the walk there is the documented blocker — the real-input crawl wedges at
//! the east duct mouth (x = 40.98, the lintel head bump [D probe14_eastduct;
//! walkreal_07000014_live's two #[ignore]d legs]) — so the interior spots are reached by the
//! documented stand-in transit; the operation itself always runs through the real input. The
//! vent-room tests (the plate, the grapple) walk from the spawn with no teleport at all.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Cold load + the intro release, the same warm-up the level's other live tests use.
fn load_cold() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014"));
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

/// The documented stand-in transit (see the module header): place Bond, settle him standing
/// (the spot's floor found by the world's own down ray — the y in the placement records is the
/// object's, not the floor's).
fn teleport(w: &mut Walker, pos: [f32; 3], yaw: f32) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = pos;
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = yaw;
    }
    w.wait(40);
}

/// Aim the view at a point (the walker's yaw convention `atan2(dx, dz)`, the look units
/// `atan(slope)/π/2` [M BL+0x8a8·π/2]).
fn aim(sim: &mut nf_view::scene::Sim, to: [f32; 3]) {
    let head = sim.bond.head();
    let d = [to[0] - head[0], to[1] - head[1], to[2] - head[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1] / l).atan() / std::f32::consts::FRAC_PI_2;
}

/// Hold the use key through the real `Player_Activate` path until `pred` turns true (≤ `ticks`
/// steps, one edge every 6 ticks like the walker's door presses).
fn use_until(w: &mut Walker, ticks: u32, mut pred: impl FnMut(&nf_view::scene::Sim) -> bool) -> bool {
    for k in 0..ticks {
        if pred(w.scene.sim.as_ref().unwrap()) { return true; }
        w.tick(nf_game::player::Input::default(), k % 6 == 0);
    }
    pred(w.scene.sim.as_ref().unwrap())
}

// ---------------------------------------------------------------------------
// The plate: TouchOnce #601 "oDroneTrigger1" (40.88, 3.43, 62.47) out ch 0xB8, mask 8 [D
// placement; docs/levels/07000014.md §1a] — the vent touch. Its 0xB8 feeds AND #336
// [0xDE, 0xB8] → 0x61, the alarm channel [D §1a]. Fired by the walk-over alone (mask 8 =
// the player contact [M Trigger_Hit 0x1b1fc8's 0x8 arm]).
// ---------------------------------------------------------------------------

/// Bond CRAWLS out of the spawn (40.81, 2.40, 65.08, crouched [D p3=3]) and the walk-over
/// fires ch 0xB8 — the whole leg real input, no teleport.
#[test]
fn the_vent_touchonce_walkover_fires_chb8_from_the_spawn() {
    let mut w = load_cold();
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0xb8),
        "sister: ch 0xB8 is off at load — the touch is its only writer on this route [D §1a]");
    // The touch sits on the south duct between the spawn and the grate (z 62.47 [D]); walk
    // south through it. The crawl keeps the spawn's crouch — the doc's own finding is "fires
    // as Bond crawls past it" [D §1a].
    let mut fired = false;
    for _ in 0..600 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb8) { fired = true; break; }
        if w.scene.sim.as_ref().unwrap().bond.pos[2] < 61.0 { break; }
    }
    assert!(fired, "the walk-over fires the TouchOnce's out ch 0xB8 [D placement (40.88,3.43,62.47) out 0xB8 mask 8; M Trigger_Hit 0x1b1fc8], pos {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    // TouchOnce latches: it fired once and the channel stays (the walk passes in ~1 s).
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xb8),
        "the fired TouchOnce's channel holds [M Trigger_Hit 0x1b1fc8]");
}

// ---------------------------------------------------------------------------
// The gadget target: grapple point 0x3a "GRAPPLEPOINT_SkyBridge" (45.30, 4.79, 62.93) [D
// placement] — the watch gadget's wire target the Q-Claw bolt latches (class-0x2C scan
// [M Bullet_Delete 0x12e660:50-55 → Player_SetGrapplePoint 0x195da0]).
// ---------------------------------------------------------------------------

/// From the crossing stance (40.8, floor 2.40, z 65.5 — walked there from the spawn), the
/// Q-Claw bolt (weapon 0x50) latches the point and reels to the hang [M Player_SSGrapple
/// 0x192860]. Sister: no wire before the bolt.
#[test]
fn the_qclaw_latches_the_skybridge_grapple_point() {
    let mut w = load_cold();
    // The spawn crouch walks to the crossing stance, then stands (the bolt-legal arc needs the
    // standing head [D probe14_aim]) — walkreal's leg A, verbatim.
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the crossing stance");
    w.scene.sim.as_mut().unwrap().bond.crouched = false;
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false); }
    assert!(w.scene.sim.as_ref().unwrap().bond.grapple.is_none(), "sister: no wire before the bolt");
    // Aim at the point [D (45.30, 4.79, 62.93)] on the lower arc (pitch 0.20 rad [D probe14_aim]).
    let point = [45.29f32, 5.10, 62.94];
    let want_yaw = (point[0] - 40.8).atan2(point[2] - 65.5);
    let want_look = 0.20f32 / std::f32::consts::FRAC_PI_2;
    for i in 0..200 {
        let (byaw, bpitch) = { let s = w.scene.sim.as_ref().unwrap(); (s.bond.yaw, s.bond.pitch) };
        let inp = nf_game::player::Input {
            turn: wrap_pi(want_yaw - byaw).clamp(-0.08, 0.08),
            look: (want_look - bpitch).clamp(-0.08, 0.08),
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
    // The reel pulls toward the point and hangs within 2.0 [M 0x192860:79-121, :91-94].
    for _ in 0..400 {
        w.tick_fire(nf_game::player::Input::default());
        let sim = w.scene.sim.as_ref().unwrap();
        match &sim.bond.grapple {
            Some(g) if g.phase == 2 => {
                let p = sim.bond.pos;
                let d = ((p[0] - g.point[0]).powi(2) + (p[1] - g.point[1]).powi(2) + (p[2] - g.point[2]).powi(2)).sqrt();
                assert!(d < 2.5, "the hang sits within 2.0 of the point [M 0x192860:91-94], got {d:.2}");
                return;
            }
            None => panic!("the wire dropped before the hang [M 0x192860]"),
            _ => {}
        }
    }
    panic!("the reel never reached the hang phase [M Player_SSGrapple 0x192860:79-121]");
}

// ---------------------------------------------------------------------------
// The doors: OBJECT_Door #618 (42.47, 1.34, 30.59) with no lock (param 3 = 0 [D]) opens on
// the E [M Door_Activate]; the Door_Slide_11A/B pair (−52.22, 1.20, 40.13, lock ch 0x85
// [D param 3:0x85]) refuses the same E while 0x85 is off [M Door_Activate's lock gate].
// ---------------------------------------------------------------------------

#[test]
fn a_plain_door_opens_on_use_and_the_locked_pair_refuses() {
    let mut w = load_cold();
    // The plain door: the E is the real input; the transit is the documented stand-in (the
    // vent-mouth wedge, module header).
    let (dpos, di) = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::door_near(sim, [42.47, 1.34, 30.59]).expect("OBJECT_Door #618 exists [D]")
    };
    assert!(((dpos[0] - 42.47).abs() < 0.5) && ((dpos[2] - 30.59).abs() < 0.5),
        "the nearest door to the #618 spot is #618 itself, got {dpos:?}");
    // Stand 1.6 south of the door like the model test's stance (its use sphere is head+look
    // [M 0x1a5170:100-108] vs the door's world sphere [M Collide_PickObj 0x1eae80]).
    teleport(&mut w, [dpos[0], dpos[1] + 0.6, dpos[2] + 1.6], std::f32::consts::PI);
    w.wait(30);
    eprintln!("[014-door] bond {:?} door {dpos:?} state {:#x}",
        w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.doors.list[di].state);
    let before = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        aim(sim, [dpos[0], dpos[1] + 1.0, dpos[2]]);
    }
    let opened = use_until(&mut w, 240, |sim| {
        let st = sim.drones.doors.list[di].state;
        st & 0xc != 0 || st != before
    });
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(opened,
        "the unlocked OBJECT_Door opens on the E [M Door_Activate; D #618 param 3 = 0], {before:#x} → {st:#x}");
    // The sister: Door_Slide_11A/B (−52.22, 1.20, 40.13, lock ch 0x85 [D #477/#478 param 3]).
    let li = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.doors.list.iter()
            .position(|d| d.ch_lock == 0x85 && (d.pos[0] + 52.22).abs() < 1.0 && (d.pos[2] - 40.13).abs() < 1.0)
            .expect("the 0x85-locked door pair exists [D]")
    };
    let lpos = w.scene.sim.as_ref().unwrap().drones.doors.list[li].pos;
    teleport(&mut w, [lpos[0], lpos[1] + 0.6, lpos[2] + 1.6], std::f32::consts::PI);
    w.wait(30);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x85),
        "sister: ch 0x85 is off at load — nothing armed [D]");
    let lbefore = w.scene.sim.as_ref().unwrap().drones.doors.list[li].state;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        aim(sim, [lpos[0], lpos[1] + 1.0, lpos[2]]);
    }
    use_until(&mut w, 120, |sim| sim.drones.doors.list[li].state != lbefore);
    let lst = w.scene.sim.as_ref().unwrap().drones.doors.list[li].state;
    assert_eq!(lst, lbefore,
        "sister: the door locked on ch 0x85 must not move before the cascade [M Door_Activate's lock gate; D #477 param 3:0x85], {lbefore:#x} → {lst:#x}");
}

// ---------------------------------------------------------------------------
// The switch: 0x29 "alarmswitchplayer" (56.36, 1.64, 42.90) ch 0xDE, param 4 = 1 (the busy
// one-shot) [D placement]. The E toggles the channel [M Switch_Activate 0x1b02a8]; the busy
// latch makes it one-shot [M 0x1adea8:22/29 + 0x1b02a8's busy gate].
// ---------------------------------------------------------------------------

#[test]
fn the_alarm_switch_e_toggles_chde_once() {
    let mut w = load_cold();
    let (sp, busy) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sim.drones.useobjs.switches.iter().find(|s| s.ch == 0xde)
            .expect("the alarmswitchplayer with ch 0xDE exists [D placement]");
        (s.pos, s.busy)
    };
    assert!(busy, "the switch's param-4 busy bit is set [D placement]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0xde),
        "sister: ch 0xDE is off at load [D]");
    teleport(&mut w, [sp[0], sp[1] + 0.2, sp[2] + 1.6], std::f32::consts::PI); // south of the panel, facing −z
    // E through the real use path (the switch's pick target is its embedded SP's entity
    // [M Switch_Create 0x1adcc0:60-66 + Player_Activate's SP route]).
    let got = use_until(&mut w, 240, |sim| sim.drones.channels.on(0xde));
    assert!(got, "the E toggles ch 0xDE [M Switch_Activate 0x1b02a8; D placement ch param 0:0xde]");
    // The busy one-shot: further Es never toggle it back [M Switch_Activate's busy gate —
    // +0x18 != 0 returns; Switch_Update 0x1adea8:22,29 bumps the latch on the change].
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), true); }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xde),
        "the busy latch holds the toggle (one-shot) [M 0x1b02a8:12-16]");
}

// ---------------------------------------------------------------------------
// The fusebox: 0x31 OBJECT_FuseBox1 #617 (42.19, −0.20, 30.40), group 0 = ch 0x22 [D params
// 1:0x22 3:0x22]. The cut is the Laser beam into the wire sub-object [M FuseBox_Update
// 0x1adfe8:88-123]; the E press is inert (no class-0x3C arm [M Player_Activate
// 0x1a5170:95-160]).
// ---------------------------------------------------------------------------

/// The wire object's visible geometry (walkreal_07000014's helpers, verbatim).
fn wire_world_box(sim: &nf_view::scene::Sim, obj: usize) -> Option<([f32; 3], [f32; 3])> {
    match sim.drones.scripts.objects.list.get(obj)? {
        Some(nf_game::nis::Obj::Entity { hash, matrix, .. }) => {
            let (lo, hi) = sim.drones.wire_geo.get(hash)?;
            let (mut wlo, mut whi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for cx in 0..2 { for cy in 0..2 { for cz in 0..2 {
                let p = [if cx == 0 { lo[0] } else { hi[0] }, if cy == 0 { lo[1] } else { hi[1] }, if cz == 0 { lo[2] } else { hi[2] }];
                let wpos = nf_formats::placement::transform(matrix, p);
                for c in 0..3 { wlo[c] = wlo[c].min(wpos[c]); whi[c] = whi[c].max(wpos[c]); }
            } } }
            Some((wlo, whi))
        }
        _ => None,
    }
}

fn wire_aim_point(sim: &nf_view::scene::Sim, obj: usize) -> [f32; 3] {
    if let Some((lo, hi)) = wire_world_box(sim, obj) {
        return [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    }
    nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), obj).unwrap_or([0.0; 3])
}

#[test]
fn the_east_wing_fusebox_cuts_through_the_laser_and_the_e_is_inert() {
    let mut w = load_cold();
    let want = 0x22u8;
    let fb = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.fuseboxes.iter().position(|f| f.ch[0] as u32 == want as u32)
            .unwrap_or_else(|| panic!("the fusebox with group-0 channel {want:#x} exists [D #617]"))
    };
    let (bp, wire, wire_obj) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let obj = sim.drones.fuseboxes[fb].alive_wires().find(|(g, _, _)| *g == 0)
            .or_else(|| sim.drones.fuseboxes[fb].alive_wires().next())
            .map(|(_, _, o)| o)
            .expect("the script registered wires (groups 0 and 1 [M 0x1d94c0:172-175])");
        (sim.drones.fuseboxes[fb].pos, wire_aim_point(sim, obj), obj)
    };
    // Sister: the E press is inert at the box — the port's use path has no class-0x3C arm
    // [M 0x1a5170:95-160] and the box's SP mask 1 has no bit 4 [M 0x1dc338].
    teleport(&mut w, [bp[0] + 1.2, bp[1] + 2.4, bp[2] + 0.2], -std::f32::consts::FRAC_PI_2);
    aim(w.scene.sim.as_mut().unwrap(), wire);
    let got_e = use_until(&mut w, 60, |sim| sim.drones.channels.on(want as u32));
    assert!(!got_e, "sister: the E press at the fusebox writes nothing [M 0x1a5170:95-160]");
    // Stand inside the Laser's range of the wire (4.0 [L weapon_data 0x4E +0x1C]).
    let cands = [[0.9f32, -0.6, 0.9], [-0.9, -0.6, 0.9], [0.0, -0.6, 1.4], [1.4, -0.6, 0.0], [-1.4, -0.6, 0.0]];
    let mut stood = false;
    for c in &cands {
        let spot = [bp[0] + c[0], bp[1] + 2.4 + c[1], bp[2] + c[2]];
        let floor = w.scene.sim.as_ref().unwrap().world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
        if !matches!(floor, Some(f) if (f - (bp[1] - 2.4)).abs() < 3.0) { continue; }
        teleport(&mut w, [spot[0], spot[1], spot[2]], 0.0);
        let head = w.scene.sim.as_ref().unwrap().bond.head();
        let d = [wire[0] - head[0], wire[1] - head[1], wire[2] - head[2]];
        if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 3.4 { stood = true; break; }
    }
    assert!(stood, "a stance inside the Laser's range exists [L weapon_data 0x4E +0x1C]");
    for _ in 0..40 { w.tick(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false); }
    // The cut: steer the crosshair at the visible cable and hold fire (walkreal's burn rule).
    let mut delta = [0.0f32; 3];
    let start = w.total_ticks;
    loop {
        let (n0, want_on) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.player_bullets.len(), sim.drones.channels.on(want as u32))
        };
        if want_on { break; }
        assert!(w.total_ticks - start < 600, "the laser cut never wrote ch {want:#x} [M 0x1adfe8:88-123]");
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let t = [wire[0] + delta[0], wire[1] + delta[1], wire[2] + delta[2]];
            let s = w.scene.sim.as_mut().unwrap();
            aim(s, t);
        }
        w.tick_fire(nf_game::player::Input::default());
        // The round's miss against the wire's world slab shifts the aim.
        let round = w.scene.sim.as_ref().unwrap().drones.player_bullets.get(n0)
            .filter(|b| b.weapon == 0x4e).map(|b| (b.pos, b.dir, b.speed));
        if let Some((pos, dir, speed)) = round {
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some((lo, hi)) = wire_world_box(sim, wire_obj) {
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
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(want as u32),
        "the laser cut fires the group-0 channel [M FuseBox_Update 0x1adfe8:88-123; D #617 p1 0x22]");
}

// ---------------------------------------------------------------------------
// The pickup: 0xf0 PICKUP_BodyArmourHeavy (32.26, 0.87, 31.14), kind 3, 50 [D params 0:0x3
// 2:0x32] — the server room's armour. The walk-over grants armour 50 [M Pickup_Handler
// 0x18e650 case 3]. Sister: armour 0 before.
// ---------------------------------------------------------------------------

#[test]
fn the_server_room_armour_is_taken_by_the_walkover() {
    let mut w = load_cold();
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.health.armour, 0.0, "sister: no armour before");
    let p = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .find(|p| p.kind == 3 && (p.m[3][0] - 32.26).abs() < 0.6 && (p.m[3][2] - 31.14).abs() < 0.6)
        .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
        .expect("the server-room BodyArmourHeavy exists [D placement]");
    teleport(&mut w, [p[0], p[1] + 1.8, p[2] + 2.0], std::f32::consts::PI);
    w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.2), "onto the BodyArmourHeavy");
    w.wait(20);
    let a = w.scene.sim.as_ref().unwrap().drones.health.armour;
    assert!((a - 50.0).abs() < 1.0,
        "the walk-over grants armour 50 [M Pickup_Handler 0x18e650 case 3; D params 0:0x3 2:0x32], got {a}");
}

// ---------------------------------------------------------------------------
// The breakable: 0x20 ServerRoomWindow (params [0:0x2, 1:0x0, 2:0x20] [D]) — shootable
// glass [M Break_Create 0x12c230 + Break_Kill 0x12b970]. The gunshot through the real fire
// path kills the pane. Sister: the pane stands before.
// ---------------------------------------------------------------------------

#[test]
fn the_server_room_window_breaks_on_the_gunshot() {
    let mut w = load_cold();
    // The placed panes are the level's three 0x20 placements [D]; the port poses them from the
    // object's own verts (the placement sits at the origin [D]). Find the server-room one by
    // its bounds: the server room spans x ≈ 30..36, z ≈ 30..40 (the fusebox #1159 at 32.29,
    // 0.60, 36.00 and the armour at 32.26, 0.87, 31.14 [D]).
    let bi = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.breaks.list.iter().position(|b| b.script_obj.is_none() && b.kind == 2
            && b.lo[0] > 20.0 && b.hi[0] < 45.0 && b.lo[2] > 25.0 && b.hi[2] < 45.0)
            .unwrap_or_else(|| panic!("the ServerRoomWindow pane exists among the placed breaks [D]"))
    };
    let (lo, hi) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.drones.breaks.list[bi].lo, sim.drones.breaks.list[bi].hi) };
    let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    eprintln!("[014-window] pane bounds lo {lo:?} hi {hi:?}");
    assert!(!w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead, "sister: the pane stands before the shot [M Break_Kill 0x12b970]");
    // A stance west of the pane with a clean line (the stand-in transit; the shots are real).
    teleport(&mut w, [centre[0] + 2.6, centre[1], centre[2]], 0.0);
    for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); } // settle
    // The disc's filter: param 2 = 0x20 → tag+0x08 = ~0x20, and `Collide_FilterBullets` drops
    // every contact whose weapon word meets ~0x20 | 0x106 = 0xffdf [M Break_Create 0x12c230:31-39
    // + Break_Update 0x12bfd8:13 + 0x1ece30:24-33] — the disc's guns carry word 0x0008, the laser
    // 0x0002, the watch 0x0100 [L ee_start_05.bin weapon_data +0x10]. A real gunshot therefore
    // does NOT break these panes — the pin:
    let mut broke = false;
    for _ in 0..60 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            aim(sim, centre);
        }
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead { broke = true; }
    }
    assert!(!broke, "the gunfire never breaks the pane [M Collide_FilterBullets 0x1ece30 drops word 0x8 against ~0x20|0x106; D param 2:0x20]");
    // The pane's killer is the BLAST: `Explode_Propagate`'s case 0x20 drains the break targets
    // with NO weapon filter [M 0x17c518:163-167 → Break_ApplyDamage]. The grenade itself is the
    // walk_07000014's documented stand-in (the level's grenade pickups sit in the west pick
    // room, 60 u west — the real-input throw needs that transit; the BLAST is the game's own
    // code path, weapon 0x34 [D pickup params 1:0x34], landed at the pane's wall).
    w.scene.sim.as_mut().unwrap().drones.grenades.push(nf_game::grenade::Grenade {
        pos: [centre[0] + 1.2, centre[1], centre[2]], velocity: [0.0; 3], speed: 0.0,
        travelled: 59.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34, mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None,
        phase: None,
    });
    w.wait(240); // the grenade's fuse + the blast tick
    let dead = w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead;
    assert!(dead, "the grenade blast kills the ServerRoomWindow pane [M Explode_Propagate case 0x20 → Break_ApplyDamage 0x17c518:163-167; D placement params [2, 0, 0x20]]");
}

// ---------------------------------------------------------------------------
// The monitor: 0x30 SWITCH_Consoles_Evilbase (47.45, 2.19, 36.81) group 1 [D param 0:0x1];
// the group-1 cameras are the sensors (26.31, 3.54, 28.99) and (26.63, 3.54, 30.92) [D
// param 6:0x1 ×2]. The E shows the feed [M Monitor_Activate 0x1b0058], the next E cycles to
// the other group camera, the next exits [M Monitor_Update 0x1afcc8].
// ---------------------------------------------------------------------------

#[test]
fn the_security_monitor_use_shows_the_group_camera_and_cycles() {
    let mut w = load_cold();
    let mi = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.monitors.list.iter().position(|m| m.group == 1 && (m.pos[0] - 47.45).abs() < 1.0)
            .expect("the group-1 monitor console exists [D placement param 0:0x1]")
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.monitors.list[mi].state, 0, "sister: the monitor idles at load [M Monitor_Create 0x1b08b0]");
        assert!(sim.drones.monitors.list[mi].watched.is_none(), "sister: nothing watched before the use");
    }
    let mp = w.scene.sim.as_ref().unwrap().drones.monitors.list[mi].pos;
    eprintln!("[014-monitor] console pos {mp:?}");
    // The stance: the console's own walkway. Probe the offsets' floors (the down ray from just
    // above the console) and take the first whose floor sits at the console's base — the
    // (47.45, 38.4) offset above is open shaft down to the silo (the probe: −32.9).
    let mut spot: Option<[f32; 3]> = None;
    for off in [[0.0f32, 1.2], [0.0, -1.2], [1.2, 0.0], [-1.2, 0.0], [0.0, 0.4], [2.2, 0.0], [-2.2, 0.0], [0.0, 2.2]] {
        let c = [mp[0] + off[0], mp[1] + 0.6, mp[2] + off[1]];
        let floor = w.scene.sim.as_ref().unwrap().world.segment(c, [c[0], c[1] - 6.0, c[2]]).map(|t| c[1] - 6.0 * t);
        if let Some(f) = floor { if f > mp[1] - 2.5 && f < mp[1] + 0.5 { spot = Some([c[0], f + 0.1, c[2]]); break; } }
    }
    let spot = spot.expect("a stance on the console's walkway exists [D the console stands on its floor]");
    teleport(&mut w, spot, std::f32::consts::PI);
    eprintln!("[014-monitor] bond {:?} head {:?} (console {mp:?})",
        w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.head());
    // E → the group's first camera [M Monitor_Activate 0x1b0058] (the reach is the head's
    // 2.2 sphere [M 0x1a5170's monitor arm]).
    let got = use_until(&mut w, 120, |sim| sim.drones.monitors.list[mi].state == 2);
    eprintln!("[014-monitor] bond {:?} head {:?} state {}",
        w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.head(),
        w.scene.sim.as_ref().unwrap().drones.monitors.list[mi].state);
    assert!(got, "the E activates the monitor (state 2) [M Monitor_Activate 0x1b0058; D param 0:0x1]");
    let w1 = w.scene.sim.as_ref().unwrap().drones.monitors.list[mi].watched
        .expect("the feed picked a camera [M 0x1b0058]");
    eprintln!("[014-monitor] watched {w1} of {} sensors, ids {:?}",
        w.scene.sim.as_ref().unwrap().drones.sensors.list.len(),
        w.scene.sim.as_ref().unwrap().drones.sensors.list.iter().map(|s| (s.monitor_id, s.listed, s.state)).collect::<Vec<_>>());
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.sensors.list[w1].monitor_id, 1,
        "the watched camera is a group-1 sensor [D #640/#1108 param 6:0x1]");
    // The next E cycles within the group [M Monitor_Update 0x1afcc8's action-0xE arm re-picks
    // the watched camera]. The third-E exit (the cooldown states 9→4→0 [M 0x1afcc8's
    // none-left arm]) needs the list-order cycle the decompile's walk implies — the port's
    // first-other pick alternates the pair here, so the exit is a live check, not pinned [S].
    let got2 = use_until(&mut w, 120, |sim| {
        sim.drones.monitors.list[mi].watched.map_or(false, |wch| wch != w1)
    });
    assert!(got2, "the next E cycles the feed to another camera [M Monitor_Update 0x1afcc8's action-0xE arm]");
    let w2 = w.scene.sim.as_ref().unwrap().drones.monitors.list[mi].watched.unwrap();
    assert_ne!(w1, w2, "the cycle moved off the first camera [M 0x1afcc8]");
}

// ---------------------------------------------------------------------------
// The hint: 0x33 "Hint Small Plate" (−89.29, −1.32, 31.27) label 0x100020E, suppression
// channel 0x8C [D params 0:0x100020e 4:0x8c] — the sabotage room's hint, up while the
// sabotage is NOT done. The neighbour (−89.31, −1.32, 32.35) label 0x100020D arms ON 0x8C
// [D param 3:0x8c]. Firing posts the type-1 INFO message [M Hint_Update 0x1b3380:33].
// ---------------------------------------------------------------------------

#[test]
fn the_sabotage_hints_gate_on_the_8c_channel() {
    let mut w = load_cold();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.hints.iter().any(|h| h.label == 0x100_020e && h.off_channel == 0x8c),
            "the sabotage hint marker exists [D placement params 0:0x100020e 4:0x8c]");
        assert!(sim.hints.iter().any(|h| h.label == 0x100_020d && h.on_channel == 0x8c),
            "the done-side hint marker exists [D params 0:0x100020d 3:0x8c]");
        assert!(!sim.drones.channels.on(0x8c), "sister: 0x8C off at load — the sabotage not done [D]");
    }
    // Walk into the not-done marker's range: the INFO queue gains label 0x100020E [M
    // Hint_Update 0x1b3380 posts Txt_BindLabel(param 0)].
    teleport(&mut w, [-89.29, 1.0, 32.6], std::f32::consts::PI);
    w.wait(40);
    assert_eq!(w.scene.sim.as_ref().unwrap().hud.info_msgs.front().map(|m| m.label), Some(0x100_020e),
        "the not-done hint posts its label [M Hint_Update 0x1b3380:33; D label param 0:0x100020e]");
    // Sister: the done-side marker (3 u north, arms ON 0x8C [D param 3:0x8c]) stayed silent
    // while 0x8C is off.
    assert!(!w.scene.sim.as_ref().unwrap().hud.info_msgs.iter().any(|m| m.label == 0x100_020d),
        "sister: the 0x8C-armed marker stays silent while the sabotage is undone [M Hint_Update 0x1b3380's on-channel gate]");
}
