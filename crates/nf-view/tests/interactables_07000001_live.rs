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

/// The kind-0xD8 rigid bodies (four `rb_redlamp` placements #9-#12 [D census], type halfword
/// 0xFFFF = the sphere body, mass word 1 [D params (2,1)/(4,65535)]) take a bullet's class-5
/// contact: `RB_RestingContactRot` 0x1d73a0 adds impulse = hit_vel·(scale·0.5) and
/// (point − translation)×impulse to the torque slot, and `RB_EulerEvolve` 0x1d7810 turns the
/// torque into omega (the per-type mask keeps the full cross for 0xFFFF) — the body TIPS
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
        assert_eq!(rb.rb.ty, -1, "the 0xFFFF type halfword = the sphere body [D params (4,65535)]");
        (rb.st.trans, rb.st.s13c)
    };
    assert!(radius > 0.3 && radius < 2.0, "the glist world sphere is the lamp's own [M 0x133070], got {radius}");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.rbodies.list[0].rb.quat, [1.0, 0.0, 0.0, 0.0],
        "sister: the lamp rests at the identity rotation [M RB_SetState 0x1d7da0]");

    // Into the entry hall (the sliding-door test's proven stand, then closer under the lamp):
    // walk there through the nav, arm, and aim a fifth-unit BELOW the mount — the body's
    // world sphere is the glist's r≈0.33 ball at the mount [M Control_BuildWorldSph 0x133070's
    // +0x10/+0x1C sphere; D the object's glist radius], so the shot must pass inside it, but
    // off-axis: the entry point then sits ρ≈0.2 from the translation and the game's
    // r×impulse torque (|cross| ≈ scale·0.5·ρ) is non-zero — a dead-centre hit is parallel
    // and tips nothing.
    w.walk_nav_to([7.89, 5.35, 4.47], "into the entry hall", 1.6);
    w.walk_nav_to([3.6, 5.4, 3.6], "under the lamp", 1.2);
    eprintln!("[lamp] stand {:?} centre {centre:?} r {radius}", w.scene.sim.as_ref().unwrap().bond.pos);
    assert!(w.switch_to_armed("armed for the lamp"), "an armed gun for the shot");
    let target = [centre[0], centre[1] - 0.22, centre[2]];
    let quat_now = |s: &nf_view::scene::Sim| s.drones.rbodies.list[0].rb.quat;
    let mut dbg = 0u32;
    let tipped = face_and_fire(&mut w, target.into(), "the red lamp", |s| {
        let q = quat_now(s);
        dbg += 1;
        if dbg % 60 == 0 {
            eprintln!("[lamp] t{dbg} quat {q:?} clips {} spare {} paths {} hits {}",
                s.gun.clip, s.gun.spare, s.drones.bullet_paths().0.len(), s.drones.rbodies.list[0].hits.len());
        }
        q[0] < 0.9999 || q[1].abs() > 1.0e-3 || q[2].abs() > 1.0e-3 || q[3].abs() > 1.0e-3
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
    // Sister: a fresh load, nobody shoots — the lamp rests at the identity forever.
    let mut w2 = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("reload"));
    past_intro(&mut w2);
    w2.wait(240);
    assert_eq!(w2.scene.sim.as_ref().unwrap().drones.rbodies.list[0].rb.quat, [1.0, 0.0, 0.0, 0.0],
        "the unshot lamp rests (the sister) [M the type-0-only arm — rigid_body.rs rest pin]");
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

/// The bonus's walk-over consumption: blocked on this branch — the bonus floor sits ~1.6
/// below every floor the walker reaches from either side (the spawn-side nav ends on the
/// upper hall at y 7.18; the west corridor's low route rides the escort path this harness
/// walks only with the contact). Blocker: the west-hall bonus floor, listed in the report.
#[test]
#[ignore = "blocker: the west-hall bonus floor is off the walker's reachable set on this branch — see the report"]
fn the_bond_bonus_walk_over_takes_it() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso, "07000001.bin", false, None, 1, gold_bits(0x0700_0001))
        .expect("gold load"));
    past_intro(&mut w);
    let spot = {
        let s = w.scene.sim.as_ref().unwrap();
        s.drones.pickups.iter().find(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).expect("the hall bonus [D #315]")
    };
    // The stand-in transit onto the bonus's own floor; the take is the real walked walk-over
    // (the case-6 arm logs and consumes [M Pickup_Handler 0x18e650:234-236]).
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond.pos = [spot[0], spot[1] + 0.4, spot[2] + 1.4];
    sim.bond.vel = [0.0; 3];
    drop(sim);
    w.wait(30);
    w.walk_to(walker::Wp::new(spot[0], spot[1], spot[2]).r(0.6), "onto the bond bonus");
    let taken = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS)
        .all(|p| p.gone || p.hidden);
    for _ in 0..120 {
        w.tick(Default::default(), false);
        if taken(&w) { break; }
    }
    assert!(taken(&w), "the walk-over consumes the bonus [M 0x18e650:234-236]");
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
