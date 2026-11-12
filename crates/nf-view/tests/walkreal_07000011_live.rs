//! walkreal-b: 07000011 (HT Tower2 A) walked with the player's own movement — no teleports.
//! Route (docs/levels/07000011.md + this task's surface scan): the spawn #146 (−0.40, 46.97, 25.83
//! [D]; the TouchOnce #282 spawn tile → ch 0x7E) → east across the top office floor to the
//! stairwell door (13.8, 46.8, 7.1 [D]) → E-opened — the doorway's clear band is its SOUTH half
//! (the leaf's hinge sits on the north edge z 7.1; the swung leaf blocks z 7.0-7.2 [D probe]) →
//! the stairwell's top landing (15.1, 6.4) → the stairwell DESCENT WALKED: the ring of flights
//! around the central well (the full chain in leg 2 below; the surface scan [D probe] resolved
//! what the old point-probe pass could not) → the lower floor (15.0) → the Load Point #158
//! (14.58, 16.41, −3.94 [D]) → E → level_exit 0x07000012 ([M Trigger_Activate 0x1b2318 — the E
//! arm falls through with the gate off]).
//!
//! THE EXTENSION (level-11-every-objective-3): after the 07000012 spawn stand the walk CONTINUES
//! on real input through every remaining objective of mission 0x07000011 to the mission's end —
//! the next level's load: the override console E (#731 → ch 0x49 completes 0x04000037), the lift
//! call, the shaft entrance (#683 → ch 0x1F), the gated exit #669 auto-firing into 0700004A
//! ([M Trigger_Update 0x1b1a78 case 10]), then the carried load's spawn drop into 0700004A's
//! touch #277 (ch 0x4D completes 0x04000038 — its writer lives one level ahead [D]). The route on
//! 07000012 is tower2's, shared with walkreal_07000012_live; its TWO documented stand-ins (the
//! door[19] corner pincer and the north-wall funnel — both measured, probes 12/12b/12c/12d, the
//! game passes them on the same key data) are carried over with it and listed in the report's
//! live checks. Everything else on both levels is walked input.
//!
//! THE MISSION ROWS ON THE WALK [D c07000011.rs; M 0x1827d8's machine]:
//! - 0x04000035 "Escape from the tower" (ch 0x64 = 100, the mission-success AND [M 0x1827d8:119-123])
//!   announces at load and completes only on 07000013, where the AND over the whole mission first
//!   fires [D docs/levels/07000011.md §2] — asserted announced (3) here, never complete.
//! - 0x04000036 "Gain access to lower office floor" (ch 0x48) announces at load; the writer hunt
//!   (level-11-every-objective-2, docs/reports/level-11-every-objective-2.md) found no writer on
//!   07000011 — that absence is [S] unverified (an assertion may not rest on a missing writer;
//!   the port may simply lack it), so the walk does NOT assert it: it logs the channel each leg
//!   and asserts the [M]-backed monitor states instead. The writer on 07000012 is TouchOnce
//!   #679 [D]: the spawn stands inside its tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62 [D])
//!   and ch 0x48 fires under Bond's feet — the record asserts COMPLETE (state 4) there
//!   [M 0x1827d8 case 3].

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The G-055 first-attack flush [M 0x148010:100-131] puts this level's stairwell guards on their
/// SECOND behaviour block at their first attack — the blocks charge to contact (the merged-tree
/// real-damage death: 100.0 across 29 hits / 13 fights, the 18.9-strip pair's melee 65 of it,
/// drones #13/#14 at d 0.7-1.4; the agent sister's leg then wedged in the shared resupply's
/// 22-u tour because every fight spent the 7-round spawn pistol [M 0x1a7ef8:275-281]). A player
/// scoops the dropped guns at his feet between fights — the walk-over take [M Pickup_Handler
/// 0x18e650] files the weapon + its clip — and holds the best-fed owned gun. Test-local (the
/// g055-4 idiom): the shared resupply and every other walk stay master's.
fn loot_the_drops(w: &mut Walker, label: &str) {
    loot_the_drops_up_to(w, label, 1.5);
}

/// The stairwell-bottom variant (fx-muzzle-repin-4): the fights' earlier kill-drops lie UP the
/// flights Bond has already descended (the shifted stream left #17's drop at y 24.0 and #14's
/// at 21.3 above the 18.9 strip), and the |Δy| < 1.5 band ratchets him back up one take at a
/// time — the 2026-11-05 run's loot tour climbed to y 24.01 and the flight-A assert fired
/// ("the descent never reached the lower floor"). A player descending scoops the drops at his
/// FEET: the band's up-side caps at the standing floor (drops above it are left behind).
fn loot_the_drops_floor(w: &mut Walker, label: &str) {
    loot_the_drops_up_to(w, label, 0.0);
}

fn loot_the_drops_up_to(w: &mut Walker, label: &str, up: f32) {
    for _ in 0..4 {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let t = sim.drones.pickups.iter().filter(|p| !p.gone)
            .filter(|p| (p.m[3][1] - bp[1]).abs() < 1.5)
            .filter(|p| p.m[3][1] - bp[1] <= up)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)))
            .filter(|t| walker::dxz(*t, bp) < 8.0);
        let Some(t) = t else { break };
        w.walk_nav_to(t, &format!("{label} loot"), 0.6);
    }
    // The deep-firearm cycle (g055-4): hold a real gun with rounds — the plain cycle can land
    // on the frag 0x34.
    for _ in 0..12 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            if (2..=0x47).contains(&g.weapon) && g.weapon != 0x34 && g.weapon != 0x37
                && g.clip + g.spare >= 5 { return; }
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    }
    w.switch_to_armed(label); // anything armed beats the fists
}

/// los-mask-repins-11 (00 Agent): the visible-guard answer. Live, enabled, non-WAIT_SWITCH
/// hostiles the eye rays can reach, nearest first, a `stand_and_fire` each — Bond HOLDS his
/// spot (the shoot loop's opening strafe drifts him 2 u west: the measured mouth run died at
/// d 1.1 of #19 after two stands drifted him off the mouth [L NF_RF_TRACE this task]). A
/// player clears the guards he can SEE from a strong spot before he walks into their
/// crossfire; a guard the stand cannot drop is left to the leg's own return-fire gate.
fn answer_visible_guards(w: &mut Walker, label: &str, reach: f32) {
    for _ in 0..6 {
        let g = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|&(i, d)| d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                    && d.state != nf_game::drone::state::WAIT_SWITCH
                    && walker::dxz(d.pos, b) < reach && (d.pos[1] - b[1]).abs() < 6.0
                    && walker::DamageLog::visible(sim, i))
                .min_by(|x, y| walker::dxz(x.1.pos, b).total_cmp(&walker::dxz(y.1.pos, b)))
                .map(|(i, _)| i)
        };
        let Some(g) = g else { break };
        eprintln!("[011] {label}: answering guard #{g}");
        if w.scene.sim.as_ref().unwrap().gun.clip < 5 {
            // A player reloads between guards (the measured #18 stand emptied the clip and the
            // #19 stand opened on clip 1 [L NF_RF_TRACE this task]).
            w.reload_full();
        }
        let k = w.stand_and_fire(g, 600);
        eprintln!("[011] {label}: guard #{g} killed {k} health {}",
            w.scene.sim.as_ref().unwrap().drones.health.health);
        if !k { break; }
    }
}

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent / 3 00
/// Agent ride the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_spawn_to_the_lower_floor_load_point_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000011.bin", false, None, difficulty)
            .expect("load")
    });
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[011] d{difficulty} at load: {} drones, clip {} spare {}",
            sim.drones.list.len(), sim.gun.clip, sim.gun.spare);
        for p in sim.drones.pickups.iter().filter(|p| !p.gone) {
            eprintln!("[011]   placed pickup at ({:.2},{:.2},{:.2}) kind {} weapon {:#x} rounds {}",
                p.m[3][0], p.m[3][1], p.m[3][2], p.kind, p.weapon, p.rounds);
        }
    }
    // The grenade launcher #290 lies in the locked storeroom [D].
    w.no_fetch.push([10.54, 47.47, 18.10]);
    // The two armour vests (kind 3, `nf_game::pickup::kind::ARMOUR` [D placements]) carry
    // no rounds: a dry Bond does not walk to them for ammo.
    w.no_fetch.push([23.12, 47.62, 22.88]);
    w.no_fetch.push([15.79, 15.08, 4.57]);
    if difficulty != 1 {
        assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
            "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] + 0.40).abs() < 1.5 && (p[1] - 46.97).abs() < 2.0 && (p[2] - 25.83).abs() < 1.5,
            "the spawn #146 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x7e),
        "the spawn tile TouchOnce #282 fires ch 0x7E [D]");
    // The mission rows after the first monitor ticks [D c07000011.rs; M 0x182270 init + 0x1827d8's
    // case-0 announce (record.level <= GameState → state 3; both records' enable is 0)]:
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "obj 0x04000035 'Escape from the tower' announced at load [D c07000011.rs; M 0x1827d8:41-54]");
        assert_eq!(sim.drones.mission.states[15][1], 3,
            "obj 0x04000036 'Gain access to lower office floor' announced at load [D c07000011.rs; M 0x1827d8:41-54]");
    }
    // The stairwell's guards (0x11/0x13/0x14/0x36/0x37 wake on the flight plates [D §5]) shoot
    // during the descent; the walk pins the route, not the firefight — the AI runs for real.
    w.top_up_health = true;
    // The real-health tactic (level-11-start-fight-real-health): on the flights the guards step
    // INTO the gun line while the fight holds on the farther target — the 2026-11-02 real-damage
    // run stood ~143 ticks on west flight B while the blocker (drone #18, 1.6 u, weapon 0x19)
    // landed 15 rounds for 51 health and the stand never tapped [L the run's rf-trace: fight#17
    // t60/t120 "blocked true", clip 18 throughout]. A player shoots the hostile who walks into
    // his line: under real damage the fights retarget a live hostile blocker
    // (DamageLog::shoot_blockers). Inert under the top-up (the fights never run there).
    if !w.tops_up() { w.dmg.shoot_blockers = true; }
    // The precise act (level-11-start-fight-real-health-3, opt-in, real damage only): the fights
    // tap only with the crosshair inside 0.01 rad of the exposed part and the line clear — the
    // stands fire from the hip by default (`Input { aiming: self.dmg.aimed_fights }`, default
    // false) and the loose 0.03/0.04 gate spent rounds the 7-round budget did not have [M
    // Player_InitWeapon 0x1a7ef8:275-281 — the 0x7000011 fresh equip is weapon 0x10 with SEVEN
    // rounds; case 0x7000012 RamLoads and, with BL+0x27a clear, re-runs the same equip — so both
    // legs' ammo is the 7]. The 2026-11-03 repro (repro10.out) fired all 7 by tick ~1020 — two of
    // them from a stand whose aim error printed −2.3/−2.5 rad at t0 — and went DRY at the keypad
    // with 24.5 health still to lose to #5 and 54.5 to #10. The mission-11 pistol carries no
    // 0x400000 record flag [L ee_start_05.bin 0x2bf150 + 0x10·0x10C + 0x68 = 0x1000], so the aim
    // bit does not change its cone (the spread stays the record's 7.0 + the pull bloom [M
    // Bullet_init 0x12c5a0:139-170]); and the AIM BUTTON ITSELF is excluded on purpose — with it
    // the port's free-aim path reinterprets the fight's turn/look as cursor moves and the rounds
    // fly off the aligned view (DamageLog::precise_fights' doc, repro11c). Inert under the top-up
    // (the fights never run there).
    if !w.tops_up() { w.dmg.precise_fights = true; }

    // THE 00 AGENT RESUPPLY MODE (difficulty 3; AGENT TOO on the merged tree): the d3 placements
    // grow the fight count past the fresh 7-round pistol's budget [M 0x1a7ef8:275-281 — the
    // mission-11 equip is 7] and the shared resupply then targeted the floor's only placed ammo
    // box — the grenade launcher at (10.54, 47.47, 18.10) [D #290] 22 u away on desk geometry —
    // whose tour ate the descent leg's cap. The lane mode (the 07000013 walk's opt-in, per-walk
    // only) plus the floor-drop band counts the guard drops the fights themselves leave ON the
    // floor (the drop's centre rests on the floor ray, 1.09 under Bond's pos y — outside the lane
    // 0.5 band): the run's NE-landing pass walked onto a fresh kill drop and armed clip 30 [L the
    // 2026-11-03 run]. Agent (d2) rode the shared rules while the guards held their first blocks;
    // the merged G-055 flush (the second-block chargers, the real-damage death measured this
    // session: 100.0 across 29 hits / 13 fights, the agent sister's shared-resupply wedge at the
    // same 22-u box) spends the 7 exactly like d3 — the same two levers apply.
    if difficulty >= 2 {
        w.dmg.lane_fights = true;
        w.dmg.lane_floor_drops = true;
    }
    if difficulty == 3 {
        // The close-guard law: under 3.0 u every drone round hits [M DroneWeap_DoBulletAccuracy
        // 0x179740:113-115] — this run's ledger finishes Bond with the connector guard #18 at
        // 1.9 u (4.0, the last health) and round 1's flight-B blocker stood 1.6 u. A player
        // back-pedals out of the free-hit band while shooting (the opt-in steps back, aim held).
        w.dmg.back_off = Some(2.6);
        // The eye-ray stands (real-health-00agent-11-3): the strip guards fire across the
        // flights while the lane fights' muzzle stand-in rays clip the flight lips — the
        // measured crossings drained the armour 43→0 with NO fight triggered (the pick returned
        // None while the volleys kept landing). Answer what he sees (the opt-in flag; every
        // other walk keeps the lane gun-ray pick).
        w.dmg.eye_ray_stands = true;
        // los-mask-repins-11: the guard-sight fights MIX. The run's death stand (fight#29) held
        // a shooter the eye rays never saw (aim None the whole stand — he was walking the 23.0
        // strip above the flight lip) while #19 (2.6 u) and #36 (the connector above) burst
        // health 39 → 0 in 90 ticks [L NF_RF_TRACE this task]. The hurting-first laws (the
        // 07000008 Agent sisters' opt-in): the pick answers the shooter who is actually HURTING
        // Bond, drops the cooled candidates, and the fight retargets every 30 ticks.
        w.dmg.hurting_first = true;
    }

    // THE 00 AGENT PRE-LEG (real-health-00agent-11-2): take the top-floor BodyArmour BEFORE the
    // first fight. Round 1 died at the north strip 31.1 with 15 hits for 100.0 — the fights
    // themselves traded 70 of it (#14 56.0 + #15 28.0 while their partners crossed-fired) and
    // #17/#16 finished the last 9. The game's own law: a bullet pain's armour fraction is 1.0 —
    // armour absorbs first, health only what the armour cannot cover [M Player_HandlePain
    // 0x1902c8:84-104 → the type-0 arm; PlayerHealth::apply's `absorbed = (d·frac).min(armour)`],
    // and the take is the walk-over `Pickup_Handler` [M 0x18e650 — refused only at armour ≥ 50;
    // Bond loads with 0]. The disc places #261/#266 BodyArmour at (23.12, 47.62, 22.88) on the
    // top office floor [D; the load print], 23 u east of the spawn along the open floor — the
    // player's own first move on 00 Agent. (The mid-leg take of round 1 wedged the 6000-tick cap
    // inside resupply recursion; as a PRE-LEG the fights it meets are the spawn guards' free
    // kills and the take happens before any fight has cost anything.)
    // THE AGENT SISTER RIDES THE SAME PRE-LEG (civ-repin-11-13): the 2026-11-06 d2 run died at
    // connector 2 (t16236, 23 hits for 100.0 / 13 fights) after its pre-fight low-restock tour
    // (the lane 12 u fetch) targeted the GL #290 at d 11.9 and wedged twice at the 6000-tick cap
    // on the desk lip — 12000 ticks and a dry descent (clip 4 → 2 → 1, the NE-landing drop armed
    // clip 30 only at t14400+) [L the clean per-test run]. The armour take + the #6/#5 drop detour
    // (the pre-leg's own ammo stop) are the player's answer: the 130 budget (80 health + 50
    // armour) the d3 run measured on these flights, armed from the first fight. The d1 sister
    // keeps her timeline (she passes without it — her death was 07000012's, fixed on that walker).
    if difficulty >= 2 {
        // The east office floor is a desk maze the game's own nav never spans [D: path 1 has NO
        // node path from the spawn-side n39 to the armour-side n64; path 0's ring stays z 7-16]
        // and the direct steer wedges on the mid-floor wall run (measured: stuck at (12.5, 13.5),
        // segment hit 0.56 ahead, sidesteps exhausted). The chain below is a corridor-BFS over
        // the world's own segments (0.5 u cells, 3 body-width rays a step, walkreal probe this
        // task) — the walkable way a player threads the desks, out to the armour pocket and back
        // to the stairwell-door ledge where leg 1's own chain takes over.
        let maze: &[(&str, f32, f32, f32)] = &[
            ("the spawn pocket south", 0.0, 22.5, 0.8),
            // The #6/#5 spawn fights' own gun drop rests at (−3.0, 17.9) [L the run logs] — the
            // pre-leg's ammo stop (resupply_off forbids the tours; the fights ahead need the
            // rounds). The coarse probe has z 16-19 x −2..5 open — the detour rejoin is proven
            // geometry on both ends.
            ("the spawn-fight drop detour", -3.0, 17.9, 0.9),
            ("the detour rejoin", 2.0, 17.2, 0.9),
            ("south of the west desks", 4.5, 18.5, 0.8),
            ("the mid-west jog", 5.5, 16.5, 0.8), ("the mid corridor west", 9.5, 16.5, 0.8),
            ("the desk corner 1", 10.5, 16.0, 0.6), ("the desk corner 2", 11.0, 15.0, 0.6),
            ("the desk corner 3", 11.5, 14.5, 0.6), ("the desk corner 4", 12.5, 14.0, 0.6),
            ("the mid-wall south", 12.5, 9.5, 0.7), ("the ledge jog 1", 13.0, 9.0, 0.6),
            ("the ledge jog 2", 13.0, 7.5, 0.6), ("the stairwell-door ledge", 13.5, 6.5, 0.7),
            ("the ledge east", 17.0, 6.5, 0.8), ("the east jog 1", 17.5, 8.0, 0.6),
            ("the east run", 19.5, 9.5, 0.7), ("the east jog 2", 20.5, 10.0, 0.6),
            ("the east wall north", 20.5, 16.5, 0.7), ("the armour run east", 22.5, 16.5, 0.8),
            ("the armour run north", 22.5, 20.0, 0.8), ("the armour take spot", 22.5, 21.5, 0.6),
        ];
        for (label, x, z, r) in maze {
            w.walk_to(Wp::new(*x, 47.4, *z).r(*r), &format!("d3 pre-leg: {label}"));
            if *label == "the east jog 2" {
                // los-mask-join: the armour room's pair is next, and the corridor fights left
                // the clip at 2-3 rounds (spare 7) — the hp-24 #2 then outlived the clip at the
                // doorway and his 0x1d volleys at 1.9 u took all 80 [L the masked run; the
                // unmasked run won the same stand by a few ticks]. A player reloads before he
                // opens the door.
                w.reload_full();
            }
            if *label == "the east wall north" {
                // The armour room's pair, held at its doorway: #1 (placement 122, unarmed) and #2
                // (placement 123, weapon 0x1d). With the 5.0/6.0 engage ranges [M 0x14b300
                // LAB_0014bb44] #1 now closes to a 40.0 punch on Bond in the doorway, and #2's
                // point-blank volleys at 2.0 u finished him at tick 2033 (8 x 8.0 in one tick) as
                // he walked in. A player shoots them from the doorway as they come: any guard on
                // this floor within 7 is fought; one still within 12 is waited for (bounded).
                // capsule-join-5: the stand never walks INTO the captain. On this tree #2 (24 hp,
                // weapon 0x1d) stood unseen behind the door frame (aim None), the unseen-cover
                // creep carried Bond toward him, and at 1.7 u one tick's volley took 80.0 in 16
                // hits [L NF_RF_TRACE run: fight#2 aim None, creep true, death t2427]. A player
                // facing a shotgun captain he cannot see holds back: the stand keeps 4 u off a
                // live guard (DamageLog::back_off — the crowd arm steps back and stops the creep).
                let prev_back_off = w.dmg.back_off;
                if difficulty >= 3 { w.dmg.back_off = Some(4.0); } // the Agent sister passes as she is
                let start = w.total_ticks;
                while w.total_ticks - start < 900 {
                    let (near, far) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let live = |d: &&nf_game::drone::Drone| d.present && !d.corpse && d.health > 0.0 && d.team == 1
                            && (d.pos[1] - sim.bond.pos[1]).abs() < 2.0;
                        (sim.drones.list.iter().position(|d| live(&d) && walker::dxz(d.pos, sim.bond.pos) < 7.0),
                         sim.drones.list.iter().any(|d| live(&d) && walker::dxz(d.pos, sim.bond.pos) < 12.0))
                    };
                    if let Some(di) = near {
                        let k = w.stand_and_fire(di, 600);
                        eprintln!("[011] d3 armour-room stand: #{di} killed {k} health {}", w.scene.sim.as_ref().unwrap().drones.health.health);
                    } else if far {
                        w.tick(nf_game::player::Input::default(), false);
                    } else { break; }
                }
                w.dmg.back_off = prev_back_off;
                // The doorway stand's own turns and recoil can carry Bond off the wall lane
                // (join-reds2-0, 00 Agent on the stream join: the run east then started from the
                // desk side and wedged on a desk top at (20.87, 48.28, 14.89), grounded false).
                // A player steps back onto the lane before the run east.
                let off = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [*x, 47.4, *z]);
                if off > 0.7 {
                    eprintln!("[011] d3 armour-room stand left Bond {off:.2} u off the wall lane — back onto it");
                    w.walk_to(Wp::new(*x, 47.4, *z).r(0.45), "d3 pre-leg: the east wall north, back on the lane");
                }
            }
        }
        // The take fires from the ring around the armour (radius 0.477 + 1.1 with a clear
        // sphere→centre line [M 0x18e148:203-214; the walkreal probe this task]) — the take spot
        // (22.5, 21.5) the corridor-BFS proved, plus the lenient inner hop (23.0, 22.0); a
        // wedged hop is skipped (the desk lip east of the pickup) — the stand inside the ring
        // has already taken it.
        w.try_walk_to(Wp::new(23.0, 47.4, 22.0).r(0.4), "d3 pre-leg: the armour ring inner cell");
        {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(sim.drones.health.armour > 0.0,
                "the pre-leg took the BodyArmour (walk-over take [M 0x18e650], armour {})",
                sim.drones.health.armour);
        }
        // Back west along the same corridor to the ledge, where leg 1's direct chain resumes.
        // The x-20.5 wall run has the desk rows half a unit east — tight arrival radii keep the
        // steering overshoot off the desk tops (measured: a 0.8 arrive corner climbed the desk at
        // (21.5, 15.9) and wedged grounded=false).
        for (label, x, z, r) in [
            ("the armour run back south", 22.5f32, 16.5f32, 0.8f32),
            ("the east wall back", 20.5, 16.5, 0.45), ("the east wall back 2", 20.5, 15.5, 0.45),
            ("the east wall back 3", 20.5, 14.0, 0.45), ("the east wall back 4", 20.5, 12.0, 0.5),
            ("the east jog back", 20.5, 10.0, 0.5),
            ("the east run back", 19.5, 9.5, 0.6), ("the ledge east back", 17.0, 6.5, 0.8),
        ] {
            w.walk_to(Wp::new(x, 47.4, z).r(r), &format!("d3 pre-leg return: {label}"));
        }
        eprintln!("[011] d3 pre-leg: armour {} on at +{} ticks",
            { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.health.armour }, w.total_ticks);
    }

    // Leg 1: east across the top floor to the stairwell door (the nav web [D paths 0/1] has no
    // cross-floor route — the direct chain), then through the doorway's south band — walked.
    w.stop_on_exit = false;
    // Operative: a fight on the doorway leg can send the resupply 17 u east to the far drops
    // ((23.8, 22.4)); the leg's straight walk back then crossed the desk rows and climbed the desk
    // at (21.5, 15.9) (grounded=false, the 6000-tick cap) once the room drawn gate moved the
    // fights. The errand walks home first (the opt-in the 13 chain uses).
    if difficulty == 1 { w.resupply_home = true; }
    w.follow_nav_route([12.7, 47.4, 6.4], "across the top floor to the stairwell door");
    w.walk_to(Wp::new(16.0, 47.4, 6.4).r(0.9), "through the doorway (the south band)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[011] post-leg1: exit {:?} ch100 {} ch62 {} ch61 {} ch60 {} ch63 {} pos {:?}",
            sim.drones.level_exit, sim.drones.channels.on(100), sim.drones.channels.on(0x62),
            sim.drones.channels.on(0x61), sim.drones.channels.on(0x60), sim.drones.channels.on(0x63), sim.bond.pos);
        assert!(!sim.drones.channels.on(0x62), "no death on the walked top floor");
    }
    // Leg 2 — THE STAIRWELL DESCENT, WALKED (the old teleport stand-in is gone). The stairwell is
    // a ring of flights around a central well (x 17.5-21, z −1..3 stays open at every level [D
    // probe]); the descent walks its rim, alternating long flights (the west strip x 14.2-16.6,
    // z −3.5..5.5) with the quarter-turn connectors [D surface scan, this task]:
    //   office floor (the seam slopes onto the flight at x 10.5-13, z −4.2..−5)
    //   → north flight (z −5.5..−4.5, x 12.5→19: 46.8→44.4) → NE landing 43.1 (x 21-24, z −5..−2)
    //   → east flight (x 22-24, z −3→4.4: 43.1→39.1) → SE landing 39.1 (x 19-25, z 4.4-6.4)
    //   → south run (z 6.0-6.4, x 21→14.6: 39.1→35.2) → SW landing 35.2 (x 14.2-16.6, z 4.4-6.5)
    //   → west flight C (x 14.2-16.6, z 5.5→−2.5: 35.2→31.1) → north strip 31.1 (x 14-23)
    //   → east connector (x 23, z −3→4.4: 31.1→27.1) → south strip 27.1 (x 14.6-23, z 3.5-5.5)
    //   → west flight B (27.1→23.0) → north strip → east connector (23.0→18.9) → south strip
    //   → west flight A (18.9→15.0) → the lower floor (15.0, flat to the Load Point).
    let band = |w: &Walker, lo: f32, hi: f32, what: &str| {
        let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
        assert!(y > lo && y < hi, "{what}: y {y} not in ({lo}, {hi}) — the descent left the flights");
    };
    w.walk_to(Wp::new(12.0, 47.4, 6.6).r(0.9), "the ledge west");
    // The well strip (z 1.5-4.5) splits the top level; the crossing is the west bridge at
    // x ≈ 1-1.5 (the office floor is continuous z −2..7 at x 1.0 [D probe]; the nav's own
    // office→balcony link n12 (2.1, 47.2, 6.6) → n45 (2.2, 47.2, −0.3) rides the same bridge
    // [D nav path 1]).
    w.walk_to(Wp::new(2.2, 47.4, 6.6).r(0.9), "the ledge's west end (n12)");
    w.walk_to(Wp::new(1.2, 47.4, 3.0).r(0.6), "the west bridge south");
    w.walk_to(Wp::new(1.4, 47.4, -1.0).r(0.8), "onto the north balcony (n45)");
    w.walk_to(Wp::new(5.0, 47.4, -1.8).r(0.9), "the balcony east (n46/n47)");
    w.walk_to(Wp::new(9.0, 47.4, -2.2).r(0.9), "the north strip east");
    w.walk_to(Wp::new(9.6, 47.4, -4.4).r(0.8), "the strip west of the parapet");
    // The well's parapets at chest height [D ray-fan]: the diagonal W1 (9.2,−5.8)→(12.75,−3.5) and
    // the north-east pair W2 (11.5,−1.9)→(12.7,−3.4) meet at (12.7,−3.45) — the office floor's
    // tongue ends in their pocket; the descent crosses at W1's south-west end (below).
    w.walk_to(Wp::new(10.7, 47.3, -4.1).r(0.4), "the gap's north approach");
    // THE STAIRWELL DOOR #192 IS THE CROSSING (this session's fix of the old 2 m pocket stand-in):
    // the well's parapets W1/W2 are the stairwell's own walls, and the disc puts the E
    // door_large_stairwell #192 (10.22, 46.75, −5.21 [D], lock ch 0x12) IN that wall line — the
    // doorway is the opening; there is no walk-around (the parapets are continuous — W1
    // (9.2,−5.8)→(12.75,−3.5), the probes hit its 0x03/0x15 faces at body height [D] — and the
    // game's own nav graph has no link across the pocket either: the office-floor component ends
    // at n45/n46/n47, the stair chain starts fresh at n94 [D nav BFS]). The keypad lock #177 is
    // create-unlocked on Operative [D #177 p6 = 1; M 0x1ae1f8:67-69] and Lock_Update case 1
    // (0x1ae3d8) holds ch 0x12 on, so the door answers E from level start.
    // THE SISTERS' KEYPAD (difficulty 2/3): `Lock_Create` 0x1ae1f8:67-69 calls `Lock_Unlock` at
    // create ONLY while GameState+0x40 == 1 — on Agent/00 Agent the keypad #177 is created
    // LOCKED, ch 0x12 cold [M; the interactables_07000011 sister pins the cold load]. The player's
    // way, all real input: the aim + E pulse arms the mini-game (state 3 [M Player_Activate
    // 0x1a5170:158-166 → Lock_Activate 0x1aed00's kind-0 no-key arm]), the digit presses run
    // C_KEYPAD_Handler 0x216810 msg 0x4b, the case-4 match fires `Lock_Unlock` [M 0x1ae3d8:262-296]
    // and the next case-1 tick holds ch 0x12 [M 0x1ae3d8:192-196] — the door answers E again.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if difficulty == 1 {
            assert!(sim.drones.channels.on(0x12),
                "the keypad's Operative unlock holds ch 0x12 [D #177 p6; M 0x1ae3d8 case 1]");
        } else {
            assert!(!sim.drones.channels.on(0x12),
                "Agent/00 Agent: the keypad #177 is created LOCKED, ch 0x12 cold [M Lock_Create 0x1ae1f8:67-69]");
        }
    }
    w.walk_to(Wp::new(10.22, 47.4, -3.6).r(0.9), "the stairwell door's front tile");
    if difficulty != 1 {
        // The stand north-west of the keypad (11.58, 48.20, −4.21 [D]) — its wall (the parapet W1
        // (9.2,−5.8)→(12.75,−3.5) [D]) faces the office floor, so the front side is here; the use
        // sphere is head+look within 1.0 + r [M 0x1a5170:100-108] and the facing gate reads
        // `l.facing · (body − l.pos) ≥ 0` [M 0x1a5170:158-166]. The aim loop is
        // walkreal_0700000a's real-input keypad entry (turn/look input, one use-key pulse).
        w.walk_to(Wp::new(10.7, 47.4, -4.1).r(0.5), "the keypad's front side");
        let kp: [f32; 3] = {
            let sim = w.scene.sim.as_ref().unwrap();
            let l = sim.drones.locks.iter().find(|l| l.placement == 177).expect("keypad #177 [D]");
            l.pos
        };
        let mut armed = false;
        'kp: for _cycle in 0..8u32 {
            for t in 0..30u32 {
                {
                    let sim = w.scene.sim.as_ref().unwrap();
                    if sim.drones.keypad.is_some() { armed = true; break 'kp; }
                    let bp = sim.bond.pos;
                    let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
                    let dx = kp[0] - bp[0];
                    let dz = kp[2] - bp[2];
                    let dy = kp[1] - eye;
                    let desired = dx.atan2(dz);
                    let err = walker::wrap_pi(desired - sim.bond.yaw);
                    let perr = (dy / walker::dxz(bp, kp).max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
                    let input = nf_game::player::Input {
                        turn: err.clamp(-0.05, 0.05),
                        look: perr.clamp(-0.05, 0.05),
                        ..Default::default()
                    };
                    let hold = t == 15 && err.abs() < 0.2;
                    w.scene.sim.as_mut().unwrap().step_bond(input, false, false, hold);
                }
                w.total_ticks += 1;
            }
        }
        assert!(armed, "the keypad #177 E arms the mini-game (state 3) [M 0x1aed00's kind-0 no-key arm]");
        // The 4 digits (C_KEYPAD_Handler 0x216810 msg 0x4b — the keypad page's digit-control
        // dispatch; the cursor/page UI is the view's presentation, the audit §5). The code is the
        // lock's p1 entry of the boot table (the game's KeyCodes 0x2a37a8 [M Locks_Init 0x1b0548]).
        let code = {
            let sim = w.scene.sim.as_ref().unwrap();
            let idx = sim.drones.locks.iter().find(|l| l.placement == 177).expect("keypad #177 [D]").code;
            sim.drones.key_codes.code(idx)
        };
        for d in code {
            assert!(d >= b'0' && d <= b'9', "the keypad code digits are ASCII");
            w.scene.sim.as_mut().unwrap().drones.keypad_digit((d - b'0') as u32);
            w.wait(1);
        }
        w.wait(2); // the case-4 match fires on the last digit's tick; the case-1 hold writes ch 0x12 the NEXT tick [M 0x1ae3d8 — the hold loop runs before the keypad tick in the object pass]
        {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(sim.drones.channels.on(0x12),
                "the solved keypad's case-1 hold writes ch 0x12 [M 0x1ae3d8:192-196]");
        }
    }
    let d192 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 10.22).abs() < 0.5 && (d.pos[2] + 5.21).abs() < 0.5)
        .expect("door_large_stairwell #192 [D]");
    w.use_door(d192, "the stairwell door (the ch-0x12 unlock)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let st = sim.drones.doors.list[d192].state;
        assert!(st & 0xc != 0, "the stairwell door opens on the walk's E, state {st:#x}");
    }
    w.wait(45); // the swing finishes before the doorway walk (the auto-close is 360 ticks [M Door_Update])
    w.walk_to(Wp::new(11.0, 47.4, -5.5).r(0.9), "through the stairwell doorway");
    loot_the_drops(&mut w, "the doorway"); // g055-4 kill-loot: the fights rearm from their own drops
    w.walk_to(Wp::new(13.0, 46.9, -6.2).r(0.9), "the top flight east");
    w.walk_to(Wp::new(19.5, 44.6, -5.2).r(1.0), "the north flight down (46.8→44.4)");
    w.walk_to(Wp::new(22.3, 43.6, -3.2).r(1.0), "the north-east landing 43.1");
    band(&w, 42.6, 44.6, "the NE landing");
    loot_the_drops(&mut w, "the NE landing");
    w.walk_to(Wp::new(23.0, 39.5, 3.6).r(1.0), "the east flight down (43.1→39.1)");
    w.walk_to(Wp::new(20.5, 39.5, 5.8).r(1.0), "the south-east landing 39.1");
    band(&w, 38.6, 40.5, "the SE landing");
    loot_the_drops(&mut w, "the SE landing");
    w.walk_to(Wp::new(15.2, 35.6, 6.0).r(1.0), "the south run down (39.1→35.2)");
    w.walk_to(Wp::new(14.8, 35.4, 5.0).r(0.9), "the south-west landing 35.2");
    band(&w, 34.6, 36.6, "the SW landing");
    loot_the_drops(&mut w, "the SW landing");
    // The real-health run's fights pause the descent (a return-fire stand or a pain stagger
    // on the flight), so the dxz-arrival can sample the band a step or two up the flight —
    // measured 32.516 against this 32.5 cap (walkreal_07000011_real_damage, 2026-11-02). The
    // UPPER caps here carry +0.2 of fight-pause slack; the LOWER caps (the descent reached
    // the flight's bottom) are the pins and stay.
    w.walk_to(Wp::new(14.8, 31.4, -2.6).r(1.0), "the west flight C down (35.2→31.1)");
    band(&w, 30.4, 32.7, "flight C's bottom");
    w.walk_to(Wp::new(22.4, 31.3, -3.0).r(1.0), "the north strip 31.1 east");
    loot_the_drops(&mut w, "the 31.1 strip");
    w.walk_to(Wp::new(23.0, 27.4, 3.8).r(1.0), "the east connector down (31.1→27.1)");
    band(&w, 26.4, 29.5, "connector 1's bottom");
    w.walk_to(Wp::new(15.0, 27.4, 4.8).r(1.0), "the south strip 27.1 west");
    if difficulty == 3 {
        // los-mask-join (00 Agent): the 27.1 take climbed back up connector 1 after a drop
        // lying on its treads (Bond wedged at y 30.3 over the 29.0 floor, 6000 ticks [L the
        // masked run]). A player descending scoops at his feet — the 18.9 strip's rule.
        loot_the_drops_floor(&mut w, "the 27.1 strip");
    } else {
        loot_the_drops(&mut w, "the 27.1 strip");
    }
    if difficulty >= 3 {
        // capsule-join (00 Agent): measured from the 27.1 strip's west end before the descent
        // (Bond at (15.9, 28.2, 4.7)): the 23.0 strip's #33/#34 (CombatNoSight 0x5c at
        // (22-23, 24.0, −3)) are already exposed down the flight, and #20/#21 (0x5a, the 31.1
        // strip's pursuers above) too; at the flight's foot the same pair fire 0x68 at 4-5 u
        // while the pursuers close behind [L capsule-join exposure probe] — the strip walk took
        // the crossfire from both sides (74 → 0 at t6380). A player takes the strip pair from
        // the flight's mouth, where they are exposed and he is above them, then any pursuer he
        // can see, before he goes down. Live guards only (the WaitForSwitch 0x1 placements at the
        // foot are not yet enabled), nearest-below first, a bounded stand each.
        for _ in 0..6 {
            let g = {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|&(i, d)| d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                        && d.state != nf_game::drone::state::WAIT_SWITCH
                        && walker::dxz(d.pos, b) < 14.0 && (d.pos[1] - b[1]).abs() < 6.0
                        && walker::DamageLog::visible(sim, i))
                    .min_by(|x, y| (x.1.pos[1], walker::dxz(x.1.pos, b)).partial_cmp(&(y.1.pos[1], walker::dxz(y.1.pos, b))).unwrap())
                    .map(|(i, _)| i)
            };
            let Some(g) = g else { break };
            if !w.shoot_bounded(g, &format!("flight B's mouth guard {g}"), 600) { break; }
        }
    }
    // The mouth stands' own repositioning can carry Bond down the flight and east along the
    // 23.0 strip (measured: (22.40, 23.23, −0.26) — the strip's east end, where the straight
    // flight leg back west met the wall for its cap): a Bond already down on the 23.0 floor has
    // made the descent; one that the stands carried on down the east connector (measured
    // y 21.07) comes back up to the strip through the nav web.
    if w.scene.sim.as_ref().unwrap().bond.pos[1] > 25.5 {
        w.walk_to(Wp::new(14.8, 23.3, -2.6).r(1.0), "the west flight B down (27.1→23.0)");
    } else {
        w.walk_nav_to([22.4, 23.2, -3.0], "back onto the 23.0 strip after the mouth fights", 1.0);
    }
    band(&w, 22.4, 25.0, "flight B's bottom");
    w.walk_to(Wp::new(22.4, 23.2, -3.0).r(1.0), "the north strip 23.0 east");
    // capsule-join-5: the 23.0 strip takes the drops at Bond's FEET (loot_the_drops_floor, the
    // 18.9 strip's rule): the |dy| < 1.5 band ratcheted him back up flight B one take at a time
    // — the d3 run's tour ended wedged on the 27.1 strip at (15.33, 27.55, 2.40) under a drop at
    // y 27.09, the d1 run's at the flight's foot (15.33, 24.69, −1.23), and the straight
    // connector leg from there climbed flight B to (15.33, 27.66, 2.57) for its 6000-tick cap
    // [L both runs on this branch]. A tour that still leaves him off the strip's east end walks
    // back along it first (the 18.9 strip's "back along" leg).
    loot_the_drops_floor(&mut w, "the 23.0 strip");
    if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [22.4, 23.2, -3.0]) > 2.0 {
        w.walk_nav_to([22.4, 23.2, -3.0], "back along the 23.0 strip", 1.0);
    }
    if difficulty >= 3 {
        // los-mask-repins-11 (00 Agent): the connector-2 TOP. The guard sight turned the
        // connector's own ends into a crossfire: the measured run died at its bottom with the
        // top guard #29 (23.6, 23.8, −1.4, 4.6 u — 17.5 in 5 hits) and the elevated well guard
        // #36 (19.7, ~21, 0.4, 3.7 u — 39.0 in 11 hits) shooting from above while the flight's
        // lip blocked every eye ray back up at them [L NF_RF_TRACE this task]. A player answers
        // what he can see from the strip's east end BEFORE he commits to the flight: from here
        // the top guard is 1.7 u and the well guard is a down-look over the same lip that
        // blocks the up-look.
        answer_visible_guards(&mut w, "d3 the connector-2 top", 12.0);
    }
    w.walk_to(Wp::new(23.0, 19.3, 3.8).r(1.0), "the east connector down (23.0→18.9)");
    band(&w, 18.2, 21.3, "connector 2's bottom");
    if difficulty >= 3 {
        // los-mask-repins-11 (00 Agent): the 18.9 strip's MOUTH. With the game's sight the
        // strip's cluster engages across the mouth itself: the run's fight#18 opened with the
        // melee charger #18 already 1.1 u in (the kill cost 35.0 in 5 punches), and after it
        // the unseen-fight fallback stood blind on the north-strip shooter #29 (aim None the
        // whole stand) while #19 (2.6 u on the strip, weapon 0x19, never damaged) and #36 (the
        // connector above, 5.6 u) burst health 39 → 0 — death tick 19894 at the mouth's east
        // end [L NF_RF_TRACE this task]. A player stops at the connector's foot and answers the
        // guards he can see on the strip BEFORE he crosses (the flight-B mouth idiom above):
        // the stands killed #18/#19/#28 from here on the measured run [L this task].
        answer_visible_guards(&mut w, "d3 the 18.9 strip mouth", 12.0);
    }
    w.walk_to(Wp::new(15.0, 19.2, 4.8).r(1.0), "the south strip 18.9 west");
    // The strip's return-fire fights (#13/#14 on the d1 walk with the game's sight round robin
    // and the own-cel boundary push) can turn Bond onto the flight-B foot; the straight leg
    // west then climbs flight B and arrives on the 27.1 strip in x/z (15.04, 28.09, 3.81)
    // [L this branch's d1 run]. A Bond off the 18.9 floor comes back down over the nav web.
    if w.scene.sim.as_ref().unwrap().bond.pos[1] > 21.3 {
        w.walk_nav_to([15.0, 19.2, 4.8], "back down onto the 18.9 strip", 1.0);
    }
    loot_the_drops_floor(&mut w, "the 18.9 strip");
    // A drop can lie at the strip's east end: back to the flight head before the descent.
    w.walk_to(Wp::new(15.0, 19.2, 4.8).r(1.0), "back along the 18.9 strip");
    w.walk_to(Wp::new(14.8, 15.4, -2.6).r(1.0), "the west flight A down (18.9→15.0)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.bond.pos[1];
        assert!(y < 17.2, "the descent never reached the lower floor (y {y}) [D 15.0 probe + the 1.09 standing offset]");
        assert!(!sim.drones.channels.on(0x7c), "the descent never crossed the atrium plates (ch 0x7C) [D §4]");
        // [S] unverified — NOT a pass/fail assert: "no 0x48 writer on this level" rests on a
        // missing writer, and the port may simply lack it (level-11-every-objective-2). The
        // mission-state asserts below are the [M]-backed pins.
        eprintln!("[011] [S] unverified: ch 0x48 off at the stairwell bottom (expected: no lower-floor writer on this level)", );
        assert!(!sim.drones.channels.on(100), "no mission success on this level [D §2; M 0x1827d8:119-123]");
        assert_eq!(sim.drones.mission.states[15], [3, 3],
            "both records still announced at the stairwell bottom [M 0x1827d8 case 3]");
    }
    eprintln!("[011] the stairwell descent WALKED at +{} ticks", w.total_ticks);
    if difficulty >= 3 {
        // capsule-join-5 (00 Agent): the lower floor's guard #36 (weapon 0x19) stands 6.3 u off
        // the Load Point and fired 7 hits for 22.0 into the E press while no fight opened (the
        // press legs are short; Bond died at (14.32, 16.14, −2.64) with 22.0 left [L this branch's
        // run]). A player at the flight's foot answers the guns he can see on the floor before
        // he turns his back to them for the door — the flight-B mouth idiom: live, visible,
        // within 12 u and 6 u of height (#36 was still on the connector, y 20.4, when the
        // descent ended [L the probe]), nearest first, a bounded stand each.
        for _ in 0..4 {
            let g = {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|&(i, d)| d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                        && d.state != nf_game::drone::state::WAIT_SWITCH
                        && walker::dxz(d.pos, b) < 12.0 && (d.pos[1] - b[1]).abs() < 6.0
                        && walker::DamageLog::visible(sim, i))
                    .min_by(|x, y| walker::dxz(x.1.pos, b).total_cmp(&walker::dxz(y.1.pos, b)))
                    .map(|(i, _)| i)
            };
            let Some(g) = g else { break };
            eprintln!("[011] d3 the stairwell foot: answering guard #{g}");
            if !w.shoot_bounded(g, &format!("the stairwell foot guard {g}"), 600) { break; }
        }
    }
    // Leg 3: the Load Point #158 and the E press ([M 0x1a5170 case 0x1d → 0x1b2318]).
    // The exit that loads is #193 "DOOR_loadlevel" (key gate 0): #158's key gate ch 0x1E [D] is off
    // at the press on this route, so `Trigger_Locked` 0x1b2fc8 refuses #158 [M 0x1b2318:76-90].
    // The use pick is the glist sphere test at head + look, radius 1.0, front side only [M
    // Player_Activate 0x1a5170:100-108 → Intersect_PointGeom 0x1e92d8: plane distance in (0, 1.0]];
    // #193's +Z faces sit at z −4.00 over x 14.00..14.62 [D tris], so Bond walks in at x 14.3 and
    // ends facing −Z with the head near z −2.8 (the old 0.9 arrival left the use point 1.04 out).
    w.walk_to(Wp::new(14.3, 17.0, -2.0).r(0.3), "in front of the Load Point");
    w.walk_to(Wp::new(14.3, 17.0, -2.8).r(0.3), "the Load Point");
    // The use plane is MARGINAL at this arrival: the head must sit near z −2.8 for the ray to
    // land inside 1.0 of the #193 use point (the old 0.9 arrival left it 1.04 out), and the
    // r-0.3 arrive samples anywhere in (−3.1, −2.5) — the 2026-11-06 d2 arrival took −2.56 and
    // the E fell 1.28 out, the exit never fired [L the run log]. One deeper press-in leg puts
    // the head against the #193 face on every run (the door's +Z faces sit at z −4.00 [D tris];
    // try_walk_to skips the leg rather than wedge).
    w.try_walk_to(Wp::new(14.3, 17.0, -3.3).r(0.3), "the Load Point, pressed in");
    // latch-walks-on-master: a return-fire stand inside the pressed-in leg (the shaft-falling
    // guard #19 lands mid-descent and engages — measured at tick 52798 on the latch branch,
    // never on master where the E fires ~60 ticks earlier) leaves the fight's yaw at the
    // arrival, and the use pick (head + look, [M 0x1a5170:100-108]) then reads the room behind
    // instead of the #193 face. A player turns back to the door and presses E: face −Z with the
    // turn channel (walker_b2 face_toward), then press.
    w.face_toward(0.0, -1.0, "face the Load Point door");
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0012),
        "E at the stairwell-bottom exit #193 loads 07000012 [M 0x1b2318]");
    assert!(loaded, "the exit fired");
    // The exit's end-of-level mission rows: neither record completed here — their writers live on
    // 07000012 (#679's spawn tile) and on 07000013 (the whole-mission AND) [D §2].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // [S] unverified — NOT a pass/fail assert: "no 0x48 writer exists" rests on a missing
        // writer, and the port may simply lack it (level-11-every-objective-2). The hunt is in
        // docs/reports/level-11-every-objective-2.md.
        eprintln!("[011] [S] unverified: ch 0x48 never fired on 07000011 (expected: no writer exists)", );
        assert!(!sim.drones.channels.on(100), "no mission success on this level [D §2; M 0x1827d8:119-123]");
        assert_eq!(sim.drones.mission.states[15], [3, 3],
            "both records announced (not complete) at the exit [M 0x1827d8 case 3]");
    }
    // THE CHAIN INTO THE NEXT LEVEL (real input all the way): the exit's own next level loads
    // [M the E arm → ResetMap_LevelToLoad 0x1b24b8], and 07000012's spawn #0 (22.49, 0.19, 19.16
    // [D]) stands inside TouchOnce #679's tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62) [D] —
    // ch 0x48 fires under Bond's feet and the monitor completes the carried "Gain access to
    // lower office floor" [M 0x1827d8 case 3 → state 4]. "Escape from the tower" stays
    // announced: its writer is the whole-mission AND, which first fires on 07000013 [D §2].
    // (level-11-every-objective-3: the load is the difficulty-3 front-end path — the exact call
    // walkreal_07000012_live calibrates tower2's route on; the spawn pins here are
    // difficulty-independent [M the 0x1827d8 announce + #679's contact write].)
    let scene2 = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None,
            if difficulty == 1 { 3 } else { difficulty })
        .expect("07000012 loads after the exit");
    let mut w2 = Walker::new(scene2);
    // The precise act on the carried leg too (real damage only): the 07000012 fights are the ones
    // the 7-round budget has to survive (the front-end load re-equips weapon 0x10 with 7 [M
    // 0x1a7ef8:282-288 — case 0x7000012 RamLoads and, BL+0x27a clear, falls into the 0x7000011
    // equip]). Inert under the top-up (the fights never run there).
    // THE 07000012 LEGS' SURVIVAL KIT (civ-repin-11-13): the 2026-11-05 d1 run died at t1011 on
    // 12's lobby south — clip 3 spare 0, and the MASTER-rules resupply tour (the 45 u fetch, no
    // tried list, the master 1.5 band that also sees the lobby's buried y 0.4-0.5 ghosts) walked
    // Bond into killer #4's line (11.7 u) for 93.0 in 7 hits while it toured nothing [L the
    // civ-alarm-repin-2 death ledger]. The 07000013 walk survives the SAME tower2 legs at the SAME
    // difficulties with the lane kit on its 12 walker [L its passing d2/d3 runs] — the same three
    // levers here: the lane resupply (the 12 u fetch, the tried list, the 0.6 walk-onto arrive,
    // the inventory ammo count), the floor-drop band (this level's fight drops rest 1.09 under
    // Bond's pos y — invisible to the lane 0.5 band, so every pass came back dry while the fights
    // ran the 7 dry), and the 3.0 back-off stand (the 13 walk's shipped value on this route; the
    // failed 2026-11-05 attempt's wedge was 2.6 on the 11 STAIRWELL walker, untouched here). The
    // top-up runs never fight (the return-fire gate reads tops_up first), so the kit is inert
    // there.
    if !w2.tops_up() {
        w2.dmg.precise_fights = true;
        w2.dmg.lane_fights = true;
        w2.dmg.lane_floor_drops = true;
        w2.dmg.back_off = Some(3.0);
        if difficulty >= 2 { tower2::EAST_WING_WEST_SIDE.with(|c| c.set(true)); }
    }
    w2.wait_enabled(3000, "07000012's intro");
    w2.wait(90);
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0012, "the exit's next level [D #158 next]");
        assert!(sim.drones.channels.on(0x48),
            "#679's spawn tile fires ch 0x48 [D (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62)]");
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "'Gain access to lower office floor' COMPLETE on the next level's spawn [M 0x1827d8 case 3]");
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "'Escape from the tower' still announced (completes on 07000013) [D §2]");
    }
    // THE EXTENSION (level-11-every-objective-3): every remaining objective on real input, to the
    // mission's end — the next level's load. The ch 0x48 writer is TouchOnce #679 (pinned above;
    // the port has it — triggers.rs). Next, 0x04000037 "Activate elevator override control in
    // security center" completes on ch 0x49 at the console E (#731 [D 07000012.md §2; M
    // Trigger_Activate 0x1b2318 type 1]); the lift call #699 (ch 0x58) and the shaft entrance
    // (#683 holds ch 0x1F) arm the gated exit #669, whose auto-fire [M Trigger_Update 0x1b1a78
    // case 10] loads 0700004A — the next level's load. The route is tower2's (shared with
    // walkreal_07000012_live; its two measured stand-ins carried over — see the header);
    // tower2's opening block re-pins the spawn states on this same walker.
    tower2::walk_12(&mut w2);
    tower2::walk_12_exit(&mut w2);
    // The mission's last reachable objective: 0x04000038 "Escape through the elevator shaft"
    // (ch 0x4D) has NO writer on 07000012 [D 07000012.md §2] — its writer is TouchOnce #277 on
    // 0700004A (the shaft's first plate, y −9.87…−7.65 [D 0700004a.md §2]). The game carries the
    // RamSave through the real load [M Player_RamLoad 0x1957a8]; 4A's start #43 (4.26, −7.71,
    // −4.76 [D]) drops Bond INTO #277's volume, ch 0x4D fires, and the monitor completes the
    // record the next frame (it runs before the object handler [M 0x182c60 from 0x1c9790:17]).
    // 0x04000035 "Escape from the tower" stays announced: its writer is the whole-mission AND
    // (ch 100 [M 0x1827d8:119-123]), which first fires on 07000013 [D §2] — the mission's
    // remaining part, beyond this walk's endpoint.
    let save = {
        let sim = w2.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };
    let mut w3 = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save))
        .expect("0700004A loads after 07000012's exit"));
    w3.wait_enabled(3000, "the 4A intro");
    let mut landed = false;
    for _ in 0..600 {
        w3.tick(nf_game::player::Input::default(), false);
        if w3.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
    }
    assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
    for _ in 0..30 { w3.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w3.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "row 15: 'Gain access to lower office floor' stays complete (the RamSave carries it) [M 0x182270]");
        assert_eq!(sim.drones.mission.states[16][1], 4,
            "'Escape through the elevator shaft' completes through the real exit load (the late AND) [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][0], 4,
            "'Activate elevator override control' stays complete [M 0x182270 keeps previous parts]");
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "'Escape from the tower' still announced (its writer is 07000013's whole-mission AND) [D §2]");
    }
    eprintln!("[011] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

/// The plain Operative walk (the topped-up allowance; the load is the replay-parity path).
#[test]
fn walk_spawn_to_the_lower_floor_load_point() {
    walk_spawn_to_the_lower_floor_load_point_at(1)
}

/// Real damage, Operative: the whole walked chain (07000011 spawn → the stairwell descent → the
/// Load Point exit → the real 07000012 load → tower2's route → the shaft exit → the real 0700004A
/// load) with the health top-up off for every walker it makes — the stairwell guards'
/// (0x11/0x13/0x14/0x36/0x37 [D §5]) fire is the port's own damage — and the walker's return
/// fire on; Bond's death panics with the death spot and the last hits (common/damage_log.rs).
/// No health/armour/ammo writes anywhere (the leg marker `w.top_up_health = true` is the
/// topped-up allowance and reads off under this wrapper [M damage_log.rs no_topup_env]); the
/// loads are the game's own exit → next-level path ([M ResetMap_LevelToLoad 0x1b24b8;
/// Player_RamLoad 0x1957a8]), not teleports.
/// The fights HEAD-HUNT and answer the desk guards (`with_real_damage_head_hunting` +
/// `with_real_damage_unseen_fights`) — the same law the 07000012 walk's own real-damage sister
/// runs on the same route (walkreal_07000012_live.rs): the level's guards are the 12-health
/// type-2 skins and the mission-11 pistol deals 10.0 a torso round [L ee_start_05.bin
/// 0x2bf150 + 0x10·0x10C + 0x0c = 10.0], so a torso fight needs 2-3 rounds a guard — the
/// 2026-11-03 aimed run spent 6 of the 7 on the lobby (fight#9's hp-30 guard ate 2 torso
/// rounds for 30→10 and MELEED 14 later) and Bond crossed the south corridor with 1 round —
/// not enough for the shotgun #10 (hp 12 needs 2 torso rounds, 1 head hit [M
/// NDrone2_HitDamage 0x145558 × DroneDamage_Head]). One head hit drops a 12-health guard.
#[test]
fn walk_spawn_to_the_lower_floor_load_point_real_damage() {
    walker::with_real_damage_head_hunting(||
        walker::with_real_damage_unseen_fights(||
            walker::with_real_damage(||
                walk_spawn_to_the_lower_floor_load_point_at(1))))
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-00agent-11): the same walked
/// route at the menu's Agent pick — the front-end's launch path, GameState+0x40 = 2 [M
/// C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]. On this level group the pick scales
/// Bond's pain by Plr_DMod_Normal 0.7 (vs Operative's Plr_DMod_Easy) and the guards' hit chance
/// rides DroneFiring_Accuracy_Normal 0.7 (vs Easy 0.5) [M Player_HandlePain 0x1902c8:29-37;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88]; the min-difficulty placements pass Drone_Create's
/// gate [M 0x136be0:49]. The keypad #177 is created LOCKED at difficulty 2 (the p6 tag
/// auto-unlocks only on Operative [M 0x1ae1f8:67-69]) — the walk solves it on real input (the
/// aim + E arm, the digit presses, the case-4 match). Real input only — the load path is the
/// only other difference.
#[test]
fn walk_spawn_to_the_lower_floor_load_point_real_damage_agent() {
    walker::with_real_damage_head_hunting(||
        walker::with_real_damage_unseen_fights(||
            walker::with_real_damage(||
                walk_spawn_to_the_lower_floor_load_point_at(2))))
}

/// 00 Agent-difficulty sister of `..._real_damage_agent`: the same walked route at the menu's
/// 00 Agent pick — GameState+0x40 = 3 [M C_SBNFDFCTY 0x216ca8:15]. The pick scales Bond's pain
/// by Plr_DMod_Hard 1.2 (vs Agent's 0.7), the guards' hit chance rides
/// DroneFiring_Accuracy_Hard 1.0 (vs 0.7) and their rounds by DroneDamage_Hard 0.8 [M
/// Player_HandlePain 0x1902c8:40-41; DroneWeap_DoBulletAccuracy 0x179740:74-88; damage.rs];
/// the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input only.
///
/// UN-IGNORED (real-health-00agent-11-3, 2026-11-06): GREEN — the round-2 hole (the
/// 23.0/18.9 pairs' crossfire, the ~40-health binder) closed by the `eye_ray_stands` opt-in
/// alone: the strip guards' boxes sit behind their flight lips — the lane fights' muzzle
/// stand-in rays could never see them (the pick returned None) while their own sight cleared
/// the lip, so every crossing drained the armour with no fight triggerable. With the stands
/// answering what Bond SEES (the plain eye ray [M DamageLog::exposed_part], the opt-in flag,
/// damage_log.rs), the pairs die in the walk's own fights (the measured run: #33/#34 on the
/// 23.0 strip, #28 at flight A, #36 at the Load Point) and the walked chain reaches the Load
/// Point exit and the real 07000012 load on the 130 budget with 17.0 to spare. The frag
/// pair-break levers were built, thrown and measured THREE ways (4.7 u / 7.9 u / 9.0 u aimed)
/// and are REMOVED: the lip occludes the blast from above too (zero guard damage each time,
/// 21-63 self-cost) — the grenade lever is dead against lip-covered strips. Round 2's ledger
/// lives in docs/reports/real-health-00agent-11-2.md.
/// UN-IGNORED AGAIN (los-mask-repins-11, 2026-11-10): GREEN under the game's guard sight —
/// the fights moved onto the stairwell's connector-2 ends and the 18.9 strip (the round-3
/// ledger: Bond reached the strip at 46 health and died at tick 19894 to #19 and #36 at
/// 2.6-5.6 u). Three walk-local levers, all real input, all d3-gated:
/// 1. `hurting_first` (the 07000008 Agent sisters' opt-in): the run's death stand (fight#29)
///    held a shooter the eye rays never saw (he was walking the 23.0 strip above the flight
///    lip) while #19 (2.6 u) and #36 (the connector above) burst health 39 → 0 in 90 ticks
///    [L NF_RF_TRACE this task] — the pick now answers the shooter who is actually HURTING
///    Bond and the fight retargets every 30 ticks.
/// 2. The connector-2 TOP stands (`answer_visible_guards`): from the 23.0 strip's east end the
///    top guard is 1.7 u and the elevated well guard (#36, (19.7, ~21, 0.4)) is a down-look
///    over the same flight lip that blocks the up-look — the measured run killed #27/#26/#18
///    there before committing to the flight.
/// 3. The 18.9 strip MOUTH stands after the descent: the strip cluster (#18 the melee charger
///    at 1.6 u, #19 weapon 0x19, #28) dies from the connector's foot instead of mid-strip —
///    the stands killed #18/#19/#28 on the measured run [L this task]. `stand_and_fire`, not
///    `shoot_bounded`: the shoot loop's opening strafe drifts Bond 2 u off the mouth (the
///    second measured run died at d 1.1 of #19 that way); a player reloads between guards.
#[test]
fn walk_spawn_to_the_lower_floor_load_point_real_damage_00agent() {
    walker::with_real_damage_head_hunting(||
        walker::with_real_damage_unseen_fights(||
            walker::with_real_damage(||
                walk_spawn_to_the_lower_floor_load_point_at(3))))
}
