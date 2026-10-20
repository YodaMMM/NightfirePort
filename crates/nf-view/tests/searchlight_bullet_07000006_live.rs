//! searchlight-bullet-06 (U-041 ported): a player bullet against a 07000006 courtyard searchlight
//! head goes through the game's contact path — the flying bullet's pick tests the head's world
//! sphere (obj+0x80/0x8C, the posed glist bounds [M Control_BuildWorldSph 0x133070]; the type-0x201
//! segment test [M Collide_PickObj 0x1eae80:92-141]) and the head's update breaks on a class-5
//! contact with damage > 0.0 [M Searchlight_Update 0x1cb900:406-462]. Driven through the REAL path
//! (`nf_view::load` + `Sim::step_bond` inputs; Bond is PLACED at a computed stand spot with
//! `Bond::stand_at`, the same driver-placement stand-in the walkreal/flip tests use — everything
//! after it is real input). Opt-in via NF_ISO.
//!
//! Fail-first vs the removed stand-in: the old code tested the head's AND the mount's gfx boxes and
//! hid every link including the mount — T1's mount assertion and T2's no-break both fail there.
//!
//! Docs: docs/systems/searchlight-bullet.md.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A head's posed world sphere (obj+0x80/0x8C): `Control_BuildWorldSph` poses the glist's local
/// bounds centre by the object's matrix (radius · |obj+0xE8|, obj+0xE8 = 1.0 [M 0x134630:17]).
fn posed_sphere(m: &[[f32; 4]; 4], c: [f32; 3], r: f32) -> ([f32; 3], f32) {
    ([m[0][0] * c[0] + m[0][1] * c[1] + m[0][2] * c[2] + m[3][0],
      m[1][0] * c[0] + m[1][1] * c[1] + m[1][2] * c[2] + m[3][1],
      m[2][0] * c[0] + m[2][1] * c[1] + m[2][2] * c[2] + m[3][2]], r)
}

/// Bond's look vector from yaw/pitch (`Bond::look` [M RotMatrixX 0x1e46a0]): yaw 0 = +Z, larger
/// turns left; pitch in units of π/2. Inverse: place Bond so look() points at `target`.
fn aim_angles(from: [f32; 3], target: [f32; 3]) -> (f32, f32) {
    let d = [target[0] - from[0], target[1] - from[1], target[2] - from[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    (d[0].atan2(d[2]), (d[1] / len).asin() * std::f32::consts::FRAC_2_PI)
}

fn load_settled(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!sim.disabled, "Bond is free after the intro");
    // The boot NIS stages the heads' on-channels [D 0x06000066 stream 0]; wait for a sweeping head.
    let mut lit = false;
    for _ in 0..240u32 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.searchlights.parts.iter().any(|p| p.kind == 1 && p.state == 0) { lit = true; break; }
    }
    assert!(lit, "a courtyard head sweeps (the boot raised its on-channel)");
    scene
}

/// T1: a bullet through the head's world sphere breaks the light like the game — state 2, the
/// spot/beam/corona deleted (links[2..4] cleared, the spot's light node freed), the MOUNT kept
/// [M 0x1cb900:418-456]. The old stand-in hid the mount too (and tested boxes, not the sphere).
#[test]
fn a_player_bullet_breaks_a_courtyard_head_like_the_game() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_settled(&iso);
    let spot_idx;
    {
    let sim = scene.sim.as_mut().unwrap();

    // The head: a LIT kind-1 part (state 0/1) with a linked spot (the fullest break set) and a mount.
    let head = (0..sim.drones.searchlights.parts.len())
        .find(|&i| sim.drones.searchlights.parts[i].kind == 1
            && matches!(sim.drones.searchlights.parts[i].state, 0 | 1)
            && sim.drones.searchlights.parts[i].links[2].is_some()
            && sim.drones.searchlights.parts[i].links[0].is_some())
        .expect("a lit 07000006 head with a spot and a mount");
    let (centre, radius) = {
        let h = &sim.drones.searchlights.parts[head];
        eprintln!("[sl] head {head} at {:?} sph_centre {:?} r {} state {}", h.pos, h.sph_centre, h.sph_radius, h.state);
        assert!(h.sph_radius > 0.2, "the head's glist sphere has a real radius");
        posed_sphere(&h.matrix, h.sph_centre, h.sph_radius)
    };
    eprintln!("[sl] posed sphere centre {centre:?} r {radius}");

    // Bond stands at the first candidate spot with a clear shot at the sphere centre (the driver
    // placement stand-in; candidates are deterministic — ring offsets around the head).
    let mut placed = false;
    for &(dx, dz) in &[(0.0f32, 9.0), (0.0, -9.0), (9.0, 0.0), (-9.0, 0.0), (7.0, 7.0), (-7.0, 7.0), (7.0, -7.0), (-7.0, -7.0)] {
        let spot = [centre[0] + dx, centre[1] - 0.5, centre[2] + dz];
        if !sim.world.line_of_sight([spot[0], spot[1] + 1.5, spot[2]], centre) { continue; }
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
        placed = true;
        eprintln!("[sl] bond at {spot:?} (offset {dx},{dz})");
        break;
    }
    assert!(placed, "a clear stand spot exists around the head");

    // The start weapon with ammo (the probe prints it; the shot needs a real bullet). The spot part
    // index is captured BEFORE the break clears links[2] — the tracked light record points at it.
    spot_idx = sim.drones.searchlights.parts[head].links[2].unwrap();
    eprintln!("[sl] gun weapon {} clip {}", sim.gun.weapon, sim.gun.clip);
    let mut shots = 0u32;
    for t in 0..1200u32 {
        // Aim at the posed centre every tick (the head sweeps; the sphere centre moves with it).
        let (c, _) = { let h = &sim.drones.searchlights.parts[head]; posed_sphere(&h.matrix, h.sph_centre, h.sph_radius) };
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye, sim.bond.pos[2]];
        let (yaw, pitch) = aim_angles(eye, c);
        sim.bond.yaw = yaw;
        sim.bond.pitch = pitch;
        // The semi-auto trigger: one shot per fresh press — pulse the fire bit.
        let fire = t % 4 != 0;
        let input = nf_game::player::Input { fire, ..Default::default() };
        sim.step_bond(input, fire, false, false);
        if fire { shots += 1; }
        if sim.drones.searchlights.parts[head].state == 2 { eprintln!("[sl] broken after {shots} shots (t {t})"); break; }
    }
    eprintln!("[sl] head state {} after {shots} fire ticks", sim.drones.searchlights.parts[head].state);
    assert_eq!(sim.drones.searchlights.parts[head].state, 2,
        "the head breaks (state 2) — the bullet's contact reached its obj+0xD0 list [M 0x1cb900:418]");

    // The game's break effects [M 0x1cb900:438-456]: spot/beam/corona deleted (links cleared), the
    // MOUNT kept, the glass 0xE6 in the sound events. The old stand-in hid the mount — this fails there.
    let (spot, beam, corona, mount) = {
        let h = &sim.drones.searchlights.parts[head];
        (h.links[2], h.links[3], h.links[4], h.links[0])
    };
    assert_eq!((spot, beam, corona), (None, None, None), "links[2..4] cleared [M 0x1cb900:438-453]");
    assert!(mount.is_some() && sim.drones.searchlights.parts[mount.unwrap()].drawn,
        "the mount is NOT deleted [M 0x1cb900:438-453 never touches links[0]]");
    if let Some(sp) = spot { assert!(!sim.drones.searchlights.parts[sp].drawn, "the spot object deleted"); }
    assert!(sim.drones.searchlights.parts[spot_idx].light.is_none(),
        "the psiLight node freed (the part's node marker cleared) [M 0x1cb900:430-433]");
    }
    // The tracked light (track = the spot part index) is inactive through the same gate the renderer
    // reads — the psiLight_Delete equivalent [M 0x1cb900:430-433].
    let lights = scene.frame_lights();
    let dead = lights.lights.iter().filter(|l| l.kind == 2 && l.track == Some(spot_idx as u32)).all(|l| !l.active);
    assert!(dead, "the head's tracked light is inactive after the break (psiLight_Delete)");
}

/// T2 (the stand-in discriminator): a shot line through the MOUNT's volume but outside the head's
/// sphere does NOT break the light — the mount is its own class-0x22 object whose update returns
/// before the contact walk [M 0x1cb900:89], so nobody reads its contacts. The old stand-in walked
/// the mount's box and broke here.
#[test]
fn a_mount_only_shot_does_not_break_the_head() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_settled(&iso);
    let sim = scene.sim.as_mut().unwrap();
    let head = (0..sim.drones.searchlights.parts.len())
        .find(|&i| sim.drones.searchlights.parts[i].kind == 1
            && matches!(sim.drones.searchlights.parts[i].state, 0 | 1)
            && sim.drones.searchlights.parts[i].links[0].is_some())
        .expect("a lit head with a mount");
    let (mount, (centre, radius)) = {
        let h = &sim.drones.searchlights.parts[head];
        (h.links[0].unwrap(), posed_sphere(&h.matrix, h.sph_centre, h.sph_radius))
    };
    let mpos = sim.drones.searchlights.parts[mount].pos;
    let mbox = (sim.drones.searchlights.parts[mount].box_min, sim.drones.searchlights.parts[mount].box_max);
    eprintln!("[sl] head {head} sphere {centre:?} r {radius}; mount {mount} at {mpos:?} box {mbox:?}");
    // The discriminator's geometry: the mount's world box (a yaw rotation only — its matrix keeps the
    // local y extent) spans mount.y − 0.326..+0.126; the head's sphere reaches radius below its
    // centre. The window between the box's bottom and the sphere's reach is the shot line: inside the
    // box, outside the sphere (the probe prints the numbers; the margins are a few hundredths).
    let box_bottom = mpos[1] + mbox.0[1];
    let aim = [centre[0], (box_bottom + (centre[1] - radius)) * 0.5, centre[2]];
    eprintln!("[sl] aim y {:.3} (box bottom {:.3}, sphere reach {:.3}) — dist below centre {:.3} vs r {:.3}",
        aim[1], box_bottom, centre[1] - radius, centre[1] - aim[1], radius);
    assert!(aim[1] > box_bottom && centre[1] - aim[1] > radius,
        "the window exists: inside the mount's box, outside the head's sphere");
    let mut placed = false;
    for &(dx, dz) in &[(0.0f32, 9.0), (0.0, -9.0), (9.0, 0.0), (-9.0, 0.0), (7.0, 7.0), (-7.0, 7.0), (7.0, -7.0), (-7.0, -7.0)] {
        let spot = [aim[0] + dx, aim[1] - 0.5, aim[2] + dz];
        if !sim.world.line_of_sight([spot[0], spot[1] + 1.5, spot[2]], aim) { continue; }
        // The discriminator's premise: the line from the eye through `aim` passes the mount's box
        // (the stand-in would break) while missing the head's sphere by a clear margin.
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
        placed = true;
        eprintln!("[sl] bond at {spot:?}");
        break;
    }
    assert!(placed, "a clear stand spot exists for the mount shot");
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye, sim.bond.pos[2]];
    let (yaw, pitch) = aim_angles(eye, aim);
    eprintln!("[sl] aim angles yaw {yaw:.3} pitch {pitch:.3} (eye {eye:?} aim {aim:?})");
    for t in 0..600u32 {
        sim.bond.yaw = yaw;
        sim.bond.pitch = pitch;
        // The semi-auto trigger: one shot per fresh press — pulse the fire bit.
        sim.step_bond(nf_game::player::Input { fire: t % 4 != 0, ..Default::default() }, t % 4 != 0, false, false);
        if t % 120 == 0 {
            let n = sim.drones.bullet_paths_len();
            eprintln!("[sl] t {t} paths {n} head state {}", sim.drones.searchlights.parts[head].state);
        }
        assert_ne!(sim.drones.searchlights.parts[head].state, 2,
            "the head must NOT break from a mount-only shot [M 0x1cb900:89 — the mount's contacts are unread]");
    }
    assert!(matches!(sim.drones.searchlights.parts[head].state, 0 | 1), "still sweeping or tracking");
}
