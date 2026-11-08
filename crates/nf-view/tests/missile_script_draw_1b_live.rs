//! 0700001b: the creator-owned missile and laser scripts' entities DRAW.
//!
//! `Create_SpaceMissile` 0x1a9ef0 / `SpaceMissile_Update` 0x1aa118 load the kind-0x39 placements'
//! p0 (launch), p1 (launched), p2 (disarmed) and side scripts p8..p11 with their own
//! `Script_Load` [M 0x1a9ef0:46-73, disasm 0x1aa23c-0x1aa350]; `SpaceLaser_Update` 0x1aa3f8 its
//! kind-0x38 p0/p1 tracking and p2 dead scripts [M :87-126, :176]. No ScriptPlayer owns them, so
//! the scene's draw-slot walk (rooted at the placed players' scripts) gave their entities no slot:
//! the bays and missile bodies collided on the decks but never drew. Every test here fails on
//! that tree.
//!
//! Disc data [D 0700001b placements #129-137]: missiles 1/3/5/7 share p1 0x060007c6 and 2/4/6/8
//! share 0x060007e1 (each the launched script of four missiles, all four able to play at once);
//! p8..p11 are 0 on every missile; the laser #130's p0/p1 are 0x060007c0/c1, p2 0x060007e3.
//!
//! Draw gate [M]: `Control_SetGList` 0x134960 copies the glist's +0x44 flags into obj+0xF0 and
//! `View_AddObjects` 0x1e62d8:33 skips `(obj+0xF0 & 0x8010) != 0`; only `Script_HideObj(si, 0)`
//! 0x1db808 clears 0x10, and only `SP_HideObj` 0x1dcbf0 calls it. So the owned scripts' 0x10
//! glists never draw: the tracking scripts' 0x20007ce (flags 0x30 [D]), a small red cube on
//! the laser dish.

#[path = "common/walker.rs"]
mod walker;

use nf_game::nis::Obj;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w
}

/// Every live entity of `scripts` has a draw slot that resolves to it: for each (src, hash) the
/// slots k = 0..n-1 exist, n = the live count (`Sim::script_entity_k` serves the k-th newest).
fn assert_all_drawn(w: &Walker, scripts: &[u32], what: &str) -> usize { assert_all_drawn_but(w, scripts, &[], what) }

fn assert_all_drawn_but(w: &Walker, scripts: &[u32], skip: &[u32], what: &str) -> usize {
    let sim = w.scene.sim.as_ref().unwrap();
    let mut live: std::collections::BTreeMap<((u32, u8), u32), usize> = Default::default();
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let Obj::Entity { hash, src, kind, .. } = o {
            if scripts.contains(&src.0) && !skip.contains(hash) && !matches!(kind, 0xea | 0xeb | 0xf5 | 0xf2) {
                *live.entry((*src, *hash)).or_default() += 1;
            }
        }
    }
    assert!(!live.is_empty(), "{what}: the scripts have live entities");
    for (&((s, st), h), &n) in &live {
        for k in 0..n {
            assert!(w.scene.attachments.iter().any(|a| a.script == Some((s, st, h, k as u8))),
                "{what}: {s:#x} stream {st} entity {h:#x} copy {k} has a draw slot");
            assert!(sim.script_entity_k((s, st), h, k).is_some(), "{what}: the slot resolves");
        }
    }
    live.values().sum()
}

/// The placed kind-0x39 p0 launch scripts' entities (the bays, hatches, missile bodies) draw
/// from the load [D the kind-0x39 dump #129/#131-137].
#[test]
fn the_launch_scripts_entities_have_draw_slots() {
    let w = load();
    let p0 = [0x0600092au32, 0x0600092b, 0x06000929, 0x06000a30, 0x06000927, 0x0600092c, 0x0600092e, 0x0600092d];
    let n = assert_all_drawn(&w, &p0, "p0");
    assert!(n >= 7 * 3, "seven decks carry at least the three t0 bodies each [D], got {n}");
}

/// The shared p1s get four slot sets each: four missiles name each one [D params], and all four
/// can play at once (each missile loads its own copy [M 0x1aa23c-0x1aa350]).
#[test]
fn the_shared_launched_scripts_have_four_copies() {
    let w = load();
    for (p1, ent) in [(0x060007e1u32, 0x020007e1u32), (0x060007c6, 0x020007e5)] {
        let ks: Vec<u8> = w.scene.attachments.iter().filter_map(|a| match a.script {
            Some((s, _, h, k)) if s == p1 && h == ent => Some(k),
            _ => None,
        }).collect();
        let mut ks2 = ks.clone();
        ks2.sort_unstable();
        ks2.dedup();
        assert_eq!(ks2, vec![0, 1, 2, 3], "{p1:#x} entity {ent:#x}: copies 0..3, got {ks:?}");
    }
    // Sister: a missile's own p0 (one placement) keeps one slot set.
    let ks = w.scene.attachments.iter().filter(|a| matches!(a.script, Some((0x0600_0927, _, 0x0200_07e4, _)))).count();
    assert_eq!(ks, 1, "06000927's 0x20007e4: one copy");
}

/// The launch branch's p1 entities draw (the flame 0x20007e1 and the flying body).
#[test]
fn the_launch_branch_entities_draw() {
    let mut w = load();
    let mut launched = false;
    for _ in 0..3300u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(193, 1, sim.drones.frame);
            sim.drones.channels.set(194, 1, sim.drones.frame);
        }
        w.tick(Default::default(), false);
        let m4 = w.scene.sim.as_ref().unwrap().drones.space_missiles.iter().find(|m| m.index == 4).unwrap().clone();
        if m4.state == 3 && m4.launch_flag == 1 { launched = true; break; }
    }
    assert!(launched, "missile 4 takes the launch branch [M 0x1aa31c]");
    w.wait(3);
    assert_all_drawn(&w, &[0x060007e1, 0x060007c6], "p1");
}

/// The laser's tracking script entities draw: the body 0x20007ce (t0) and the beam 0x20007cc
/// (t120) [D 0x060007c0/c1].
#[test]
fn the_laser_tracking_entities_draw() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for m in sim.drones.space_missiles.iter_mut() { m.state = 4; }
    }
    let mut beam = false;
    for _ in 0..1200u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(19, 1, sim.drones.frame);
        }
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        beam = sim.drones.scripts.objects.list.iter().flatten().any(|o| matches!(o, Obj::Entity { hash: 0x020007cc, src, .. } if src.0 == 0x060007c0 || src.0 == 0x060007c1));
        if beam { break; }
    }
    assert!(beam, "the tracking script reaches its beam [D t120]");
    let sim = w.scene.sim.as_ref().unwrap();
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let Obj::Entity { hash, src, matrix, .. } = o {
            if src.0 == 0x060007c0 || src.0 == 0x060007c1 { eprintln!("tracking {:#x}/{} {hash:#x} at {:?}", src.0, src.1, &matrix[3][..3]); }
        }
    }
    assert_all_drawn_but(&w, &[0x060007c0, 0x060007c1], &[0x020007ce], "tracking");
    // The 0x10-flagged marker 0x20007ce is live (the fire origin) but has no draw slot [M the gate above].
    assert!(sim.drones.scripts.objects.list.iter().flatten().any(|o| matches!(o, Obj::Entity { hash: 0x020007ce, .. })), "the marker is live");
    assert!(!w.scene.attachments.iter().any(|a| matches!(a.script, Some((0x0600_07c0 | 0x0600_07c1, _, 0x0200_07ce, _)))),
        "the 0x10-flagged 0x20007ce gets no draw slot");
}

