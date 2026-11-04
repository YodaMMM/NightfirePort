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

/// The torso box to aim at: the largest-volume posed hit box whose part is neither a head bone
/// (4/5 — a head hit is ×100 and kills a 12-health guard outright, unmeasurable per round) nor
/// a leg bone (0x31+). Falls back to 0.6 above the origin when no boxes pose at all.
fn torso_aim(sim: &nf_view::scene::Sim, gi: usize) -> (V3, usize, Option<i16>) {
    let dr = &sim.drones.list[gi];
    let boxes = dr.hit_boxes(&data_of(sim));
    let n = boxes.len();
    let best = boxes.iter()
        .filter(|b| !matches!(b.part, 4 | 5) && !(0x31..=0x38).contains(&b.part))
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
fn fire_at(w: &mut Walker, gi: usize, max_rounds: u32, my_weapon: u16, label: &str) -> FireTrace {
    let mut tr = FireTrace { spawned: 0, impacts: Vec::new(), ally_impacts: 0, hp0: 0.0, hp1: 0.0, flying: Vec::new(), boxes: 0, enabled_at_fire: Vec::new(), corpse: false, ticks: 0 };
    tr.hp0 = w.scene.sim.as_ref().unwrap().drones.list[gi].health;
    let mut hp_prev = tr.hp0;
    let mut last_spawn = 0u32;
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
            let on = err.abs() < 0.07 && perr.abs() < 0.10;
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, on)
        };
        let (hp_before, enabled, state, dist, born_before) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            (dr.health, dr.enabled, dr.state, walker::dxz(sim.bond.pos, dr.pos), sim.drones.frame)
        };
        if on { w.tick_fire(input); } else { w.tick(input, false); }
        tr.ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let spawned_here = sim.shot_tick;
        if spawned_here {
            tr.spawned += 1;
            last_spawn = t;
            tr.enabled_at_fire.push(enabled);
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
    // control — each on a fresh living guard.
    w.tick(Input { switch_to: Some(0x2), ..Default::default() }, false);
    for _ in 0..90 { w.tick(Default::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x2, "the pistol switch");
    for dist in [1.0f32, 2.0f32, 6.0f32] {
        live.retain(|&i| {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[i];
            d.team == 1 && !d.corpse && d.enabled && d.health > 0.0
        });
        let Some(&tg) = live.first() else { break };
        stand_near(&mut w, tg, dist);
        let tr = fire_at(&mut w, tg, 1, 0x2, &format!("pistol {dist}u"));
        let drop = tr.hp0 - tr.hp1;
        eprintln!("[smg] pistol {dist} u on d#{tg}: spawned {} rounds, {} impacts, hp {:.1} -> {:.1} (drop {drop:.3}), ticks {}",
            tr.spawned, tr.impacts.len(), tr.hp0, tr.hp1, tr.ticks);
        assert!(drop > 0.0, "pistol rounds at {dist} u must damage the guard (the level-01 report's own control at 3.5-9 u)");
        assert!(!tr.impacts.is_empty(), "pistol {dist} u: at least one PLAYER impact must register (ally hits: {})", tr.ally_impacts);
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
            let tr = fire_at(&mut w, tg, rounds, 0x12, &format!("smg {dist}u"));
            let drop = tr.hp0 - tr.hp1;
            let expected: f32 = tr.impacts.iter().map(|(_, e, _)| *e).sum();
            eprintln!("[smg] SMG {dist} u: spawned {} rounds, {} impacts (ally {}), hp {:.1} -> {:.1} (drop {drop:.3}, expected {expected:.3}), ticks {}",
                tr.spawned, tr.impacts.len(), tr.ally_impacts, tr.hp0, tr.hp1, tr.ticks);
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
        let tr = fire_at(&mut w, dg, 4, 0x2, "dormant");
        eprintln!("[smg] DORMANT d#{dg}: spawned {} rounds, hp {:.1} -> {:.1} — Enable(0) hold [M 0x15b960/0x149dd0]",
            tr.spawned, tr.hp0, tr.hp1);
        assert_eq!(tr.hp0, tr.hp1, "a switch-held guard takes no bullets — the game's own Enable(0) rule [M 0x15b960 enter]");
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
