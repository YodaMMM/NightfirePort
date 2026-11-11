//! Guard rounds and glass, on every level with placed glass panes.
//!
//! `Bullet_update` 0x12e7b0:390 arms the flight's surface mask from the weapon record alone:
//! `HITTEST+0x9a = (+0x6C & 0x800 == 0) << 5`, and `Collide_Update` 0x1ea398:222-224 walks past
//! a class-0x10 contact under that bit [M]. Nothing in the arm reads who fired the round, so a
//! guard's round passes glass exactly as Bond's does, and the pane takes the contact on its
//! obj+0xD0 chain: `Break_Update` 0x12bfd8:36-37 drains the pane's health from it [M].
//!
//! The scene per level: a pane, Bond standing behind it, an armed guard in front of it, the
//! guard alarmed through the game's own msg 0x1e path [M `DroneVision_EnemyAlerts` 0x177c18:113-114].
//! The guard sees Bond through the pane (vision mask 0xa27 drops class 0x10 [M 0x1ebaa8:315-317]),
//! fires, and the round must drain the pane and reach Bond.
//!
//! Opt-in: set `NF_ISO`.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::msg;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The SP levels with kind-2 (glass) panes [D the breakable census].
const LEVELS: [&str; 10] = ["07000001", "07000003", "07000006", "07000007", "07000008", "07000009", "0700000a", "07000012", "07000013", "07000014"];

struct Outcome { drained: bool, filtered: bool, hurt: bool }

fn try_pane(iso: &str, lv: &str, pane: usize) -> Option<Outcome> {
    let mut w = nf_view::load(iso, &format!("{lv}.bin"), false).map(walker::Walker::new).ok()?;
    w.wait_enabled(6000, "level start");
    w.wait(60);
    let sim = w.scene.sim.as_mut().unwrap();
    // The `nth` wall pane with a floor on both sides at the pane's chest height.
    // A see-through pane: every collision triangle of class 0x10 — the class the guard's 0xa27
    // vision drops [M Collide_Intersect 0x1ebaa8:314-316, mask bit 0x2]. A kind-2 pane of another
    // surface class (07000003's mat-0x05 #338/#339 [D the pane's collision materials]) stops the
    // guard's sight ray like any wall [M 0x1eae80:70-87 admits it, 0x1ebaa8 drops nothing], so the
    // guard never sees Bond behind it and is no scene for this rule.
    let see_through = |br: &nf_game::breakable::Break| sim.world.shape_materials(br.shape_key)
        .is_some_and(|m| !m.is_empty() && m.iter().all(|&x| x & 0x3f == 0x10));
    let scene = sim.drones.breaks.list.iter().filter(|b| b.kind == 2 && !b.dead && see_through(b)).filter_map(|br| {
        let (lo, hi) = (br.lo, br.hi);
        let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
        // A wall pane: thin on one horizontal axis, at least a metre wide and tall.
        let thin = if size[0] < size[2] { 0 } else { 2 };
        let wide = 2 - thin;
        if size[thin] > 0.3 || size[wide] < 1.0 || size[1] < 1.0 { return None; }
        let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
        let stand = |off: f32| -> Option<[f32; 3]> {
            let mut p = c;
            p[thin] += off;
            let f = sim.world.floor_below([p[0], c[1] + 0.5, p[2]], 3.0)?;
            // The shooter's chest line and the target's body cross the pane.
            (hi[1] > f + 0.9 && lo[1] < f + 1.5 && c[1] > f).then_some([p[0], f, p[2]])
        };
        let pairs = [1.5f32, 2.0, 3.0].iter().flat_map(|&b| [4.0f32, 3.0, 6.0, 2.0].map(move |g| (b, g)));
        let (b, g) = pairs.flat_map(|(b, g)| [(b, -g), (-b, g)])
            .find_map(|(bo, go)| Some((stand(bo)?, stand(go)?)))?;
        Some((br.placement, br.health, thin, b, g))
    }).nth(pane);
    let (placement, health0, thin, bond_at, guard_at) = scene?;
    let gi = sim.drones.list.iter().position(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team != 2 && d.weapon != 0 && d.clip > 0 && d.watching
        && d.state != nf_game::drone::state::CIVILIAN_MISSION
        // A gun: the records with `+0x6C & 0x800` (the fists) stop on glass [M 0x12e7b0:390].
        && sim.weapons.records.get(d.weapon as usize).is_some_and(|r| r[0x6d] & 0x08 == 0))?;
    sim.bond.pos = [bond_at[0], bond_at[1] + 1.03, bond_at[2]];
    for (i, d) in sim.drones.list.iter_mut().enumerate() { if i != gi { d.present = false; } }
    {
        let d = &mut sim.drones.list[gi];
        d.pos = [guard_at[0], guard_at[1] + 1.0, guard_at[2]];
        let (dx, dz) = (bond_at[0] - guard_at[0], bond_at[2] - guard_at[2]);
        d.yaw = dx.atan2(dz);
    }
    let difficulty = sim.drones.difficulty;
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
        ai_points: &sim.ai_points,
    };
    let frame = sim.drones.frame;
    sim.drones.list[gi].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
    let mut hurt = false;
    for _ in 0..900 {
        sim.step_bond(Input::default(), false, false, false);
        sim.bond.pos[0] = bond_at[0];
        sim.bond.pos[2] = bond_at[2];
        hurt |= sim.drones.health.health < 100.0;
        sim.drones.health.health = 100.0;
        sim.drones.health.armour = 0.0;
    }
    let after = sim.drones.breaks.list.iter().find(|b| b.placement == placement)?;
    let drained = after.dead || after.health < health0;
    // `Collide_FilterBullets` 0x1ece30:24-33: the weapon's category word against the pane's
    // mask | 0x106 drops the contact, and the round still passes [M Break_Update 0x12bfd8:17].
    let r = &sim.weapons.records[sim.drones.list[gi].weapon as usize];
    let filtered = u16::from_le_bytes([r[0x10], r[0x11]]) & (after.filter | 0x106) != 0;
    eprintln!("{lv} pane #{placement}: guard #{gi} state {:#x} at {guard_at:?}, Bond at {bond_at:?}: drained {drained} filtered {filtered} (health {health0} -> {}, dead {}), Bond hurt {hurt}", sim.drones.list[gi].state, after.health, after.dead);
    Some(Outcome { drained, filtered, hurt })
}

#[test]
fn guard_rounds_pass_and_drain_glass_on_every_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut failed = Vec::new();
    let mut covered = 0;
    for lv in LEVELS {
        let mut tried = 0;
        let mut ok = false;
        for pane in 0..3 {
            let Some(o) = try_pane(&iso, lv, pane) else { break };
            tried += 1;
            if o.hurt && o.drained != o.filtered { ok = true; break; }
        }
        eprintln!("{lv}: {tried} scene(s), {}", if ok { "pass" } else { "FAIL" });
        if tried > 0 && !ok { failed.push(lv); }
        if tried > 0 { covered += 1; }
    }
    // 07000001 (a pane that breaks) and 07000012 (the sky bridge's 0xffdf-filtered panes, passed
    // but not drained) carry a class-0x10 scene. 07000003's two scenes (#338/#339) were mat-0x05
    // panes the guard's sight cannot cross (see `see_through`), and its class-0x10 panes have no
    // armed guard and floor pair; the rest have no armed guard and floor pair at a pane.
    assert!(covered >= 2, "only {covered} level(s) had a pane scene");
    assert!(failed.is_empty(), "guard rounds stopped at the glass on {failed:?}");
}
