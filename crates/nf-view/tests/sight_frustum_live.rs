//! sight-frustum (ai-sight-drawn-gate): the drawn gate (`obj+0xFC`) runs the game's camera frustum
//! instead of the old 60°/2-unit aim cone.
//!
//! The game's writer is `View_AddObjects` 0x1e62d8:71 [M]: obj+0xFC = 2 when the object's world
//! sphere passes `Vision_InView` (0x1e89e0 [M]) — the four side planes of the camera frustum
//! (`Vision_Calculate_View_Fustrum` 0x1e7df8 [M]: corners at depth 200, x = ±tan(view/2)·aspect·200,
//! y = ±tan(view/2)·200; view angle 0x3F860A92 = 60° [M Camera_CreateCameras 0x1bb2b8], aspect 4/3
//! [M Camera_CalcViewAngles 0x1baf70:29-33; L ee_snapshot.bin @ 0x2a37c4 = 0]) — over the
//! portal-visible cels. The viewer's matrix is the player camera's [M Camera_SetToPlayer 0x1bc048:
//! yaw matrix × RotMatrixX(−pitch·π/2), translation = Player_GetHeadPos], so the frustum tilts WITH
//! the look pitch. The old stand-in cone (`dot > 0.5` on the HORIZONTAL aim, or within 2 units) was
//! pitch-blind: looking up left every ahead guard "drawn" that the game's tilted frustum puts below
//! the bottom plane. The ray walk (the BULLET segment from the eye) stays — it stands in for the
//! portal/cel room gate the sim has no graph for (guards-0c).
//!
//! Both tests drive the REAL path: `nf_view::load` + `Sim::step_bond` inputs only — no state pokes;
//! the guards walk their own patrol on the shared RNG. Opt-in via `NF_ISO`.
//!
//! Numbers (deterministic on the slim ISO, difficulty 1): 07000001 start, 120 idle ticks —
//! Bond's eye (−8.5, 7.1, −9.8), look (0.00, 0.00, 1.00); guard placement 158 at 6.4 units,
//! 17.4° off the aim; placement 161 at 2.4 units, 19.9° — both bullet-clear from the eye and
//! inside the frustum. 40 look-up ticks (input.look = 0.01/tick, the on-foot pitch path
//! [M Player_Update's BL+0x8a8 integration, the port's player.rs :1495]) put Bond at pitch
//! 0.40 (≈ 37° up) with both still bullet-clear and still inside the old cone — but below the
//! tilted bottom plane: the port marks them undrawn.
//!
//! Census note (gas-rng-draws-3): the original third census guard, placement 261 (18.6 u,
//! 38.3° — inside only via the world-sphere slack), is DELETED before tick 120 on the shared
//! RNG stream: he carries prop 0x59 and the GO_TO_GOAL arrival tail deletes such guards
//! [M 0x165640:47-82]; on the re-landed stream he finishes his route during the intro. The
//! census is re-derived onto the staying guards 158/161 (the only other present, clear,
//! in-frustum guards at the start view — every other candidate is occluded). The world-sphere
//! slack's start-view coverage left with 261; `in_view`'s slack term itself is unchanged code
//! [M 0x133070 obj+0x8C].

use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> Sim {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    scene.sim.take().expect("sim")
}

/// One step through the real path; the level's intro cut-scene is skipped like the walkers do.
fn step(sim: &mut Sim, input: nf_game::player::Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}

fn guard<'a>(sim: &'a Sim, placement: usize) -> &'a nf_game::drone::Drone {
    sim.drones.list.iter().find(|d| d.placement == placement && d.present)
        .unwrap_or_else(|| panic!("guard placement {placement} missing or not present — re-derive the census"))
}

/// Bullet-clear from Bond's eye to the guard (the room-gate leg of `visible_to`): the rejection
/// assertions below are the frustum's verdict, not an occluder's.
fn ray_clear(sim: &Sim, d: &nf_game::drone::Drone) -> bool {
    sim.world.segment_bullet(sim.bond.head(), d.pos).is_none()
}

/// The frustum verdict the port computes for the guard at this tick.
fn frustum(sim: &Sim, d: &nf_game::drone::Drone) -> bool {
    nf_game::drone::in_view(sim.bond.head(), d.pos, nf_game::drone::DRONE_DRAWN_SLACK, &nf_game::drone::frustum_sides(sim.bond.look()))
}

/// The old stand-in's verdict (the pitch-blind horizontal cone) — cited in the report, not the
/// port: looking up keeps this true while the game's frustum rejects.
fn old_cone(sim: &Sim, d: &nf_game::drone::Drone) -> bool {
    let look = sim.bond.look();
    let v = [d.pos[0] - sim.bond.head()[0], d.pos[2] - sim.bond.head()[2]];
    let a = [look[0], look[2]];
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let al = (a[0] * a[0] + a[1] * a[1]).sqrt();
    l < 2.0 || (al > 1e-6 && (v[0] * a[0] + v[1] * a[1]) / (l * al) > 0.5)
}

/// The fix proof: Bond looks up and the ahead guards drop out of the drawn set — the game's tilted
/// frustum, not the pitch-blind cone. FAILS on the stand-in (it kept drawn=true: cone && clear ray).
#[test]
fn looking_up_leaves_the_below_plane_guards_undrawn() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = load();
    let idle = nf_game::player::Input::default();
    for _ in 0..120 { step(&mut sim, idle); }

    // Baseline at the start view (pitch ≈ 0): both guards are on-screen and drawn — the sister's
    // state, asserted here so the rejection below is provably the pitch's work.
    // (261, the original baseline guard, is deleted on arrival by the game's own law
    // [M 0x165640:47-82]; the census is re-derived onto the staying 158/161.)
    for p in [158usize, 161] {
        let d = guard(&sim, p);
        assert!(ray_clear(&sim, d), "placement {p} occluded at the start view — geometry moved, re-derive");
        assert!(d.drawn, "placement {p} undrawn at the start view (frustum {})", frustum(&sim, d));
    }

    // The real input path: look up 0.01 (π/2 units) per tick for 40 ticks → pitch ≥ 0.40 (≈ 37°).
    let mut up = idle;
    up.look = 0.01;
    for _ in 0..40 { step(&mut sim, up); }
    assert!(sim.bond.pitch >= 0.39, "the look input did not raise the pitch ({})", sim.bond.pitch);

    // placements 158 (≈ 5.0 units) and 161 (≈ 2.5 units) after the look-up: still bullet-clear,
    // still inside the old cone — and below the tilted bottom plane. [M 0x1e62d8:71 + 0x1e89e0:
    // the object is drawn only inside the four side planes; a point this far below fails the
    // bottom plane.]
    for p in [158usize, 161] {
        let d = guard(&sim, p);
        assert!(ray_clear(&sim, d), "placement {p} occluded after the look-up — geometry moved, re-derive");
        assert!(old_cone(&sim, d), "placement {p} left the old cone too — the discriminator moved, re-derive");
        assert!(!frustum(&sim, d), "placement {p} still inside the frustum after the look-up — re-derive");
        assert!(!d.drawn, "placement {p} drawn while below the tilted frustum — the stand-in behaviour");
    }
}

/// The sister: at the start view the on-screen guards stay drawn — the frustum must not over-reject
/// (guards-0c's slide fix rides this: an undrawn guard slides with no gait [M NDrone2_DoAnimation
/// 0x149f88]). Census re-derived onto the staying guards 158/161 (see the header note: 261 is
/// deleted on arrival [M 0x165640:47-82]; his 0.7°-outside-the-half-angle slack coverage left
/// with him — the slack term itself is unchanged [M 0x133070 obj+0x8C]).
#[test]
fn in_frustum_guards_stay_drawn_at_the_start_view() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = load();
    let idle = nf_game::player::Input::default();
    for _ in 0..120 { step(&mut sim, idle); }
    assert!(sim.bond.pitch.abs() < 0.05, "the start view is not level ({})", sim.bond.pitch);
    for p in [158usize, 161] {
        let d = guard(&sim, p);
        assert!(ray_clear(&sim, d), "placement {p} occluded at the start view — geometry moved, re-derive");
        assert!(frustum(&sim, d), "placement {p} outside the frustum at the start view — re-derive");
        assert!(d.drawn, "placement {p} undrawn at the start view — the frustum over-rejects");
    }
}
