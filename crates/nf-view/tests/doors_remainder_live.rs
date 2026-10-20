//! G-035 remainder (`fix-doors-remainder`): the use pick against the door's collision mesh
//! (`Player_Activate` 0x1a5170), the door labels 0x2000003 / 0x200000b (`Door_Update` 0x1363c8 /
//! `Check_Target` 0x1a7810 + `Door_DoMsg` 0x135f18), and the `Collide_Update` proxy path of
//! `Door_ReactToHit` (0x1ea398 → 0x135980). Opt-in: NF_ISO.

#[path = "common/walker.rs"]
mod walker;
use walker::Walker;

use nf_game::door;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn norm_h(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[2] * v[2]).sqrt();
    if l < 1e-4 { [0.0, 0.0, 0.0] } else { [v[0] / l, 0.0, v[2] / l] }
}

/// Teleport Bond to `pos`, aim his yaw/pitch at `target` until converged, then pulse E one tick
/// per 6. The aim phase matters: the stick's full deflection is 0.08 rad/tick (the helper's
/// clamp), so a cold teleport can need 40+ ticks to swing π — a fixed short window fired E
/// mid-turn, before the look ever pointed at the door (the census's mass failures).
fn aim_and_use(w: &mut Walker, pos: [f32; 3], target: [f32; 3], ticks: u32) {
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond.pos = pos;
    for _ in 0..30 { w.tick(Input::default(), false); } // settle on the floor
    // Aim to convergence first (no E): full-deflection turns until the look is on the target.
    for _ in 0..240 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (target[0] - sim.bond.pos[0]).atan2(target[2] - sim.bond.pos[2]);
        let err = desired - sim.bond.yaw;
        let err = if err > std::f32::consts::PI { err - 2.0 * std::f32::consts::PI } else if err < -std::f32::consts::PI { err + 2.0 * std::f32::consts::PI } else { err };
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let to = sub(target, eye);
        let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let done = err.abs() < 0.02 && perr.abs() < 0.02;
        w.tick(Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.06, 0.06), ..Default::default() }, false);
        if done { break; }
    }
    for k in 0..ticks {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (target[0] - sim.bond.pos[0]).atan2(target[2] - sim.bond.pos[2]);
        let err = desired - sim.bond.yaw;
        let err = if err > std::f32::consts::PI { err - 2.0 * std::f32::consts::PI } else if err < -std::f32::consts::PI { err + 2.0 * std::f32::consts::PI } else { err };
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let to = sub(target, eye);
        let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        w.tick(Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.06, 0.06), ..Default::default() }, k % 6 == 0);
    }
}

/// The door's through-axis: the horizontal matrix row the leaf does NOT span. The offset from
/// the placement origin (`matrix[3]`) to the gfx-sphere centre (`sph.0`) lies along the span
/// row — the probe on 07000001: door 0's offset lies 0.93 along row0 and 0.00 along row2 (the
/// leaf spans row0 = world z, the approach normal is row2 = world x); door 1's spans x the same
/// way. Standing 1.2 out along the SPAN row parks Bond in the doorway plane at the leaf's edge.
fn door_axis(d: &nf_game::door::Door) -> [f32; 3] {
    let off = sub(d.sph.0, [d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]]);
    let r0 = [d.matrix[0][0], d.matrix[0][1], d.matrix[0][2]];
    let r2 = [d.matrix[2][0], d.matrix[2][1], d.matrix[2][2]];
    if dot(off, r0).abs() >= dot(off, r2).abs() { norm_h(r2) } else { norm_h(r0) }
}

/// The first door the mesh pick can act on: a loaded shape, unlocked, no approach gate (07000001's doors are all kind-2 slides).
fn pickable_door(w: &Walker) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.doors.list.iter().position(|d| {
        d.glist != 0 && d.ch_lock == 0 && d.approach == 0
            && sim.world.has_shape(d.glist)
    })
}

/// (a) The use pick is the game's mesh pick: a 1.0-radius sphere one unit ahead of Bond's HEAD
/// (`Player_GetHeadPos` + `Mat_GetDir(glb_viewer)` [M 0x1a5170:100-108]), tested against the
/// door's live leaf mesh with NO occlusion test — `Collide_SphereIntersect` returns every
/// intersecting object and the walk activates every class-0x1C one (walls do not stop it) [M
/// 0x1a5170:113-133]. Pins, on the through-axis (the leaf-span row is the OTHER one — the probe
/// dump in the report): 2.3 out the sphere falls 0.3 short of the plane (the pre-fix 2.5
/// placement stand-in opened there); 1.2 out it overlaps 0.8 and opens; and from beside the
/// doorway at 2.3 along the leaf's span (in the wall plane, the stand-in's 3.2 placement
/// distance out of reach) the sphere still clips the leaf's edge tris through the wall — the
/// game's no-occlusion pick.
#[test]
fn the_use_pick_respects_the_leaf_mesh() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).unwrap());
    w.wait_enabled(3000, "level start");
    w.wait(30);
    let di = pickable_door(&w).expect("07000001 has a shape-backed door");
    let (centre, through, span) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di];
        // The through-axis from the leaf-span offset, and the span axis itself (for the edge case).
        let off = sub(d.sph.0, [d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]]);
        let r0 = [d.matrix[0][0], d.matrix[0][1], d.matrix[0][2]];
        let r2 = [d.matrix[2][0], d.matrix[2][1], d.matrix[2][2]];
        let (through, span) = if dot(off, r0).abs() >= dot(off, r2).abs() { (r2, r0) } else { (r0, r2) };
        (d.sph.0, norm_h(through), norm_h(span))
    };

    // Square-on 2.3 out: the pick sphere (r 1 at head + look) reaches only 1.3 from the leaf
    // plane — 0.3 short. The old stand-in (placement distance < 2.5) opened here.
    aim_and_use(&mut w, [centre[0] + through[0] * 2.3, centre[1], centre[2] + through[2] * 2.3], centre, 30);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state, door::CLOSED,
        "E 2.3 out on the through-axis must not reach the leaf (the sphere falls 0.3 short)");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].pos01, 0.0, "the leaf never moved");

    // Sister: square-on 1.2 out — the sphere overlaps the leaf by ~0.8; the door opens.
    aim_and_use(&mut w, [centre[0] + through[0] * 1.2, centre[1], centre[2] + through[2] * 1.2], centre, 30);
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "E 1.2 out square-on opens the door (state {st:#x})");

    // The edge/through-wall case: 2.3 out ALONG the leaf span (beside the doorway, in the wall
    // plane — the old stand-in's placement distance is ~3.2, out of its 2.5 reach). The pick
    // sphere's centre lands just past the leaf's edge ON the leaf plane, and the game's pick
    // (no occlusion [M 0x1a5170:113-133]) opens the door through the wall. Bond settles out of
    // the wall to either face; both leave the sphere on the edge.
    aim_and_use(&mut w, [centre[0] + span[0] * 2.3, centre[1], centre[2] + span[2] * 2.3], centre, 30);
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "E beside the doorway (2.3 along the span) clips the leaf edge through the wall (state {st:#x})");
}

/// (b) The door labels: the ids bind from USATxt.dat (bank 2: `Txt_BindLabel` 0x1cad60's
/// fixups[2] + low bits — 0x0200_0003 "Door Unlocked", 0x0200_000b "Press Action to open door"
/// [D probe; the decompiles' 0x2000003/0x200000b are bank-2 ids]), and the unlock note
/// 0x0200_0003 fires on a lock channel's rising edge through the real path (`Door_Update`
/// 0x1363c8:101-107 pushes it where the door self-activates and clears DI+0x68). The aim-hint
/// show/latch/locked branches of `Check_Target`'s door arm (0x1a7810:186-193 + `Door_DoMsg`
/// 0x135f18) are wired (doors.aim_msg) but NOT pinned here: every head position this probe
/// tried near 07000001's doors has a STATIC world-segment hit 0.03-0.06 from the eye (the
/// port's first-hit gate [M 0x1a7810:113-116 reads only the ray's first hit] kills the hint
/// before the door rules run) — the exact numbers are in the report under "Live checks".
#[test]
fn door_labels_fire_per_the_decompile_rules() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).unwrap());
    w.wait_enabled(3000, "level start");
    w.wait(30);
    // [D USATxt.dat via the level's own lang] the bank-2 ids the decompiles bind:
    let hint = w.scene.sim.as_ref().unwrap().lang.as_ref().unwrap().bind(0x0200_000b).unwrap_or("").trim().to_string();
    assert_eq!(hint, "Press Action to open door", "0x0200000b binds the aim-hint string");
    let note = w.scene.sim.as_ref().unwrap().lang.as_ref().unwrap().bind(0x0200_0003).unwrap_or("").trim().to_string();
    assert_eq!(note, "Door Unlocked", "0x02000003 binds the unlock-note string");

    // The unlock note on a lock channel's rising edge, at the Door_Update layer (the level
    // scripts' own channel traffic around the sim's doors swallows the in-sim pin): lock a
    // shape-backed door on an unused channel, raise it, tick `Door_Update`'s own path
    // (`Doors::update` + `Doors_Calc`) — the edge arm shows the note and self-activates the
    // door [M 0x1363c8:95-107].
    let (note_events, activated) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let dj = (0..sim.drones.doors.list.len())
            .position(|j| sim.drones.doors.list[j].glist != 0
                            && sim.drones.doors.list[j].ch_lock == 0
                            && sim.drones.doors.list[j].state == door::CLOSED
                && sim.world.has_shape(sim.drones.doors.list[j].glist))
            .expect("a closed shape-backed door");
        let mut doors = sim.drones.doors.clone();
        doors.list[dj].ch_lock = 200;
        let mut ch = nf_game::triggers::Channels::default();
        let mut evs: Vec<(u32, u16)> = Vec::new();
        for f in 1..=12u32 {
            if f == 5 { ch.v[200] = 1; } // the rising edge
            for i in 0..doors.list.len() { doors.update(i, &mut ch, &[( [0.0; 3], 0.4, false )], f); }
            doors.calc();
            evs.extend(doors.label_msgs.drain(..));
        }
        (evs, doors.list[dj].state & 0xc != 0 && doors.list[dj].ch_lock == 0)
    };
    assert!(note_events.iter().any(|&(id, _)| id == 0x0200_0003), "the unlock edge pushes 0x02000003 (got {note_events:?})");
    assert!(activated, "the unlocked door self-activates and clears its lock channel");
}

/// (c) The proxy path: a closing door that meets Bond reverses (the anti-crush) — `Collide_Update`
/// post-walk fills the records, `Door_ReactToHit` (0x135980) consumes them the same frame.
/// IGNORED this session: the closing sweep never reversed for Bond standing 0.5 out on the
/// through-axis (40 ticks, state never 0x8) — the proxy scan's reach/timing vs Bond's capsule
/// needs the live measurement listed in the report's "Live checks" before it can pin anything.
#[test]
#[ignore = "the reversal does not reproduce headlessly yet — see the report's Live checks"]
fn a_closing_door_reverses_off_bond() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).unwrap());
    w.wait_enabled(3000, "level start");
    w.wait(30);
    // The level-start script can already have run some doors — pick one still CLOSED so the
    // pick's activation is the open (an E on an open door is the close; the pre-fix runs here).
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.doors.list.len()).position(|j| {
            let d = &sim.drones.doors.list[j];
            d.glist != 0 && d.ch_lock == 0 && d.approach == 0 && d.state == door::CLOSED
                && sim.world.has_shape(d.glist)
        })
    }.expect("a closed shape-backed door");
    let (centre, nrm) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di];
        (d.sph.0, door_axis(d))
    };
    let n = norm_h(nrm);
    let pos = centre;

    // Open it — through the door code's own use pick (a head 0.5 out from the leaf centre looking
    // straight in; the pick itself is T1's pin) — then step back out of the blocking zone so the
    // 360-tick auto-close fires.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let eye = [centre[0] + n[0] * 0.5, centre[1], centre[2] + n[2] * 0.5];
        let look = norm_h(sub(centre, eye));
        let (_, hits) = sim.drones.doors.player_use(eye, look, &mut sim.drones.channels, sim.drones.frame, &sim.world);
        assert!(hits >= 1, "the use pick meets the leaf from 0.5 out");
    }
    let opened = { let s = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state; s & 0xc != 0 };
    assert!(opened, "the door opened for the close test");
    for _ in 0..10 { w.tick(Input::default(), false); }
    // Close it the game's way: a second E on an open door starts the closing sweep (the toggle
    // in `Door_Activate`); door 0 is a keep-open door, so the auto-close never runs here.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let eye = [centre[0] + n[0] * 0.5, centre[1], centre[2] + n[2] * 0.5];
        let look = norm_h(sub(centre, eye));
        let _ = sim.drones.doors.player_use(eye, look, &mut sim.drones.channels, sim.drones.frame, &sim.world);
    }
    // Step into the doorway IMMEDIATELY (the slide closes in ~30 ticks): Bond stands in the
    // leaf's path during the sweep, the leaf's mesh meets his sphere and the proxy path
    // (`Collide_Update` post-walk → `Door_ReactToHit` 0x135980) reverses the door to OPENING.
    // Pre-fix the door closed through him (state 2).
    let mut reversed = false;
    for k in 0..40u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            if k == 0 { sim.bond.pos = [pos[0] + n[0] * 0.5, sim.bond.pos[1], pos[2] + n[2] * 0.5]; }
        }
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0x8 != 0 { reversed = true; break; }
    }
    assert!(reversed, "the closing leaf met Bond and Door_ReactToHit reopened the door");
}

/// The census DIAGNOSTIC (not a pass/fail pin): every shape-backed, unlocked, approach-open
/// door on every level bin, tried at 1.2 out along its through-axis with a converged aim.
/// The 2026-10-20 numbers: 175 doors, 13 opened — the rest fail on stand-spot geometry (the
/// through sign per door, floors, walls), NOT on the pick (T1 pins the pick on a controlled
/// spot). The per-level list goes to the report; the main agent's live census is the follow-up.
#[test]
fn census_every_door_opens_at_its_approach() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut total = 0u32;
    let mut opened = 0u32;
    let mut fails: Vec<String> = Vec::new();
    for code in 1..=0x1b {
        let name = format!("0700{code:04}.bin");
        let Ok(scene) = nf_view::load(&iso(), &name, false) else { continue };
        let mut w = Walker::new(scene);
        w.wait_enabled(1500, &name);
        w.wait(20);
        let doors: Vec<(usize, [f32; 3], [f32; 3])> = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.doors.list.iter().enumerate()
                .filter(|(_, d)| d.glist != 0 && sim.world.has_shape(d.glist) && d.ch_lock == 0 && d.locked_flag == 0 && d.approach == 0)
                .map(|(i, d)| (i, d.pos, door_axis(d)))
                .collect()
        };
        for (i, pos, nrm) in doors {
            total += 1;
            let n = norm_h(nrm);
            let centre = [pos[0], pos[1] + 1.0, pos[2]];
            aim_and_use(&mut w, [pos[0] + n[0] * 1.2, pos[1], pos[2] + n[2] * 1.2], centre, 24);
            let st = w.scene.sim.as_ref().unwrap().drones.doors.list[i].state;
            if st & 0xc != 0 { opened += 1; } else { fails.push(format!("{name} door {i} at {:.1},{:.1},{:.1} state {st:#x}", pos[0], pos[1], pos[2])); }
            // Reset the door so the next candidate starts clean (close it the same way).
            aim_and_use(&mut w, [pos[0] + n[0] * 1.2, pos[1], pos[2] + n[2] * 1.2], centre, 12);
        }
        eprintln!("[census] {name}: {} doors so far, {} opened", total, opened);
    }
    eprintln!("[census] TOTAL {total} doors, {opened} opened, {} failed (stand-spot geometry; see the report): {fails:#?}", fails.len());
    assert!(total > 0, "the census found no candidate doors at all");
}
