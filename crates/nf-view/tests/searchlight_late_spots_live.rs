//! searchlight-late-spots (GAPS G-036 item 1): the "late spot creation" premise is false — the
//! disc carries only 2 kind-2 spots for 07000005's 7 heads (the whole-FILES.BIN sweep [D]), and
//! no code creates class-0x22 parts outside the load-time placement hook [M Searchlight_Create
//! 0x1cb4f8's sole caller is parsemap_create_dynamic_objects 0x1d04f0; Script_CreateEntity
//! 0x1d8968 creates no class 0x22; the dynamic walk 0x1d1b68 runs twice per load, never later].
//! The port already matches the game: exactly 1 of 7 heads linked at start [L ee_start_05.bin],
//! forever. No drawing changes. Opt-in via NF_ISO; docs/systems/searchlight-late-spots.md.

use nf_game::searchlight::Part;
use nf_view::scene::Sim;

const LEVEL: &str = "07000005.bin";

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The disc's census for 07000005's main map, placement order = creation order [D].
const HEADS: [usize; 7] = [27, 194, 199, 205, 210, 230, 261];
const SPOTS: [usize; 2] = [31, 33];
const ORPHANS: [usize; 6] = [194, 199, 205, 210, 230, 261];
const LINKED_HEAD: usize = 27;
const LINKED_SPOT: usize = 31;
/// The down-aiming orphan used for the arm test (its cone covers walkable ground below).
const ARM_ORPHAN: usize = 194;

fn load_level(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, LEVEL, false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    scene
}

fn part<'a>(sim: &'a Sim, placement: usize) -> &'a Part {
    sim.drones.searchlights.parts.iter().find(|p| p.placement == placement).unwrap_or_else(|| panic!("part for placement {placement}"))
}

fn linked_heads(sim: &Sim) -> Vec<&Part> {
    sim.drones.searchlights.parts.iter().filter(|p| p.kind == 1 && p.links[2].is_some()).collect()
}

/// The start state equals the original snapshot: 29 parts (the disc's census [D]), exactly 1 of
/// 7 heads linked [L ee_start_05.bin], the linked pair the disc's order-dependent one (head 27 ←
/// spot 31, created after the head [M 0x1cb6c0–0x1cb798 branch 4b]), both spots at the kind-2
/// −10000 hold [M 0x1cb8a0–0x1cb8cc]. Sister: the 6 orphan heads are unlinked — fails if the
/// linking ever over-reaches the game's order-dependent rule.
#[test]
fn the_start_state_is_the_discs_census_with_one_linked_head() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let scene = load_level(&path);
    let sim = scene.sim.as_ref().unwrap();
    let sl = &sim.drones.searchlights;
    let count = |k: u8| sl.parts.iter().filter(|p| p.kind == k).count();
    assert_eq!(sl.parts.len(), 29, "29 class-0x22 parts — the disc's census [D]");
    assert_eq!((count(0), count(1), count(2), count(3), count(4)), (7, 7, 2, 6, 7), "[D 07000005: 7 heads, 2 spots, 7 discs, 6 beams, 7 bulbs]");
    let linked = linked_heads(sim);
    assert_eq!(linked.len(), 1, "1 of 7 heads linked [L ee_start_05.bin]");
    assert_eq!(linked[0].placement, LINKED_HEAD, "the linked head is placement {LINKED_HEAD} [D order]");
    let spot = &sl.parts[linked[0].links[2].unwrap()];
    assert_eq!(spot.placement, LINKED_SPOT, "linked to spot placement {LINKED_SPOT} [D order]");
    let (h, s) = (part(sim, LINKED_HEAD), part(sim, LINKED_SPOT));
    let d = [(h.pos[0] - s.spawn_pos[0]), (h.pos[1] - s.spawn_pos[1]), (h.pos[2] - s.spawn_pos[2])]
        .iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(d < 3.0, "the pair sits within the 3-unit walk ({d:.2}) [M 0x1cb6d0]");
    // both spots hold at placement − 10000 until their head's first ray hit [M the kind-2 epilogue]
    for spl in SPOTS {
        let sp = part(sim, spl);
        assert!((sp.pos[0] - (sp.spawn_pos[0] - 10000.0)).abs() < 0.5, "spot {spl} at the -10000 hold");
    }
    // sister: the 6 orphans never linked (the −10000 orphan rule [L spot 016e8ec0])
    for pl in ORPHANS {
        assert!(part(sim, pl).links[2].is_none(), "orphan head {pl} must be unlinked");
    }
}

/// No late creation: after 700 ticks of play the part surface is exactly the load's (the game's
/// Searchlight_Create has no caller after the two load-time walk passes [M 0x1d1cf0:65,
/// 0x1d6360:146] and the disc carries no further spots [D]). Sister: the orphans are still
/// unlinked — the thing a late spot's create-walk would change.
#[test]
fn no_late_creation_the_part_surface_never_grows() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load_level(&path);
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..700 { sim.step_bond(Default::default(), false, false, false); }
    let sl = &sim.drones.searchlights;
    assert_eq!(sl.parts.len(), 29, "no part appears late [M 0x1cb4f8's sole caller is the load-time hook]");
    assert_eq!(linked_heads(sim).len(), 1, "still 1 linked head");
    assert_eq!(linked_heads(sim)[0].placement, LINKED_HEAD);
    for pl in ORPHANS {
        assert!(sl.parts.iter().find(|p| p.placement == pl).unwrap().links[2].is_none(), "orphan {pl} still unlinked");
    }
}

/// The arm law is spot-only [M 0x1cbf14–0x1cbf54: the 0→1 transition reads the spot's position,
/// and links[2] == 0 skips the whole block — 0x1cbdb0 `beqz $s1`]: an orphan head never leaves
/// the sweep even with Bond parked where its beam cone meets the ground. Sister (can fail): the
/// same head with a test-side link poke — the ONLY change is the link — arms to state 1 under
/// the same Bond, proving the blocker is the (correct) missing link and not the arm path.
#[test]
fn an_orphan_head_never_tracks_but_a_poked_link_arms_it() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load_level(&path);
    let sim = scene.sim.as_mut().unwrap();
    // the orphan's cone centre: the game's own two stored angles at the sweep's centre are
    // (yaw, pitch) = (inst+0x14, inst+0x3C) — the port's (base_yaw, mid) [M 0x1cba68–0x1cbacc];
    // the beam direction is `spherical(r, yaw, -pitch)` [M 0x1e3948 via 0x1cbdcc].
    let h = part(sim, ARM_ORPHAN);
    // The sweep's beam directions: yaw = base_yaw + half·sinθ, pitch = mid + half·cosθ [M
    // 0x1cba68–0x1cbacc] — the beam traces that circle once per 480 ticks, so any sampled θ is a
    // direction the live beam actually points. Sample the ring, keep the first sample whose ray
    // meets walkable ground (normal y > 0.5) — Bond parks there and the sweep crosses him.
    let mut park: Option<[f32; 3]> = None;
    for k in 0..12 {
        let th = k as f32 * std::f32::consts::TAU / 12.0;
        let (yaw, pitch) = (h.base_yaw + h.half * th.sin(), h.mid + h.half * th.cos());
        let dir = nf_game::searchlight::spherical(1.0, yaw, -pitch);
        // start 2 units out: past the head's own housing (the game's spot ray excludes the head's
        // own object [M Collide_PickObj 0x1eae80:57-59]; the port's spot ray skips its own box).
        let from = [h.pos[0] + dir[0] * 2.0, h.pos[1] + dir[1] * 2.0, h.pos[2] + dir[2] * 2.0];
        let end = [h.pos[0] + dir[0] * 100.0, h.pos[1] + dir[1] * 100.0, h.pos[2] + dir[2] * 100.0];
        let Some((t, n, _)) = sim.world.segment_surface(from, end, true) else { continue };
        if n[1] <= 0.5 { continue; }
        park = Some([h.pos[0] + (end[0] - h.pos[0]) * t, h.pos[1] + (end[1] - h.pos[1]) * t, h.pos[2] + (end[2] - h.pos[2]) * t]);
        break;
    }
    let hit = park.expect("the orphan's sweep ring meets walkable ground");
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [hit[0], hit[1] + 0.5, hit[2]], 0.0);
    let before = part(sim, ARM_ORPHAN).yaw;
    for i in 0..700 {
        sim.step_bond(Default::default(), false, false, false);
        let st = part(sim, ARM_ORPHAN).state;
        assert_ne!(st, 1, "tick {i}: the orphan head armed without a spot — the arm block must be spot-only [M 0x1cbf14]");
        assert_ne!(st, 2, "tick {i}: the orphan broke (no bullet flew)");
    }
    assert!(part(sim, ARM_ORPHAN).yaw != before, "the head swept through the session (the test is not vacuous)");
    // sister: poke the link (test-side only) — the same Bond, the same head, now it arms.
    let spot_idx = sim.drones.searchlights.parts.iter().position(|p| p.placement == LINKED_SPOT).unwrap();
    sim.drones.searchlights.parts.iter().position(|p| p.placement == ARM_ORPHAN).map(|hi| {
        sim.drones.searchlights.parts[hi].links[2] = Some(spot_idx);
    });
    let mut armed = false;
    for _ in 0..900 {
        sim.step_bond(Default::default(), false, false, false);
        if part(sim, ARM_ORPHAN).state == 1 { armed = true; break; }
    }
    assert!(armed, "the poked head armed under the same Bond — the arm path works, the link was the blocker");
}
