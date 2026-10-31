//! door-guard-pairing on 07000007 — the only story level
//! that places door guards [D census below]: 6 guards whose mode resolves to 0x16 and whose start
//! state is CivilianDoorGuard 0x31 (`DroneTypeSettings` row 22: start 0x31 [L the table dump;
//! docs/fields/dronetype-tables.md]). The port had the const with no arms (WORKLOG 3969).
//!
//! The game's path [M] (corrected by the daily audit 2026-10-17 — the old header called the two
//! +0x34 gates "yaw" gates and built the plane normal on (0,0,1); both wrong):
//! - `NDrone2_SetDronePartner(0x16, dc)` 0x141360 at the 0x31 ENTER [M 0x160694]: the nearest
//!   other mode-0x16 guard (enabled `+0x4F8 & 0x600 == 0 && & 0x100 != 0`, health > 0, class
//!   != 0x11, not deleted) whose **position Y is within 2.0 of the caller's** — the +0x34 gate
//!   at :90 is a HEIGHT gate (+0x30 is the position vector, so +0x34 is Y; the yaw is +0x54,
//!   which is what :121 reads) — and partnerless; the pair is mutual (`+0x2BC` each side);
//!   `Vec_AngleDifference(my_yaw, atan2(dir))` < 0 marks the CALLER's +0x2CC = 0xff and builds
//!   the planes on the PARTNER, else the PARTNER gets 0xff and the planes are built on the
//!   CALLER — in both cases v1 = owner→partner, v2 = (0,1,0) [M disasm 0x14150c-0x141554 /
//!   0x1415f4-0x141614], anchored at the owner's own position [M 0x141360:115-174].
//! - `NDrone2_DoorGuardHandler` 0x1416d8 at the 0x31 UPDATE [M 0x160814]: no partner or no
//!   opponent (+0x170) → 0; the +0x2CC == 0xff holder returns the partner's +0x2CC (the zone
//!   the owner's handler tail stored there [M :75-77]); the planes' owner evaluates:
//!   |opp Y − my Y| ≥ 2.0 → 0 (a HEIGHT gate [M :22 + disasm 0x14170c-0x141738]), else the
//!   plane1/plane2 gates then the plane-0 bands s0 < −6 → 0, < 0 → 3, < 2 → 2, < 6 → 1, else 0
//!   [M 0x1416d8:16-73].
//! - The planes [M 0x141360:115-174 + 0x1e4498]: plane0.n = cross(v0, (0,1,0)) = (−v0.z, 0, v0.x)
//!   through the OWNER — horizontal; plane1.n = (−n0.z, 0, n0.x) = −v0 through the MIRROR;
//!   plane2.n = (n0.z, 0, −n0.x) = +v0 through the OWNER. No degenerate zero normal exists (a
//!   pair along z gives n = (∓1, 0, 0)).
//! - The 0x31 UPDATE arms [M 0x160648:38-105]: an armed opponent → prop-0x33 shout + Attack;
//!   zone 1 → face + the idle-anim replay on the step change; zone 2 → anim 0x5f variant 1 + the
//!   ChooseSFX(0x4f, 0x50) speech (the planes' owner only) + face; zone 3 → ChooseSFX(0x8f, 0x92)
//!   + `DroneFunc_SetMissionFailReason(drone, 0xc)` + Attack; zone 0 → the idle-anim replay.
//! - TIMER_C (case 0xc): the idle cycle 0x24 → 0x25 → 0x26 → 0x24 by the `+0xCC8` payload,
//!   re-armed at frame + (Rand(20) + 5)·FRAME_RATE_INT [M 0x160648:124-150; FRAME_RATE_INT = 30 —
//!   the port's DEAD-arm comment `DAT_00298F58 = 10 · FRAME_RATE_INT` = 300 ticks].
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{state, Data, Drones, Player, Rng};
use std::collections::HashMap;

pub struct L {
    tables: drone::DroneAnimData,
    props: drone::PropertyTables,
    weapons: WeaponData,
    level: Level,
    world: nf_game::world::CollisionWorld,
    nav: nf_game::nav::Network,
    covers: Vec<nf_game::cover::CoverNode>,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    scripts: HashMap<u32, nf_formats::anim::Script>,
    sequences: HashMap<u32, nf_formats::anim::Sequence>,
    skeletons: HashMap<u8, nf_formats::character::Skeleton>,
    models: HashMap<u32, nf_game::pickup::Model>,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
}

impl L {
    pub fn load() -> Option<L> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000007.bin")).unwrap();
        let raw = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&raw).unwrap();
        let lb = LevelBin::parse(&raw).unwrap();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let m = level.main_map().unwrap();
        let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap(), &world, 0x0700_0007);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, 0x0700_0007);
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| nf_formats::anim::Script::parse(lb.data(&raw, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&raw, e)).ok().map(|s| (s.id as u8, s))).collect();
        let models_hold = nf_game::pickup::models(&level);
        Some(L {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, 0x0700_0007); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            world, nav, covers, volumes, scripts, sequences: HashMap::new(), skeletons,
            // `models` derives from `&level` before the move.
            level, models: models_hold,
        })
    }

    pub fn data(&self) -> Data<'_> {
        Data {
            tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0007,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning,
            hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: None, models: &self.models, surfaces: None, voices: None, difficulty: 1, ai_points: &[],
        }
    }

    /// The full roster with one Rng through `level_drones` (the game's order), then a fresh `Drones`
    /// rng for the tick stream.
    pub fn sim(&self) -> (Drones, Rng) {
        let d = self.data();
        let mut rng = Rng::new(0x1234_5678);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        let s = Drones::new(drones, Rng::new(0x777));
        (s, Rng::new(0x777))
    }
}

pub fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None } }

pub fn idx(sim: &Drones, placement: usize) -> usize {
    sim.list.iter().position(|dr| dr.placement == placement).unwrap_or_else(|| panic!("placement {placement} on the roster"))
}

/// The door guards reach CivilianDoorGuard 0x31 and pair through `SetDronePartner`'s
/// sequential ENTER scans [M 0x141360 + 0x160694]. Fail-first: the const had no arms,
/// `door_partner` never filled.
#[test]
fn door_guards_pair_into_mutual_nearest_pairs() {
    let Some(l) = L::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (mut sim, _rng) = l.sim();
    // Settle two ticks: the started ENTERs, then the pairing pass (the port pairs at the tick
    // boundary after the ENTER [*S one-tick deviation from the in-ENTER pairing]).
    let p = bond_at([-500.0, 0.0, -500.0]);
    sim.tick(&d, &p);
    sim.tick(&d, &p);
    let (g404, g417, g441, g442) = (idx(&sim, 404), idx(&sim, 417), idx(&sim, 441), idx(&sim, 442));
    for &i in &[g404, g417, g441, g442] {
        assert_eq!(sim.list[i].state, state::CIVILIAN_DOOR_GUARD,
            "pl {} reached CivilianDoorGuard, state {:#x}", sim.list[i].placement, sim.list[i].state);
    }
    // The exact partner graph the game's sequential ENTERs leave [M 0x160694 — the 0x31 ENTER
    // calls SetDronePartner unconditionally; 0x141360:96 — the claim `me->0x2BC = cand` is
    // written INSIDE the scan; :113 — the mutual write `cand->0x2BC = me` lands after the scan
    // and overwrites any earlier claim on the candidate]: the list-order scans chain. 404
    // claims 417; 417's own ENTER still scans (nothing gates the caller) and claims the nearer
    // partnerless guard 442 (7.6 u vs 8.9 u); 441 finds every candidate already claimed; 442's
    // own ENTER claims 441. Only 441↔442 ends mutual; 404→417 and 417→442 are the stale claims
    // the later scans left behind. (The pre-audit test asserted two clean mutual pairs — that
    // outcome depended on the old wrong yaw gate filtering the cross-pair candidates [M
    // 0x141360:90 is a HEIGHT gate].)
    assert_eq!(sim.list[g404].door_partner, Some(g417), "404's ENTER claim");
    assert_eq!(sim.list[g417].door_partner, Some(g442), "417's own re-scan claim");
    assert_eq!(sim.list[g441].door_partner, Some(g442), "441's scan finds everything claimed");
    assert_eq!(sim.list[g442].door_partner, Some(g441), "442's own re-scan claim — the one mutual pair");
    // The HEIGHT gate [M 0x141360:90]: each claim's two objects' +0x34 (position Y) differ by
    // less than 2.0. (All six guards stand at Y 44.0 on this level [D the roster dump], so the
    // gate never separates them here — pin it anyway.) The old test asserted a yaw difference
    // here — the daily audit 2026-10-17 correction: +0x30 is the position vector, +0x54 the yaw.
    for (a, b) in [(g404, g417), (g417, g442), (g441, g442)] {
        assert!((sim.list[a].pos[1] - sim.list[b].pos[1]).abs() < 2.0,
            "pl {} and pl {} claim only within the 2.0 height gate", sim.list[a].placement, sim.list[b].placement);
    }
    // The planes [M 0x141360:115-174 + 0x1e4498 + disasm]: each final build is anchored at its
    // OWNER with v0 = owner→partner of the claim that built it. audit-fix-door-guard-3: the
    // pairing runs at the deferred watch UPDATE with the [S] watch-across rule (drone.rs
    // `door_guard_pass`), so every build takes the IF outcome — planes on the CANDIDATE, v0 =
    // −dir = candidate→caller. The three builds: 404's (flipped) lands on 417 with v0 = 417→404;
    // 417's own (natural diff < 0 [M 0x141360:121]) lands on 442 with v0 = 442→417; 442's own
    // (flipped) lands on 441 with v0 = 441→442. 441 owns the last build and evaluates; 404/417/442
    // hold 0xff (404 and 442 marked by their own flipped builds, 417 by his own natural-IF build).
    // plane0.n = cross(v0, (0,1,0)) = (−v0.z, 0, v0.x) — the game's second vector is (0,1,0)
    // [M disasm 0x14150c-0x141554 / 0x1415f4-0x141614], NOT (0,0,1), so the normal is HORIZONTAL
    // and the sign is fixed; a pair along z gives n = (∓1, 0, 0) — the old test's "the game's
    // own degenerate zero normal" never happens. plane1 = −v0 through that build's MIRROR,
    // plane2 = +v0 through the OWNER [M 0x141360:134-149 / 158-173].
    for (owner, mirror) in [(g417, g404), (g442, g417), (g441, g442)] {
        let got = sim.list[owner].door_planes;
        let (ox, oy, oz) = (sim.list[owner].pos[0], sim.list[owner].pos[1], sim.list[owner].pos[2]);
        let (mx, my, mz) = (sim.list[mirror].pos[0], sim.list[mirror].pos[1], sim.list[mirror].pos[2]);
        let dx = mx - ox;
        let dy = my - oy;
        let dz = mz - oz;
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        let v0 = [dx / len, dy / len, dz / len];
        let n = [-v0[2], 0.0f32, v0[0]]; // cross(v0, (0,1,0)) [M 0x1e4498:11-17]
        let d0 = -(n[0] * ox + n[1] * oy + n[2] * oz);
        let (a, b, c) = (n[0], n[1], n[2]);
        let expect = [
            [a, b, c, d0],
            // +0x2F0: {−c, b, a, d0 + a·owner.x + c·mirror.x + c·owner.z − a·mirror.z}
            // [M 0x141360:141-149 / 165-173 — algebraically −n1·mirror]
            [-c, b, a, d0 + a * ox + c * mx + c * oz - a * mz],
            // +0x300: {c, b, −a, d0 + (a − c)·owner.x + (c + a)·owner.z}
            // [M 0x141360:144-145 / 168-169 — algebraically −n2·owner]
            [c, b, -a, d0 + (a - c) * ox + (c + a) * oz],
        ];
        let close = |x: f32, y: f32| (x - y).abs() < 1e-3 || (x - y).abs() < 1e-3 * x.abs().max(1.0);
        let all = expect.iter().zip(got.iter()).all(|(e, g)| e.iter().zip(g.iter()).all(|(&x, &y)| close(x, y)));
        assert!(all, "pl {}'s planes match the [M] formula (mirror pl {}): got {got:?}",
            sim.list[owner].placement, sim.list[mirror].placement);
        // The normal is horizontal and unit-length — INCLUDING for the 441↔442 build whose axis
        // runs along z, where the old test expected the game's "degenerate zero normal"
        // [M disasm 0x14150c-0x141554: n = (∓1, 0, 0) there, never zero].
        let nlen = (a * a + b * b + c * c).sqrt();
        assert!((nlen - 1.0).abs() < 1e-3 && b.abs() < 1e-6,
            "plane0's normal is horizontal unit length: n = ({a},{b},{c}) len {nlen}");
        // The semantic pin, on the port's STORED planes: plane0 through the owner, plane1
        // through that build's mirror, plane2 through the owner [M the d-store chains cancel].
        let side = |pl: [f32; 4], q: [f32; 3]| pl[0] * q[0] + pl[1] * q[1] + pl[2] * q[2] + pl[3];
        assert!(side(got[0], sim.list[owner].pos).abs() < 1e-2, "plane0 passes through the owner");
        assert!(side(got[1], sim.list[mirror].pos).abs() < 1e-2, "plane1 passes through the build's mirror");
        assert!(side(got[2], sim.list[owner].pos).abs() < 1e-2, "plane2 passes through the owner");
    }
    // The 0xff markers after the chain [M the branches at 0x141360:123/151 set exactly one side
    // per build; audit-fix-door-guard-3's [S] watch-across rule routes the diff ≥ 0 builds through
    // the IF branch]: 404's flipped build marked HIMSELF; 417's own natural-IF build marked
    // himself; 442's flipped build marked herself — 441 (the last build's candidate) never marked.
    assert_eq!(sim.list[g404].door_side, 0xff, "404 marked by his own flipped build");
    assert_eq!(sim.list[g417].door_side, 0xff, "417 marked by his own natural-IF build");
    assert_eq!(sim.list[g441].door_side, 0, "441 owns 442's build");
    assert_eq!(sim.list[g442].door_side, 0xff, "442 marked by her own flipped build");
    // 404 holds no planes [M the three builds above all landed on 417/442/441].
    assert!(sim.list[g404].door_planes.iter().all(|p| p.iter().all(|&v| v == 0.0)),
        "404 never had planes built on him");
}

/// The zone bands drive the UPDATE arms [M 0x1416d8:55-73 + 0x160648:59-103]. With the game's
/// planes every normal is horizontal, so all three side values are horizontal offsets — none is
/// a height: s0 is the signed LATERAL offset from the owner↔partner line, s1 ≥ 0 and s2 ≥ 0
/// select the slab between the two guards (plane1 = −v0 through the mirror, plane2 = +v0 through
/// the owner [M 0x141360:134-149 / 158-173]). The bands on s0: [−6,0) → 3 (the mission-fail
/// side), [0,2) → 2 (the challenge: the 0x5f anim once, the planes' owner speaks), [2,6) → 1.
/// The read only happens within the HEIGHT gate [M 0x1416d8:22]: |opp Y − my Y| ≤ 2.0.
#[test]
fn zone_bands_drive_the_challenge_and_the_mission_fail() {
    let Some(l) = L::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (mut sim, _rng) = l.sim();
    let guards: Vec<usize> = [404usize, 417, 441, 442].iter().map(|&pl| idx(&sim, pl)).collect();
    let p0 = bond_at([-500.0, 0.0, -500.0]);
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    let side = |pl: [f32; 4], p: [f32; 3]| pl[0] * p[0] + pl[1] * p[1] + pl[2] * p[2] + pl[3];
    // For each planes' owner, probe standing spots in front of him (along his facing) whose
    // sides land in the challenge band, with a clear line. (The gate is the OPPONENT's height
    // against the evaluator's own +0x34 [M 0x1416d8:22] — the probe spots stand on the guard's
    // own floor, and no aim is involved anywhere in the function.)
    let mut challenge: Vec<(usize, [f32; 3])> = Vec::new();
    let mut fail: Vec<(usize, [f32; 3])> = Vec::new();
    let mut raised: Vec<(usize, [f32; 3])> = Vec::new();
    for &ev in &guards {
        if sim.list[ev].door_side == 0xff { continue; } // the mirror holds no planes
        let planes = sim.list[ev].door_planes;
        let anchor = sim.list[ev].pos;
        let face = [sim.list[ev].yaw.sin(), 0.0, sim.list[ev].yaw.cos()];
        for k in [1usize, 2, 3, 4] {
            for (dx, dz) in [(0i32, 0i32), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                let cand = [anchor[0] + face[0] * k as f32 + dx as f32 * 0.3, anchor[1], anchor[2] + face[2] * k as f32 + dz as f32 * 0.3];
                let (s0, s1, s2) = (side(planes[0], cand), side(planes[1], cand), side(planes[2], cand));
                if !d.world.line_of_sight([cand[0], cand[1] + 1.0, cand[2]], [anchor[0], anchor[1] + 1.4, anchor[2]], nf_game::world::LOS_GUARD_VISION) { continue; }
                if (0.0..2.0).contains(&s0) && s1 >= 0.0 && s2 >= 0.0 {
                    challenge.push((ev, cand));
                    // The height-gate sister spot: the same spot with Bond 2.5 above the guard's
                    // floor — past the [M 0x1416d8:22] gate — provided the world ray from the
                    // raised eye still reaches the guard (so vision cannot mask the gate).
                    let up = [cand[0], anchor[1] + 2.5, cand[2]];
                    if d.world.line_of_sight([up[0], up[1] + 1.0, up[2]], [anchor[0], anchor[1] + 1.4, anchor[2]], nf_game::world::LOS_GUARD_VISION) {
                        raised.push((ev, up));
                    }
                }
                if (-6.0..0.0).contains(&s0) && s1 >= 0.0 && s2 >= 0.0 { fail.push((ev, cand)); }
            }
        }
    }
    assert!(!challenge.is_empty(), "no challenge spots (the geometry probe)");
    // The ARMED sister [M 0x160648:38-58]: `NDrone2_NearArmedPerson` runs BEFORE the zone read —
    // an armed opponent at the same spot turns the guard hostile (the prop-0x33 shout) once the
    // sight is acked. The port's `bond_weapon` gate [M the armed set 2..0x47].
    let (guard, pos) = challenge[0].clone();
    {
        let (mut sim, _rng) = l.sim();
        sim.tick(&d, &p0);
        sim.tick(&d, &p0);
        let ev = idx(&sim, sim.list[guard].placement);
        let p = bond_at(pos);
        let mut hostile = false;
        for _ in 0..90 {
            sim.tick(&d, &p);
            if matches!(sim.list[ev].state, state::ATTACK | state::DRAW_WEAPON | state::COMBAT | state::AIM_STAND | state::AIM_STAND_FIRE | state::HEARD_NOISE_AWARE | state::HEARD_NOISE_ALERT) {
                hostile = true;
                break;
            }
        }
        assert!(hostile, "the armed opponent never turned the door guard hostile (state {:#x})", sim.list[ev].state);
        // (No "no challenge" assertion: `NearArmedPerson` needs the reaction-acked sight +0x228 & 4
        // [M 0x1787c8:37], so for the reaction window the guard's case 3 falls through to
        // `DoorGuardHandler` [M 0x160648:39 → :58] and the zone-2 challenge may start before the
        // armed switch — the old instant-los stand-in never let it.)
    }
    // The challenge: stand UNARMED Bond there (fists, weapon 1 — outside the armed set). No aim
    // is set toward the guard — the game's zone read has no aim input [M 0x1416d8:16-73].
    let (mut sim, _rng) = l.sim();
    sim.bond_weapon = 1;
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    let ev = idx(&sim, sim.list[guard].placement);
    let p = bond_at(pos);
    let mut saw_challenge = false;
    for _ in 0..90 {
        sim.tick(&d, &p);
        if sim.list[ev].door_zone == 2 && sim.list[ev].door_zone_step == 2 { saw_challenge = true; break; }
        assert!(sim.list[ev].fail_reason.is_none(), "the challenge band must not fail the mission");
    }
    assert!(saw_challenge, "the challenge band never fired: zone {} step {} los {}",
        sim.list[ev].door_zone, sim.list[ev].door_zone_step, sim.list[ev].los);

    // The HEIGHT-gate sister [M 0x1416d8:22]: the same challenge spot with Bond 2.5 ABOVE the
    // guard's floor never reads the challenge band — the opponent's |ΔY| against the
    // evaluator's own +0x34 is past the 2.0 gate — while the vision path still sees him
    // (`los` true: the zero zone is the height gate's doing, not vision's).
    if let Some((guard, pos)) = raised.first().cloned() {
        let (mut sim, _rng) = l.sim();
        sim.bond_weapon = 1;
        sim.tick(&d, &p0);
        sim.tick(&d, &p0);
        let ev = idx(&sim, sim.list[guard].placement);
        let p = bond_at(pos);
        let mut saw_challenge = false;
        for _ in 0..90 {
            sim.tick(&d, &p);
            if sim.list[ev].door_zone == 2 && sim.list[ev].door_zone_step == 2 { saw_challenge = true; break; }
        }
        assert!(sim.list[ev].los, "the raised opponent must still be seen (los true) — pl {} at {pos:?}", sim.list[guard].placement);
        assert!(!saw_challenge, "a Bond 2.5 above the evaluator must not read the challenge band (the 0x1416d8:22 height gate): zone {}", sim.list[ev].door_zone);
    } else {
        eprintln!("[sister skipped: no raised challenge spot with a clear line on this level's pairs — the gate itself is [M 0x1416d8:22]]");
    }

    // The restricted band: a seen player with s0 ∈ [−6, 0) — the mission-fail SIDE of the
    // owner↔partner line, inside the slab — → the fail reason 0xc + hostile.
    let (mut sim, _rng) = l.sim();
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    if let Some((guard, pos)) = fail.first().cloned() {
        let ev = idx(&sim, sim.list[guard].placement);
        sim.bond_weapon = 1; // the zones only run for an unarmed opponent [M NearArmedPerson's gate]
        let p = bond_at(pos);
        let mut saw_fail = false;
        for _ in 0..90 {
            sim.tick(&d, &p);
            if sim.list[ev].fail_reason.is_some() { saw_fail = true; break; }
        }
        assert!(saw_fail, "the restricted band never raised the fail reason (zone {})", sim.list[ev].door_zone);
        assert!(matches!(sim.list[ev].state, state::ATTACK | state::COMBAT | state::AIM_STAND | state::AIM_STAND_FIRE | state::AIM_CROUCH | state::AIM_CROUCH_FIRE),
            "the guard went hostile, state {:#x}", sim.list[ev].state);
    } else {
        eprintln!("[sister skipped: no restricted-band spot with a clear line on this level's pairs — the arm itself is [M 0x160648:89-94]]");
    }
}
/// The waiting pair (pl 448/449, switch channel 21) never pairs while the channel is off — the
/// pairing only happens at a 0x31 ENTER [M 0x160694] — and Bond at their feet raises nothing.
/// Pulsing the channel activates them into 0x31 and they pair [M WaitSwitch 0x15b960 case 3 →
/// Enable(1) + SetState(+0x5A2)].
#[test]
fn the_waiting_pair_activates_through_its_channel() {
    let Some(l) = L::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (mut sim, _rng) = l.sim();
    let a = idx(&sim, 448);
    let b = idx(&sim, 449);
    sim.bond_weapon = 1; // the fists: an UNARMED opponent never trips the 0x31 UPDATE's NearArmedPerson arm [M 0x1787c8] — this test's subject is the channel→pair path, not the armed arm
    let p = bond_at([sim.list[a].pos[0] + 1.0, sim.list[a].pos[1], sim.list[a].pos[2]]);
    for _ in 0..30 {
        sim.tick(&d, &p);
        assert_eq!(sim.list[a].state, state::WAIT_SWITCH, "the waiting pair waits");
        assert!(sim.list[a].door_partner.is_none() && sim.list[b].door_partner.is_none(), "no pairing while waiting");
    }
    // The channel pulse [D the level's ch 21 waits; the port's Channels::set mirrors the game's
    // switch writer].
    sim.channels.set(21, 1, sim.frame);
    let mut paired = false;
    for _ in 0..30 {
        sim.tick(&d, &p);
        if sim.list[a].state == state::CIVILIAN_DOOR_GUARD && sim.list[a].door_partner.is_some() { paired = true; break; }
    }
    assert!(paired, "the activated guards entered 0x31 and paired: a state {:#x} partner {:?}", sim.list[a].state, sim.list[a].door_partner);
}

/// guard-opponent-pointer: the zone bands never read Y — the planes' normals are horizontal
/// (n = cross(v0, (0,1,0)) [M 0x141360:115-174 + 0x1e4498], the b coefficient 0) — so the ONLY
/// Y input in `NDrone2_DoorGuardHandler` is the [M 0x1416d8:22] height gate
/// |opp +0x34 − my +0x34| ≤ 2.0. The original test's raised sister pins the >2.0 refuse side;
/// this pins the pass side's margin: Bond 1.9 BELOW the guard's floor, inside the challenge
/// band, still reads the zone-2 challenge. (The lost-sight zone case is unreachable in this
/// level's ray world — every in-band spot has a clear eye ray, and floors cull their back
/// face [M 0x10a8c8:168] — it is a live check in the report.)
#[test]
fn the_zone_bands_read_a_bond_below_the_floor_within_the_height_gate() {
    let Some(l) = L::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (mut sim, _rng) = l.sim();
    let p0 = bond_at([-500.0, 0.0, -500.0]);
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    let side = |pl: [f32; 4], p: [f32; 3]| pl[0] * p[0] + pl[1] * p[1] + pl[2] * p[2] + pl[3];
    let guards: Vec<usize> = [404usize, 417, 441, 442].iter().map(|&pl| idx(&sim, pl)).collect();
    let mut below_challenge: Option<(usize, [f32; 3])> = None;
    for &ev in &guards {
        if sim.list[ev].door_side == 0xff { continue; }
        let planes = sim.list[ev].door_planes;
        let anchor = sim.list[ev].pos;
        let face = [sim.list[ev].yaw.sin(), 0.0, sim.list[ev].yaw.cos()];
        'outer: for k in [1usize, 2, 3, 4] {
            for (dx, dz) in [(0i32, 0i32), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                let cand = [anchor[0] + face[0] * k as f32 + dx as f32 * 0.3, anchor[1] - 1.9, anchor[2] + face[2] * k as f32 + dz as f32 * 0.3];
                let (s0, s1, s2) = (side(planes[0], cand), side(planes[1], cand), side(planes[2], cand));
                if (0.0..2.0).contains(&s0) && s1 >= 0.0 && s2 >= 0.0 { below_challenge = Some((ev, cand)); break 'outer; }
            }
        }
        if below_challenge.is_some() { break; }
    }
    let Some((guard, pos)) = below_challenge else { panic!("no below-floor challenge spot (the geometry probe)") };
    sim.bond_weapon = 1;
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    let ev = idx(&sim, sim.list[guard].placement);
    let p = bond_at(pos);
    let mut saw = false;
    for _ in 0..90 {
        sim.tick(&d, &p);
        if sim.list[ev].door_zone == 2 && sim.list[ev].door_zone_step == 2 { saw = true; break; }
        assert!(sim.list[ev].fail_reason.is_none(), "the challenge band must not fail the mission");
    }
    assert!(saw, "a Bond 1.9 below the floor (inside the [M 0x1416d8:22] gate) never challenged: zone {} step {}",
        sim.list[ev].door_zone, sim.list[ev].door_zone_step);
}

/// guard-opponent-pointer: the zone dies with Bond, NOT with sight. Health 0 zeroes the fold's
/// pointer [M 0x1424e0:111-114] and the handler's first gate returns 0 [M 0x1416d8:9-11] —
/// from a spot the guard still sees (los true), so the zero is the POINTER's doing.
#[test]
fn the_zone_dies_with_bond_even_seen() {
    let Some(l) = L::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (mut sim, _rng) = l.sim();
    let p0 = bond_at([-500.0, 0.0, -500.0]);
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    let side = |pl: [f32; 4], p: [f32; 3]| pl[0] * p[0] + pl[1] * p[1] + pl[2] * p[2] + pl[3];
    let guards: Vec<usize> = [404usize, 417, 441, 442].iter().map(|&pl| idx(&sim, pl)).collect();
    let mut seen_challenge: Option<(usize, [f32; 3])> = None;
    for &ev in &guards {
        if sim.list[ev].door_side == 0xff { continue; }
        let planes = sim.list[ev].door_planes;
        let anchor = sim.list[ev].pos;
        let face = [sim.list[ev].yaw.sin(), 0.0, sim.list[ev].yaw.cos()];
        'outer: for k in [1usize, 2, 3, 4] {
            for (dx, dz) in [(0i32, 0i32), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                let cand = [anchor[0] + face[0] * k as f32 + dx as f32 * 0.3, anchor[1], anchor[2] + face[2] * k as f32 + dz as f32 * 0.3];
                if !d.world.line_of_sight([cand[0], cand[1] + 1.0, cand[2]], [anchor[0], anchor[1] + 1.4, anchor[2]], nf_game::world::LOS_GUARD_VISION) { continue; }
                let (s0, s1, s2) = (side(planes[0], cand), side(planes[1], cand), side(planes[2], cand));
                if (0.0..2.0).contains(&s0) && s1 >= 0.0 && s2 >= 0.0 { seen_challenge = Some((ev, cand)); break 'outer; }
            }
        }
        if seen_challenge.is_some() { break; }
    }
    let Some((guard, pos)) = seen_challenge else { panic!("no seen challenge spot (the geometry probe)") };
    sim.bond_weapon = 1;
    sim.tick(&d, &p0);
    sim.tick(&d, &p0);
    let ev = idx(&sim, sim.list[guard].placement);
    let p = bond_at(pos);
    let mut saw = false;
    for _ in 0..90 {
        sim.tick(&d, &p);
        if sim.list[ev].door_zone == 2 && sim.list[ev].door_zone_step == 2 { saw = true; break; }
    }
    assert!(saw, "the seen challenge never fired: zone {} step {} state {:#x}",
        sim.list[ev].door_zone, sim.list[ev].door_zone_step, sim.list[ev].state);
    // The sight evidence is the probe's FRESH ray (the port's `los` field is the rotating
    // per-tick budget — its value at this tick proves nothing): the spot was chosen with a
    // clear line and the stationary guard has not moved.
    assert!(d.world.line_of_sight([pos[0], pos[1] + 1.0, pos[2]], [sim.list[ev].pos[0], sim.list[ev].pos[1] + 1.4, sim.list[ev].pos[2]], nf_game::world::LOS_GUARD_VISION),
        "the spot must be IN sight for the pin to bite");
    // Bond dies: the fold zeroes +0x170 [M 0x1424e0:111-114] → the zone gate [M 0x1416d8:9-11]
    // zeroes the zone on the next read — the same tick's sight stamp keeps `has_opponent` true
    // (the grace [M 0x14ad58:11-14]), which the handler does NOT read.
    for _ in 0..8 { sim.health.health = 0.0; sim.tick(&d, &p); }
    assert_eq!(sim.list[ev].door_zone, 0, "a dead opponent zeroes the zone [M 0x1416d8:9-11]");
}

