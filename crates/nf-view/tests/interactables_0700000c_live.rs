//! interactables-0c: the player-touchable placement kinds on 0700000c (Chain Reaction part 1,
//! the power station) that the level's earlier tests don't already pin. The kinds already green
//! elsewhere (interact_0700000c_live.rs: the plain door E, the TouchOnce walk-over, the armour
//! pickup, the oil-barrel shot, the param-less fuseboxes, the zipline, the Q-Claw, the music
//! trigger; walk_0700000c_live.rs: the ST-1 plate → objective 1, the CamSubject photograph, the
//! LoadLevel gate; walkreal_0700000c_live.rs: the yard ladders; fix_level_0c_live.rs: the
//! hinge-lock burns, the AND mux, the SS hatch sweeps; guards-0c/-b: the drones/spawners):
//! this file adds the secret-door chain — the E-operated trigger volume (mask 4), the body-touch
//! volume, the AND and the swinging SS door, with the chain's own SoundTrigger start — plus the
//! hint plate's global-voice rule, the weapon pickup, the medal-gated kind-6 bonus and the
//! destroyables' bullet-filter mask. Each test drives the real input (the walk, the E, the
//! trigger pull) and asserts the game's effect with its sister.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes. A teleport reaches only the
//! spots whose walk is a separately documented blocker; the operations themselves always run
//! through the real input.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Load, wait for the intro to release the controls, settle.
fn ready() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000c.bin", false).expect("load 0700000c"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    w
}

/// Teleport Bond to a spot and let him land (the reach stand-in: the spot's own walk is not the
/// operation under test; every operation below is real input from the settled stance).
fn drop_at(w: &mut Walker, p: [f32; 3]) -> [f32; 3] {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = p;
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    w.scene.sim.as_ref().unwrap().bond.pos
}

/// The floor height under a spot (the settle target for the drops).
fn floor_under(w: &Walker, x: f32, y: f32, z: f32) -> f32 {
    w.scene.sim.as_ref().unwrap().world.segment_hit([x, y + 2.0, z], [x, y - 30.0, z])
        .map(|(t, _)| y + 2.0 - 32.0 * t)
        .unwrap_or(y)
}

/// Fire at an arbitrary world point through the real gun path (aims with turn/look, taps only
/// when the view is on the point, reloads when the clip empties).
fn face_and_fire(w: &mut Walker, target: [f32; 3], label: &str) {
    for t in 0..900u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let dx = target[0] - bp[0];
        let dz = target[2] - bp[2];
        let dy = target[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / walker::dxz(bp, target.into()).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let clip_empty = sim.gun.clip == 0 && sim.gun.spare > 0;
        let on_target = err.abs() < 0.03 && perr.abs() < 0.035 && t % 16 == 0;
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        let reload = clip_empty && t % 20 == 0;
        w.scene.sim.as_mut().unwrap().step_bond(input, on_target && !clip_empty, reload, false);
        w.total_ticks += 1;
        if w.scene.sim.as_ref().unwrap().gun.clip == 0 && w.scene.sim.as_ref().unwrap().gun.spare == 0 && t > 60 {
            eprintln!("[fire] {label}: out of ammo at +{t}");
            return;
        }
    }
}

/// The barrel leaf of one main-map placement (the destroy vec keeps the placement index).
fn barrel_of(w: &Walker, placement: usize) -> usize {
    w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
        .position(|d| d.placement == placement)
        .unwrap_or_else(|| panic!("no destroyable for placement {placement}"))
}

/// The secret-door spot (yard warehouse, box [52.24, 0.90, −174.45]..[53.29, 3.20, −172.73]
/// [D #558/#559]): the E-interactable #558 (out ch 3, mask 4 — the use-key flag) and the
/// body-touch #559 (out ch 2, mask 8) share it, and the AND #560 (inputs 2,3 [D]) writes ch 8,
/// which the secret door `Door_secret1` #544 sweeps on [D #544 p3=8; M SS_Update 0x1a9948].
/// The E write is `Trigger_Activate` 0x1b2318's type-2 arm: the `mask & 4` gate then the out
/// write [M triggers.rs activate_request]; the body write is `Trigger_Hit`'s mask-8 player arm
/// [M triggers.rs hit]. Sister: the same E from one step north — ch 3 rises, but with the body
/// outside the volume ch 2 never rises and the AND holds ch 8 off and the door shut.
#[test]
fn the_secret_door_needs_the_body_touch_and_the_e_together() {
    let mut w = ready();
    // The SS at rest: record the door leaf's pose before anything.
    let si = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
        .position(|s| s.placement == 544).expect("the Door_secret1 SS #544 [D]");
    let rest = w.scene.sim.as_ref().unwrap().drones.sss.list[si].matrix;
    let rest_full = rest; // the whole posed matrix for the swing check
    // The stand one step north of the spot's box: the walk capsule (r 0.55 [M 0x1953f4]) must
    // clear the box's z edge −172.73, so the stand sits at z −171.5 (reach −172.05).
    let f = floor_under(&w, 52.7, 2.5, -171.5);
    drop_at(&mut w, [52.7, f + 1.0, -171.5]);
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let (t558, t559) = {
        let d = &sim.drones;
        (d.triggers.iter().position(|t| t.placement == 558).expect("trigger #558 [D]"),
         d.triggers.iter().position(|t| t.placement == 559).expect("trigger #559 [D]"))
    };
    assert_eq!(sim.drones.triggers[t558].mask, 0x4, "#558's mask is the use-key flag [D]");
    assert_eq!(sim.drones.triggers[t559].mask, 0x8, "#559's mask is the player body [D]");
    assert!(!sim.drones.channels.on(2), "sister: ch 2 off at the stand [D #559 body box]");
    assert!(!sim.drones.channels.on(3), "sister: ch 3 off at load");
    assert!(!sim.drones.channels.on(8), "sister: the AND's ch 8 off at load [D #560]");
    assert!(!sim.drones.sound_triggers.playing(1534), "sister: the door's sound silent [D #562]");
    assert!((0..3).all(|k| (sim.drones.sss.list[si].matrix[3][k] - rest_full[3][k]).abs() < 0.01),
        "sister: the door holds its rest pose");
    // The E from the stand: the use point (head + 1 m of look [M 0x1a5170]) lands inside
    // #558's box expanded by the pick radius, the body's capsule does not reach the volume.
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI; // face −z, the spot
    let mut e_fired = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), w.total_ticks % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(3) { e_fired = true; break; }
    }
    assert!(e_fired, "the E fires the mask-4 trigger #558 → ch 3 [M 0x1b2318:23-33; D #558]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(2), "sister: the body one step north never touches #559 [D box]");
    assert!(!sim.drones.channels.on(8), "sister: the AND lacks ch 2 — ch 8 stays off [D #560 list 2,3]");
    // The walk in (real steps across the box edge — no teleport inside).
    let f_in = floor_under(&w, 52.7, 2.5, -173.6);
    w.walk_to(Wp::new(52.7, f_in.max(f) + 0.2, -173.6).r(0.5), "into the secret-door spot");
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(2),
        "the body touch writes ch 2 [D #559 mask 8 out 2; M Trigger_Hit's mask-8 arm]");
    // The same E now closes the AND.
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), w.total_ticks % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(8) { break; }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(8),
        "the AND raises ch 8 on (2 AND 3) [D #560; M Trigger_Update case 3]");
    // The ch-8 rise starts the door's own sound trigger #562 (on ch 8 → id 1534, non-global
    // [D #562 p0=8 p2=1534 p5=0]) — the SoundTrigger kind's start arm [M 0x1b2790:16-28].
    assert!(w.scene.sim.as_ref().unwrap().drones.sound_triggers.playing(1534),
        "the ch-8 rise starts SoundTrigger #562's id 1534 [D #562; M 0x1b2790]");
    // The door sweeps off its rest pose [D #544 ch 8; M SS_Update 0x1a9948]. The door's motion
    // can swing (the rest word 0.75 = the 75 % opening [D]), so the whole posed matrix is compared.
    w.wait(500);
    let s = &w.scene.sim.as_ref().unwrap().drones.sss.list[si];
    let moved: f32 = (0..4).map(|r| (0..3).map(|k| (s.matrix[r][k] - rest_full[r][k]).abs()).sum::<f32>()).sum();
    assert!(moved > 0.1, "the secret door sweeps on ch 8 [M SS_Update 0x1a9948], moved {moved:.4} state {} (rest z {})", s.state, rest_full[3][2]);
}

/// The hint plate #455 (0xEA TouchOnce, out ch 0x55, lock ch 0x1c, mask 8 [D triggers dump])
/// fires on the walk-over while the photograph channel is off, and its SoundTrigger #456 (on
/// ch 0x55 → id 1517, global [D #456 p0/p5]) then attempts to start. The fresh boot's mission
/// voice #550 (global [D p5=1]) holds the single GlobalVoiceHandle from the spawn walkway's
/// TouchOnce #551, so the game's own rule kills the hint object at the attempt [M 0x1b2790:19-28].
/// Sister: without the crossing the hint object stays alive in the list.
#[test]
fn the_hint_plate_fires_ch55_and_the_global_rule_kills_456_while_550_holds() {
    let mut w = ready();
    // Plate #455 spans z −142.15..−141.36 over x −4.42..14.64, y 2.37..14.02 [D box]; the
    // display ground below it is the walk real route uses. The walk stand clears the plane by
    // more than the walk capsule's reach.
    let f = floor_under(&w, 5.0, 7.0, -140.2);
    drop_at(&mut w, [5.0, f + 1.0, -140.2]);
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x55), "sister: ch 0x55 off before the walk");
        assert!(sim.drones.sound_triggers.list.iter().any(|t| t.id == 1517), "#456 alive before [D #456]");
    }
    // The real walk-over across the plate plane.
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI; // face −z, the plane
    let mut crossed = false;
    for _ in 0..240 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x55) { crossed = true; break; }
    }
    assert!(crossed, "the walk-over fires TouchOnce #455 → ch 0x55 [D #455; M Trigger_Update case 1]");
    // The game's global-voice rule: #456 attempts to start while #550's mission voice holds the
    // slot (it fired with the spawn walkway's TouchOnce #551) — the object kills itself
    // [M 0x1b2790:19-28; D #456/#550 p5=1].
    w.wait(5);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.sound_triggers.list.iter().any(|t| t.id == 1517),
        "the hint object dies on the busy global slot [M 0x1b2790:19-28; D #456 p5=1]");
    assert!(sim.drones.channels.on(0x43), "the mission voice's ch 0x43 fired at the spawn [D #551]");
    // Sister: a fresh load that never crosses the plate — the hint object stays alive.
    let mut w2 = ready();
    let f3 = floor_under(&w2, 5.0, 7.0, -140.2);
    drop_at(&mut w2, [5.0, f3 + 1.0, -140.2]);
    for _ in 0..120 { w2.tick(nf_game::player::Input::default(), false); }
    let sim2 = w2.scene.sim.as_ref().unwrap();
    assert!(!sim2.drones.channels.on(0x55), "no walk-over → ch 0x55 off (the sister)");
    assert!(sim2.drones.sound_triggers.list.iter().any(|t| t.id == 1517),
        "no attempt → #456 stays alive (the sister)");
}

/// The weapon pickup (0xF0 #557 Pickup_Crossbow, p0=0 p1=17 p2=15 [D]) arms the weapon on the
/// walk-over: `Pickup_Handler` 0x18e650 case 0 equips an un-owned weapon (owned + a full clip
/// and the rounds beyond it carried) [M]. Sister: the same spot with the pickup un-touched —
/// un-owned and silent.
#[test]
fn the_crossbow_pickup_arms_the_weapon_on_the_walk_over() {
    let mut w = ready();
    let (pp, floor) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.pickups.iter()
            .find(|p| (p.m[3][0] - 46.72).abs() < 0.5 && (p.m[3][2] + 156.42).abs() < 0.5)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .expect("the crossbow pickup #557 [D]");
        let fl = floor_under(&w, p[0], p[1], p[2]);
        (p, fl)
    };
    assert!(!w.scene.sim.as_ref().unwrap().drones.inventory.owned[0x11],
        "sister: the crossbow 0x11 un-owned before [D the 0c loadout; M 0x1a7ef8 case 0x700000c]");
    // The deck the pickup rests on; drop south of it and walk the last steps onto it.
    drop_at(&mut w, [pp[0], floor + 1.4, pp[2] + 2.0]);
    w.walk_to(Wp::new(pp[0], floor + 0.2, pp[2]).r(0.7), "onto the crossbow");
    w.wait(20);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.inventory.owned[0x11],
        "the walk-over arms the crossbow 0x11 [M 0x18e650 case 0; D p1=17]");
    let total: i16 = sim.drones.inventory.clip.iter().sum::<i16>()
        + sim.drones.inventory.carried.iter().sum::<i16>();
    assert!(total > 0, "the pickup brought rounds [D p2=15], total {total}");
    assert!(sim.drones.pickups.iter()
        .filter(|p| (p.m[3][0] - pp[0]).abs() < 0.5 && (p.m[3][2] - pp[2]).abs() < 0.5)
        .all(|p| p.gone || p.hidden), "the pickup is consumed by the take [M 0x18e650]");
    // Sister: a fresh load, stand beside it without the walk-over.
    let mut w2 = ready();
    drop_at(&mut w2, [pp[0], floor + 1.4, pp[2] + 2.0]);
    for _ in 0..60 { w2.tick(nf_game::player::Input::default(), false); }
    let sim2 = w2.scene.sim.as_ref().unwrap();
    assert!(!sim2.drones.inventory.owned[0x11], "no walk-over → un-owned (the sister)");
}

/// The four `Pickup 007` kind-6 rows (#546-549 [D]) exist only with the level's gold medal in
/// the `bonus[0]` word: `Pickup_Create` 0x18dda0:93-95 creates a kind-6 only when
/// `PlrStats_HasGoldMedal` 0x199a40 holds for the level's BaseMap — record 11's level hash
/// 0x0700000c [L ee_start_05.bin @0x2a4478]. The take path logs the bonus [M 0x18e650:234-236].
/// Sister: the fresh-boot zero word creates none.
#[test]
fn the_medal_gated_bond_bonuses_create_and_take() {
    // The gold word for the BaseMap [M Menu_SetLevelBonus 0x1ffa10's award arm; the 0b model's
    // gold_bits].
    let bits = {
        let mut p = nf_game::progress::Progress::default();
        p.set_level_bonus(0x0700_000c, 3, 0);
        p.bonus[0]
    };
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "0700000c.bin", false, None, 1, bits)
        .expect("load with the gold word"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.pickups.len(), 15,
            "the gold word creates the 4 kind-6 rows on top of the 11 [D #546-549; M 0x18dda0:93-95]");
        assert_eq!(sim.drones.pickups.iter().filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS).count(), 4,
            "four Pickup 007 bonuses [D]");
    }
    // Walk over the south-yard bonus #547 (85.81, 0.29, −223.60 [D] — the ground-level row;
    // #549's walkway step stops the walker 1.3 m short).
    let (pp, floor) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.pickups.iter()
            .find(|p| p.kind == nf_game::pickup::kind::BOND_BONUS
                && (p.m[3][0] - 85.81).abs() < 0.5 && (p.m[3][2] + 223.60).abs() < 0.5)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .expect("bonus #547 [D]");
        let fl = floor_under(&w, p[0], p[1], p[2]);
        (p, fl)
    };
    drop_at(&mut w, [pp[0], floor + 1.4, pp[2] + 1.8]);
    w.walk_to(Wp::new(pp[0], floor + 0.2, pp[2]).r(0.7), "onto the Pickup 007");
    w.wait(20);
    assert!(w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| (p.m[3][0] - pp[0]).abs() < 0.5 && (p.m[3][2] - pp[2]).abs() < 0.5)
        .all(|p| p.gone || p.hidden), "the bonus is consumed on the walk-over [M 0x18e650:234-236]");
    // Sister: the fresh-boot zero word — no kind-6 at all [M 0x18dda0:93-95].
    let mut bare = ready();
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 rows are not created [M 0x18dda0:93-95]");
}

/// The destroyables' bullet-category mask (tag+0x14 = ~param4 [M 0x12c318:47-52]) decides which
/// gunfire lands: `Collide_FilterBullets` 0x1ece30:26-33 drops a class-5 contact whose weapon
/// word ANDs non-zero against it. The lit barrel #564 carries p4=8 → 0xfff7, which drops the
/// handheld word 0x0008 [D #564; D weapon_data] — sustained pistol fire kills nothing. The yard
/// barrel #492 carries p4=248 → 0xff07, which admits it (0x0008 & 0xff07 = 0 [D #492]) — the
/// same fire kills it. Sister pair in one test, one weapon.
#[test]
fn the_bullet_filter_drops_handheld_fire_on_564_and_admits_it_on_492() {
    let mut w = ready();
    let fired = {
        let g = &w.scene.sim.as_ref().unwrap().gun;
        g.clip + g.spare
    };
    // The lit barrel #564 by the exit (94.43, 6.90, −207.86 [D]).
    let i564 = barrel_of(&w, 564);
    let f = floor_under(&w, 91.8, 8.4, -207.9);
    drop_at(&mut w, [91.8, f + 1.0, -207.9]);
    face_and_fire(&mut w, [94.43, 7.4, -207.86], "the lit barrel #564");
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.props.destroy[i564].dead,
            "the 0xfff7 mask drops the handheld shots [M 0x1ece30:26-33; D #564 p4=8]");
        assert_eq!(sim.drones.props.destroy[i564].mask, 0xfff7, "the mask word [M 0x12c318:47-52; D p4=8]");
    }
    // The same weapon drained — the shots really flew (the fire-path control).
    let spent = fired - {
        let g = &w.scene.sim.as_ref().unwrap().gun;
        g.clip + g.spare
    };
    assert!(spent > 0, "the pistol really fired (spent {spent})");
    // The yard barrel #492 (63.17, 0.38, −182.78 [D], p4=248 → 0xff07): the same fire kills it.
    let mut w2 = ready();
    let i492 = barrel_of(&w2, 492);
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.props.destroy[i492].placement, 492, "the yard barrel's placement [D]");
        assert_eq!(sim.drones.props.destroy[i492].mask, 0xff07, "the admitted mask [D #492 p4=248]");
    }
    let f2 = floor_under(&w2, 60.6, 2.0, -182.8);
    drop_at(&mut w2, [60.6, f2 + 1.0, -182.8]);
    face_and_fire(&mut w2, [63.17, 0.9, -182.78], "the yard barrel #492");
    for _ in 0..60 { w2.tick(nf_game::player::Input::default(), false); }
    assert!(w2.scene.sim.as_ref().unwrap().drones.props.destroy[i492].dead,
        "the 0xff07 mask admits the handheld shots — the barrel dies [M 0x1ece30:26-33; D #492]");
}
