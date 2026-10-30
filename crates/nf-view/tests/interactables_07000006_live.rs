//! interactables-06: the 07000006 player-touchable KINDS the sister files do not already drive,
//! one live test per KIND, driven the way a player drives it (the walk, the trigger pull), each
//! asserting the game's effect with its sister. Model: interactables_0700000b_live.rs.
//!
//! The level's per-kind census and its already-covered rows:
//! - doors: #47 E-opens, #369 (lock ch 200) refuses — interact_07000006_live.rs.
//! - ScriptPlayers: the ambient radio #807 (mode 8 load beat) and the trigger-start #810 —
//!   interact_07000006_live.rs; the mask-2 bullet SP arm on the radio — urn_break_live.rs.
//! - pickups (walk-over take + the parked sister): the BodyArmour #755 —
//!   interact_07000006_live.rs. The kind-6 BOND_BONUS token gate is THIS file's third test.
//! - fusebox #747 (laser-cut groups → ch 57/103/55): fusebox_06_live.rs.
//! - TouchOnce plates + the inverter + the siren (the alarm chain): alarm_panel_06_live.rs;
//!   the route plates walkreal_07000006_live.rs.
//! - searchlights: searchlight_flip/fix_searchlight_alarm lives. LoadPoint #746: walk_07000006.
//! - hint volumes: fix_context_hints_live. The inert Hurt_A #373: interact_07000006_live.
//! - the drawn 0xE2 barrel props (posed pass): simplescript_live.rs — draw only. The behavioural
//!   pin (the knock) is THIS file's second test.
//! - explodeables (0xE1 → `Destroy_Create`): NO 06 live test existed — THIS file's first test
//!   (the 0c twin is interact_0700000c_live's barrel, 20+ units from its sister; the 06 pair
//!   #321/#322 stands 0.82 m apart, which is what exposes the chain).
//!
//! Kinds NOT player-touchable on this level (no test): room/key cels, sky-lights, snow, AI
//! furniture (cover nodes, AI volumes/points), spawners (channel-driven), starts, sound triggers
//! (ride the alarm chain). No 0x29 switch, 0x2E lock, lift, copter or music trigger exists on
//! 07000006 [D census, docs/levels/07000006.md §5].
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes (docs/levels/07000006*.md), [L]
//! the EE snapshot. A teleport only reaches spots whose walk is a documented stand-in transit
//! (neither the east-gate barrel cluster nor the token yard is on walkreal's legs); the operation
//! itself always runs through the real input.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The warm-load channels the real run hands this level: the truck-ride spawn gate 0x1D plus the
/// two interior route gates (the 06 harness's carry [M Player_Start 0x190fd0's gate pick]).
fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn load_06() -> Walker {
    let scene = nf_view::load_with_carry(&iso(), "07000006.bin", false, Some(&carry(&[0x1d, 0x45, 0x70])))
        .expect("load 07000006");
    let mut w = Walker::new(scene);
    w.wait_enabled(3000, "the intro");
    w
}

/// The warm load with the gold-medal `bonus[0]` word `bits` (`0` = the fresh-boot state; the
/// kind-6 gate reads it [M Pickup_Create 0x18dda0:93-95 — `type_ == BOND_BONUS && !has_gold_medal`
/// → no pickup, the port's pickup.rs:174 arm]).
fn load_medal_bits(bits: u64) -> Walker {
    let scene = nf_view::load_with_carry_medals(&iso(), "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70])), 1, bits).expect("load 07000006 warm");
    let mut w = Walker::new(scene);
    w.wait_enabled(3000, "the intro");
    w
}

/// The gold-medal `bonus[0]` word for `base_map` [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm].
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}

/// Teleport Bond onto the floor at (x, z) — the documented stand-in transit — and settle him.
fn drop_at(w: &mut Walker, x: f32, y_guess: f32, z: f32) {
    let floor = w.scene.sim.as_ref().unwrap().world
        .segment([x, y_guess, z], [x, y_guess - 30.0, z]).map(|t| y_guess - 30.0 * t);
    let y = floor.unwrap_or(y_guess) + 0.1;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [x, y, z];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
}

/// Aim at `target` and fire through the real trigger path until `done` reads true (the 0c
/// barrel pattern `face_and_fire`: hip-fire aim walk — turn/look error per tick, the trigger
/// pulses on-target every 16th tick, an empty clip reloads [M Player_Activate's shot block]).
fn fire_until(w: &mut Walker, target: [f32; 3], max_ticks: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    loop {
        if done(w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[fire] MISS: bond {:?} yaw {:.3} pitch {:.3} target {target:?} weapon {} sw {} clip {} spare {}",
                sim.bond.pos, sim.bond.yaw, sim.bond.pitch, sim.gun.weapon, sim.gun.switch_state, sim.gun.clip, sim.gun.spare);
            return false;
        }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let clip_empty = sim.gun.clip == 0 && sim.gun.spare > 0;
            let on_target = err.abs() < 0.03 && perr.abs() < 0.035 && (w.total_ticks - start) % 16 == 0;
            let input = nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                look: perr.clamp(-0.08, 0.08),
                ..Default::default()
            };
            let reload = clip_empty && (w.total_ticks - start) % 20 == 0;
            let fire_input = if on_target { nf_game::player::Input::default() } else { input };
            w.scene.sim.as_mut().unwrap().step_bond(fire_input, on_target && !clip_empty, reload, false);
        }
        w.total_ticks += 1;
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    }
}

/// KIND explodeable (0xE1 → `Destroy_Create` 0x12c318, class 0x21): a player round detonates the
/// oil barrel #321, the death writes its out channel 0x29 and spawns the type-0xa blast script
/// 0x0600052 [M Destroy_CollisionHandler 0x12bd48:30-49 → Destroy_Smash 0x12bb60:40-76], and the
/// blast TAKES THE TWIN #322 0.82 m away — `Explode_Propagate` case 0x21 smashes every reached
/// destroyable whose falloff ≥ 1.0 and whose tag+0x14 mask opens the 0x20 bit [M 0x17c518:168-176;
/// D #321/#322 p4 0xf8 → mask 0xFF07]. Sisters: both barrels whole and ch 0x29 silent before the
/// shot; the fire itself (no hit yet) writes nothing.
#[test]
fn a_player_round_detonates_the_oil_barrel_and_the_blast_takes_the_twin() {
    let mut w = load_06();
    let (i321, i322) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let idx = |pl: usize| sim.drones.props.destroy.iter().position(|d| d.placement == pl)
            .unwrap_or_else(|| panic!("oil barrel #{pl} in the destroy list [D]"));
        (idx(321), idx(322))
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.drones.props.destroy[i321];
        // The disc row [D 07000006 #321: p0 0x29, p1 0, p2 0xa, p3 0x14, p4 0xf8, p5 0x12c; M
        // Destroy_Create 0x12c318:31-48's param map — p0 the OUT channel, p2 the smash type, p3
        // the health pool, p4 the ~bullet mask, p5·0.01 the blast damage].
        assert_eq!((b.channel, b.kind), (0x29, 0xa), "the barrel's out channel and smash type [D p0/p2]");
        assert_eq!(b.mask, 0xff07, "~p4 = 0xFF07 [D p4 0xf8; M 0x12c318:41-47]");
        assert!((b.health - 20.0).abs() < 0.01, "the health pool p3 0x14 [D]");
        assert!((b.blast_damage - 3.0).abs() < 0.01, "the blast damage p5 0x12c·0.01 [D; M 0x12c318:48]");
        assert!(!b.dead, "sister: the barrel stands before the shot");
        assert!(!sim.drones.props.destroy[i322].dead, "sister: the twin #322 stands before the shot");
        assert!(!sim.drones.channels.on(0x29), "sister: ch 0x29 silent before the shot");
    }
    // The stand-in transit: the barrels stand west of the tunnel doors (the walkreal's leg
    // anchors are east of them); the shot is the real input. Stand ~2.9 m north of #321.
    drop_at(&mut w, -10.9, 43.0, -44.6);
    { let sim = w.scene.sim.as_mut().unwrap(); sim.gun.switch_request = 2; }
    w.wait(60);
    let wpos = {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.drones.props.destroy[i321];
        [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0]
    };
    let hit = fire_until(&mut w, wpos, 900, |sim| sim.drones.props.destroy[i321].dead);
    assert!(hit, "no round reached barrel #321 at {wpos:?} — the destroyable never took the contact");
    // The kill tick's effects [M Destroy_CollisionHandler 0x12bd48 → Destroy_Smash 0x12bb60]:
    // ch 0x29 rises, the blast object exists with the type-0xa script 0x0600052, propagated.
    let mut ch_up = false;
    let mut twin_dead = false;
    for _ in 0..8 {
        let sim = w.scene.sim.as_ref().unwrap();
        ch_up |= sim.drones.channels.on(0x29);
        twin_dead |= sim.drones.props.destroy[i322].dead;
        if ch_up && twin_dead { break; }
        w.wait(1);
    }
    assert!(ch_up, "the death writes switch_channels[0x29] [M Destroy_Smash 0x12bb60:40]");
    assert!(twin_dead, "the blast takes the 0.82 m twin #322 [M Explode_Propagate 0x17c518 case 0x21 \
        — falloff ≥ 1.0, mask 0xFF07 opens the 0x20 bit; D #322 p4 0xf8]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.props.destroy[i321].dead, "the shot barrel is dead");
        assert!(sim.drones.blasts.iter().any(|e| e.script_hash == 0x0600_0052 && e.propagated),
            "the type-0xa blast script 0x0600052 played and propagated [M Destroy_Smash 0x12bb60:59-76; \
             Explode_CollisionHandler 0x17cde0's one-shot → Explode_Propagate]");
    }
}

/// KIND SimpleScript barrel-roll trap (0xE2, the west cluster's out ch 14): the rolls carry mask
/// 0x1/0x2 — NO use bit and NO walk-in bit — so the E is dead on them [M SS_Activate 0x1a9ea8's
/// `& 4` gate] and a walk-through moves nothing [M SS_Operate 0x1a9df8:21-40 — class 3 → 0x8,
/// no barrel mask carries it]. The player's real input is the BULLET: a round through the
/// mask-0x2 roll #354 operates it [M 0x1a9df8:35-39], the 0x40 arm latches the sweep [M
/// SS_Update 0x1a9948:85-90], and — mask bit 0 clear — the latch writes out ch 14 [M
/// 0x1a9948:20-23], whose rising value operates the mask-0x1 riders through the channel half [M
/// 0x1a9df8:47-51]. The sweep is one-shot: wrap_flag 1 disarms to state 0 holding the end pose
/// [M 0x1a9948:119-128]. Sisters: ch 14 silent and both playheads at rest before the shot; the
/// E and the walk-in leave the cluster armed.
#[test]
fn a_player_round_knocks_the_barrel_roll_and_the_channel_riders_follow() {
    let mut w = load_06();
    let (gi, ri) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let g = sim.drones.sss.of_placement(354).expect("the mask-0x2 roll #354 [D]");
        let r = sim.drones.sss.of_placement(353).expect("the mask-0x1 rider roll #353 [D]");
        (g, r)
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        // The disc row [D 07000006 #354: 0:0x40 1:0x1 2:0x2 3:0xe 4:0x11b 5:0 6:0x1 7:0; M
        // SS_Create 0x1a9c58's param map — p0 state, p1 wrap, p2 mask, p3 channel, p4 sound].
        assert_eq!((s.state, s.wrap_flag, s.mask, s.channel, s.sound), (0x40, 1, 0x2, 0xe, 0x11b),
            "the roll's disc params [D #354]");
        let r = &sim.drones.sss.list[ri];
        assert_eq!((r.state, r.mask), (0x40, 0x1), "the rider's disc params [D #353: mask 0x1]");
        assert!(!sim.drones.channels.on(0xe), "sister: ch 14 silent before the shot");
        assert_eq!((s.t, r.t), (0.0, 0.0), "sister: both playheads at rest");
    }
    // The stand-in transit: the west cluster sits by the courtyard canopy, off walkreal's legs
    // (the nearest walked anchor is the hedge gap, 17 m east); the E attempt and the shot are
    // the real inputs. Stand beside the ammo #784 spot (9.36, 38.01, 3.86 [D]), ~3.9 m from #354.
    drop_at(&mut w, 7.6, 39.6, 5.2);
    // The E-inert pin: close enough that a use-bit mask would take the pick sphere's reach.
    let ey = {
        let sim = w.scene.sim.as_ref().unwrap();
        let f = sim.world.segment([7.0, 40.0, 3.4], [7.0, 10.0, 3.4]).map(|t| 40.0 - 30.0 * t);
        f.unwrap_or(39.0)
    };
    w.walk_to(Wp::at(7.0, ey + 0.1, 3.4, Act::Use { use_ticks: 240 }), "E at the roll cluster");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        assert_eq!((s.state, s.t), (0x40, 0.0),
            "the E is dead on the roll [M SS_Activate 0x1a9ea8's mask & 4 gate — 0x2 & 4 == 0]");
        assert!(!sim.drones.channels.on(0xe), "the E wrote nothing [M 0x1a9ea8 returns before the write]");
    }
    // The walk-in pin: stand in the cluster for a moment — the masks carry no class-3 bit.
    drop_at(&mut w, 6.6, 39.6, 3.9);
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        assert_eq!((s.state, s.t), (0x40, 0.0),
            "the walk-in moves nothing [M SS_Operate 0x1a9df8:21-40 — no barrel mask carries the class-3 0x8]");
    }
    // The shot: back to the stand, aim at the roll's live matrix spot, fire the real trigger.
    drop_at(&mut w, 7.6, 39.6, 5.2);
    { let sim = w.scene.sim.as_mut().unwrap(); sim.gun.switch_request = 2; }
    w.wait(60);
    let target = w.scene.sim.as_ref().unwrap().drones.sss.list[gi].pos();
    let hit = fire_until(&mut w, target, 2400, |sim| {
        let s = &sim.drones.sss.list[gi];
        s.t > 0.0 || s.state != 0x40
    });
    assert!(hit, "no round reached roll #354 at {target:?} — the mask-0x2 contact never registered");
    // The operate tick: state 1 latched, ch 14 written (mask bit 0 clear), the sound beat queued.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        assert_eq!(s.state, 1, "the shot latches state 1 [M 0x1a9948:85-90]");
        let ids = sim.drones.audio.active_ids();
        eprintln!("[roll] audio after the operate: {ids:?} (the 0x11b=283 beat is the sound layer's view)");
    }
    // The riders: the channel half operates every mask-0x1 roll in the cluster [M 0x1a9df8:47-51].
    let mut riders_moved = false;
    for _ in 0..30 {
        let sim = w.scene.sim.as_ref().unwrap();
        riders_moved = sim.drones.sss.list[ri].t > 0.0 || sim.drones.sss.list[ri].state != 0x40;
        if riders_moved { break; }
        w.wait(1);
    }
    assert!(riders_moved, "the ch-14 write operates the mask-0x1 rider #353 [M 0x1a9df8:47-51]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xe),
        "the latch wrote ch 14 [M 0x1a9948:20-23 — mask 0x2's bit 0 is clear]");
    // The one-shot wrap: the sweep runs out and disarms to state 0 holding the end pose [M
    // 0x1a9948:119-128 — wrap_flag 1 → state 0], and ch 14 STAYS (one-shot; no other placement
    // on the level writes 0xe [D census §5.9]).
    let (keys_n, rate) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        (s.keys.len(), s.rate)
    };
    let last = (keys_n - 1) as f32;
    let step = last / rate as f32;
    let mut max_t = 0.0f32;
    let start = w.total_ticks;
    let wrapped = loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        max_t = max_t.max(s.t);
        if s.t == 0.0 && s.state == 0 { break true; }
        if w.total_ticks - start > rate + 120 { break false; }
        w.tick(nf_game::player::Input::default(), false);
    };
    assert!(wrapped, "roll #354 never wrapped after the shot (t peaked {max_t}, last {last})");
    assert!(max_t >= last - step, "the sweep peaked at t {max_t} but the last key is {last} (step {step})");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[gi];
        assert_eq!(s.state, 0, "wrap_flag 1 disarms the one-shot trap [M 0x1a9948:119-128]");
        let end = nf_game::door::interp(&s.keys, 0, last as u16, last, s.spline).expect("the end pose interpolates");
        assert_eq!(s.matrix, end, "the disarm holds the LAST key's pose (the knocked-over barrel)");
        assert!(sim.drones.channels.on(0xe),
            "ch 14 stays after the wrap — the write is one-shot HIGH, nothing clears it [D census: no ch-0xe writer]");
    }
}

/// KIND pickup, the kind-6 BOND_BONUS arm: the two "Pickup 007" tokens (#750/#751, hash
/// 0x0200092a [D §5.6]) are created ONLY with the level's gold medal recorded — `Pickup_Create`
/// 0x18dda0:93-95 drops the kind-6 row when `PlrStats_HasGoldMedal` reads 0 (the port's
/// pickup.rs:174 arm). 07000006 plays under BaseMap 0x07000005 [L ee_start_05 0x2a4378: the
/// mission row (level 0x07000006, base 0x07000005, order 1)]. Sister: the fresh-boot word 0
/// creates no kind-6 pickup anywhere; the medal load creates them, and the walk-over takes one.
#[test]
fn the_007_tokens_need_the_gold_medal_and_the_walk_over_takes_one() {
    // The fresh-boot arm: no medal → no kind-6 pickup exists at all [M 0x18dda0:93-95].
    let mut bare = load_medal_bits(0);
    {
        let sim = bare.scene.sim.as_ref().unwrap();
        assert!(sim.drones.pickups.iter().all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
            "no gold medal → the kind-6 tokens are not created [M 0x18dda0:93-95]");
        // Sister for the kind itself: ordinary pickups still exist on the same load.
        assert!(sim.drones.pickups.iter().any(|p| !p.gone && !p.hidden),
            "ordinary pickups exist on the bare load (the gate is the kind-6 row, not the list)");
    }
    // The medal arm: the token at (92.05, 34.36, 16.65) [D §5.6 #750] exists, spinning.
    let mut w = load_medal_bits(gold_bits(0x0700_0005));
    let spot = {
        let sim = w.scene.sim.as_ref().unwrap();
        let tok = sim.drones.pickups.iter()
            .find(|p| p.kind == nf_game::pickup::kind::BOND_BONUS
                && (p.m[3][0] - 92.05).abs() < 1.5 && (p.m[3][2] - 16.65).abs() < 1.5)
            .expect("the gold-medal token #750 at the east-gate yard [D]")
            .m[3];
        [tok[0], tok[1], tok[2]]
    };
    // The walk-over take [M Pickup_Handler 0x18e650's take — the same path the armour test pins
    // on this level]: the stand-in transit to the east-gate yard, then the real walk onto it.
    drop_at(&mut w, spot[0] - 1.4, spot[1] + 1.5, spot[2]);
    let taken = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS
            && (p.m[3][0] - spot[0]).abs() < 1.5 && (p.m[3][2] - spot[2]).abs() < 1.5)
        .all(|p| p.gone || p.hidden);
    assert!(!taken(&w), "sister: the token sits before the walk");
    let mut input = nf_game::player::Input { forward: 0.4, ..Default::default() };
    for _ in 0..120 {
        w.tick(input, false);
        if taken(&w) { break; }
    }
    input.forward = 0.0;
    assert!(taken(&w), "the walk-over takes the medal token [M 0x18e650's take path]");
}
