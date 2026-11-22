//! level-0a-civilian-slide-cels: the office civilians must never slide without a walk while in
//! view. The game's drawn byte (`obj+0xFC`) is written by `View_AddObjects` 0x1e62d8:71 [M] for an
//! object whose world sphere passes `Vision_InView` 0x1e89e0 [M] over the PORTAL-VISIBLE cels —
//! `View_AddCels` 0x1e6ba0 draws each VisCelList cel's object list, and the VisCelList is the
//! portal walk from the viewer's cel (`Vision_Portal_Recurse` 0x1e86e8 + `Vision_Recurse`
//! 0x1e82e0). The pair holds NO rays: a civilian in a visible room behind a thin post
//! (LaserTrackVert, mat 0x06) or non-pass foliage still draws in the game. The port's sim gate
//! used to run bullet-segment rays as the room stand-in, so the civilian #695 (danim 392, AI 0x14
//! CivilianPatrol) glided at the danim speed with no gait while on screen (`NDrone2_DoAnimation`
//! 0x149f88's not-drawn arm [M]). The gate now runs the game's own room rule.
//!
//! #695 is not on the floor at the load: he is one of the four group-2 templates (#690/#694/#695/
//! #696) of spawner #692 (AI_DroneSpawner 0xf1: mode 2, max 1, waves 1, start ch 0x84, min
//! distance 3 [D params]). Ch 0x84 is the drone-zone plates #713/#714 (oDroneTrigger3 at
//! (−17.0, 0.7, 17.5) / (−26.9, 0.7, 17.5), touch-once, mask 8 [D]) on the west offices' north
//! corridor. On the plate's tick `DroneSpawner_Control` 0x137e68 latches the start, and
//! `DroneSpawner_SpawnDrone` 0x137cb0 draws ONE template with `Rand_Rand(4)` over the usable
//! (unused, at least 3 from Bond) walk, aborting on a spot Bond can see [M]; the one spawn fills
//! the wave and the single wave ends the spawner [M 0x137e68:126-172]. So exactly one of the four
//! ever walks, and which one rides the shared stream's phase at the plate. The port's spawner
//! releases group 2 on the same plate (asserted below); the route's 3-tick stand in the west hall
//! before the plate puts the draw on #695 (stands of 0/1/2/3 ticks drew #694/#690/#694/#695 on
//! this route).
//!
//! #695's patrol is a loop through the west offices (about x −9..−36, z 2.6..13.5 on this route).
//! The watch stays at the plate, no teleports anywhere: the load's own arrival, out of the cab,
//! the lobby, the crouched west-hall crossing under beam #607, the plate. Bond faces south-west
//! down the west corridor while #695 walks the loop's west end. Each tick the test runs the
//! room walk itself (`portal_visible_cels_from_cell_doors` from Bond's cel, the five depth-0
//! planes) and checks: whenever #695's cel is portal-visible and his sphere is in the frustum,
//! the sim reads him drawn, his walk anim advances and he moves with it. The old bullet-ray gate
//! read him undrawn there (the corridor's west wall stands between) and slid him.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn turn_to(w: &mut Walker, yaw: f32) {
    for _ in 0..600 {
        let err = walker::wrap_pi(yaw - w.scene.sim.as_ref().unwrap().bond.yaw);
        if err.abs() < 0.02 { return; }
        let input = Input { turn: err.clamp(-0.08, 0.08), ..Input::default() };
        w.tick(input, false);
    }
    panic!("could not turn to yaw {yaw}");
}

/// The game's room rule for one drone this tick: his linked cel sits in the VisCelList the
/// portal walk from Bond's cel builds (`View_AddCels` 0x1e6ba0 over `Vision_Portal_Recurse`
/// 0x1e86e8 + `Vision_Recurse` 0x1e82e0, the doors leg on this tick's closed flags [M]).
fn cel_visible(sim: &nf_view::scene::Sim, gi: usize) -> bool {
    let d = &sim.drones.list[gi];
    let (Some(start), Some(his)) = (sim.bond.cel, d.obj_cel) else { return false };
    let doors = sim.drones.doors.closed_flags();
    let sides = nf_game::drone::frustum_sides(sim.bond.look());
    let eye = sim.bond.head();
    let mut depth0: [[f32; 4]; 5] = [[0.0; 4]; 5];
    for (k, n) in sides.iter().enumerate() {
        depth0[k] = [n[0], n[1], n[2], -(n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2])];
    }
    let fw = sim.bond.look();
    let l = (fw[0] * fw[0] + fw[1] * fw[1] + fw[2] * fw[2]).sqrt();
    let fe = fw[0] / l * eye[0] + fw[1] / l * eye[1] + fw[2] / l * eye[2];
    depth0[4] = [-fw[0] / l, -fw[1] / l, -fw[2] / l, fe + 200.0];
    nf_view::scene::portal_visible_cels_from_cell_doors(&sim.cel_graph, start, eye, &depth0, Some(&doors))
        .is_some_and(|vis| vis.get(his).copied().unwrap_or(false))
}

struct Sample { pos: [f32; 3], danim: u16, ftime: f32, in_frustum: bool, cel_vis: bool, drawn: bool }

/// Turn Bond to `yaw` where he stands, then sample #695 for `ticks` while he patrols.
fn watch(w: &mut Walker, yaw: f32, ticks: u32, gi: usize) -> Vec<Sample> {
    turn_to(w, yaw);
    let mut out = Vec::new();
    for _ in 0..ticks {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.level_exit.is_none(), "the mission ended during the watch");
        let d = &sim.drones.list[gi];
        let in_frustum = nf_game::drone::in_view(sim.bond.head(), d.pos, nf_game::drone::DRONE_DRAWN_SLACK, &nf_game::drone::frustum_sides(sim.bond.look()));
        out.push(Sample { pos: d.pos, danim: d.anim.danim, ftime: d.anim.ftime, in_frustum, cel_vis: cel_visible(sim, gi), drawn: d.drawn });
    }
    out
}

/// The crouch toggle through the real input edge (walkreal_0700000a_live's `set_crouch`).
fn set_crouch(w: &mut Walker, on: bool) {
    for t in 0..400u32 {
        if w.scene.sim.as_ref().unwrap().bond.crouched == on { return; }
        w.tick(Input { crouch: t % 70 == 0, ..Input::default() }, false);
    }
    panic!("the crouch toggle never reached {on}");
}

#[test]
fn the_west_office_civilian_spawns_on_his_plate_and_walks_his_gait_while_drawn() {
    let cold = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load 0700000a warm"));
    // #695 and his spawner at the load: a held template (no object until the spawn [M 0x137cb0]).
    let gi = {
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = sim.drones.list.iter().position(|d| d.placement == 695).expect("drone placement #695 [D]");
        let d = &sim.drones.list[gi];
        assert!(!d.present && d.group == 2, "#695 is a group-2 spawner template, absent at the load [D]");
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 692).expect("spawner #692 [D]");
        assert_eq!((sp.group, sp.start, sp.max_alive, sp.waves), (2, 0x84, 1, 1), "spawner #692: group 2, start ch 0x84, max 1, one wave [D]");
        assert!(sp.templates.contains(&gi), "#695 is one of #692's templates");
        gi
    };
    // The real route prefix: the arrival plate fires under Bond's feet, out of the cab, the lobby.
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x47), "the arrival plate #58 → ch 0x47 [D]");
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    // Beam #607 across the west-hall mouth: crouched under it (walkreal_0700000a_live's crossing).
    set_crouch(&mut w, true);
    w.walk_to(Wp::new(-7.4, 0.4, 16.6).r(0.6), "the west hall's doorway line (crouched)");
    w.walk_to(Wp::new(-10.0, 0.4, 16.6).r(0.6), "through beam #607's plane (crouched)");
    w.walk_to(Wp::new(-11.6, 0.4, 16.6).r(0.6), "west, clear of beam #607's reach (crouched)");
    set_crouch(&mut w, false);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x84), "ch 0x84 is still off before the plate");
    assert!(!w.scene.sim.as_ref().unwrap().drones.list[gi].present, "#695 is still held before the plate");
    // The stand that phases the spawn draw onto #695 (see the header), then plate #713.
    w.wait(3);
    w.walk_to(Wp::new(-16.6, 0.4, 17.4).r(0.6), "the west offices' north corridor (plate #713)");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x84), "plate #713 → ch 0x84 [D]");
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 692).unwrap();
        let got: Vec<usize> = sp.spawned.iter().map(|&i| sim.drones.list[i].placement).collect();
        assert_eq!(got, vec![695], "spawner #692 released one group-2 civilian on ch 0x84, #695 on this route's draw [M 0x137cb0]");
        assert!(sp.gone, "one spawn, one wave: spawner #692 is done [M 0x137e68:126-143]");
        let d = &sim.drones.list[gi];
        assert!(d.present && d.enabled, "#695 is on the floor after the spawn");
        eprintln!("[civ] #695 list[{gi}] on the floor at tick {} pos ({:.2},{:.2},{:.2})", w.total_ticks, d.pos[0], d.pos[1], d.pos[2]);
    }
    // The watch: Bond stays on the plate and faces south-west down the west corridor while #695
    // walks the loop's west end (2400 ticks covers one lap of the loop on this route).
    let samples = watch(&mut w, -2.07, 2400, gi);
    let (mut seen, mut drawn_seen, mut walk_seen) = (0u32, 0u32, 0u32);
    let (mut gait, mut moved) = (0.0f32, 0.0f32);
    for k in 1..samples.len() {
        let (a, b) = (&samples[k - 1], &samples[k]);
        // Both ends in view: the sim's gate runs before his move inside a tick, so the tick he
        // crosses into the frustum reads the old side (one sample on this route).
        if !(a.in_frustum && a.cel_vis && b.in_frustum && b.cel_vis) { continue; }
        seen += 1;
        if b.drawn { drawn_seen += 1; } else { eprintln!("[undrawn] sample {k} pos {:?}", b.pos); }
        if b.danim == 392 && a.danim == 392 && a.drawn && b.drawn {
            walk_seen += 1;
            gait += (b.ftime - a.ftime).abs();
            moved += walker::dxz(a.pos, b.pos);
        }
    }
    eprintln!("[civ] samples {} seen {seen} drawn {drawn_seen} walk {walk_seen} gait {gait:.1} moved {moved:.2}", samples.len());
    assert!(seen >= 300, "#695 was not in a portal-visible cel and the frustum for long enough ({seen}) — re-derive the watch");
    assert_eq!(drawn_seen, seen, "#695 read undrawn in a portal-visible cel inside the frustum — the slide (the old ray gate) [M View_AddObjects 0x1e62d8:71]");
    assert!(walk_seen >= 200, "#695 was not on his walk danim 392 while drawn ({walk_seen})");
    assert!(gait > 100.0, "the walk anim did not advance while drawn ({gait:.1})");
    assert!(moved > 2.0, "#695 did not patrol while drawn ({moved:.2})");
    // Sister: Bond faces away (north-east) — #695 leaves the frustum and reads undrawn.
    let away = watch(&mut w, 1.07, 300, gi);
    let drawn_away = away.iter().filter(|s| !s.in_frustum && s.drawn).count();
    let in_frustum_away = away.iter().filter(|s| s.in_frustum).count();
    assert_eq!(in_frustum_away, 0, "facing away, #695 stayed in the frustum — re-derive the sister");
    assert_eq!(drawn_away, 0, "facing away, #695 stayed drawn ({drawn_away}) — the gate over-draws");
}
