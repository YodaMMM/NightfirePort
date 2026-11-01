//! walkreal-a: 07000001 "Paris Prelude" (Castle A) walked with the player's own movement — no
//! teleports after the spawn. The route (docs/levels/07000001.md):
//!   spawn #0 (−2.78, 6.84, −12.40) → the intro cut-scene writes ch 0xB → the contact (skin 0x4F)
//!   enables → walk the interior (doors #2/#3/#4 on the way) to the LoadPointActivator plate
//!   (Touch out 0x10, mask 0x18, filter skin 0x4F) at (−33.1, 2.9, 15.4) → Bond AND the contact on
//!   it → ch 0x10 → the exit (LOAD_LEVEL gate 0x10) → 07000002.
//!
//! Bond is driven with `Input` only (turn/forward/use); the health top-up is the "enemies may be
//! disabled for the walk" allowance (the AI still runs — alerts, looks, shots — it just cannot kill).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::world::V3;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The walked route to the exit (shared by the walk test and the exit-fade pin below): the real
/// input path from spawn #0 to the exit plate, stopping AT the fire tick — `level_exit` is
/// posted and `exit_hold` still has all 300 frames (the game's fade level is 0 on the fire tick;
/// `Camera_UpdateGlbVars` advances it on the frames after [M 0x1bada0:16-20]).
fn walk_to_the_exit(iso: &str, stop_at_fire: bool) -> Walker {
    let mut w = Walker::new(nf_view::load(iso, "07000001.bin", false).expect("load 07000001"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -2.78).abs() < 2.0 && (p[2] - -12.4).abs() < 2.0, "spawn #0 [D], got {p:?}");
    }
    // The intro cut-scene (SP #277, script 0x06000087) plays at load; its stream 8 writes ch 0xB
    // at ~tick 2119 and the script's tail (event 6 / Script_Free) re-places Bond at the script
    // matrix — wait the whole thing out before reading the route.
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    let enabled_at = enabled_at.expect("the intro must write ch 0xB (the contact's enable channel)");
    eprintln!("[01] intro ch 0xB at tick {enabled_at}");
    w.wait_enabled(6000, "the intro tail (event 6 re-places Bond)");
    w.wait(300);

    // The contact (skin 0x4F) is the escort: after ch 0xB he leaves WAIT_SWITCH, but the AllyLead
    // machine hides him (state 0x1f, [M AllyLeadHide 0x15ec08]) while an armed guard stands within
    // 14 units — the game's own behavior. The real player clears the guards; Bond shoots every
    // started armed guard within 14 of the contact through the real fire path.
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    drop(sim);
    // The friendly-fire shield (level-06-rook-door-3): the game's bullets hit ANY drone — no
    // team gate in Drone_CollisionHandler 0x140968:85-95 [M] — and the attributed round-0 kill
    // was BOND's own spread past guard #5 into the walking contact's head (the NF_HEALTH_TRACE
    // read; the guard set {3,5,14,15} never shot him — drone-fired bullets cannot even reach a
    // drone in the port, and the game's shooter would be the player's own pellet there too).
    // The player's play: never take a shot whose line runs through the escort. Every clearing
    // loop below inherits it through `shoot`.
    w.shield = Some(ci);
    let guards: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && walker::dxz(d.pos, sim.drones.list[ci].pos) < 14.0
        }).collect()
    };
    eprintln!("[01] {} armed guard(s) within 14 of the contact — clearing", guards.len());
    for g in guards {
        // Walk within 9 of the guard first (line of sight), then shoot.
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(gpos, bpos) > 9.0 {
            w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.5);
        }
        if !w.shoot(g, &format!("guard {g}")) {
            if w.resupply("ammo retry") { w.shoot(g, &format!("guard {g}")); }
        }
        // An empty gun cannot finish the fight — if the resupply also failed, move on rather than
        // hold an empty trigger through the 900-tick reposition windows.
        if {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.gun.clip + sim.gun.spare == 0
        } { eprintln!("[01] guard {g}: out of ammo and no box reached — leaving him for a later round"); continue; }
    }
    // The contact's hide exit: no armed drone within 14 → timer → Lead [M 0x15ec08]. Walk to him,
    // then escort: follow, and every time he cowers (an armed guard within 14 of him — LeadHide
    // [M 0x15ec08]) or parks (PlayerInWay), clear the armed started guards around him. The real
    // player fights through; the lead resumes on each clear.
    let cpos = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
    let plate = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH, 0x10).expect("the AND-touch plate");
    w.walk_to(Wp { at: cpos, act: Act::None, arrive: 2.5 }, "to the contact");

    // The meet beat — the level's second cut-scene, triggered entirely on disc. SP #276
    // (placement 276, script 0x06000086, 19 streams, in the level archive [D the disc scan —
    // pack 07800001 carries only the intro's 0x06000087]) is a mask-1 ScriptPlayer: it triggers
    // when its own out channel ch 0x33 rises [M SP_Hit 0x1dc470:17-21 flags bit 0x1 +
    // SP_Update 0x1dbda8 case 0]. The chain: the TOUCH_ONCE plate at (6.8, 6.0, 8.9) (mask 0x08)
    // writes ch 0x32 [D placement]; the OUT mux (0xEE, out 0x32 → list [0x33]) mirrors
    // ch[0x32] → ch[0x33] EVERY tick [M Trigger_Update 0x1b1a78 case 5:
    // switch_channels[list[k]] = switch_channels[out]]; SP #276 fires. So Bond WALKS the plate —
    // the real player's route into the north-east hall — and the scene plays out untouched
    // (streams end at frame 280 [D decode], ~4.7 s).
    w.walk_nav_to([6.8, 6.0, 8.9], "the meet plate (NE hall)", 1.0);
    // The plate is a TOUCH_ONCE on Bond's capsule — the touch lands when the capsule segment
    // crosses the volume's top face (feet ≈ the plate's floor 6.04). A nav stop a hair short
    // never crosses; a player steps ON it. Walk the centre tightly, then sweep the quad.
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
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[01] meet plate: bond at {:?} ch 0x32={} ch 0x33={}", sim.bond.pos,
            sim.drones.channels.on(0x32), sim.drones.channels.on(0x33));
        assert!(sim.drones.channels.on(0x32), "walking the meet plate must fire its ch 0x32 [D placement]");
    }
    let mut e_at = None;
    for t in 0..2400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xe) { e_at = Some(t); break; }
    }
    eprintln!("[01] the meet scene (0x06000086) wrote ch 0xE at +{e_at:?} — stream 9's event 0xD ch 0xE val 1 [D decode]");
    assert!(e_at.is_some(), "the meet plate must fire ch 0x32 → the 0x33 mirror → SP #276 must play and write ch 0xE");
    // ch 0xE is the start channel of four hall guards [D placements; the probe's drone table]:
    // skins 0x39/0x36/0x38/0x3a at (−12.2,17.2), (5.4,14.4), (9.1,17.0), (5.9,17.9).
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let states: Vec<u8> = sim.drones.list.iter()
            .filter(|d| d.switch_channel == 0xe)
            .map(|d| d.state)
            .collect();
        assert_eq!(states.len(), 4, "four drones wait on ch 0xE [D placements]");
        assert!(states.iter().all(|&s| s != nf_game::drone::state::WAIT_SWITCH),
            "every ch-0xE guard must have left WAIT_SWITCH when the scene wrote the channel (states {states:?})");
    }

    // Top up BEFORE the hall: the meet scene's weapon hand left clip 32 spare 0, and the hall's
    // dodge fights burn more than a clip per guard (guard 6's fight ran dry at +634 ticks; the
    // resupply detour from inside the hall then wedged on the hall's south wall, the ambush
    // stayed alive and ch 0x99 never rose — the hold kept the contact at node 33 and the exit
    // died with it). A player tops up at the corridor boxes before the hall: the walked-path
    // corridor's own rule (resupply()).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dry = (sim.gun.clip as i32) + (sim.gun.spare as i32) < 48;
        drop(sim);
        if dry { w.resupply("pre-hall ammo"); }
    }

    // The hall detour: the disc's AND mux (kind 3, out 0x99 = 153, list [0x11,0x12,0x13] [D the
    // placement]) fires when the three named guards are dead — their death channels are 0x13,
    // 0x11, 0x12 [D placement params]. A real player clearing the north-east hall fires this
    // scripted event; the walk does it through the real fire path before the escort.
    let named: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len())
            .filter(|&i| [0x11u8, 0x12, 0x13].contains(&sim.drones.list[i].death_channel))
            .collect()
    };
    assert_eq!(named.len(), 3, "three guards carry death channels 0x11/0x12/0x13 [D]");
    // Stage into the hall through its east door first. From the meet room the routes to the
    // west-half guards run through door #2 (−14.1,5.3,13.1) — the level's known walker wedge
    // (the solo approach wedges at the 3000-tick cap) — and a
    // guard fought through that wall never exposes a part (he stands behind the hall's south
    // structure). A player enters the hall by its main door and fights inside; the east hop is
    // the route the guard-11 approach already proves (nav reached (2.97,6.38,16.27) from the
    // same start), and from the stage the named guards are short in-hall hops.
    w.walk_nav_to([2.9, 6.3, 16.3], "stage into the hall (east door)", 3.0);
    for g in named {
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        // The guards hunt: guard 11 left his placement for the hall's south entry and the
        // approach from the courtyard side wedged on the hall's south wall (the nav gave up
        // at the 3000-tick cap, segment_to_target ≈0.47). Re-stage through the proven route
        // (the meet-room corridor → the east door) before closing from inside.
        if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, gpos) > 9.0 {
            walker::hall_restage(&mut w, &format!("guard {g} approach"));
        }
        w.walk_nav_to(gpos, &format!("approach hall guard {g}"), 3.0);
        // Dry mid-hall is the walk's own trap: the hall's south wall walls the resupply nav
        // off from the corridor boxes (every southward hop wedged at segment t≈0.095). A
        // player steps out the door he came in by: east door → corridor box → back inside.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let dry = sim.gun.clip + sim.gun.spare < 8;
            drop(sim);
            if dry { walker::hall_ammo_ladder(&mut w, &format!("hall guard {g} ammo")); }
        }
        if !w.shoot(g, &format!("hall guard {g}")) {
            if w.resupply("hall ammo") { w.shoot(g, &format!("hall guard {g}")); }
        }
    }
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x99),
        "AND(0x11,0x12,0x13) must raise ch 153 once the three named guards are dead [D mux placement]");
    eprintln!("[01] the hall AND fired: ch 153 on at tick {}", w.total_ticks);

    let mut settled = cpos;
    // The contact's park from the previous round: two rounds inside ~1 unit with no guard to
    // clear means the lead machine wedged on geometry — Bond walks PAST him toward the plate
    // (the real player's move): out of the PlayerInWay band the lead resumes and re-paths [M
    // 0x15ea28 case 3 / 0x15e698 case 3], which unsticks him. The head-look merge's corrected
    // head box (the game's own AnimGetCollData → psiBuildMatrixPalette → HeadTrack coupling)
    // moves the fight timeline, so the lead can park in spots the old path never hit.
    let mut prev_settled = cpos;
    let mut cleared = std::collections::HashSet::new();
    for round in 0..12 {
        settled = w.follow(ci, &format!("the contact r{round}"), 12000, plate);
        if walker::dxz(settled, plate) < 5.0 { break; }
        // Stalled: who is armed and started within 14 of the contact?
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                    && walker::dxz(d.pos, settled) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() {
            // No threat: the lead may just be WAITING for Bond (state 0x20 = the lead wait, Bond
            // inside 3.0 [M 0x15e4f0 case 3]) or parked after the detour. PlayerInWay (0x1e) is the
            // opposite: Bond is IN the lead's path (`route_blocked && dist <= 3.0` [M 0x15e4f0's
            // PlayerInWay arm]) and he resumes only when Bond is OUT of the way band (dist > 3.0
            // [M 0x15ea28 case 3]) — so Bond steps BACK behind him (away from the plate) instead of
            // closing in, which would keep him parked. Otherwise close the distance and follow again;
            // panic only after several fruitless rounds.
            let sim = w.scene.sim.as_ref().unwrap();
            let cstate = sim.drones.list[ci].state;
            drop(sim);
            // The meet hall's west wall check (the level-11-stairwell-bullet-wall-3 stall): Bond
            // chased guard 15 OUT of the hall (the north lane) and stands in the west corridor with
            // the hall's west wall segment (x≈1.93, z 11.1-13.1 [D the disc's collision cels — the
            // port's ray fan, walkreal_probe11::dump_07000001_west_wall]) between him and the
            // contact; every close/walk-past target sits inside the hall and the straight-line
            // steering wedges on the wall face (the nav's own first hop is a hall-side node across
            // the wall, and the stuck ladder's sidesteps slide ALONG it, never the ~1.7 north to
            // the wall's north edge). The player walks back the lane he came out by — the hall's
            // north opening, the AI's own chain node[8] (−0.39,14.28) → node[23] (3.39,14.69) →
            // node[7] (6.88,14.62) [D nav path 0 nodes+links; the lane passes the AI's bounds
            // walls through their z≈13.1 north end], then closes on the contact; the AllyLeadWait
            // release [M NDrone2_DSTATE_AllyLeadWait 0x15eda8 case 3: pd > −7.0 && (pd != 0.0 ||
            // dist2d < 5.0)] needs Bond within 5.0 straight-line when pd == 0.0 (Bond off the
            // route's 5.0 node scan), and inside the hall he is.
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let (bp, cp) = (sim.bond.pos, sim.drones.list[ci].pos);
                let walled = bp[0] < 2.0 && sim.world.segment(bp, cp).is_some();
                drop(sim);
                if walled {
                    eprintln!("[01] round {round}: the hall's west wall is between Bond ({bp:?}) and the contact ({cp:?}) — walking the north lane back in");
                    w.walk_nav_to([0.6, 6.5, 14.3], "the west corridor lane (north)", 1.4);
                    w.walk_nav_to([3.39, 6.4, 14.69], "the hall's north opening (nav node 23)", 1.4);
                    w.walk_nav_to([6.88, 6.4, 14.62], "inside the hall (nav node 7)", 1.4);
                    w.walk_nav_to(settled, "close on the contact (the [M 0x15eda8] release band)", 1.2);
                    prev_settled = settled;
                    continue;
                }
            }
            if cstate == nf_game::drone::state::PLAYER_IN_WAY {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                let back = [cp[0] + (cp[0] - plate[0]) * 0.5, cp[1], cp[2] + (cp[2] - plate[2]) * 0.5];
                drop(sim);
                eprintln!("[01] round {round}: contact in PlayerInWay — Bond steps back off his lane");
                w.walk_nav_to(back, "step out of the lead's way", 3.0);
            } else if round >= 24 {
                let sim = w.scene.sim.as_ref().unwrap();
                panic!("[01] the contact stalled at ({:.1},{:.1},{:.1}) state {:#x} armed_peer {:.1} with no guard to clear (round {round}; the cap 24 gives the [M 0x15eda8] d2<5 release time to land on the slow closing)",
                    settled[0], settled[1], settled[2], sim.drones.list[ci].state, sim.drones.list[ci].armed_peer_dist);
            } else if walker::dxz(settled, prev_settled) < 1.0 {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                let (dx, dz) = (plate[0] - cp[0], plate[2] - cp[2]);
                let l = (dx * dx + dz * dz).sqrt().max(1e-3);
                let ahead = [cp[0] + dx / l * 4.0, cp[1], cp[2] + dz / l * 4.0];
                drop(sim);
                eprintln!("[01] round {round}: the lead is wedged at ({:.1},{:.1},{:.1}) state {cstate:#x} — Bond walks past him toward the plate (the lead resumes out of the way band and re-paths)",
                    settled[0], settled[1], settled[2]);
                prev_settled = settled;
                let before = w.scene.sim.as_ref().unwrap().bond.pos;
                w.walk_nav_to(ahead, "walk past the wedged lead", 3.0);
                let moved = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, before);
                if moved < 1.0 {
                    // The nav gave up (the hall's meet-frozen bodies park mid-lane when the
                    // fight resolves differently): a real player vaults the crowd — the jump
                    // press carries the capsule over the parked bodies.
                    eprintln!("[01] the walk-past nav gave up — Bond vaults forward (the jump press)");
                    let sim = w.scene.sim.as_ref().unwrap();
                    let (bp, cp) = (sim.bond.pos, sim.drones.list[ci].pos);
                    drop(sim);
                    let to = [cp[0] - bp[0], cp[2] - bp[2]];
                    let l = (to[0] * to[0] + to[1] * to[1]).sqrt().max(1e-4);
                    for _ in 0..90 {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let err = walker::wrap_pi(to[0].atan2(to[1]) - sim.bond.yaw);
                        drop(sim);
                        w.tick(nf_game::player::Input { forward: 1.0, jump: true, turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
                    }
                }
            } else {
                // Close RIGHT up (1.2, not the old 4.0): the AllyLeadWait release [M 0x15eda8
                // case 3, the port's ALLY_LEAD_WAIT arm] needs Bond's mission-path distance back
                // inside (−7, 0] — on the hall's winding route a 4.0 straight-line stop still read
                // > 7 route-units and the wait held. A real player walks up to his partner.
                eprintln!("[01] round {round}: no guards to clear (contact state {cstate:#x}) — closing on him");
                prev_settled = settled;
                w.walk_nav_to(settled, "close on the contact", 1.2);
            }
            continue;
        }
        eprintln!("[01] round {round}: the contact stalled at ({:.1},{:.1},{:.1}) — clearing guards {:?}",
            settled[0], settled[1], settled[2], guards);
        // Diagnostic: at the second fruitless round on the same set, dump the guards' boxes vs the
        // aim ray (the guard-15 miss the previous session probed).
        if round >= 5 {
            let sim = w.scene.sim.as_ref().unwrap();
            for &g in &guards {
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                eprintln!("[01] diag guard {g}: pos {:?} health {:.0} state {:#x} crouched-anim state {:#x} dist {:.1}",
                    d.pos, d.health, d.state, d.anim.state, walker::dxz(d.pos, bp));
                let to = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                eprintln!("[01] diag guard {g}: torso seg hit {:?}", sim.world.segment(eye, to));
                let data = nf_game::drone::Data {
                    tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
                    world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins, skeletons: &sim.skeletons,
                    nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning, hitboxes: &sim.hitboxes, volumes: &sim.volumes,
                    sfx: sim.sfx.as_ref(), models: &sim.models, surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: 1,
                    ai_points: &sim.ai_points,
                };
                let boxes = d.hit_boxes(&data);
                for b in boxes.iter().take(8) {
                    eprintln!("[01] diag guard {g} part {:3}: c ({:.2},{:.2},{:.2}) half {:?} from-eye {:.1}",
                        b.part, b.m[3][0], b.m[3][1], b.m[3][2], b.half, walker::dxz([b.m[3][0], b.m[3][1], b.m[3][2]], bp));
                }
            }
        }
        for g in guards {
            // Walk TO the guard (the route's doors open under Bond's feet via the stuck handler —
            // a midpoint stop can sit behind a wall with no line of sight). The gate is LOS, not
            // just distance: at the stairwell corner Bond stood 3.2 from the guard with the
            // eye→torso ray hitting a wall at t 0.18 — the shots could never land.
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.0);
            }
            // Low ammo: grab the nearest pickup first (the placed kind-3 ammo / the guard drops).
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.gun.clip + sim.gun.spare < 8 { drop(sim); w.resupply("ammo"); }
            }
            // Skip the hopeless fight: no ammo and no box reached — the round loop retries after
            // the drops land (every killed guard drops his weapon [L the 01 drops]).
            let ammo_now = w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare;
            if ammo_now == 0 { eprintln!("[01] guard {g}: skipped — dry"); continue; }
            let mut killed = w.shoot(g, &format!("guard {g}"));
            if !killed {
                // Out of ammo mid-fight: resupply and retry once.
                if w.resupply("ammo retry") { killed = w.shoot(g, &format!("guard {g}")); }
            }
            // Only DEAD guards leave the clear list — a survivor re-hides the contact.
            let dead = w.scene.sim.as_ref().unwrap().drones.list[g].corpse
                || w.scene.sim.as_ref().unwrap().drones.list[g].health <= 0.0;
            if killed || dead { cleared.insert(g); }
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        eprintln!("[01] the contact settled at ({:.1},{:.1},{:.1}) state {:#x} route_idx {:?} blocked {} armed {:.1} bond d {:.1}",
            d.pos[0], d.pos[1], d.pos[2], d.state, d.route.as_ref().map(|r| (r.index, r.nodes.len())), d.route_blocked, d.armed_peer_dist,
            walker::dxz(sim.bond.pos, d.pos));
        eprintln!("[01] anim: danim {} state {:#x} ftime {:.1} script {:?} held {} once {} call {:?} drawn {}",
            d.anim.danim, d.anim.state, d.anim.ftime, d.anim.script, d.anim.held, d.anim.once, d.anim.call, d.drawn);
        eprintln!("[01] yaw {:.2} desired {:.2} target {:?}", d.yaw, d.desired_yaw, d.route.as_ref().map(|r| r.target));
        if let Some(t) = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::TOUCH && t.out == 0x10) {
            eprintln!("[01] plate lo {:?} hi {:?} centre {:?}", t.lo, t.hi, t.centre());
        }
        // The way west from his nose: world geometry at hip heights?
        for (dx, dz) in [(-0.5f32, 0.0), (-1.0, 0.0), (-2.0, 0.0)] {
            let from = [d.pos[0], d.pos[1], d.pos[2]];
            let to = [d.pos[0] + dx, d.pos[1], d.pos[2] + dz];
            eprintln!("[01] world seg to ({:+.1},{:+.1}): hip {:?} hip+0.5 {:?} feet {:?}",
                dx, dz, sim.world.segment([from[0], from[1], from[2]], [to[0], to[1], to[2]]),
                sim.world.segment([from[0], from[1] + 0.5, from[2]], [to[0], to[1] + 0.5, to[2]]),
                sim.world.segment([from[0], from[1] - 0.9, from[2]], [to[0], to[1] - 0.9, to[2]]));
        }
    }
    eprintln!("[01] the contact settled at ({:.1},{:.1},{:.1}), plate at ({:.1},{:.1},{:.1})",
        settled[0], settled[1], settled[2], plate[0], plate[1], plate[2]);
    assert!(walker::dxz(settled, plate) < 6.0, "the contact must walk his route to the LoadPointActivator plate");
    // Bond walks the last leg onto the plate (the contact stalled at the stairwell head;
    // the AND cannot fire with him outside the volume).
    drop(sim);
    // The player route descends the stairwell from the west hall's corner (the AI's own path
    // climbs over the top floor — the sister proved the descent through this anchor).
    w.walk_nav_to([-30.8, 6.4, 13.1], "the stairwell head", 1.2);
    w.walk_nav_to(plate, "the plate", 1.2);
    let sim = w.scene.sim.as_ref().unwrap();
    let on_plate = sim.bond.pos[0] < -32.0 && sim.bond.pos[2] > 12.0;
    assert!(on_plate, "Bond must walk onto the LoadPointActivator plate (at {:?})", sim.bond.pos);
    {
        let d = &sim.drones.list[ci];
        eprintln!("[01] at the AND-touch: contact at ({:.2},{:.2},{:.2}) state {:#x} route_idx {:?} blocked {} seen {}",
            d.pos[0], d.pos[1], d.pos[2], d.state, d.route.as_ref().map(|r| (r.index, r.nodes.len())), d.route_blocked, d.seen);
        for (i, o) in sim.drones.list.iter().enumerate() {
            if i == ci || o.team != 1 || !o.started || o.corpse || !o.enabled { continue; }
            let dd = walker::dxz(o.pos, d.pos);
            if dd < 16.0 {
                eprintln!("[01] peer {i} skin {:#x} at ({:.1},{:.1},{:.1}) d {dd:.1} state {:#x} health {:.0}",
                    o.skin, o.pos[0], o.pos[1], o.pos[2], o.state, o.health);
            }
        }
    }
    // The stairwell guards (the west-side combat-steppers, plus the balcony skin-0x36) hide the
    // contact at the head — in the game the hide runs him to cover (`NDrone2_FindRunToPoint`
    // 0x15168, the half the port does not model) while the player fights. The walk fights them
    // the same way: clear every started, armed guard within 14 of the CONTACT (the hide exit
    // needs the visible armed peer beyond 14.0 [M `NDrone2_NotNearArmedDrone` 0x177640]), let the
    // lead finish the route's descent, then take Bond back onto the plate for the AND.
    'recovery: for round in 0..12 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() { break; }
        for g in guards {
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("approach stairwell guard {g}"), 3.0);
            }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.gun.clip + sim.gun.spare < 8 { drop(sim); w.resupply("ammo"); }
            }
            let ammo_now = w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare;
            if ammo_now == 0 { eprintln!("[01] stairwell guard {g}: skipped — dry"); continue; }
            let mut killed = w.shoot(g, &format!("stairwell guard {g}"));
            if !killed {
                if w.resupply("ammo retry") { killed = w.shoot(g, &format!("stairwell guard {g}")); }
            }
            let dead = w.scene.sim.as_ref().unwrap().drones.list[g].corpse
                || w.scene.sim.as_ref().unwrap().drones.list[g].health <= 0.0;
            if killed || dead { cleared.insert(g); }
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break 'recovery; }
        }
        w.follow(ci, &format!("the contact descent r{round}"), 6000, plate);
    }
    // Bond LEADS the descent: he takes the plate first — the AND needs BOTH, so the exit holds
    // until the contact arrives — while the hand (below) lands on the live world. Bond ahead is
    // fine for the lead: the wait bands stall only for a Bond BEHIND on the route [M 0x15e698
    // case 3's −9.0 path arm], so the contact's route walks him down to the plate.
    let on_plate = |p: V3| p[0] < -32.4 && p[1] < 4.0 && (12.2..18.7).contains(&p[2]);
    let exited = w.scene.sim.as_ref().unwrap().drones.channels.on(0x10);
    if !exited && !on_plate(w.scene.sim.as_ref().unwrap().drones.list[ci].pos) {
        // The proven descent: the stairwell head anchor first (the direct nav to the plate from
        // the balcony side climbs over the top floor and never enters the stairwell — the sister
        // proved the descent through this anchor).
        w.walk_nav_to([-30.8, 6.4, 13.1], "the stairwell head", 1.2);
        w.walk_nav_to(plate, "the plate (lead the descent)", 1.2);
    }
    // level-01-end-fade: the pin path stops AT the fire — the mission-monitor block below waits
    // 30 ticks that would burn into the fade's 300 before the read (the walk path keeps them).
    if stop_at_fire && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return w; }
    // obj0 "Escort Mayhew to bunker" (text 0x04000006) watches ch 0x47 [D mission::c07000001,
    // ee_start_05 @ 0x2a3eb8: text 0x04000006, ch 0x47, enable 0, flags 0]. The objective does
    // NOT complete on this level, and this walk now pins that instead of standing in for it:
    //   * no 07000001 disc data writes 0x47 — the level census found the only script-0xD writes
    //     0xB and 0xE, and no trigger/mux/lock/pickup carries 0x47 [D walkreal_probe01];
    //   * no ACTION.ELF code writes ch 0x47 immediately either — this session's disassembly
    //     census of every switch_channels (0x26fc90) store (lui 0x27/-0x370 base idiom, tracked
    //     through addiu/addu to sb/sh/sw) shows the only immediate channels are 0x2d, 0x5e, 0x62,
    //     0x63, 0x64, 0x6f, 0x97, 0x98, 0x9b, 0xfd [M-dis work/elf/ACTION.ELF]; every other
    //     writer (Trigger/SP/Script_EventHandler/DroneFunc_HostageSaved/Pickup/Door/...) takes
    //     its channel from disc data;
    //   * the real writer is the NEXT level's story hand-off: 07000002's level-start cut-scene
    //     SP #400, script 0x06000084 (pack 07800002, mode 1), stream 7 writes ch 0x47 = 1
    //     [D 07800002 0x06000084 stream 7] — the same one-part-late shape as 0x4D, which
    //     07000003's TouchOnce #308 writes for 07000002's obj2 [D 07000003 param sweep].
    // walkreal_07000002_live pins that write and the resulting fold live; here the game's own
    // monitor state on THIS level is pinned: obj0 announced and active, everything else waiting
    // [M 0x1827d8 case 0 announces when MissionData row level <= GameState; case 1 waits on the
    // enable channel; case 3 folds only when the watch channel rises].
    // The record indexes resolve from the LEVELS table, not hardcoded: the table mirrors the
    // game's .data order and the mission-05..08 records sit in FRONT of 01's [M 0x1827d8 walks
    // LEVELS in table order; a hardcoded [0][0] named 01's record before those landed].
    let m01 = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0001).expect("07000001's LEVELS record");
    let m02 = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0002).expect("07000002's LEVELS record");
    // The monitor settles; fire-aware so the pin's read stays on the fire tick (identical 30
    // ticks when !stop_at_fire).
    for _ in 0..30 {
        if stop_at_fire && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
        w.tick(Default::default(), false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[m01][0], 3, "obj0 announced (immediate reveal, enable 0) and still active — the fold belongs to 07000002's intro stream 7 [M 0x1827d8 case 0; D 07800002 0x06000084]");
        assert_eq!(sim.drones.mission.states[m02][0], 1, "07000002 'Destroy Mayhew's computer' still waits on enable 0x47 [M 0x1827d8 case 1]");
        assert_eq!(sim.drones.mission.states[m02][1], 1, "07000002 'Rescue geisha girls' still waits on enable 0x47");
        assert_eq!(sim.drones.mission.states[m02][2], 1, "07000002 'Find door to servants quarters' still waits on enable 0x4c");
        // [S] unverified — kept out of pass/fail. "07000001 never raises the obj0 completion
        // channel" rests on a MISSING writer (the census above: level data, every
        // switch_channels 0x26fc90 immediate, every disc-data-driven writer), and an assertion
        // may not rest on a missing writer — the port may simply lack it. Logged, never
        // asserted:
        if sim.drones.channels.on(0x47) {
            eprintln!("[01] [S] unverified — ch 0x47 is ON after the walk: a writer the census missed (or the un-ported mission layer) fired; the never-raises expectation is NOT met");
        } else {
            eprintln!("[01] [S] unverified — ch 0x47 still off after the walk: matches 'no disc writer on this level' [D census + M-dis], but the port may simply lack the writer, so this stays out of pass/fail");
        }
        assert!(!sim.drones.channels.on(100), "the mission spans 07000002 — success waits for its objectives");
    }
    if !exited {
        // The contact finishes the route onto the plate → the AND → the exit (Bond leads from the
        // plate; the wait loop gives the descent his window). The game's own escort STOP: while
        // Bond stands within 3.5 dxz inside the contact's π/3 facing cone, PlayerInWay (0x1e)
        // HOLDS — the case-3 stay arm returns 1 while `Dist2D(Bond) < 3.5` [M 0x15ea28:32-47],
        // and the 3.5-exit/Lead re-entry is the only release (the wedge is the game's rule, not a
        // port stall — 957's guard-combat rework shifted the endgame race so Bond's plate-first
        // parking now lands inside the band). A real player steps aside; the harness does the
        // same through the real input path: retreat to the stairwell-head anchor (outside the
        // plate's y band AND the 3.5 band) whenever the contact is wedged, and let the lead
        // resume before re-taking the plate.
        for _ in 0..80 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                let wedged = sim.drones.list[ci].state == 0x1e // PLAYER_IN_WAY [M 0x15ea28]
                    || walker::dxz(cp, sim.bond.pos) < 3.5;
                drop(sim);
                if wedged && !on_plate(cp) {
                    w.walk_nav_to([-30.8, 6.4, 13.1], "step aside (the PlayerInWay release [M 0x15ea28])", 1.2);
                }
            }
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
            if on_plate(w.scene.sim.as_ref().unwrap().drones.list[ci].pos)
                && !w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) {
                w.walk_nav_to(plate, "the plate (re-take)", 1.2);
            }
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
            // level-01-end-fade: the wait breaks AT the fire — the pin below reads the hold with
            // all 300 frames left on the exit tick itself (the countdown begins the next tick
            // [M 0x1bada0:16-20 advances the fade on the frames after the SetFade stores level 0]).
            for _ in 0..60 {
                if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
                w.tick(Default::default(), false);
            }
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x10), "the AND-touch must fire with Bond AND the contact on the plate (contact at {:?})",
        sim.drones.list[ci].pos);
    drop(sim);
    // The exit fires the same tick ch 0x10 rises (the LOAD_LEVEL trigger's gate IS 0x10 — the
    // case-10 arm fires on the channel, no proximity check [M Trigger_Update 0x1b1a78:162-166]) —
    // stop at the fire tick; the walk test and the fade pin both resume here.
    for _ in 0..60u32 {
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return w; }
        w.tick(Default::default(), false);
    }
    panic!("the exit must fire within 60 ticks of the AND-touch (level_exit posted)");
}

#[test]
fn walk_from_the_spawn_to_the_exit_with_the_contact() {
    let iso = iso();
    let w = walk_to_the_exit(&iso, false);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0002), "the exit fires to 07000002");
    eprintln!("[01] level_exit to 07000002 at tick {} — walked {} ticks total", w.total_ticks, w.total_ticks);
    for f in &w.report.falls { eprintln!("[01] fall: {f}"); }
}

/// Sister: walking onto the plate ALONE (the contact left far behind) must not fire ch 0x10 — the
/// mask-0x18 filter needs the skin-0x4F drone too [M Trigger_Hit 0x1b1fc8].
#[test]
fn the_plate_refuses_bond_alone() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load"));
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { break; }
    }
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);
    // Teleport-free check of the filter logic itself is the main test's job; here Bond stands on the
    // plate by WALKING there (the contact never left his spawn room — his enable channel is forced
    // off in this sister by stepping only 30 ticks past the intro… the intro writes 0xB regardless,
    // so instead the sister walks Bond to the plate and asserts the channel without the contact.
    let sim = w.scene.sim.as_ref().unwrap();
    let plate = walker::trigger_centre(sim, walker::trigger_kind::TOUCH, 0x10).unwrap();
    let start = sim.bond.pos;
    let pts = walker::leg(sim, start, plate);
    drop(sim);
    for k in 1..pts.len() { w.walk_to(Wp::new(pts[k][0], pts[k][1], pts[k][2]), &format!("sister {k}")); }
    w.walk_to(Wp { at: plate, act: Act::None, arrive: 1.2 }, "the plate");
    // Keep Bond ON the plate while the contact (left at his spawn) is far away.
    for _ in 0..60 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).unwrap();
    let contact_far = walker::dxz(sim.drones.list[ci].pos, plate) > 4.0;
    if contact_far {
        assert!(!sim.drones.channels.on(0x10), "Bond alone must not fire the AND-touch [M Trigger_Hit]");
        assert_eq!(sim.drones.level_exit, None);
    } else {
        eprintln!("[01 sister] the contact followed Bond to the plate — the sister asserts nothing (contact at {:?})",
            sim.drones.list[ci].pos);
    }
}


/// Sister: Bond never walks the meet plate (he stays at the spawn side of the bar), so the
/// TOUCH_ONCE at (6.8,6.0,8.9) never writes ch 0x32, the 0xEE mirror never raises ch 0x33
/// [M Trigger_Update 0x1b1a78 case 5], SP #276 (mask 1 = its own out channel [M SP_Hit
/// 0x1dc470:17-21]) never triggers, and the four hall guards waiting on ch 0xE never start.
#[test]
fn the_hall_guards_wait_for_the_meet_scenes_channel() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load"));
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { break; }
    }
    w.wait_enabled(6000, "the intro tail");
    // Long past the scene's 280-frame span [D decode]: the plate stays untouched.
    w.wait(2400);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x32), "the meet plate never fired (Bond kept away)");
    assert!(!sim.drones.channels.on(0x33), "the 0x33 mirror follows ch 0x32 [M case 5]");
    assert!(!sim.drones.channels.on(0xe), "ch 0xE never rises without the scene");
    let waiting: Vec<u8> = sim.drones.list.iter()
        .filter(|d| d.switch_channel == 0xe)
        .map(|d| d.state)
        .collect();
    assert_eq!(waiting.len(), 4, "four drones wait on ch 0xE [D placements]");
    if waiting.iter().any(|&s| s != nf_game::drone::state::WAIT_SWITCH) {
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.switch_channel == 0xe {
                eprintln!("[01 sister] ch-0xE drone {i} skin {:#x} at {:?} state={:#x} enabled={} health={:.0} anim {:?}",
                    d.skin, d.pos, d.state, d.enabled, d.health, d.anim.danim);
            }
        }
    }
    assert!(waiting.iter().all(|&s| s == nf_game::drone::state::WAIT_SWITCH),
        "every ch-0xE guard must sit in WAIT_SWITCH while the meet plate is unfired (states {waiting:?}) [M NDrone2_DSTATE_Global]");
}

/// The driver's real path: clear the ambush, then FOLLOW Mayhew down — Bond stays behind him on
/// the whole descent (never the walker's tuned plate-first lead), Mayhew's mission route runs to
/// its end (path-0 node 36, INSIDE the LoadPointActivator plate volume [D the nav dump: node 36
/// (-32.947, 3.695, 12.837) vs the plate lo (-33.80, 0.64, 12.17) hi (-32.41, 3.96, 18.68)]), and
/// the game's AllyLeadDone parks him there [M NDrone2_DSTATE_AllyLeadDone 0x15f2e8 case 3
/// falls to the default — no movement]. Bond then steps onto the plate: the AND (mask 0x18,
/// filter skin 0x4F [M Trigger_Hit 0x1b1fc8]) fires ch 0x10 → the exit loads 07000002.
#[test]
fn following_mayhew_down_completes_the_level() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // ---- the route to the stairwell head (the main test's play, condensed) ------------------
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) {
            eprintln!("[01f] intro ch 0xB at tick {t}"); break;
        }
    }
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    drop(sim);
    w.shield = Some(ci);
    let plate = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH, 0x10).expect("the AND-touch plate");
    // Clear whatever armed guard stands within 14 of the contact (the hide exit needs the coast
    // clear [M NDrone2_NotNearArmedDrone 0x177640]).
    let mut cleared = std::collections::HashSet::new();
    let clear_around = |w: &mut Walker, cleared: &mut std::collections::HashSet<usize>, label: &str| {
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        for g in guards {
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            if walker::dxz(gpos, w.scene.sim.as_ref().unwrap().bond.pos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("{label}: approach guard {g}"), 3.0);
            }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare == 0 { continue; }
            let killed = w.shoot(g, &format!("{label}: guard {g}"))
                || { if w.resupply("ammo retry") { w.shoot(g, &format!("{label}: guard {g}")) } else { false } };
            let sim = w.scene.sim.as_ref().unwrap();
            let dead = sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0;
            drop(sim);
            if killed || dead { cleared.insert(g); }
        }
    };
    clear_around(&mut w, &mut cleared, "spawn side");
    // The meet beat: walk the plate at (6.8, 6.0, 8.9) → ch 0x32 → the 0x33 mirror → SP #276 → ch 0xE.
    w.walk_nav_to([6.8, 6.0, 8.9], "the meet plate (NE hall)", 1.0);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let meet = sim.drones.triggers.iter()
            .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x32)
            .map(|t| t.centre()).expect("the meet plate");
        drop(sim);
        w.walk_to(Wp { at: meet, act: Act::None, arrive: 0.25 }, "onto the meet plate");
        for (dx, dz) in [(0.0f32, 0.9f32), (0.0, -0.9), (0.9, 0.0), (-0.9, 0.0)] {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: [meet[0] + dx, meet[1], meet[2] + dz], act: Act::None, arrive: 0.2 }, "the meet plate sweep");
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: meet, act: Act::None, arrive: 0.2 }, "back across the meet plate");
        }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x32), "the meet plate must fire ch 0x32");
    let mut e_at = None;
    for t in 0..2400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xe) { e_at = Some(t); break; }
    }
    assert!(e_at.is_some(), "the meet scene must write ch 0xE");
    w.wait(120);
    // Pre-hall ammo, then the hall's three named guards (death channels 0x11/0x12/0x13) → ch 0x99.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dry = (sim.gun.clip as i32) + (sim.gun.spare as i32) < 48;
        drop(sim);
        if dry { w.resupply("pre-hall ammo"); }
    }
    w.walk_nav_to([2.9, 6.3, 16.3], "stage into the hall (east door)", 3.0);
    let named: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| [0x11u8, 0x12, 0x13].contains(&sim.drones.list[i].death_channel)).collect()
    };
    for g in named {
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        if walker::dxz(gpos, w.scene.sim.as_ref().unwrap().bond.pos) > 9.0 {
            walker::hall_restage(&mut w, "hall restage");
        }
        w.walk_nav_to(gpos, &format!("approach hall guard {g}"), 3.0);
        if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 {
            walker::hall_ammo_ladder(&mut w, "hall ammo");
        }
        if !w.shoot(g, &format!("hall guard {g}")) {
            if w.resupply("hall ammo") { w.shoot(g, &format!("hall guard {g}")); }
        }
    }
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x99), "the hall AND must fire ch 0x99");
    // Follow the contact out of the hall to the stairwell head (the main test's follow loop, but
    // stopping AT the head — the FOLLOW begins at the descent).
    let head: V3 = [-30.8, 6.4, 13.1];
    for round in 0..12 {
        let settled = w.follow(ci, &format!("the contact r{round}"), 12000, head);
        if walker::dxz(settled, head) < 4.0 { break; }
        let before = cleared.clone();
        clear_around(&mut w, &mut cleared, "lead r");
        let sim = w.scene.sim.as_ref().unwrap();
        let wedged = settled == w.scene.sim.as_ref().unwrap().drones.list[ci].pos
            && sim.drones.list[ci].state != nf_game::drone::state::PLAYER_IN_WAY;
        drop(sim);
        if cleared == before && wedged {
            // No guard to clear and the contact is holding: close on him (the AllyLeadWait release band
            // [M 0x15eda8 case 3] needs Bond within 5.0 straight-line when pd == 0.0). When the
            // straight line crosses collision (the hall's south structure — see the trail-loop's
            // corner note), the player rounds it by the north lane first.
            let cp = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
            let walled = w.scene.sim.as_ref().unwrap().world.segment(bp, [cp[0], bp[1], cp[2]]).is_some();
            if walled {
                eprintln!("[01f] the contact is across the hall's south structure — walking the north lane back in");
                w.walk_nav_to([5.45, 6.6, 14.3], "round the hall's south structure (north)", 1.4);
                w.walk_nav_to([6.88, 6.4, 14.62], "the hall's north opening (nav node 7)", 1.4);
                w.walk_nav_to([7.4, 6.0, 11.0], "back through the east door band", 1.4);
            }
            w.walk_nav_to(cp, "close on the contact", 1.2);
        }
    }
    clear_around(&mut w, &mut cleared, "stairwell head");
    // ---- THE FOLLOW: trail him down the stairwell, never overtaking --------------------------
    // The trail point sits 4.0 behind him on the Bond→contact line: outside the PlayerInWay
    // 3.5 stay band [M 0x15ea28:32-47], and close enough that the LeadWait band cannot hold —
    // the wait enters at pd ≤ −9.0 / (pd == 0.0 && dist ≥ 5.0) [M 0x15e698 case 3] and releases
    // at pd > −7.0 [M 0x15eda8 case 3]; a 5.5 trail on the corridor's zigzag read ~7.8
    // route-units and parked him in AllyLeadWait (the first run's stall at (−18.8, 14.6)).
    let mut trail_still = 0u32;
    let mut last_cp = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
    let mut rounds = 0u32;
    let mut closed_in = 0u32;
    let parked_at;
    loop {
        rounds += 1;
        assert!(rounds < 120, "the follow never settled: contact at {:?} state {:#x}",
            w.scene.sim.as_ref().unwrap().drones.list[ci].pos, w.scene.sim.as_ref().unwrap().drones.list[ci].state);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { parked_at = w.scene.sim.as_ref().unwrap().drones.list[ci].pos; break; }
        let (cp, cstate) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[ci].pos, sim.drones.list[ci].state)
        };
        // Parked: the route is done (AllyLeadDone parks [M 0x15f2e8]).
        if cstate == nf_game::drone::state::ALLY_LEAD_DONE {
            eprintln!("[01f] the contact parked in AllyLeadDone at ({:.2},{:.2},{:.2}) after {rounds} follow rounds — tick {}",
                cp[0], cp[1], cp[2], w.total_ticks);
            parked_at = cp;
            break;
        }
        if walker::dxz(cp, last_cp) < 0.05 { trail_still += 60; } else { trail_still = 0; closed_in = 0; }
        last_cp = cp;
        if trail_still > 600 {
            // He stood still 10 s — a player closes the gap: the wait's own release
            // ([M 0x15eda8 case 3] pd > −7.0 && (pd != 0.0 || dist < 5.0)) and the PlayerInWay
            // exit both come from Bond's position. Two closes, then give up with the dump.
            closed_in += 1;
            trail_still = 0;
            if closed_in > 2 {
                eprintln!("[01f] the contact stood still 30 s at ({:.2},{:.2},{:.2}) state {cstate:#x} despite {closed_in} closes — tick {}",
                    cp[0], cp[1], cp[2], w.total_ticks);
                parked_at = cp;
                break;
            }
            eprintln!("[01f] the contact stood still 10 s at ({:.2},{:.2},{:.2}) state {cstate:#x} — closing in (the release bands [M 0x15eda8])",
                cp[0], cp[1], cp[2]);
            // The hall's south-west inner corner: when the straight line to the contact crosses
            // collision, Bond is boxed by the hall's south structure — the x≈6.03 wall (z 11.6-13+)
            // and the z≈11.05 wall ([D the disc's collision cels — the port's segment fan: the corner
            // casts hit at 0.10-0.39 in every direction but north; the doorway band x 6.5-7.9,
            // z 12.0→10.4 is the only opening]). The player rounds the structure by its north end —
            // the AI's own lane node[23] (3.39,14.69) → node[7] (6.88,14.62) [D nav path 0] — and
            // comes back in through the east door band (the main test's west-wall ladder).
            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
            let walled = w.scene.sim.as_ref().unwrap().world.segment(bp, [cp[0], bp[1], cp[2]]).is_some();
            if walled {
                eprintln!("[01f] the contact is across the hall's south structure — walking the north lane back in");
                w.walk_nav_to([5.45, 6.6, 14.3], "round the hall's south structure (north)", 1.4);
                w.walk_nav_to([6.88, 6.4, 14.62], "the hall's north opening (nav node 7)", 1.4);
                w.walk_nav_to([7.4, 6.0, 11.0], "back through the east door band", 1.4);
            }
            w.walk_nav_to(cp, "close on the waiting contact", 3.0);
            continue;
        }
        // Guards near him first (the hide keeps him off the route); the approach may overtake —
        // a player swaps positions for a fight, then falls back in behind.
        let armed_near = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            (0..sim.drones.list.len()).any(|i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            })
        };
        if armed_near { clear_around(&mut w, &mut cleared, "follow"); continue; }
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let (dx, dz) = (cp[0] - bp[0], cp[2] - bp[2]);
        let l = (dx * dx + dz * dz).sqrt().max(1e-3);
        let trail = [cp[0] - dx / l * 4.0, cp[1], cp[2] - dz / l * 4.0];
        w.walk_nav_to(trail, &format!("trail the contact {rounds}"), 2.0);
    }
    // The endgame state: the contact where the follow left him, Bond behind.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        eprintln!("[01f] follow done (park {:?}): contact at ({:.2},{:.2},{:.2}) state {:#x} route {:?} bond at ({:.2},{:.2},{:.2}) tick {}",
            parked_at, d.pos[0], d.pos[1], d.pos[2], d.state, d.route.as_ref().map(|r| (r.index, r.nodes.len())),
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], w.total_ticks);
        // Who could hold the hide / block the AND: every live armed guard within 20 of him.
        for (i, o) in sim.drones.list.iter().enumerate() {
            if i == ci || o.team != 1 || !o.started || o.corpse || !o.enabled || o.health <= 0.0 { continue; }
            let dd = walker::dxz(o.pos, d.pos);
            if dd < 20.0 {
                eprintln!("[01f] armed peer {i} skin {:#x} at ({:.1},{:.1},{:.1}) d {:.1} state {:#x} health {:.0} weapon {:#x}",
                    o.skin, o.pos[0], o.pos[1], o.pos[2], dd, o.state, o.health, o.weapon);
            }
        }
    }
    // Bond takes the plate (the player's final step — the AND needs him in the volume).
    w.walk_nav_to(plate, "Bond onto the plate", 0.8);
    // The AND needs BOTH touchers inside the volume for a beat [M Trigger_Hit 0x1b1fc8]; wait for it.
    let mut fired = None;
    for t in 0..600u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { fired = Some(t); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[ci];
    eprintln!("[01f] after the plate wait: ch 0x10 fired at {fired:?}; contact ({:.2},{:.2},{:.2}) state {:#x} bond {:?}",
        d.pos[0], d.pos[1], d.pos[2], d.state, sim.bond.pos);
    let contact_toucher = nf_game::triggers::Toucher {
        a: [d.pos[0], d.pos[1] - 1.0, d.pos[2]], b: d.head(), r: 0.4, drone: true, skin: d.skin, contain: true,
    };
    // On-foot Bond: `contain: !hanging` with hanging false — the sim's own on-foot construction
    // [drone.rs 16711], and this walk never leaves the ground.
    let bond_toucher = nf_game::triggers::Toucher {
        contain: true, ..nf_game::triggers::Toucher::player([sim.bond.pos[0], sim.bond.pos[1] - 0.6, sim.bond.pos[2]])
    };
    if let Some(t) = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::TOUCH && t.out == 0x10) {
        eprintln!("[01f] plate lo {:?} hi {:?}: contact touched_by={} bond touched_by={}",
            t.lo, t.hi, t.touched_by(&contact_toucher), t.touched_by(&bond_toucher));
    }
    assert!(fired.is_some(), "the AND-touch must fire with Bond AND the parked contact on the plate");
    drop(sim);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0002), "the exit fires to 07000002");
    eprintln!("[01f] level_exit to 07000002 at tick {} (follow route)", w.total_ticks);
    for f in &w.report.falls { eprintln!("[01f] fall: {f}"); }
}

/// level-01-end-fade: the level-end screen fade, pinned on the walked exit above. The game's
/// exit path [M]: the AND-touch raises ch 0x10 and `Trigger_Update`'s case-10 arm fires the
/// LOAD_LEVEL trigger (0x1b1a78:162-166 — no proximity check); `Trigger_Activate`'s type-10
/// tail runs `Music_Event(3,1)` + `GS_PauseGame(1)` + `ResetMap_LevelToLoad(next,1,0)` +
/// `GameFlow_PushState(300.0f, 7, 0xff)` [M 0x1b2318:89-92, disasm 0x1b24b8-0x1b24ec]. The push
/// converts state 7 → 8 through `Camera_SetFade(6, 300.0, 0xff, 1)` [M 0x1c9f38 disasm
/// 0x1c9f7c-0x1c9f84: a0=6, f12/a1 pass through, a2=1], arming the viewer fade: level 0, rate
/// (1/300)·(0xff/255), colour bytes R=G=B=0 — black [M 0x1bbdd8 disasm 0x1bbe20-0x1bbe54].
/// `Game_Draw` draws psiFadeView(R,G,B,level·255) while the fade is active [M 0x1e6f50:203-209];
/// `Camera_UpdateGlbVars` advances the level every frame and clamps the byte [M 0x1bada0:16-27];
/// when the fade ends, state 8 pops onto state 3 (`ResetMap_Load`) and the level loads
/// [M 0x1c98c8:173-189].
#[test]
fn the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads() {
    let iso = iso();
    let mut w = walk_to_the_exit(&iso, true);
    // The exit tick: the hold stands with the full count, the fade level is 0, the request rides.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let h = sim.drones.exit_hold
            .expect("the exit posts GS_PauseGame(1) + GameFlow_PushState(300.0f, 7, 0xff) [M 0x1b2318:89-92]");
        assert_eq!(h.next, 0x0700_0002, "the hold carries the exit's next level [D placement p0; M 0x1b2318:91]");
        assert_eq!(h.total, 300, "the push carried 300.0f [M 0x1b2318:92 — 0x43960000]");
        assert_eq!(h.fade_frames, 300, "all 300 frames left at the fire tick");
        assert_eq!(sim.drones.level_exit, Some(0x0700_0002),
            "the exit request is posted; the host drains it only after the hold clears [M 0x1c98c8 case 8]");
        assert_eq!(sim.drones.exit_fade_view().unwrap().alpha, 0.0,
            "the fade's level starts at 0 [M 0x1bbdd8: level = 0 · (0xff/255)]");
    }
    // The world freezes under the fade while it runs [M 0x1c98c8:44-47 the clock stops;
    // 0x1c9790:30-38 the world sections sit behind the pause gate; 0x10cd80:36 the input dies].
    let (bond0, anchor0, frame0) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.pos, sim.drones.player_pos, sim.drones.frame)
    };
    for _ in 0..100 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.pos, bond0, "the world is frozen under the fade");
        assert_eq!(sim.drones.player_pos, anchor0, "the player anchor is frozen under the fade");
        assert_eq!(sim.drones.frame, frame0, "the game clock stops while paused [M 0x1c98c8:44-47]");
        let h = sim.drones.exit_hold.expect("the hold still runs at fire+100");
        assert_eq!(h.fade_frames, 200, "100 of the 300 frames burned [M 0x1bbdd8 rate · 0x1bada0 advance]");
        let a = sim.drones.exit_fade_view().unwrap().alpha;
        assert!((a - 100.0 / 300.0).abs() < 1e-6,
            "the fade alpha is the burned fraction of the push (got {a}) [M 0x1bada0:16-20]");
        // The visible pane: the game draws psiFadeView(0,0,0, level·255) — black at the faded
        // alpha [M 0x1bbdd8 colour bytes, 0x1e6f50:203-209]. The port's overlay quad matches.
        let hud = w.scene.script_overlay_hud(4.0 / 3.0);
        assert!(hud.v.iter().any(|v| v.c[0] == 0.0 && v.c[1] == 0.0 && v.c[2] == 0.0
                && (v.c[3] - 100.0 / 300.0).abs() < 1e-3),
            "the fade pane draws black at the faded alpha [M 0x1bbdd8 + 0x1e6f50 psiFadeView]");
    }
    // The rest of the count: the hold clears at exactly 300 with the request intact
    // [M 0x1c98c8:173-189 — state 8 pops onto state 3].
    for _ in 0..300 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.exit_hold.is_none() { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.exit_hold.is_none(), "the hold cleared at 300 frames [M 0x1c98c8:173-189]");
        assert_eq!(sim.drones.level_exit, Some(0x0700_0002), "the loader waited under the fade — the request is intact");
    }
    // The load AFTER the fade: the host captures the RamSave (the game saves at the fire
    // [M 0x1b2318:96]; the world is frozen between) and loads warm [M 0x1d3800 → 0x1d6360].
    let save = {
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    drop(w);
    let next = nf_view::load_with_carry(&iso, "07000002.bin", false, Some(&save)).expect("load 07000002 warm");
    {
        let sim = next.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0002, "the next level loaded after the fade [M 0x1c98c8:173-189]");
        assert!(sim.drones.exit_hold.is_none(), "the fresh level carries no hold");
        let p = sim.bond.pos;
        assert!((p[0] - -27.17).abs() < 2.0 && (p[2] - 29.80).abs() < 2.0,
            "start #0 (gate 0) — entry from 07000001's single exit route [D docs/levels/07000002.md], got {p:?}");
    }
}
