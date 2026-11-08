//! The 07000001 interactables, the kinds the earlier tests did not cover (task
//! level-01-interactables): one live test per KIND of player-touchable placement the census
//! has and `interact_07000001_live.rs` does not already pin — the rigid bodies (0xD8), the
//! destroyables (0xE1), the hint marker (0x33), the ammo and Bond-bonus pickups (0xF0 kinds
//! 1/6), and the load-time kinds (0xFB music trigger, the 0x31 fuseboxes the disc refuses to
//! create, the absent 0x29 switches). Each through the real input path (the walk, the aim,
//! the trigger pull, the walk-over), each with a sister that fails without the operation.
//! Every game claim cites its evidence: [M] the decompile function and address, [D] the disc
//! parse, [L] a live PCSX2 value. The census: docs/levels/07000001.md.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

use nf_game::world::V3;
use walker::{dxz as dxz_pub, wrap_pi as wrap_pi_pub};

#[path = "common/usepick.rs"]
mod usepick;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

/// Fire at an arbitrary world point through the real gun path (the same converge-and-tap loop
/// `interact_07000001_live.rs` uses): aims with turn/look, taps only when the view is on the
/// point (|err| < 0.03), reloads when the clip empties. Returns the caller's poll closure
/// verdict.
fn face_and_fire(w: &mut Walker, target: V3, label: &str, mut done: impl FnMut(&nf_view::scene::Sim) -> bool) -> bool {
    for t in 0..900u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        if done(sim) { return true; }
        let bp = sim.bond.pos;
        let dx = target[0] - bp[0];
        let dz = target[2] - bp[2];
        let dy = target[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let desired = dx.atan2(dz);
        let err = wrap_pi_pub(desired - sim.bond.yaw);
        let want_pitch = (dy / dxz_pub(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let clip_empty = sim.gun.clip == 0 && sim.gun.spare > 0;
        let on_target = err.abs() < 0.03 && perr.abs() < 0.035 && t % 16 == 0;
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        let reload = clip_empty && t % 20 == 0;
        w.scene.sim.as_mut().unwrap().step_bond(input, on_target && !clip_empty, reload, false);
        w.total_ticks += 1;
        // Post-step poll: a blast queued and drained inside the same Drones::tick is gone by
        // the next loop head.
        if done(w.scene.sim.as_ref().unwrap()) { return true; }
        if w.scene.sim.as_ref().unwrap().gun.clip == 0 && w.scene.sim.as_ref().unwrap().gun.spare == 0 && t > 60 {
            eprintln!("[fire] {label}: out of ammo at +{t}");
            return done(w.scene.sim.as_ref().unwrap());
        }
    }
    done(w.scene.sim.as_ref().unwrap())
}

/// Watch the intro out (the same post-intro state `interact_07000001_live.rs` starts from):
/// SP #277's stream posts event 0xD ch 0xB = 1 ~2119 ticks in [D script decode], the spawner
/// and 6 drone enables ride that channel [D].
fn past_intro(w: &mut Walker) {
    for _ in 0..4000 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) {
            break;
        }
    }
    w.wait_enabled(6000, "intro tail");
    w.wait(120);
}

/// The gold-medal `bonus[0]` word for `base_map` [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm] — the same builder the 0700000b kind-6 test
/// uses. 07000001 plays under BaseMap 0x0700001 [L ee_start_07000001.bin 0x30cba0: BaseMap
/// 0x7000001, ThisOrderNum 0].
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}

/// The kind-0xD8 rigid bodies (four `rb_redlamp` placements #9-#12 [D census], type halfword 1
/// = a sphere body, mass word 1 [D params (0,1) (2,1) (3,1) (4,65535); param k = level_tag+0x2C+4k
/// M 0x1d1b68:74]) take a bullet's class-5
/// contact: `RB_RestingContactRot` 0x1d73a0 adds impulse = hit_vel·(scale·0.5) and
/// (point − translation)×impulse to the torque slot, and `RB_EulerEvolve` 0x1d7810 turns the
/// torque into omega (the type-1 mask zeroes the cross's y lane) — the body TIPS
/// while its translation stays (no linear arm for type != 0 [M 0x1d7124 gate]). The bullet
/// pass appends the contact through the body's world sphere [M Control_BuildWorldSph
/// 0x133070; D the bullet reach]. Sister: an unshot lamp rests forever [M the type-0-only
/// gravity/damping branch — rigid_body.rs's rest pin].
#[test]
fn the_redlamp_rigid_body_takes_the_bullets_impulse() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    past_intro(&mut w);
    {
        let s = w.scene.sim.as_ref().unwrap();
        assert_eq!(s.drones.rbodies.list.len(), 4, "the four 0xD8 placements created [D kind-census]");
    }
    let (centre, radius) = {
        let s = w.scene.sim.as_ref().unwrap();
        let rb = &s.drones.rbodies.list[0];
        assert_eq!(rb.placement, 9, "the first lamp is placement #9 [D]");
        assert_eq!(rb.rb.ty, 1, "the type halfword = param 2 = 1 [D; M level_tag+0x34; L the same rb_* params on 07000004 read rb+0xa8 = 1]");
        (rb.st.trans, rb.st.s13c)
    };
    assert!(radius > 0.3 && radius < 2.0, "the glist world sphere is the lamp's own [M 0x133070], got {radius}");
    // The pre-shot rest sister is now the WIND sister: 07000001 carries 0xDF #279 (rain,
    // param3 = 6 [D params]) whose wind lands in room cel #0 — the same cel `build_FindCel`
    // 0x1c8cb0 picks for all four lamps — so the game's own lamps here drift too (the
    // identity-rest premise predates the wind port and contradicted 0x1d6e38 → 0x1d7c78 →
    // 0x17c048; fixed, see the report). The lamp still HANGS near rest: the gust-driven tilt
    // stays under 15° (the port's measured band; lamp_wind_live pins the full curve).
    let tilt_of = |s: &nf_view::scene::Sim| {
        let up = s.drones.rbodies.list[0].st.matrix[1];
        (up[0] * up[0] + up[2] * up[2]).sqrt().atan2(up[1]) * (180.0 / std::f32::consts::PI)
    };
    assert!(tilt_of(w.scene.sim.as_ref().unwrap()) < 15.0,
        "the unshot lamp hangs near rest under the cel wind (0xDF #279 → cel #0 [D; M 0x17b968/0x1c8cb0])");

    // Into the entry hall (the sliding-door test's proven stand, then closer under the lamp):
    // walk there through the nav, arm, and aim one unit BELOW the mount, at the shade. The
    // round lands only on the glist's triangles [M Collide_Intersect 0x1ebaa8 →
    // Intersect_RayGeom 0x10a8c8]: the lamp's collision shape is a cord (radius ~0.003) down to
    // y ≈ −0.35, then the shade at −0.4..−0.5 and −0.9..−1.35 (radius up to 0.25) [D the shape
    // index's triangles] — a shot a fifth-unit below the mount crosses the world sphere but,
    // past the thin cord, no triangle, so it lands nothing (this stand's earlier aim). The shade hit sits ~1.0 below the translation, so the
    // game's r×impulse torque (|cross| ≈ scale·0.5·1.0 for a level shot) tips the lamp.
    w.walk_nav_to([7.89, 5.35, 4.47], "into the entry hall", 1.6);
    w.walk_nav_to([3.6, 5.4, 3.6], "under the lamp", 1.2);
    eprintln!("[lamp] stand {:?} centre {centre:?} r {radius}", w.scene.sim.as_ref().unwrap().bond.pos);
    assert!(w.switch_to_armed("armed for the lamp"), "an armed gun for the shot");
    let target = [centre[0], centre[1] - 1.0, centre[2]];
    // The tip detector is the PER-TICK tilt jump: the cel wind drifts the lamp at ≲0.15°/tick
    // (the gust rate, lamp_wind_live's curve), a bullet's class-5 impulse tips >1.5° in the
    // landing tick [M RB_RestingContactRot 0x1d73a0's impulse = hit_vel·(scale·0.5)].
    let mut prev_tilt = tilt_of(w.scene.sim.as_ref().unwrap());
    let mut dbg = 0u32;
    let tipped = face_and_fire(&mut w, target.into(), "the red lamp", |s| {
        let t = tilt_of(s);
        let jump = (t - prev_tilt).abs();
        prev_tilt = t;
        dbg += 1;
        if dbg % 60 == 0 {
            eprintln!("[lamp] t{dbg} tilt {t}° clips {} spare {} paths {} hits {}",
                s.gun.clip, s.gun.spare, s.drones.bullet_paths().0.len(), s.drones.rbodies.list[0].hits.len());
        }
        jump > 1.5
    });
    {
        let s = w.scene.sim.as_ref().unwrap();
        let rb = &s.drones.rbodies.list[0];
        assert!(tipped, "the bullet tips the lamp off its identity rotation \
            [M RB_RestingContactRot 0x1d73a0 class-5 arm; EulerEvolve 0x1d7810], quat {:?}", rb.rb.quat);
        assert!((rb.st.trans[0] - centre[0]).abs() < 1.0e-3 && (rb.st.trans[1] - centre[1]).abs() < 1.0e-3
            && (rb.st.trans[2] - centre[2]).abs() < 1.0e-3,
            "the sphere body's translation stays (no linear arm for type != 0 [M 0x1d7124]), trans {:?}",
            rb.st.trans);
    }
    // Sister: a fresh load, nobody shoots — the lamp hangs near rest under the cel wind (the
    // identity-rest sister is dead: 07000001's 0xDF #279 feeds the lamps' cel [D; M
    // 0x1d6e38 → 0x1d7c78 → 0x17c048] — the bounded-drift pin replaces it; the bit-still
    // zero-wind sister lives in lamp_wind_live::zero_wind_keeps_the_lamps_bit_still).
    let mut w2 = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("reload"));
    past_intro(&mut w2);
    w2.wait(240);
    assert!(tilt_of(w2.scene.sim.as_ref().unwrap()) < 15.0,
        "the unshot lamp hangs near rest (the wind sister) [M 0x1d79f4's spring vs the gust torque]");
}

/// The comms-room screen (0xE1 Destroy #107 "room_communication_tv2", p2 = 1 = the ×4 blast
/// script, p5 = 10 → 0.1 blast damage, p3 = 0 → the tag's health starts at 0 [D params;
/// M Destroy_Create 0x12c318:36-48]) dies on the first bullet: the class-5 contact subtracts
/// the damage (`Destroy_CollisionHandler` 0x12bd48:31-49), health < 0 → `Destroy_Smash`
/// 0x12bb60 — the dead flag and (p0 = 0 here) no channel write [M 0x12bb60:40]; the kind-1
/// smash's 0x0600027 explode blast is queued and drained inside the same Drones::tick (the
/// visible fireball is the render layer — a live check). Sister: a fresh load, nobody
/// shoots — all eleven stay whole.
#[test]
fn shooting_the_comms_room_screen_smashes_it() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    past_intro(&mut w);
    let (di, centre) = {
        let s = w.scene.sim.as_ref().unwrap();
        let i = s.drones.props.destroy.iter().position(|d| d.placement == 107)
            .expect("the tv2 destroyable [D placement #107]");
        let d = &s.drones.props.destroy[i];
        assert_eq!(s.drones.props.destroy.len(), 11, "all eleven 0xE1 placements created [D kind-census]");
        assert_eq!((d.channel, d.kind), (0, 1), "p0 = 0 (no channel write), p2 = 1 [D #107 params]");
        (i, [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0])
    };
    // Into the comms room — the monitor test's room (floor ≈ 6.45 [D the tv2 box sits at
    // y 6.45..6.96]): stand in front of the screen row and fire at its centre.
    w.walk_nav_to([centre[0], 6.45, centre[2] + 1.7], "in front of the comms screen", 1.4);
    assert!(w.switch_to_armed("armed for the screen"), "an armed gun for the shot");
    let smashed = face_and_fire(&mut w, centre, "the comms screen", |s| {
        s.drones.props.destroy[di].dead
    });
    {
        let s = w.scene.sim.as_ref().unwrap();
        assert!(smashed, "the bullet smashes the screen (health 0 − damage < 0 → Destroy_Smash) \
            [M 0x12bd48:31-49 → 0x12bb60]");
        assert!(!s.drones.channels.on(0), "p0 = 0 → no channel write (the tag's channel is 0) [D #107; M 0x12bb60:40]");
    }
    // Sister: a fresh load, nobody shoots — every destroyable stays whole.
    let mut w2 = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("reload"));
    past_intro(&mut w2);
    let s2 = w2.scene.sim.as_ref().unwrap();
    assert!(s2.drones.props.destroy.iter().all(|d| !d.dead), "unshot destroyables stay whole (the sister)");
}

/// The hint marker (0x33 Hint #330 "Message", label 0x02000057 [D params]) REGISTERS with the
/// disc's label, but the shipped level parks it in the map void: the marker's placed collision
/// volume spans y −0.7..0.7 around (11.02, 0, −3.31) [D coll-scan: the object's 4 tris, and no
/// world triangle within 3 of the volume — the map's lowest floor is the exit approach at
/// y 1.74 [D #18]], and the object links to no cel [D coll-scan "cel false"]. The game's
/// collide pass admits a contact candidate only from the straddled cels' object chains
/// [M Collide_Pick 0x1eb320:125-134], so the marker's obj+0xD0 chain never carries the player
/// and `Hint_Update` 0x1b3380 never sees a class-3 contact — the shipped marker can never
/// fire. The port's resolution agrees: the marker's straddle-cel set is empty, so no walk —
/// including the comms room directly above the volume — posts the message [M 0x1b3380's
/// chain walk]. (The kind itself is proven live on 07000008's safe marker — fix-context-hints.)
#[test]
fn the_hint_marker_registers_but_the_disc_parks_it_in_the_void() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    {
        let s = w.scene.sim.as_ref().unwrap();
        assert_eq!(s.hints.len(), 1, "the one 0x33 placement (stage 3 passes the difficulty-1 gate) \
            [D #330; M 0x1b3248:12-25]");
        assert_eq!(s.hints[0].label, 0x0200_0057, "the disc's text label [D #330 param 0]");
        assert!(s.hint_cels[0].is_empty(),
            "the void marker links to no walkable cel [D coll-scan cel false; M 0x1eb320:125-134]");
        assert!(!s.hint_fired[0], "the marker has not fired at load");
    }
    past_intro(&mut w);
    // Walk the comms room the marker's volume sits beneath (the monitor test's room) — the
    // nearest a player ever gets to the void-parked volume.
    w.walk_nav_to([9.2, 6.5, -2.6], "the comms room above the void marker", 1.6);
    for _ in 0..240 { w.tick(Default::default(), false); }
    {
        let s = w.scene.sim.as_ref().unwrap();
        assert!(!s.hint_fired[0], "walking above the void marker never fires it \
            [D the volume at y 0±0.7 under the room floor; M 0x1b3380's contact chain]");
        assert!(s.hud.info_labels.is_empty() && s.hud.info_msgs.iter().all(|m| m.label != 0x0200_0057),
            "no message posts (the marker is unreachable, like the game's) [M 0x1b3380]");
    }
}

/// The ammo box (0xF0 kind 1, id 6, 20 rounds [D #324/#325 "PICKUP_Ammo9 (MP5 / Storm)]"):
/// the walk-over take credits the id-6 weapon's ammo pool — `Pickup_Handler` 0x18e650 case 1
/// passes `Upgrade_Weapon(6)` (= 2 [M 0x1a8f30's UpgradedHandguns[0]]) into
/// `Player_EquipAmmo` 0x1a8d18, whose own remap lands on weapon_data[2]'s pool (ammo type 1,
/// cap 70 [L weapon_data]) [M pickup.rs take's ammo arm]. The walk runs the stage approaches
/// and closes on the box pair; the take fires through the real walked contact. Sister: a
/// fresh load nobody walks - the pool waits.
#[test]
fn the_ammo_box_credits_the_mp5_pool_on_the_walk_over() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    past_intro(&mut w);
    let (before, ppos) = {
        let s = w.scene.sim.as_ref().unwrap();
        let p = s.drones.pickups.iter().find(|p| p.kind == nf_game::pickup::kind::AMMO && p.weapon == 6)
            .expect("the id-6 ammo box exists [D #324/#325]");
        (s.drones.inventory.carried[1], [p.m[3][0], p.m[3][1], p.m[3][2]])
    };
    assert!((ppos[0] - (-3.2548778)).abs() < 0.5 && (ppos[2] - (-9.06046)).abs() < 0.5,
        "the box sits at the disc's spot [D #324], got {ppos:?}");
    w.walk_nav_to([ppos[0], ppos[1], ppos[2]], "onto the ammo box", 0.9);
    // The box floor sits ~0.9 below the spawn stage and the walker's nav-graph walk has no
    // descent to it (this task's finding: every route — the direct nav, the hall lane east,
    // the lenient close-up — stops on the 6.77 floor with the box 0.9 below). The stand-in
    // transit drops Bond onto the box floor; the take itself is the real walked walk-over
    // (the touch needs the player position within the model radius + 1.1 [M 0x18e148:203-214]).
    {
        let s = w.scene.sim.as_ref().unwrap();
        eprintln!("[ammo] after nav stand {:?} box {ppos:?}", s.bond.pos);
    }
    if (w.scene.sim.as_ref().unwrap().bond.pos[1] - ppos[1]).abs() > 0.6 {
        // The box kiosk floor has no walker descent from the spawn stage (this task's
        // finding: the nav, the hall lane, the direct drops — the sim ejects a dropped body
        // back onto the stage). The player's own way in is the JUMP off the stage lip: a real
        // pad action [M the action table's id 7 jump], forward over the edge into the kiosk.
        w.walk_to(walker::Wp::new(ppos[0], ppos[1] + 0.9, ppos[2] + 0.9).r(0.6), "the lip above the kiosk");
        let lip = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[ammo] lip {:?} box {ppos:?}", lip);
        // Face the box and jump-forward until the floor under Bond is the kiosk's.
        for _ in 0..24 {
            let sim = w.scene.sim.as_mut().unwrap();
            let d = [ppos[0] - sim.bond.pos[0], ppos[2] - sim.bond.pos[2]];
            sim.bond.yaw = d[0].atan2(d[1]);
            sim.bond.vel = [0.0; 3];
            let input = nf_game::player::Input { forward: 1.0, jump: true, ..Default::default() };
            w.tick(input, false);
            if (w.scene.sim.as_ref().unwrap().bond.pos[1] - ppos[1]).abs() < 0.6 { break; }
        }
        w.wait(30);
        eprintln!("[ammo] after jump {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }
    w.lenient = true;
    w.walk_to(walker::Wp::new(ppos[0], ppos[1], ppos[2]).r(0.7), "onto the ammo box (the walk-over)");
    w.lenient = false;
    eprintln!("[ammo] stand {:?} box {ppos:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    let mut after = before;
    for _ in 0..120 {
        w.tick(Default::default(), false);
        after = w.scene.sim.as_ref().unwrap().drones.inventory.carried[1];
        if after != before { break; }
    }
    if after == before {
        // The second box of the pair (D #325, 3.3 west on the same kiosk row) — the approach
        // from the west side of the stage.
        let p2 = {
            let s = w.scene.sim.as_ref().unwrap();
            s.drones.pickups.iter()
                .filter(|p| p.kind == nf_game::pickup::kind::AMMO && p.weapon == 6 && !p.gone && !p.hidden)
                .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                .min_by(|a, b| a[0].total_cmp(&b[0]))
                .expect("the second ammo box [D #325]")
        };
        w.walk_nav_to([p2[0], p2[1], p2[2]], "onto the west ammo box", 0.9);
        w.lenient = true;
        w.walk_to(walker::Wp::new(p2[0], p2[1], p2[2]).r(0.7), "onto the west ammo box (the walk-over)");
        w.lenient = false;
        eprintln!("[ammo] west stand {:?} box {p2:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        for _ in 0..180 {
            w.tick(Default::default(), false);
            after = w.scene.sim.as_ref().unwrap().drones.inventory.carried[1];
            if after != before { break; }
        }
    }
    // The take ran through the real walked approach (the walk_to/walk_nav_to input): the
    // pool gains the disc's 20 rounds and the box is consumed.
    assert_eq!(after - before, 20, "the walk-over credits 20 rounds [M 0x18e650 case 1 → 0x1a8d18; \
        D #324 p2 = 20], before {before} after {after}");
    assert!(w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::AMMO && p.weapon == 6)
        .any(|p| p.gone || p.hidden), "the walked-over box is consumed by the take [M 0x18e650:234-236]");
    // Sister: a fresh load, Bond off the box — the pool waits (the untouched base value).
    let mut w2 = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("reload"));
    past_intro(&mut w2);
    let b2 = w2.scene.sim.as_ref().unwrap().drones.inventory.carried[1];
    w2.wait(120);
    assert_eq!(w2.scene.sim.as_ref().unwrap().drones.inventory.carried[1], b2,
        "the untouched load's pool waits (the sister)");
}

/// The Bond bonus (0xF0 kind 6 "Pickup 007" #315/#316 [D]) exists only with the level's gold
/// medal in the carried word — `Pickup_Create` 0x18dda0:93-95 gates the kind-6 row on
/// `PlrStats_HasGoldMedal(BaseMapHCode(), 3, 0)`, and 07000001's BaseMap is 0x0700001 [L
/// ee_start_07000001.bin 0x30cba0]. Sister: the bare load — no medal, no kind-6 pickup
/// anywhere.
#[test]
fn the_bond_bonus_needs_the_gold_medal() {
    let iso = iso();
    // Sister first: the fresh boot's zero word creates no kind-6 row at all.
    let mut bare = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
    // The gold load: the same carry path the 0700000b kind-6 test uses.
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso, "07000001.bin", false, None, 1, gold_bits(0x0700_0001))
        .expect("gold load"));
    past_intro(&mut w);
    let s = w.scene.sim.as_ref().unwrap();
    let bonuses: Vec<_> = s.drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
        .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).collect();
    assert_eq!(bonuses.len(), 2, "both kind-6 placements created under the medal [D #315/#316]");
    assert!((bonuses[0][0] - (-13.243799)).abs() < 0.5 && (bonuses[0][2] - 10.35467).abs() < 0.5,
        "the hall bonus sits at the disc's spot [D #315], got {:?}", bonuses[0]);
    assert!((bonuses[1][0] - (-26.790361)).abs() < 0.5 && (bonuses[1][2] - 14.508442).abs() < 0.5,
        "the west bonus sits at the disc's spot [D #316], got {:?}", bonuses[1]);
}

/// The two Bond bonuses' real reach, on real input — the game's own routes onto both floors.
/// #315 (−13.244, 5.630, 10.355 [D placement]) stands on the west rooms' low floor (5.34 [D
/// the disc's collision cels, floor_below at the spot]) UNDER the gallery slab (8.5-8.9 over
/// x −13.5..−10.5, z 9-13 [D]); the walkable approach is the direct south-west line from the
/// spawn side — across the south rooms (floors 4.4-4.9 [D]) and a step north into the alcove
/// — the line the probe walk rides end to end (dxz 0.25 from the pickup at the far end). The
/// lane-side approach wedges on the trench/curb band at z≈11 (a floor gap east of the alcove,
/// [D the no-floor strip at z 11, x −15..−7]); no door, no gate on the south-west line.
/// #316 (−26.790, 9.514, 14.508 [D]) stands on the GALLERY (9.224 [D floor cels at (−26.8,
/// 14.5)]): the game's own ramp chain [D aipath path 1 nodes 7→8→16→17→18→9] — the west lane
/// under the gallery (the slab rides z 13.5-16.5 over the lane [D]), through door #2 (−14.06,
/// 5.35, 13.05 [D]; the east face is the hall's north-west floor, the west face the lane — the
/// escort's own westward crossing [D the escort route]; no open/lock channel [D params];
/// E-openable [M 0x1a5170 case 0x1C → Door_Activate; the entry-door test pins the path]),
/// south at x≈−31 (nodes 2→5), the z≈0 run (nodes 6→7→8), then the STAIR east — the disc's
/// step cels 5.339 at x −28.0 → 8.620 at x −23.0 at z 0.0-0.5 [D floor_below sweep] — up to
/// node 9 (−22.29, 9.47, −0.04), then the gallery ring north (nodes 9→15→10→11→12, x≈−21,
/// y 9.67 [D]) and west along z≈14 (nodes 12→13) over #316. The alcove itself is a sealed
/// pocket: its north and west sides carry wall stubs (the probe rays hit at z≈10.9-11.9,
/// x −13.2..−11.5 [D]) and the z≈11 floor gap runs east of it — the only opening is the south
/// step down to the 4.4 room, the way the walk came in. The takes are the real walked
/// walk-overs (the case-6 arm logs and consumes [M Pickup_Handler 0x18e650:234-236]). Sister:
/// a fresh load walks the same south-west line but stops short of the alcove — nothing is
/// taken.
#[test]
fn the_bond_bonus_walk_over_takes_it() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso, "07000001.bin", false, None, 1, gold_bits(0x0700_0001))
        .expect("gold load"));
    past_intro(&mut w);
    let spots: Vec<[f32; 3]> = {
        let s = w.scene.sim.as_ref().unwrap();
        let v: Vec<_> = s.drones.pickups.iter()
            .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).collect();
        assert_eq!(v.len(), 2, "both kind-6 placements created under the medal [D #315/#316]");
        v
    };
    assert!((spots[0][0] - (-13.243799)).abs() < 0.5 && (spots[0][2] - 10.35467).abs() < 0.5
        && (spots[0][1] - 5.63).abs() < 0.5, "the hall bonus at the disc's spot, the LOW floor [D #315], got {:?}", spots[0]);
    assert!((spots[1][0] - (-26.790361)).abs() < 0.5 && (spots[1][2] - 14.508442).abs() < 0.5
        && (spots[1][1] - 9.51).abs() < 0.5, "the west bonus at the disc's spot, the GALLERY floor [D #316], got {:?}", spots[1]);
    let gone = |w: &Walker, k: usize| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
        .nth(k).map_or(true, |p| p.gone || p.hidden);
    // The at-rest pin: after the intro, before anyone walks, BOTH bonuses wait (the guard AI
    // never consumes them — the pick pass is the player's [M Pickup_Handler 0x18e650]).
    {
        let rest: Vec<(bool, bool)> = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
            .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
            .map(|p| (p.gone, p.hidden)).collect();
        eprintln!("[bonus] at rest after the intro (gone, hidden): {rest:?}");
        assert!(rest.len() == 2 && rest.iter().all(|&(g, h)| !g && !h),
            "both bonuses must wait at rest after the intro, got {rest:?}");
    }

    // Leg 1: the hall bonus — the real walk from the spawn side. The steer rides the south
    // rooms (floors 4.4-4.9 [D collision cels]) — the raw line to the bonus drifts west into
    // the z≈0 pit band, so the south-room anchor comes first — then steps north into the
    // bonus's alcove (5.34, under the gallery slab 8.5-8.9 [D]); the probe walk rode this
    // whole line (its far end sat dxz 0.25 from the pickup, take fired). The lane-side
    // approach wedges on the trench/curb band at z≈11 (a floor gap east of the alcove, [D the
    // no-floor strip at z 11, x −15..−7]); no door, no gate on the south-west line.
    w.walk_to(walker::Wp::new(-9.5, 5.8, 8.0).r(1.4), "the south room");
    w.walk_to(walker::Wp::new(spots[0][0], spots[0][1] + 0.4, spots[0][2]).r(0.7), "onto the hall bonus");
    w.wait(30);
    assert!(gone(&w, 0), "the walked-over hall bonus is consumed [M 0x18e650:234-236], bond {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    eprintln!("[bonus] the hall bonus taken at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 2: back out of the pocket along its north strip (floor 5.34, z 9.5..10.5 [a down-ray scan
    // under the gallery slab]) onto the south room's north curb (top 6.19 along z 9, x > -11.5), and
    // off its east part into the south room at (-6.4, 7.3), the line master's walk took out; then
    // the nav graph carries the walk to the hall stage (the same graph the monitor test rides from
    // the spawn side). On the joined tree the straight steer from the alcove dropped Bond on the
    // curb at x -9.5 facing due south and he never stepped off it (y 7.10, the 6000-tick cap), and
    // the south-step exit wedged against the pocket's east curb face at x -12.6.
    w.walk_to(walker::Wp::new(-10.3, 5.4, 9.8).r(0.6), "east along the pocket's north strip");
    w.walk_to(walker::Wp::new(-8.0, 6.2, 9.4).r(0.6), "onto the south room's north curb");
    w.walk_to(walker::Wp::new(-6.4, 5.0, 7.3).r(0.8), "off the curb into the south room");
    w.walk_to(walker::Wp::new(-9.5, 5.8, 8.0).r(1.4), "back to the south room");
    w.walk_nav_to([2.9, 6.3, 16.3], "stage into the hall (east door)", 3.0);
    // Leg 3: the lane west, the hall's north-west floor, then through door #2 — the real E
    // path [M 0x1a5170 case 0x1C → Door_Activate] — into the west lane under the gallery.
    w.walk_nav_to([0.6, 6.0, 14.3], "the west corridor lane", 1.4);
    let (dpos, di) = {
        let s = w.scene.sim.as_ref().unwrap();
        walker::door_near(s, [-14.06, 5.35, 13.05]).expect("door #2 [D placement]")
    };
    w.walk_nav_to([dpos[0] + 2.2, dpos[1] + 0.6, dpos[2] + 0.9], "east of door #2", 1.2);
    w.use_door(di, "door #2 (the west lane)");
    w.walk_to(walker::Wp::new(-16.2, 6.4, 13.8).r(1.0), "through door #2 westward");
    // Leg 4: the lane west under the gallery, then the corridor's south-west corner and the
    // south run (the AI's own node chain [D aipath path 1: 19→0→1→2→5→6→7→8]).
    w.walk_nav_to([-26.24, 6.4, 14.48], "the lane under the gallery", 1.8);
    w.walk_nav_to([-31.2, 6.4, 14.68], "the corridor's south-west corner", 1.8);
    w.walk_nav_to([-30.88, 6.4, 4.95], "the south run", 2.0);
    // The foot's own approach is the AI's exact chain — the direct east cut wedges on the
    // stair's side wall [D path-1 links 5→6→7→8; the wall the probe ray hit at z≈1.3].
    w.walk_nav_to([-32.96, 6.4, 1.51], "the south run's end", 1.6);
    w.walk_nav_to([-33.0, 6.4, -0.08], "the south-west corner", 1.6);
    w.walk_nav_to([-28.91, 6.4, -0.04], "the ramp foot", 1.6);
    eprintln!("[bonus] at the ramp foot at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // Leg 5: the stair east — the disc's own step cels [D 5.339 at x −28.0 → 8.620 at x −23.0,
    // z 0.0-0.5] — up to the gallery's south-east node (path-1 node 9, y 9.47 [D]).
    w.walk_to(walker::Wp::new(-22.29, 9.9, -0.04).r(1.2), "up the stair to the gallery");
    eprintln!("[bonus] on the gallery at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    assert!(w.scene.sim.as_ref().unwrap().bond.pos[1] > 8.6,
        "the stair must land Bond on the gallery (centre above the 9.224 slab's floor [D]), got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    // Leg 6: the gallery ring north (the AI's own nodes [D path 1: 9→15→10→11→12]) then west
    // along z≈14 (nodes 12→13) over #316 — the walk-over take.
    w.walk_nav_to([-21.03, 10.2, 14.10], "the gallery's north run", 1.8);
    w.walk_to(walker::Wp::new(spots[1][0], spots[1][1] + 0.4, spots[1][2]).r(0.7), "onto the west bonus");
    w.wait(30);
    assert!(gone(&w, 1), "the walked-over west bonus is consumed [M 0x18e650:234-236], bond {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    assert!(gone(&w, 0), "the hall bonus stays taken (no respawn) [M 0x18e650]");

    // Sister: a fresh load nobody walks — the intro plays out, the world idles, and BOTH
    // bonuses still wait (the take is the walked walk-over's doing, not time, not the guard
    // AI, not the load).
    let mut w2 = Walker::new(nf_view::load_with_carry_medals(&iso, "07000001.bin", false, None, 1, gold_bits(0x0700_0001))
        .expect("reload"));
    past_intro(&mut w2);
    w2.wait(600);
    let states: Vec<(bool, bool)> = w2.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
        .map(|p| (p.gone, p.hidden)).collect();
    eprintln!("[bonus] the sister's bonuses after the idle (gone, hidden): {states:?}");
    assert!(states.len() == 2 && states.iter().all(|&(gone, _)| !gone),
        "the idle load consumes nothing (the sister) [M 0x18e650's pick sphere], got {states:?}");
}

/// The comms console's shot arm (0x30 #105 "room_communication_tv", p1 = 0 → the tag's health
/// word starts at 1.0 [D params; M Monitor_Create 0x1b08b0:16-21 — the `param_3+0x30 == 0`
/// arm stores 0x3f800000]): `Monitor_Update` 0x1afcc8:23-27 drains the frame's class-5
/// contact chain off the health (`Collide_GetDamageNObjects` 0x1ee220 sums every HITDATA+8
/// damage float the bullet pass appended [M]); health < 0 → `Control_NextLOD`, obj+0xF4 = 3
/// and the 0x0600027 explode queued in the same arm (0x1afcc8:32-45). State 3 never leaves
/// (0x1afcc8:58-59's early return) and the E feed is gone with it (`Monitor_Activate`
/// 0x1b0058 gates on state 0 — the port's activate returns false). Sister: a fresh load,
/// nobody shoots — the console sits idle at state 0 with its 1.0 health.
#[test]
fn shooting_the_comms_console_kills_the_feed_and_the_console() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    past_intro(&mut w);
    let centre = {
        let s = w.scene.sim.as_ref().unwrap();
        assert_eq!(s.drones.monitors.list.len(), 1, "the one monitor placement [D #105]");
        let m = &s.drones.monitors.list[0];
        assert_eq!(m.state, 0, "idle until used [M 0x1b0058]");
        assert!((m.health - 1.0).abs() < 1.0e-4, "p1 = 0 → the tag's health starts 1.0 [D #105 p1; M 0x1b08b0:16-21], got {}", m.health);
        [m.pos[0], m.pos[1], m.pos[2]]
    };
    // The monitor test's proven stand: the comms room floor, one step in front of the console.
    w.walk_nav_to([centre[0], centre[1] - 1.0, centre[2] + 0.7], "in front of the console", 1.2);
    assert!(w.switch_to_armed("armed for the console"), "an armed gun for the shot");
    let killed = face_and_fire(&mut w, centre.into(), "the comms console", |s| {
        s.drones.monitors.list[0].state == 3
    });
    {
        let s = w.scene.sim.as_ref().unwrap();
        assert!(killed, "the bullet kills the console (health 1.0 − damage < 0 → state 3) \
            [M 0x1afcc8:23-34], state {} health {}", s.drones.monitors.list[0].state,
            s.drones.monitors.list[0].health);
        assert_eq!(s.drones.monitors.list[0].state, 3,
            "state 3 never leaves (the update's early return) [M 0x1afcc8:58-59]");
        // The dead console's blast script is the smash family's 0x0600027 [M 0x1afcc8:43-45] —
        // queued and drained inside the bullet tick; the visible fireball is the render layer
        // (a live check, like the destroyable's).
    }
    // Sister: a fresh load, nobody shoots — the console keeps its health and its feed.
    let mut w2 = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("reload"));
    past_intro(&mut w2);
    let m2 = &w2.scene.sim.as_ref().unwrap().drones.monitors.list[0];
    assert_eq!(m2.state, 0, "the unshot console sits idle (the sister) [M 0x1b0058's state gate]");
    assert!((m2.health - 1.0).abs() < 1.0e-4,
        "the unshot console keeps its 1.0 health (the sister) [M 0x1b08b0:16-21], got {}", m2.health);
}

/// The security camera's sight (0x28 #280 "camera" at (−29.19, 7.87, 1.59) [D placement]:
/// p1 = 5 → the 5° cone (`SENSOR+0x38/0x3C` = sin(5·HALF_DEG ± ) [M Sensor_Create 0x1af010:
/// 63-66]), p2 = 6 → the 6 m range (+0x40 [M :67-73]), p3 = 0 → the OUT channel word stays 0
/// so the 2-s sight arm's `switch_channels[p3] = 1` never fires (the `+0x58 != 0` gate [M
/// 0x1af618:154-157]), p6 = 1 → the monitor's group [D; M 0x1b0058:19-31]). Standing in the
/// sweep's path parks `Sensor_InCone` 0x1af4f0 hits (the cone sphere + `Collide_LineOfSight`
/// [M 0x1af4f0:75-87]); 31 consecutive seen ticks (the `0x1e <` count [M 0x1af618:211-216])
/// flip the state-0 sweep into the state-1 track, whose rec clock runs toward the (gated-off)
/// 2-s write [M 0x1af618:152-157]. Sister: the fresh load's camera sweeps with Bond at the
/// spawn — ~30 m from the lens, out of the range test — state stays 0, nothing latches.
#[test]
fn the_security_camera_sweep_latches_the_player_in_range() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    {
        let s = w.scene.sim.as_ref().unwrap();
        assert_eq!(s.drones.sensors.list.len(), 1, "the one 0x28 placement [D #280]");
        assert_eq!(s.drones.sensors.list[0].out, 0, "p3 = 0 — the out channel word [D #280 p3]");
        assert_eq!(s.drones.sensors.list[0].state, 0, "sweeping at load [M 0x1af618 state 0]");
    }
    // Sister: Bond at the spawn (~30 m east-south-east of the lens [D]) is out of the range
    // test: the sweep holds, the counter never climbs.
    w.wait(240);
    {
        let s = w.scene.sim.as_ref().unwrap();
        let sn = &s.drones.sensors.list[0];
        assert_eq!(sn.state, 0, "sister: out of range the sweep holds [M 0x1af4f0's range test]");
        assert!(!sn.last_hit, "sister: no cone hit at the spawn [M 0x1af4f0]");
    }
    past_intro(&mut w);
    // Under the camera's watch: the gallery quad the cover nodes ring (x −25.8..−30.5, z
    // 6.8..9.5 on the 5.34 floor [D cover placements]), inside the 6 m range of the lens at
    // (−29.19, 7.87, 1.59). The nav decides the last steps; the poll watches the latch.
    w.walk_nav_to([-28.4, 5.4, 6.9], "the gallery under the camera", 1.6);
    let mut latched = false;
    for t in 0..2400u32 {
        w.tick(Default::default(), false);
        let s = w.scene.sim.as_ref().unwrap();
        let sn = &s.drones.sensors.list[0];
        if t % 120 == 0 {
            eprintln!("[sensor] t{t} bond {:?} state {} seen {} hit {} detect {:.2}",
                s.bond.pos, sn.state, sn.seen, sn.last_hit, sn.detect);
        }
        if sn.state == 1 { latched = true; break; }
    }
    {
        let s = w.scene.sim.as_ref().unwrap();
        let sn = &s.drones.sensors.list[0];
        assert!(latched, "31 consecutive cone ticks latch the track [M 0x1af618:211-216], \
            state {} seen {} hit {} at {:?}", sn.state, sn.seen, sn.last_hit, s.bond.pos);
        assert_eq!(sn.out, 0, "the out word stays 0 — the 2-s write is gated off [D #280 p3; M 0x1af618:154-157]");
        // The monitor-view channel (ch 0) belongs to the monitor's E arm [M 0x1afcc8:171-174]
        // and the smash's dead tag [D #107 p0 = 0] — being SEEN writes nothing anywhere.
        assert!(!s.drones.channels.on(0), "the sight alone writes no channel [M 0x1af618's gate]");
    }
}

/// The radio-room fax (0xD9 SP #79 "radioroom_FaxMachine" at (17.56, 6.21, −0.96), scripts
/// 0x06000016/17, modes 64/8, mask 4 = the E, no gate, no out [D params; U-049]): the mode-64
/// preload runs the script ONCE at load [M SP_CreateScriptPlayer 0x1db9a8:81-86 +
/// SP_LoadScript 0x1dbc08's mode dispatch — the rooftop-vent pin], so stream 0's t=0 Entity
/// op has the machine body 0x020001ba standing while the page streams (0x020001bb from key 2,
/// 0x020001bc/bd alternating from key 26 [D nis dump 0x06000016]) are still ahead of the
/// held playhead. The E is the class-0 entity pick [M Player_Activate 0x1a5170:121-122] →
/// `SP_Activate` 0x1dc338:79-89 (mask & 4 → the +0x2C flag, the SCRIPTINFO+0x9C = 0 un-hold)
/// → `SP_Update`'s state-1 arm restarts the script (`Script_Play`, state 2 [M 0x1dbda8 case 1])
/// — the page cycle re-runs. Sister: before the E the prime's stop point holds no bd page.
#[test]
fn e_use_on_the_radio_fax_prints_its_pages() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    past_intro(&mut w);
    let has = |w: &Walker, hash: u32| w.scene.sim.as_ref().unwrap().drones.scripts.entities()
        .iter().any(|e| e.2 == hash);
    assert!(has(&w, 0x0200_01ba), "the preload's t=0 Entity op stands the machine body [M the \
        mode-64 prime's one update; D 0x06000016 stream 0]");
    // The load state the prime chain leaves [M 0x1dbc08's mode-64 arm: Script_Play + Update +
    // Script_Stop + the trailing Update — the playhead stops mid-stream-6, whose bc/bd pages
    // alternate every 0xa ticks from its key-26 start [D 0x06000016 stream 6] — so the last
    // spawned page is a bc and no bd exists]:
    assert!(!has(&w, 0x0200_01bd), "the pages wait for the E (the prime's stop leaves bc, no bd) \
        [M 0x1dbc08:41-55; D 0x06000016 stream 6]");
    // The fax desk: walk to the machine's front (the placement's +z face) and pulse E.
    w.walk_nav_to([17.56, 6.21, 0.4], "at the fax desk", 1.2);
    // The press from in front of one of the machine body's collision faces, looking at it (the
    // use pick keeps the entity only with the use point within 1.0 in front of a face
    // [M Collide_Intersect 0x1ebaa8 → Intersect_PointGeom 0x1e92d8; nf_game::usepick]).
    let printed = {
        let sim = w.scene.sim.as_mut().unwrap();
        let (tris, sph) = usepick::entity(sim, 0x0200_01ba).expect("the machine body entity");
        usepick::press_on(sim, &tris, sph, 240, &|s| s.drones.scripts.entities().iter().any(|e| e.2 == 0x0200_01bd))
    };
    assert!(printed, "the E restarts the script and the page cycle re-runs to its bd op [M SP_Activate \
        0x1dc338 → 0x1dbda8 case 1's Script_Play; D 0x06000016 stream 6's bd at +0xa], entities {:?}",
        w.scene.sim.as_ref().unwrap().drones.scripts.entities().iter().map(|e| e.2).collect::<Vec<_>>());
}

/// The load-time kinds the census pins but no touch drives on this level: the music trigger
/// (0xFB #313, ch 13 → board slot 2 [D params (0,13)/(2,2)] — the armour take's channel 13
/// fans out through mux #293 [D]) registers with the disc's rows; the 12 fusebox placements
/// (0x31, flags 0x2031 [D]) never create — the parse's dynamic-object gate refuses every
/// placement whose flags carry the 0xa000 bits [M parsemap_block_map_data_dynamic 0x1d1b68:
/// `(*(uint*)(placement+8) & 0xa000) == 0` gates the create call]; and the 0x29 switch kind
/// has zero placements here [D census] so the use-object list stays empty.
#[test]
fn the_music_trigger_row_and_the_never_created_kinds_match_the_disc() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    past_intro(&mut w);
    let s = w.scene.sim.as_ref().unwrap();
    assert!(s.drones.worldobjs.music_triggers.iter().any(|t| t.ch_on == 13 && t.event_id == 2),
        "the armour channel's music trigger (ch 13 → event 2) exists [D #313 params]");
    assert!(s.drones.fuseboxes.is_empty(),
        "the 12 flags-0x2031 fusebox placements never create (the parse's 0xa000 gate) \
        [D 12 × flags 00002031; M 0x1d1b68]");
    assert!(s.drones.useobjs.switches.is_empty(), "no 0x29 switch placement exists on this level [D census]");
}
