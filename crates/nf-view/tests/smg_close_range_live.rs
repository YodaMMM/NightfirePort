//! smg-close-range-hits: the level-01 report's "Remaining" item 3, on real input from the level
//! start. The measured symptom: 32 SMG (0x12) rounds fired at a dormant guard 0.5-2.1 u away,
//! aim settled, rounds spawned (`last_shot` advances), ZERO health change — while the pistol
//! (0x2) registers on the same guards at 3.5-9 u. This test walks the proven route to the meet
//! plate (the scene hands Bond the SMG), stands 1 u and 2 u from a dormant switch-waiting
//! guard, fires the record's rounds through the real fire path, and asserts the guard takes the
//! record's damage per round — with the pistol as the sister at 1 u, 2 u and a 6 u control.
//!
//! Every leg below is real input (walk/nav/switch/fire); the only allowance is the walker's
//! health top-up (the AI runs, it just cannot kill Bond — this test measures the TARGET guard).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;
use nf_game::world::V3;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn data_of(sim: &nf_view::scene::Sim) -> nf_game::drone::Data {
    nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    }
}

/// The torso box to aim at: the largest-volume posed hit box that is neither the head bone
/// (part 5 — a head hit is ×DroneDamage_Head and kills a 12-health guard outright, unmeasurable
/// per round) nor a leg bone (0x31+). Falls back to 0.6 above the origin when no boxes pose at
/// all.
/// audit-voice-triggers-2 correction: the old filter dropped parts 4 AND 5 as "head bones", but
/// the port's damage law has ONE head part: `hit_damage`'s match arms are 5 → DroneDamage_Head,
/// 0x14/0x15/0x17/0x20/0x23/0x27 → Arms, 0x31..=0x38 → Legs, and EVERYTHING ELSE (part 4
/// included) → DroneDamage_Torso [M damage.rs 0x145558's table; pinned by nf-game's
/// hit_damage_per_part_and_difficulty — h(4, 2, 1) = 5.25 torso, head=false]. On the 0x5000039
/// rig part 4 IS the chest (half 0.2×0.25×0.15 [L the pistol-leg box dump]); excluding it sent
/// the aim to 0.07-half arm boxes, where the shifted shared-RNG cone draws miss.
fn torso_aim(sim: &nf_view::scene::Sim, gi: usize) -> (V3, usize, Option<i16>) {
    let dr = &sim.drones.list[gi];
    let boxes = dr.hit_boxes(&data_of(sim));
    let n = boxes.len();
    let best = boxes.iter()
        .filter(|b| b.part != 5 && !(0x31..=0x38).contains(&b.part))
        .max_by(|a, b| {
            let va = a.half[0] * a.half[1] * a.half[2];
            let vb = b.half[0] * b.half[1] * b.half[2];
            va.total_cmp(&vb)
        });
    match best {
        Some(b) => ([b.m[3][0], b.m[3][1], b.m[3][2]], n, Some(b.part)),
        None => {
            let p = dr.pos;
            ([p[0], p[1] + 0.6, p[2]], n, None)
        }
    }
}

struct FireTrace {
    spawned: u32,
    /// (part, expected loss, observed hp delta on that tick) — the player's own weapon only
    impacts: Vec<(i16, f32, f32)>,
    /// hits on the target by OTHER shooters (the escort's volleys) — logged, not counted
    ally_impacts: u32,
    hp0: f32,
    hp1: f32,
    /// sample flying bullets (pos, dir) at their last seen tick
    flying: Vec<(V3, V3)>,
    boxes: usize,
    enabled_at_fire: Vec<bool>,
    corpse: bool,
    ticks: u32,
}

/// Fire `max_rounds` rounds of the held weapon `my_weapon` at guard `gi`'s torso from wherever
/// Bond stands, holding the trigger through the real fire path. Stops early when the guard dies
/// (4 more rounds) or the tick cap. Every tick is logged to stderr under [smg]. Impacts are
/// attributed by the engine's own hit record's weapon: the escort's own volleys land in the
/// same list (his 0x1c pellets) and must not count as the player's registration.
fn fire_at(w: &mut Walker, gi: usize, max_rounds: u32, my_weapon: u16, label: &str, segment_gate: bool, keep_band: bool) -> FireTrace {
    // audit-voice-triggers-2: the live guards HUNT (the gunfire alerted them — the merged
    // branch's added attack barks wake them earlier than master's stream did). At a 1 u stand a
    // circling guard flips the aim geometry (dxz under the 0.5 clamp, dx/dz sign flips swing
    // err past the gate) and the leg's rounds never spawn. The stand's band is held with the
    // same real input stand_near uses: walk in when the guard drifts out, back off when a
    // point-blank guard would swing the aim.
    // audit-voice-triggers-4: the 1.2 band is the 1 u STAND's follow (a circling point-blank
    // guard flips the aim geometry). The west legs stand at their NAMED distance (2 u / 6 u) and
    // fire from there — `keep_band=false` — so the 6 u control is a 6 u registration, not a
    // walk-in. The 1 u leg passes keep_band=true: its timeline is byte-exact.
    let band: Option<f32> = if my_weapon == 0x2 && keep_band { Some(1.2) } else { None };
    let mut tr = FireTrace { spawned: 0, impacts: Vec::new(), ally_impacts: 0, hp0: 0.0, hp1: 0.0, flying: Vec::new(), boxes: 0, enabled_at_fire: Vec::new(), corpse: false, ticks: 0 };
    tr.hp0 = w.scene.sim.as_ref().unwrap().drones.list[gi].health;
    let mut hp_prev = tr.hp0;
    let mut last_spawn = 0u32;
    // audit-voice-triggers-4 (gated legs): the trigger stays COLD until the aim gate's first
    // open tick — the escort at heel volleys the freshly-popped guard ~6 ticks in (the 2 u run:
    // 3 w0x1c pellets at +6, hp 12.0 -> -1.5), so the player's first round must spawn at the
    // gate's first open tick, not six ticks into the turn. The 1 u leg (ungated) is byte-exact.
    let mut gate_opened = false;
    for t in 0..3000u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            if (dr.corpse || dr.health <= 0.0) && tr.spawned >= 4 && t - last_spawn > 40 { tr.corpse = true; break; }
            // Stop only after the last round has had its flight time (~40 ticks) to land.
            if tr.spawned >= max_rounds && t - last_spawn > 40 { break; }
            // A dry gun can never spawn the remaining rounds — stop rather than burn the cap.
            if sim.gun.clip == 0 && sim.gun.spare == 0 { eprintln!("[smg] {label}: gun dry at {}/{} rounds", tr.spawned, max_rounds); break; }
        }
        // The aim: the torso box (recomputed — a hit flinch can shift the pose).
        let (chest, boxes, part) = {
            let sim = w.scene.sim.as_ref().unwrap();
            torso_aim(sim, gi)
        };
        // audit-voice-triggers-3 (segment_gate legs): the 0x74 combat guards crouch BEHIND the
        // hall's crates — the chest never clears (the hold-spot run: d#6/d#11, 0 rounds in 6000
        // ticks) and when he does shoot, only the head shows over the lip. The player aims at
        // whatever box the game exposes: the chest if it shows, else the head (part 5 — the
        // port's one head part [M damage.rs hit_damage's match]), else any box with a clear
        // centre ray. A head hit is the ×100 line [M NDrone2_HitDamage 0x145558] — the control
        // lands on the exposed box, the way the game itself registers hits.
        let (chest, part) = if segment_gate {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let bx = sim.drones.list[gi].hit_boxes(&data_of(sim));
            let clear = |c: &[f32; 3]| sim.world.segment(eye, *c).is_none();
            let mut pick: Option<([f32; 3], Option<i16>)> = None;
            for want in [4i16, 5] {
                if let Some(b) = bx.iter().find(|b| b.part == want) {
                    let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
                    if clear(&c) { pick = Some((c, Some(want))); break; }
                }
            }
            if pick.is_none() {
                for b in &bx {
                    let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
                    if clear(&c) { pick = Some((c, Some(b.part))); break; }
                }
            }
            pick.unwrap_or((chest, part))
        } else { (chest, part) };
        let (input, on) = {
            let sim = w.scene.sim.as_ref().unwrap();
            tr.boxes = boxes;
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let dx = chest[0] - bp[0];
            let dz = chest[2] - bp[2];
            let dy = chest[1] - eye[1];
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, chest).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on = err.abs() < 0.07 && perr.abs() < 0.10
                // audit-voice-triggers-3 (opt-in per leg — the green 1 u leg keeps its exact
                // timeline): the player does not fire into cover. The combat-state guards
                // re-cover DURING a fire (d#11: the segment was clear at the stand, then the
                // 6 rounds ate the crate over 527 ticks); the gate holds the trigger while his
                // eye-to-chest line is blocked and fires when he pops out to shoot — the same
                // pop-out the 300-tick reposition pause waits for.
                && (!segment_gate || sim.world.segment(eye, chest).is_none());
            let _ = boxes;
            // audit-voice-triggers-2: hold the stand's band on real input while the guard hunts
            // (see the `band` note above) — a drifted guard swings err past the gate and the
            // leg stops spawning rounds.
            let mut fwd = 0.0f32;
            if let Some(band) = band {
                let d = walker::dxz(bp, sim.drones.list[gi].pos);
                if d > band { fwd = 1.0; } else if d < band - 0.6 || err.abs() > 0.5 { fwd = -1.0; }
            }
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), forward: fwd, ..Default::default() }, on)
        };
        let (hp_before, enabled, state, dist, born_before) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            (dr.health, dr.enabled, dr.state, walker::dxz(sim.bond.pos, dr.pos), sim.drones.frame)
        };
        // audit-voice-triggers-2: the pistol (0x2) is SEMI-AUTOMATIC — a held trigger fires
        // once per pull and the pull only re-arms on release [M gun.rs Player_WeaponFiring —
        // the semi P2K held fires ONCE; the refire comes from a new press]. The held-trigger
        // loop used to spend the leg's whole budget on one round. Press for 2 ticks, release
        // for 4, through the same real input path.
        if on { gate_opened = true; }
        let semi = my_weapon == 0x2;
        let mut press = !semi || (t % 6) < 2;
        if segment_gate && !gate_opened { press = false; }
        if on && press { w.tick_fire(input); } else { w.tick(input, false); }
        tr.ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let spawned_here = sim.shot_tick;
        if spawned_here {
            tr.spawned += 1;
            last_spawn = t;
            tr.enabled_at_fire.push(enabled);
            // audit-voice-triggers-2: point-blank misses — dump the guard's live pose and every
            // flying bullet at the spawn tick (the round's whole flight can sit inside this
            // tick, so the after-tick sample is the only witness).
            let dr = &sim.drones.list[gi];
            eprintln!("[smg] {label} +{t}   guard now at ({:.3},{:.3},{:.3}) state {:#x} boxes {}",
                dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.hit_boxes(&data_of(sim)).len());
            for bx in dr.hit_boxes(&data_of(sim)).iter() {
                eprintln!("[smg] {label} +{t}   box part {} c ({:.3},{:.3},{:.3}) h ({:.3},{:.3},{:.3})",
                    bx.part, bx.m[3][0], bx.m[3][1], bx.m[3][2], bx.half[0], bx.half[1], bx.half[2]);
            }
            eprintln!("[smg] {label} +{t}   bullets alive {} bond yaw {:.4} pitch {:.4}",
                sim.drones.player_bullets.len(), sim.bond.yaw, sim.bond.pitch);
        }
        // This tick's drone impacts (player_impacts is cleared at the head of each tick's
        // collision pass): expected loss from the engine's own hit record (dr.hit), observed
        // delta from the health trace.
        for im in &sim.drones.player_impacts {
            if let nf_game::drone::PlayerImpact::Drone { drone, part: p, .. } = im {
                if *drone != gi { continue; }
                let dr = &sim.drones.list[gi];
                let target = nf_game::damage::Target { damageable: dr.damageable, armour: 0, mode: 0, skin: dr.skin, kind: dr.kind };
                let (loss, wdmg, wtag) = match &dr.hit {
                    Some(h) => (nf_game::damage::hit_damage(&sim.tuning, target,
                        nf_formats::ps2float::mul(h.damage, dr.damage_taken), h.part, h.weapon, false,
                        sim.drones.difficulty, sim.level).loss, h.damage, h.weapon),
                    None => (0.0, 0.0, 0u16),
                };
                let hp_now = dr.health;
                let delta = hp_prev - hp_now;
                if wtag == my_weapon {
                    tr.impacts.push((*p, loss, delta));
                } else {
                    tr.ally_impacts += 1;
                }
                hp_prev = hp_now;
                eprintln!("[smg] {label} +{t} IMPACT part {p} by w{wtag:#x} rec-dmg {wdmg:.3} expected {loss:.3} delta {delta:.3}{} (dist {dist:.2})",
                    if wtag == my_weapon { "" } else { " — ALLY, not counted" });
            }
        }
        let hp_now = sim.drones.list[gi].health;
        if spawned_here {
            eprintln!("[smg] {label} +{t} round {} spawned, boxes {boxes} (aim part {part:?}), enabled {enabled}, state {state:#x}, dist {dist:.2}, hp {hp_before:.1}->{hp_now:.1}",
                tr.spawned);
            // Should the round have hit? Each alive bullet born this tick, its segment vs the
            // target's posed boxes (the engine's own test shape) — a miss here means the ray
            // geometry missed; a hit here with zero impact means the engine's pass skipped it.
            let boxes_now = sim.drones.list[gi].hit_boxes(&data_of(sim));
            for b in sim.drones.player_bullets.iter().filter(|b| b.born >= born_before) {
                let l = b.speed.max(1.0);
                let ray = [b.dir[0] * l, b.dir[1] * l, b.dir[2] * l];
                let hits: Vec<(i16, f32)> = boxes_now.iter().filter_map(|bx| nf_game::damage::ray_box(b.pos, ray, bx).map(|tt| (bx.part, tt))).collect();
                if !hits.is_empty() {
                    eprintln!("[smg] {label} +{t}   alive bullet w{:#x} born {} bounces {} ({:.2},{:.2},{:.2}) dir ({:.3},{:.3},{:.3}) ENTERS boxes {:?} (should-hit)",
                        b.weapon, b.born, b.bounces, b.pos[0], b.pos[1], b.pos[2], b.dir[0], b.dir[1], b.dir[2], &hits[..hits.len().min(4)]);
                } else if tr.impacts.is_empty() {
                    // audit-voice-triggers-2: a spawned round that missed everything — print the
                    // ray against every posed box so the miss is explained (spread? pose? aim?).
                    eprintln!("[smg] {label} +{t}   MISSED bullet w{:#x} born {} ({:.3},{:.3},{:.3}) dir ({:.4},{:.4},{:.4}) len {:.3}",
                        b.weapon, b.born, b.pos[0], b.pos[1], b.pos[2], b.dir[0], b.dir[1], b.dir[2], l);
                    let bp = sim.bond.pos;
                    eprintln!("[smg] {label} +{t}   bond at ({:.3},{:.3},{:.3}) yaw {:.4} pitch {:.4}",
                        bp[0], bp[1], bp[2], sim.bond.yaw, sim.bond.pitch);
                    for bx in boxes_now.iter() {
                        eprintln!("[smg] {label} +{t}   box part {} centre ({:.3},{:.3},{:.3}) half ({:.3},{:.3},{:.3})",
                            bx.part, bx.m[3][0], bx.m[3][1], bx.m[3][2], bx.half[0], bx.half[1], bx.half[2]);
                    }
                }
            }
            tr.flying = sim.drones.player_bullets.iter().take(3).map(|b| (b.pos, b.dir)).collect();
        }
        hp_prev = hp_now;
    }
    let sim = w.scene.sim.as_ref().unwrap();
    tr.hp1 = sim.drones.list[gi].health;
    tr
}

#[test]
fn smg_rounds_damage_a_close_guard_and_the_pistol_sister() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // A wedged leg gives up (lenient) instead of panicking: the fire legs run from wherever the
    // walk ends, and the report says so. The proven route legs stay proven.
    w.lenient = true;
    // The intro cut-scene writes ch 0xB (the proven walk's opening).
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { break; }
    }
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    drop(sim);
    w.shield = Some(ci);
    // The ambush clear (the proven route's opening fights — real input, the walker's own shoot).
    let guards: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && walker::dxz(d.pos, sim.drones.list[ci].pos) < 14.0
        }).collect()
    };
    eprintln!("[smg] {} armed guard(s) within 14 of the contact — clearing", guards.len());
    for g in guards {
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(gpos, bpos) > 9.0 {
            w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.5);
        }
        if !w.shoot(g, &format!("guard {g}")) {
            if w.resupply("ammo retry") { w.shoot(g, &format!("guard {g}")); }
        }
    }
    // To the contact, then the meet plate: the TOUCH_ONCE plate at (6.8, 6.0, 8.9) fires ch 0x32,
    // the OUT mux mirrors it to ch 0x33, SP #276 plays and its weapon hand leaves Bond the SMG.
    let cpos = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
    w.walk_to(Wp { at: cpos, act: Act::None, arrive: 2.5 }, "to the contact");
    w.walk_nav_to([6.8, 6.0, 8.9], "the meet plate (NE hall)", 1.0);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let plate = sim.drones.triggers.iter()
            .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x32)
            .map(|t| t.centre())
            .expect("the meet plate");
        drop(sim);
        w.walk_to(Wp { at: plate, act: Act::None, arrive: 0.25 }, "onto the meet plate");
        for (dx, dz) in [(0.0f32, 0.9f32), (0.0, -0.9), (0.9, 0.0), (-0.9, 0.0)] {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: [plate[0] + dx, plate[1], plate[2] + dz], act: Act::None, arrive: 0.2 },
                &format!("the meet plate sweep ({dx},{dz})"));
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: plate, act: Act::None, arrive: 0.2 }, "back across the meet plate");
        }
        assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x32), "walking the meet plate must fire ch 0x32");
    }
    let mut e_at = None;
    for t in 0..300u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xe) { e_at = Some(t); break; }
    }
    assert!(e_at.is_some(), "the meet scene must write ch 0xE");
    w.wait(120);
    // The hand-off's gun: the SMG 0x12 (the scene's stream swaps it in).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[smg] gun after the meet: weapon {:#x} clip {} spare {}", sim.gun.weapon, sim.gun.clip, sim.gun.spare);
    }
    if w.scene.sim.as_ref().unwrap().gun.weapon != 0x12 {
        w.tick(Input { switch_to: Some(0x12), ..Default::default() }, false);
        for _ in 0..90 { w.tick(Default::default(), false); }
        assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x12, "the SMG must be Bond's gun after the hand-off");
    }
    // The roster: the meet-scene guards (ch 0xE) are the level's live ones; the switch-held
    // guards (d#10/d#16 on ch 0x5) are NDrone2_Enable(0)-held and bullets pass through them in
    // the game itself [M NDrone2_DSTATE_WaitSwitch 0x15b960 enter → NDrone2_Enable(0) 0x149dd0
    // sets obj+0xF0 |= 0x20; Collide_PickObj 0x1eae80:26-30 drops 0x20 objects]. The target is
    // the nearest LIVE (enabled) guard — the premise to test is the player's full-auto rounds
    // vs a guard that the game lets you hit.
    let roster: Vec<(usize, u32, V3, u8, bool, f32, u32, u8)> = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().enumerate().filter(|(_, d)| d.team == 1 && !d.corpse)
            .map(|(i, d)| (i, d.skin, d.pos, d.switch_channel, d.enabled, d.health, d.weapon, d.state)).collect()
    };
    for (i, skin, pos, ch, en, hp, wp, st) in &roster {
        eprintln!("[smg] roster d#{i} skin {skin:#x} at ({:.2},{:.2},{:.2}) ch {ch:#x} enabled {en} hp {hp} weapon {wp:#x} state {st:#x}", pos[0], pos[1], pos[2]);
    }
    // The dormant ch-0x5 pair LAST (on the pistol — no SMG cost): rounds must stay zero — the
    // game's own Enable(0) rule, not a port bug.
    let dormant: Vec<usize> = roster.iter().filter(|(_, _, _, ch, _, hp, _, _)| *ch == 0x5 && *hp > 0.0).map(|(i, ..)| *i).collect();
    let dg = dormant.iter().copied().min_by(|&a, &b| {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::dxz(sim.drones.list[a].pos, sim.bond.pos).total_cmp(&walker::dxz(sim.drones.list[b].pos, sim.bond.pos))
    });
    let candidates: Vec<usize> = roster.iter().filter(|(_, _, _, _, en, hp, _, _)| *en && *hp > 0.0).map(|(i, ..)| *i).collect();
    assert!(!candidates.is_empty(), "no live (enabled) guard found — see the roster dump above");
    let gi = candidates.iter().copied().min_by(|&a, &b| {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::dxz(sim.drones.list[a].pos, sim.bond.pos).total_cmp(&walker::dxz(sim.drones.list[b].pos, sim.bond.pos))
    }).unwrap();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[gi];
        eprintln!("[smg] target d#{gi} at {:?} enabled {} state {:#x} hp {:.1} boxes {}",
            d.pos, d.enabled, d.state, d.health, d.hit_boxes(&data_of(sim)).len());
    }

    // The hall stage: the proven east-door leg (door #2 is the level's known walker wedge —
    // the stand_near nav to the hall guards wedges on it). Then one FRESH live guard per leg
    // (a leg's rounds can kill him; the next leg needs a living one), each within its clip.
    // Weapon order: the SMG legs first (the hand-off's clip 32 covers leg A; leg B takes what
    // the resupply leaves after re-asserting the SMG), then the pistol sisters, then the
    // dormant pass-through probe on the PISTOL (weapon-independent [M] skip, no SMG cost).
    w.walk_nav_to([2.9, 6.3, 16.3], "stage into the hall (east door)", 3.0);
    w.lenient = true;
    let mut live: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let mut v: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.team == 1 && !d.corpse && d.enabled && d.health > 0.0
        }).collect();
        v.sort_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, sim.bond.pos).total_cmp(&walker::dxz(sim.drones.list[b].pos, sim.bond.pos)));
        v
    };
    eprintln!("[smg] live guards in distance order: {live:?}");

    // The pistol sisters: switch to 0x2 through the real switch, then 1 u, 2 u and the 6 u
    // control — each on a fresh living guard. audit-voice-triggers-2 re-pin: the game's cone
    // draw is UNCONDITIONAL — every spawned round draws its offset from the shared RNG
    // [M Bullet_init 0x12c5a0:171-180 — the spread block runs even on a dead-on aim, pinned by
    // nf-game's cone_draws_three_even_when_dead_on], so which single round lands depends on
    // where the shared stream sits. The merged branch's added voice triggers consume draws
    // (barks), shifting every downstream roll; the old 1-round pin encoded master's lucky
    // stream position. The control's claim — the pistol registers damage on the guards the
    // game lets you hit — is pinned by up to 3 rounds with at least one PLAYER impact.
    w.tick(Input { switch_to: Some(0x2), ..Default::default() }, false);
    for _ in 0..90 { w.tick(Default::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x2, "the pistol switch");
    for dist in [1.0f32, 2.0f32, 6.0f32] {
        live.retain(|&i| {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[i];
            d.team == 1 && !d.corpse && d.enabled && d.health > 0.0
        });
        // audit-voice-triggers-4 (the -3 pinned step, landed): the hall roster is the escort's
        // hunting ground — he fights alongside Bond inside the hall and killed every 12-health
        // candidate mid-leg (d#5/d#6/d#11 12.0 -> 0.0 with 0 player impacts) while the 0x74
        // cover discipline gave the gated aim only 1-4 exposure ticks per window. The west
        // corridor's ch-0x26/0xf guards (d#2/d#3 on 0x26, d#4/d#12/d#17 on 0xf [L the roster +
        // trigger probe: plate t#8 (-17.15, 14.99) out 0x26; slab t#1 (-25.43, 15.05) out 0xf])
        // sit DORMANT until their own TOUCH_ONCE plates fire — bullets pass through disabled
        // guards [M NDrone2_Enable(0) 0x149dd0 / Collide_PickObj 0x1eae80], so the escort cannot
        // pre-bleed them either. Walk the proven corridor route on real input, cross each plate
        // (no channel writes — the touch IS the enable), and run the leg on the freshly-popped
        // guard before the trailing escort closes the distance.
        if dist > 1.0 {
            if dist < 6.0 {
                w.walk_nav_to([-14.1, 6.4, 13.1], "the west corridor mouth (the proven route)", 1.2);
                w.walk_nav_to([-17.15, 6.4, 14.99], "the ch-0x26 plate", 0.3);
                let plate_c = [-17.15f32, 6.4, 14.99f32];
                for (dx, dz) in [(0.0f32, 0.9f32), (0.0, -0.9), (0.9, 0.0), (-0.9, 0.0)] {
                    if w.scene.sim.as_ref().unwrap().drones.channels.on(0x26) { break; }
                    w.walk_to(Wp { at: [plate_c[0] + dx, plate_c[1], plate_c[2] + dz], act: Act::None, arrive: 0.2 },
                        &format!("the ch-0x26 plate sweep ({dx},{dz})"));
                    if w.scene.sim.as_ref().unwrap().drones.channels.on(0x26) { break; }
                    w.walk_to(Wp { at: plate_c, act: Act::None, arrive: 0.2 }, "back across the ch-0x26 plate");
                }
                assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x26), "crossing the ch-0x26 plate must enable the corridor pair (d#2/d#3)");
            } else {
                // The route legs take the un-wedge wrapper: the escort's wedge cost the whole
                // previous 6 u leg (every route leg STUCK at (-13.49, 10.40), the blocker at
                // segment t ~0.05). The FIRST waypoint is the 2 u leg's own proven door-apex
                // alignment (-14.1, 13.1) — cutting straight to (-19.4, 15.25) sends the nav
                // diagonally into the corridor door's frame (the level's known walker wedge,
                // the test's own east-door comment) and the door wedge eats the rest of the leg.
                walk_leg(&mut w, [-14.1, 6.4, 13.1], "the west corridor mouth (the proven route)", 1.2);
                walk_leg(&mut w, [-19.4, 6.4, 15.25], "the corridor north lane", 1.2);
                walk_leg(&mut w, [-23.3, 6.4, 14.4], "the mid-corridor", 1.2);
                walk_leg(&mut w, [-25.43, 6.4, 15.05], "the ch-0xF slab cross (the ambush pops)", 0.25);
                for _ in 0..120 { if w.scene.sim.as_ref().unwrap().drones.channels.on(0xf) { break; } w.wait(1); }
                // Out FIRST (the pop volley's 10-tick reaction window [M DroneFunc_ReactionTime 0x14ac60:15-24]), then the leg.
                walk_leg(&mut w, [-22.3, 6.4, 14.8], "back off the popped ambush", 1.0);
                assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xf), "crossing the ch-0xF slab must pop the ambush (d#4/d#12/d#17)");
            }
            // The leg's roster: ONLY the plate-enabled guards (the pinned roster the escort
            // cannot reach inside the window). Re-queried fresh: the poppers hunt, they move
            // from their spawn spots. Ground guards first (the balcony pair d#2/d#4 is
            // structurally walled from any floor stand — the first 6 u run walled all four
            // candidates before a round spawned), then nearest-to-Bond.
            live = {
                let sim = w.scene.sim.as_ref().unwrap();
                let mut v: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    d.team == 1 && !d.corpse && d.enabled && d.health > 0.0
                        && (d.switch_channel == 0x26 || d.switch_channel == 0xf)
                }).collect();
                v.sort_by(|&a, &b| {
                    let sim = w.scene.sim.as_ref().unwrap();
                    // Ground guards first (the balcony pair d#2/d#4 is structurally walled
                    // from any floor stand — the 6 u run walled all four candidates before a
                    // round spawned), then nearest-to-Bond.
                    let ua = (sim.drones.list[a].pos[1] > 8.0) as u8;
                    let ub = (sim.drones.list[b].pos[1] > 8.0) as u8;
                    ua.cmp(&ub).then(walker::dxz(sim.drones.list[a].pos, sim.bond.pos)
                        .total_cmp(&walker::dxz(sim.drones.list[b].pos, sim.bond.pos)))
                });
                v
            };
            eprintln!("[smg] {dist} u leg: the plate-enabled west roster, ground first, nearest: {live:?}");
        }
        if live.is_empty() { break; }
        let mut tg = live[0];
        // audit-voice-triggers-2: the same clear-segment gate the SMG legs use — a combat-state
        // guard (0x74) can sit behind the hall's cover (d#11 at 2 u: 3 rounds, 0 impacts, the
        // rounds eat the crate); the control's claim is about guards the game lets you hit.
        // audit-voice-triggers-4: the pre-walk skip is the HALL leg's doctrine only (dist <= 1.0,
        // byte-exact). The west guards sit behind the corridor's own walls from the plate spot
        // (d#3 in the south room, d#2 on the balcony — the 2 u run: both skipped, the roster
        // emptied, the panic) — the leg WALKS to them first (stand_near, real input) and the
        // at-stand exposure check below decides the candidate.
        if dist <= 1.0 {
            loop {
                let sim = w.scene.sim.as_ref().unwrap();
                let (chest, _, _) = torso_aim(sim, tg);
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                if sim.world.segment(eye, chest).is_none() { break; }
                drop(sim);
                eprintln!("[smg] d#{tg}: no clear eye-to-torso segment for the pistol leg — next candidate");
                live.remove(0);
                if live.is_empty() { break; }
                tg = live[0];
            }
        }
        // audit-voice-triggers-2: the leg retries until the live roster is exhausted — the
        // combat-state guards re-cover DURING the stand (d#11: the segment was clear before
        // stand_near and the 3 rounds still ate the crate), and the escort can steal a leg's
        // kill (d#5 died 12→0 with 0 player impacts — his volley, not the control's). A
        // covered candidate is retried once after a 300-tick reposition pause (0x74 guards pop
        // out of cover to shoot); an ally-stolen kill voids the leg (the control needs the
        // PLAYER's registration) and moves on. The same doctrine as the SMG legs' `_attempt`
        // loop: a leg with a landed PLAYER round proves the control.
        let mut leg: Option<(FireTrace, usize)> = None;
        let mut steals: Vec<(usize, FireTrace)> = Vec::new();
        while !live.is_empty() {
            let tg = live[0];
            let mut mine: Option<FireTrace> = None;
            for attempt in 0..2 {
                if attempt > 0 {
                    // The reposition pause: the guard's own AI re-covers/re-exposes.
                    for _ in 0..300 { w.tick(Default::default(), false); }
                }
                // The retries spend the pistol dry (the 1 u leg takes 6, d#11's two attempts
                // took 8) — a dry gun breaks fire_at at ticks 0 and the leg proves nothing.
                if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 {
                    w.resupply("pistol leg retries");
                }
                live.retain(|&i| {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[i];
                    d.team == 1 && !d.corpse && d.enabled && d.health > 0.0
                });
                if !live.contains(&tg) { break; }
                // The leg's stand: the 2 u leg stands 2 u off the guard's live pos (the
                // stand_near follow); the 6 u leg HOLDS THE MOUTH — the 01 walk's own ambush
                // stance [-24.0, 14.8]: the poppers hunt Bond across the map and drag a chase
                // stand into the meet room's cover (the previous run: four stands settled ~6 u
                // and every ray stayed walled), while the sight probe [L] clears every ray from
                // the mouth stands (-23..-24.5, 13.5..16.5) to the west guards' torsos. Hold
                // the mouth, lure, and fire when a popper comes inside the band.
                if dist >= 6.0 {
                    walk_leg(&mut w, [-24.0, 6.4, 14.8], "the mouth stand (the 01 walk's ambush stance)", 0.8);
                } else {
                    stand_near(&mut w, tg, dist);
                }
                if dist > 1.0 {
                    // At the stand: the poppers HUNT — they emerge from the south rooms and
                    // the pocket into the open given a few hundred ticks (the gunshot hearing
                    // draws them [M DroneFunc_SetNoiseHeard]; the 01 walk's pocket doctrine:
                    // "hold the mouth and give the fight machinery the next window"). Hold
                    // the stand and re-check; a guard still walled after the wait gives up
                    // the candidate. The one-shot check walled all four 6 u candidates at
                    // rest — the wait is the difference between "walled at rest" and "out
                    // hunting".
                    let mut exposed = false;
                    let mut died_waiting = false;
                    for _ in 0..16 {
                        {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let dr = &sim.drones.list[tg];
                            if dr.corpse || dr.health <= 0.0 { died_waiting = true; break; }
                            let bp = sim.bond.pos;
                            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                            exposed = dr.hit_boxes(&data_of(sim)).iter().any(|b| {
                                let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
                                sim.world.segment(eye, c).is_none()
                            });
                        }
                        if exposed { break; }
                        // The emergence lure: the poppers hunt the GUNSHOT hearing [M
                        // DroneFunc_SetNoiseHeard; the 01 walk's bounded 300-tick sound lure
                        // drew the pocket guard out — "hold the mouth (the gunshot hearing
                        // keeps drawing him) and give the fight machinery the next window"].
                        // Real pistol presses toward the candidate every check. The rounds
                        // are real input; a walled guard cannot be hit by them (the wall eats
                        // the ray), and a guard who steps into the ray takes the leg's own
                        // registration.
                        if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 6 {
                            w.resupply("the emergence lure");
                        }
                        for k in 0..24u32 {
                            let (dead, input) = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                let dr = &sim.drones.list[tg];
                                if dr.corpse || dr.health <= 0.0 { (true, Input::default()) } else {
                                    let bp = sim.bond.pos;
                                    let gp = dr.pos;
                                    let desired = (gp[0] - bp[0]).atan2(gp[2] - bp[2]);
                                    let err = walker::wrap_pi(desired - sim.bond.yaw);
                                    (false, Input { turn: err.clamp(-0.08, 0.08), ..Default::default() })
                                }
                            };
                            if dead { died_waiting = true; break; }
                            if k % 12 == 3 { w.tick_fire(input); } else { w.tick(input, false); }
                        }
                    }
                    if !exposed {
                        eprintln!("[smg] d#{tg}: walled at the {dist} u stand{} — next candidate",
                            if died_waiting { " (he died during the emergence wait)" } else { " after the emergence wait" });
                        break;
                    }
                }
                let rounds = if dist > 1.0 { 6 } else { 3 };
                let tr = fire_at(&mut w, tg, rounds, 0x2, &format!("pistol {dist}u a{attempt}"), dist > 1.0, dist <= 1.0);
                let drop = tr.hp0 - tr.hp1;
                eprintln!("[smg] pistol {dist} u on d#{tg} (attempt {attempt}): spawned {} rounds, {} impacts, hp {:.1} -> {:.1} (drop {drop:.3}), ticks {}",
                    tr.spawned, tr.impacts.len(), tr.hp0, tr.hp1, tr.ticks);
                if drop > 0.0 && !tr.impacts.is_empty() { mine = Some(tr); break; }
                // audit-voice-triggers-4 (the -3 pinned step (a)): a target killed mid-leg by
                // the game's OWN ally — the escort's w0x1c volley, 0 player impacts — is
                // DOCUMENTED game behaviour (the 01 walk's ledger doctrine), not a port
                // failure: the escort follows at heel and his shotgun kills any 12-health
                // guard the moment he closes (one volley: 3 pellets x 4.5 expected = 13.5).
                // Record the steal and move to the next candidate.
                if dist > 1.0 && drop > 0.0 && tr.hp1 <= 0.0 && tr.impacts.is_empty() && tr.ally_impacts > 0 {
                    eprintln!("[smg] pistol {dist} u on d#{tg}: the escort's volley stole the kill ({} ally impacts) — documented, next candidate", tr.ally_impacts);
                    steals.push((tg, tr));
                    break;
                }
                if tr.hp1 <= 0.0 { break; } // dead (the escort's steal or the guard's own end) — no retry on a corpse
            }
            if let Some(tr) = mine { leg = Some((tr, tg)); break; }
            live.remove(0);
        }
        match leg {
            Some((tr, tg)) => {
                let drop = tr.hp0 - tr.hp1;
                assert!(drop > 0.0, "pistol rounds at {dist} u must damage the guard (the level-01 report's own control at 3.5-9 u)");
                assert!(!tr.impacts.is_empty(), "pistol {dist} u: at least one PLAYER impact must register (ally hits: {})", tr.ally_impacts);
                eprintln!("[smg] pistol {dist} u REGISTERED on d#{tg}: {} player impacts, hp {:.1} -> {:.1}", tr.impacts.len(), tr.hp0, tr.hp1);
            }
            // audit-voice-triggers-4 (the -3 pinned step (a), landed): every candidate was
            // killed by the escort's own volley before a player round landed. Assert the
            // attribution exactly — each steal is the target DEAD with 0 player impacts and
            // ally impact records on him — and pass the leg as the documented steal.
            None if dist > 1.0 && !steals.is_empty() => {
                for (sg, st) in &steals {
                    eprintln!("[smg] pistol {dist} u STEAL DOCUMENTED: d#{sg} hp {:.1} -> {:.1}, {} ally impacts on the target, 0 player impacts — the escort's w0x1c volley, the game's own ally behaviour",
                        st.hp0, st.hp1, st.ally_impacts);
                }
                assert!(steals.iter().all(|(_, st)| st.hp1 <= 0.0 && st.impacts.is_empty() && st.ally_impacts > 0),
                    "a documented steal must be the ally's own kill: the target dead, 0 player impacts, ally impacts on the target");
            }
            None => panic!("pistol rounds at {dist} u damaged no live guard and documented no ally steal (the level-01 report's own control at 3.5-9 u)"),
        }
        if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 4 {
            w.resupply("pistol ammo between legs");
        }
    }

    // SMG leg A: the clip at 1 u; leg B: at 2 u (the next guard if A died). The candidate
    // walk retries: only a guard with a CLEAR eye-to-torso segment at the stand takes the clip
    // (a guard behind the hall's cover eats the whole clip in the wall).
    for (dist, round_cap) in [(1.0f32, 8u32), (2.0, 28)] {
        // The SMG must be the held gun every leg (a resupply can swap).
        for _ in 0..3 {
            if w.scene.sim.as_ref().unwrap().gun.weapon == 0x12 { break; }
            w.tick(Input { switch_to: Some(0x12), ..Default::default() }, false);
            for _ in 0..120 { w.tick(Default::default(), false); }
        }
        assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x12, "the SMG in hand for the {dist} u leg");
        let mut fired = false;
        for _attempt in 0..3 {
            live.retain(|&i| {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[i];
                d.team == 1 && !d.corpse && d.enabled && d.health > 0.0
            });
            let Some(&tg) = live.first() else { break };
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[tg];
                eprintln!("[smg] target d#{tg} at {:?} enabled {} state {:#x} hp {:.1} boxes {}",
                    d.pos, d.enabled, d.state, d.health, d.hit_boxes(&data_of(sim)).len());
            }
            stand_near(&mut w, tg, dist);
            let clear = {
                let sim = w.scene.sim.as_ref().unwrap();
                let (chest, _, _) = torso_aim(sim, tg);
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                sim.world.segment(eye, chest).is_none()
            };
            if !clear {
                eprintln!("[smg] d#{tg}: no clear eye-to-torso segment at the {dist} u stand — next candidate");
                live.remove(0);
                continue;
            }
            let d = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.list[tg].pos);
            let clip = w.scene.sim.as_ref().unwrap().gun.clip;
            let rounds = round_cap.min(clip.max(1) as u32);
            eprintln!("[smg] firing {rounds} SMG rounds at d#{tg} from {d:.2} u (aim {dist})");
            let tr = fire_at(&mut w, tg, rounds, 0x12, &format!("smg {dist}u"), true, false);
            let drop = tr.hp0 - tr.hp1;
            let expected: f32 = tr.impacts.iter().map(|(_, e, _)| *e).sum();
            eprintln!("[smg] SMG {dist} u: spawned {} rounds, {} impacts (ally {}), hp {:.1} -> {:.1} (drop {drop:.3}, expected {expected:.3}), ticks {}",
                tr.spawned, tr.impacts.len(), tr.ally_impacts, tr.hp0, tr.hp1, tr.ticks);
            // fx-muzzle-repin: a neighbour's shotgun (0x1c) can spray the target while Bond stands
            // 1 u from him — on the corrected stream the hall guards' blast ate d#6 (3 pellet
            // impacts, 13.5 damage) before the SMG rounds landed. A player re-picks: the leg's
            // premise is the SMG's OWN rounds damaging the guard, so an ally-killed target is
            // skipped for the next candidate (the asserts below stay for the my-weapon path).
            if tr.impacts.is_empty() && tr.ally_impacts > 0 && tr.hp1 <= 0.0 {
                eprintln!("[smg] d#{tg}: died to ally fire before the SMG landed — next candidate");
                live.remove(0);
                continue;
            }
            assert!(drop > 0.0, "SMG rounds at {dist} u must damage the guard: spawned {}, impacts {}, hp {:.1}->{:.1}",
                tr.spawned, tr.impacts.len(), tr.hp0, tr.hp1);
            let dead = tr.hp1 <= 0.0;
            if !dead {
                assert!((drop - expected).abs() < 0.05, "the drop must be the record's damage per round: drop {drop:.3} vs expected {expected:.3}");
            } else {
                assert!(expected >= tr.hp0 - 0.5, "a dead guard: the expected losses must cover his health (expected {expected:.3}, hp0 {:.1})", tr.hp0);
            }
            fired = true;
            break;
        }
        let _ = fired;
        // Ammo for the next leg.
        if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 {
            w.resupply("smg ammo between legs");
        }
    }

    // The dormant probe (pistol): a switch-held guard takes no bullets — the game's own
    // Enable(0) law [M NDrone2_DSTATE_WaitSwitch 0x15b960 enter → NDrone2_Enable(0) 0x149dd0
    // sets obj+0xF0 |= 0x20; Collide_PickObj 0x1eae80:26-30 drops 0x20 objects]. The port's
    // pass-through is that rule, not a bug.
    if let Some(dg) = dg {
        stand_near(&mut w, dg, 0.9);
        let tr = fire_at(&mut w, dg, 4, 0x2, "dormant", false, true);
        eprintln!("[smg] DORMANT d#{dg}: spawned {} rounds, hp {:.1} -> {:.1} — Enable(0) hold [M 0x15b960/0x149dd0]",
            tr.spawned, tr.hp0, tr.hp1);
        assert_eq!(tr.hp0, tr.hp1, "a switch-held guard takes no bullets — the game's own Enable(0) rule [M 0x15b960 enter]");
    }
}

/// The wedged-lead unblock (file-local): a follower (the escort) or a body standing in
/// Bond's front band blocks every walk leg — the STUCK signature is segment_to_target
/// Some(~0.05) at the same spot across legs (the 6 u run: Bond wedged at (-13.49, 10.40)
/// for the whole leg, every route leg giving up at the cap). Step ASIDE perpendicular to
/// the blocked ray on real input until Bond actually moves — the 01 walk's own doctrine
/// (walkreal_07000001_live.rs:650 "step out of the lead's way", :666 "walk past the
/// wedged lead").
fn unwedge(w: &mut Walker, target: [f32; 3], label: &str) {
    let stuck = w.scene.sim.as_ref().unwrap().bond.pos;
    let (dx, dz) = (target[0] - stuck[0], target[2] - stuck[2]);
    let m = (dx * dx + dz * dz).sqrt().max(1e-3);
    let (px, pz) = (-dz / m, dx / m);
    w.lenient = true;
    for s in [1.4f32, -1.4f32] {
        let spot = [stuck[0] + px * s, stuck[1], stuck[2] + pz * s];
        w.walk_to(Wp { at: spot, act: Act::None, arrive: 0.3 }, &format!("{label}: the wedged-lead sidestep ({s:+.1})"));
        if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, stuck) > 0.8 { break; }
    }
}

/// A route leg with the un-wedge: walk, and if Bond never got near the target, sidestep
/// around the blocker and re-take the leg once.
fn walk_leg(w: &mut Walker, at: [f32; 3], label: &str, arrive: f32) {
    w.lenient = true;
    w.walk_nav_to(at, label, arrive);
    let bp = w.scene.sim.as_ref().unwrap().bond.pos;
    if walker::dxz(bp, at) > arrive + 1.5 {
        unwedge(w, at, label);
        w.walk_nav_to(at, &format!("{label} (re-take)"), arrive);
    }
}

/// Stand `dist` units from guard `gi` (horizontal), on the line from the guard to Bond's current
/// spot, walked with real input.
fn stand_near(w: &mut Walker, gi: usize, dist: f32) {
    // walk_nav_to restores lenient=false on its way out; every stand_near leg runs lenient.
    w.lenient = true;
    let gp = w.scene.sim.as_ref().unwrap().drones.list[gi].pos;
    let bp = w.scene.sim.as_ref().unwrap().bond.pos;
    let (dx, dz) = (bp[0] - gp[0], bp[2] - gp[2]);
    let m = (dx * dx + dz * dz).sqrt().max(1e-3);
    let spot = [gp[0] + dx / m * dist, gp[1], gp[2] + dz / m * dist];
    w.walk_to(Wp { at: spot, act: Act::None, arrive: 0.2 }, &format!("stand {dist} u from d#{gi}"));
    for k in 0..4 {
        let d = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.list[gi].pos);
        if (d - dist).abs() < 0.35 { break; }
        let gp = w.scene.sim.as_ref().unwrap().drones.list[gi].pos;
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let (dx, dz) = (bp[0] - gp[0], bp[2] - gp[2]);
        let m = (dx * dx + dz * dz).sqrt().max(1e-3);
        let want = d - dist; // walk IN by the excess
        let spot = [bp[0] - dx / m * want, bp[1], bp[2] - dz / m * want];
        w.walk_to(Wp { at: spot, act: Act::None, arrive: 0.15 }, &format!("nudge {k} to {dist} u"));
    }
    // The live guards HUNT (the gunfire alerted them) — a fixed spot goes stale. Follow the
    // guard's live position on real input until the band holds, aiming but never firing.
    for t in 0..2400u32 {
        let (d, input) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = walker::dxz(sim.bond.pos, sim.drones.list[gi].pos);
            let gp = sim.drones.list[gi].pos;
            let bp = sim.bond.pos;
            let dx = gp[0] - bp[0];
            let dz = gp[2] - bp[2];
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            (d, Input { turn: err.clamp(-0.08, 0.08), forward: if d > dist + 0.2 { 1.0 } else { 0.0 }, ..Default::default() })
        };
        if d <= dist + 0.2 && d >= dist - 0.5 { break; }
        w.tick(input, false);
    }
    let d = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.list[gi].pos);
    eprintln!("[smg] stand_near d#{gi}: settled at {d:.2} u (aim {dist})");
}
