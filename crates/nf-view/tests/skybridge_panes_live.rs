//! The census's second script-breakable instance (script-break-drain): 07000012's sky-bridge
//! intro — script 0x06000530 (SP #660, mode 64 preloaded, plays at level start [D
//! docs/levels/07000012.md; scripts_07000012_live]) — creates 18 obj_type-0x20 entities
//! (streams 10-27: 16 kind-2 glass panes and kind-1 panes on streams 15/26, health 1.0,
//! filter 0 [D script_break_census]). The drain (drone.rs's per-tick 0x20 sync —
//! `Break_Create` 0x12c230's kind/health/filter from the op-7 params) must register them as
//! `Breaks` rows at level start, and a pane breaks under the player's real fire path like the
//! gondola's do.
//!
//! ```bash
//! NF_ISO=… cargo test -p nf-view --test skybridge_panes_live
//! ```

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_sky_bridge_intros_panes_register_at_level_start_and_break_under_fire() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000012.bin", false).expect("load 07000012"));
    // The intro is mode 64 (preloaded) — the panes' streams create while it plays. Wait for
    // the sync to register them; a pane whose glist has no collision shape never registers
    // (the game's pane would be unhittable [M 0x12b970:61-63]) — the census pins the COMMANDS,
    // the live set is the shapable subset.
    let mut n_panes = 0usize;
    for _ in 0..1500 {
        n_panes = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter().filter(|b| b.script_obj.is_some()).count();
        if n_panes >= 18 { break; }
        w.tick(Input::default(), false);
    }
    eprintln!("[skybridge] registered panes after the wait: {n_panes}");
    assert!(n_panes >= 14, "the sky-bridge intro's panes registered [D script_break_census: 18 streams]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        for br in sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some()) {
            let o = br.script_obj.unwrap();
            let stream = match sim.drones.scripts.objects.list.get(o) {
                Some(Some(nf_game::nis::Obj::Entity { src: (_, s), .. })) => *s,
                _ => 0xff,
            };
            let c = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
            let d = ((c[0] - b[0]).powi(2) + (c[1] - b[1]).powi(2) + (c[2] - b[2]).powi(2)).sqrt();
            eprintln!("[skybridge] pane obj {o} stream {stream} kind {} health {:.1} centre ({:.1},{:.1},{:.1}) d {d:.1}",
                br.kind, br.health, c[0], c[1], c[2]);
            assert_eq!(br.health, 1.0, "health (f32)(i32)1 = 1.0 [D params; M 0x12c230 disasm 0x12c2a8-0x12c2c4]");
            assert_eq!(br.filter, 0, "param 0 → filter 0 [M 0x12c230:29-35]");
        }
        let kinds: std::collections::BTreeMap<u8, usize> = {
            let mut m = std::collections::BTreeMap::new();
            for br in sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some()) { *m.entry(br.kind).or_default() += 1; }
            m
        };
        // 17 of the census's 18 commands register: stream 12's glist has no collision shape —
        // a shapeless pane never registers (the game's own pane would be unhittable and its
        // kill silent [M 0x12b970:61-63; script_pane's no-shape skip]).
        assert_eq!(n_panes, 17, "17 shapable panes of the 18 [D census; M 0x12b970:61-63]");
        assert_eq!(kinds.get(&2).copied().unwrap_or(0), 15, "15 kind-2 panes (stream 12's is shapeless) [D census]");
        assert_eq!(kinds.get(&1).copied().unwrap_or(0), 2, "the two kind-1 panes (streams 15/26) [D census]");
    }
    // An armed GUN through the real weapon cycle (the gondola harness's arm — the weapon word
    // must pass the filter [M 0x1ece30:24-33]).
    let word = |sim: &nf_view::scene::Sim, id: u8| sim.weapons.records.get(id as usize)
        .map_or(0u16, |r| u16::from_le_bytes([r[0x10], r[0x11]]));
    for _ in 0..14 {
        let (c, s, wd) = { let sim = w.scene.sim.as_ref().unwrap();
            (sim.gun.clip, sim.gun.spare, word(sim, sim.gun.weapon)) };
        if c + s > 3 && wd & 0x106 == 0 { break; }
        w.tick(Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(Input::default(), false); }
    }
    // Walk to the bridge on real input (the spawn sits 18-38 u out with the office furniture
    // between — the shot needs the player's own feet at the glass): the panes span
    // x −12..11, z 30..37 at y 1.5 [L the registered rows]; the walk stops just south of the
    // east end pane (stream 18 at (8.3,1.5,30.3)).
    w.walk_nav_to([9.0, 0.2, 27.0], "the sky-bridge approach", 1.2);
    // The approach wedges at (9.9, 26.1), where the office wall stands 0.6 in front of the head:
    // a round fired from there stops on the wall [M Collide_Update 0x1ea398:216-229 — the
    // breakable's contact counts only ahead of the first solid stop; BREAK_WORLD_STOP rides the
    // pane ride]. The player steps back east to the approach's own nav point (12.3, 26.65), where
    // the line to the bridge's glass is clear, and shoots a pane he can see.
    w.walk_to(Wp::new(12.3, 1.1, 26.65).r(0.5), "the clear stand by the bridge");
    // A pane is in view when the flight line from the head to its centre meets its box before
    // any solid stop.
    let seen = |sim: &nf_view::scene::Sim, b: &nf_game::breakable::Break| {
        let head = sim.bond.head();
        let d = sim.data();
        let c = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
        let Some(t) = nf_game::breakable::Breaks::seg_box_t(head, c, b.lo, b.hi) else { return false };
        d.world.segment_flight_scoped(d.world.find_cel(head), head, c, true).is_none_or(|ts| t <= ts)
    };
    let any_seen = |sim: &nf_view::scene::Sim| sim.drones.breaks.list.iter()
        .any(|b| b.script_obj.is_some() && !b.dead && b.kind == 2 && seen(sim, b));
    // Then north to the bridge mouth's open east half until a pane comes into view (its west
    // half, x 9.65..11.04 on x − z = −17, is a solid mat-0x15 panel).
    'stands: for stand in [[11.8f32, 27.8], [11.8, 28.2], [11.8, 28.6]] {
        for _ in 0..180u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if any_seen(sim) { break 'stands; }
            let p = sim.bond.pos;
            if (stand[0] - p[0]).hypot(stand[1] - p[2]) < 0.25 { break; }
            let err = walker::wrap_pi((stand[0] - p[0]).atan2(stand[1] - p[2]) - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.6 { 0.0 } else { 0.6 }, ..Default::default() };
            w.tick(input, false);
        }
    }
    for _ in 0..10 { w.tick(Input::default(), false); }
    eprintln!("[skybridge] the stand {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The nearest kind-2 pane in view, then the gondola harness's converge-and-press loop (real
    // aim input, the trigger on the edge).
    let target_obj = {
        let sim = w.scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let seen = |b: &nf_game::breakable::Break| seen(sim, b);
        sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some() && !b.dead && b.kind == 2 && seen(b))
            .min_by(|a, c| {
                let d = |b: &nf_game::breakable::Break| (b.lo[0] - head[0]).powi(2) + (b.lo[1] - head[1]).powi(2) + (b.lo[2] - head[2]).powi(2);
                d(a).partial_cmp(&d(c)).unwrap()
            }).and_then(|b| b.script_obj).expect("a kind-2 pane in clear view of the stand")
    };
    eprintln!("[skybridge] target pane obj {target_obj}");
    let mut kill: Option<(usize, u8, u16)> = None;
    let mut fires = 0u32;
    for t in 0..1200u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let Some(br) = sim.drones.breaks.list.iter().find(|b| b.script_obj == Some(target_obj) && !b.dead) else { break };
        let centre = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
        let head = sim.bond.head();
        let to = [centre[0] - head[0], centre[1] - head[1], centre[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
        let pitch_err = desired_pitch - sim.bond.pitch;
        let converged = yaw_err.abs() < 0.03 && pitch_err.abs() < 0.02;
        let kills_before = sim.drones.break_kills.iter().filter(|k| k.0 == target_obj).count();
        let fire = converged && fires < 8 && t % 12 == 0;
        if fire { fires += 1; }
        let aim_input = Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.05, 0.05), ..Default::default() };
        drop(sim);
        if fire { w.tick_fire(aim_input); } else { w.tick(aim_input, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.break_kills.iter().filter(|k| k.0 == target_obj).count() > kills_before {
            kill = sim.drones.break_kills.iter().find(|k| k.0 == target_obj).copied();
            break;
        }
    }
    let (k_obj, k_kind, k_sound) = kill.expect("a sky-bridge pane broke under the real fire");
    assert!(fires >= 1, "Bond's own press broke the pane");
    assert_eq!((k_kind, k_sound), (2, 0x2E), "the glass kill [M 0x12b970:64-76]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.breaks.list.iter().all(|b| b.script_obj != Some(k_obj)), "the row dropped");
    assert!(sim.drones.scripts.objects.list.get(k_obj).is_none_or(|o| o.is_none()), "the entity deleted [M 0x12b970:60]");
    eprintln!("[skybridge] pane {k_obj} killed after {fires} presses");
}
