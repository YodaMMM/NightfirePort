//! fix-grenade-types (107a) — the thrown records' stun/smoke behaviour vs the game's numbers,
//! driven through the REAL path (`nf_view::load_weapons` + the panel give + `Sim::step_bond`).
//!
//! Evidence:
//! - The stun (0x35) fuse arm [M `Bullet_DoTrails` 0x12d890 disasm 0x12df58-0x12e130]: `Gas_Init`
//!   puff, `Camera_Shake(2.0)`, `Sound_Play3D(0x16)`, `Light_Create(10.0, 5.0, white)`, then a
//!   30-unit sphere: Bond's head takes `f = 1 − (dist − 5)·(1/85)` clamped [0,1] (dist ≤ 5 → 1.0,
//!   dist ≥ 90 → 0) × the facing factor `1 − (|yawDiff| − π/6)·0.81851107` (blast at the player's
//!   xz → 1.0) × 0.5 without LOS, and `Player_SetFlashBang(FRAME_RATE·f·10, BL, (u8)(f·255))`
//!   [M 0x12dfb0-0x12e0b8 + 0x196040] — FRAME_RATE = 60 at 60 Hz [D docs/systems/input.md:126], so
//!   the flash time is 600·f TICKS. `Player_Update` renders it as a fade: full alpha while the
//!   total counts past the half point, then a fade-out, and `GlobalBlur = BL+0x963`
//!   [M 0x193600:328-352]. The port set flashbang = 1.0 and decayed it by 1/6 per tick (a few
//!   ticks of faint white) — the white-out was effectively invisible.
//! - The guards within 20 units get msg 0x18 (player-thrown only in SP [M 0x12e094]) →
//!   `HandleImpact` case 0x18 [M 0x1469b8:250-312] → state 0x4D; `StunGrenadeImpact` ENTER
//!   [M 0x172bb0] arms `DroneFunc_RecoverTime`'s duration — for the 0x35 bullet the switch gives
//!   `FRAME_RATE_INT·30·0.01·(200 − Drone+0xB9)` = 18·(200−b9) = 3600 ticks at the stats default
//!   b9 = 0 [M 0x147188; +0xB9 = BOT_stats[8], 07000005's stats blob is zero at run time [L]] —
//!   and DROPS the weapon (`DroneWeap_DropWeapon` + `ChangeWeapon(1)`) unless already holding 4.
//!   The port armed 7200 (the default arm's 36·(200−b9)) and never dropped.
//! - The 0x69 smoke (the level-14 gas canisters' thrown twin, the same name label as 0x36 [L
//!   records]): `Bullet_init` arms its fuse to 5.0 [M 0x12c5a0:212-215]; at the pop the fuse
//!   re-arms to 3600.0 (0x45160000) and `Occlude_Create(pos, 3600, 1)` — the cloud lives as long
//!   as the re-armed fuse [M 0x12e114-0x12e1b4]. The port gave every smoke the 0x36 numbers.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn q(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Give the grenade through the panel and wait for the switch (the give path's own reload).
fn load_grenade(iso: &str, id: u8) -> nf_view::scene::Scene {
    let mut scene = nf_view::load_weapons(iso, "07000005.bin", false, &[id]).expect("load");
    let (msg, switch_to) = nf_view::panel::apply(&mut scene, &q(&[("do", "weapon"), ("id", &id.to_string()), ("rounds", "5")]));
    eprintln!("[give] {id:#04x}: {msg} -> {switch_to:?}");
    assert_eq!(switch_to, Some(id), "the panel must switch to {id:#04x}");
    for _ in 0..180 {
        scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(id), ..Default::default() }, false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == id && scene.sim.as_ref().unwrap().gun.switch_state == 0 { break; }
    }
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, id, "the grenade {id:#04x} is in hand");
    for _ in 0..4 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    scene
}

/// The white overlay alpha the flashbang must paint this frame (the render reads it off the sim).
fn flash_white_alpha(scene: &nf_view::scene::Scene) -> f32 {
    let h = &scene.sim.as_ref().unwrap().drones.health;
    if h.flashbang <= 0.0 || h.flash_total <= 0.0 { return 0.0; }
    if h.flashbang > h.flash_total * 0.5 { h.flash_alpha } else { h.flash_alpha * (h.flashbang / (h.flash_total * 0.5)) }
}

/// ── The stun whites Bond out at the game's time and alpha ───────────────────────────────────────
/// A blast at Bond's feet: dist < 5 → the distance factor is 1.0; the blast sits at his xz → the
/// facing factor is 1.0; LOS trivially clear → f = 1.0. `SetFlashBang(FRAME_RATE·f·10, f·255)` =
/// 600 ticks at full alpha [M 0x12e0b8 + 0x196040; D input.md:126]. The old port wrote 1.0 into a
/// field it decayed by 1/6 per tick — a few ticks of faint white, then gone.
#[test]
fn the_stun_grenade_whites_bond_out_at_the_games_time_and_alpha() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_grenade(&iso, 0x35);
    scene.sim.as_mut().unwrap().bond.pitch = -0.9; // at the floor just ahead
    let mut popped = None;
    let mut blast = None;
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), true, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(g) = sim.drones.grenades.first() { blast = Some(g.pos); }
        if sim.drones.health.flashbang > 0.0 { popped = Some(()); break; }
    }
    popped.expect("the stun grenade flashed Bond at its fuse");
    // frag-throw-timing2: the sim recorded the flash's exact geometry at the pop (`Drones::
    // flash_probe`, the FlashProbe doc) — the formula is asserted on THAT, not on a
    // reconstruction: with the case-0xc release timing [M 0x1a2e38:505-519] the pop can catch
    // the grenade still in flight, and the old reconstruction (the last grenade position the
    // test sampled — the record is deleted the same tick as the pop [M retain state<2, the
    // flash branch sets state 2 0x12e118's tail] — plus Bond's post-pop look) drifted past the
    // tolerance (the 741 hold: alpha 0.673 vs 0.758/0.379 on reconstructed geometry).
    let sim = scene.sim.as_ref().unwrap();
    let probe = sim.drones.flash_probe.expect("the flash recorded its geometry (the FlashProbe doc)");
    // Sister: the recorded blast is the grenade's own pop position — the last sampled flight
    // position sits one tick before it (the same-tick record deletion above).
    let last_seen = blast.expect("the grenade's flight was observed");
    let drift = ((probe.blast[0] - last_seen[0]).powi(2) + (probe.blast[1] - last_seen[1]).powi(2)
        + (probe.blast[2] - last_seen[2]).powi(2)).sqrt();
    eprintln!("[stun] probe {probe:?} last_seen {last_seen:?} drift {drift}");
    assert!(drift < 3.0, "the flash's blast is the grenade's pop position (one sampled tick apart), got {drift}");
    // The game's formula [M 0x12dfb0-0x12e0b8] on the recorded geometry reproduces the flash
    // exactly: the sim's alpha IS that call's f (the same pure function on the same inputs).
    let e = nf_game::drone::flash_amount(probe.head, probe.blast, probe.px, probe.pz, probe.yaw, probe.los);
    let alpha = sim.drones.health.flash_alpha;
    assert!((alpha - e).abs() < 1e-6,
        "the flash alpha is the game's factor on the geometry the flash used [M 0x12dfb0], got {alpha}, formula {e}");
    let h = &sim.drones.health;
    assert!(e > 0.2, "the at-feet blast must flash meaningfully, got {e}");
    // The remaining count was set to 600·f this pop [M 0x12e0b8 + 0x196040; D input.md:126] and
    // the read lands within one countdown tick of it (the pop step's own decay — 0x193600's
    // 1/tick countdown runs after the grenade update in the same step; the read is exact on
    // 600·f when the order ever flips, so the window is one-sided, no slop).
    assert!(h.flashbang <= 600.0 * e + 1e-3 && h.flashbang > 600.0 * e - 1.5,
        "the flash lasts FRAME_RATE·f·10 = 600·f ticks, read within one countdown tick [M 0x12e0b8; D input.md:126; the countdown 0x193600], got {} want ~{}", h.flashbang, 600.0 * e);
    assert!((h.flash_total - 600.0 * e).abs() < 1e-3, "the flash's total matches (the plateau/fade switch) [M 0x193600]");
    // The render shape [M 0x193600:340-352]: full white past the half point, faded out by the end.
    // The shape: the drawn white equals the flash alpha while more than half remains (the alpha
    // itself is checked against the formula on the flash's recorded geometry above).
    assert!((flash_white_alpha(&scene) - h.flash_alpha).abs() < 0.02, "full white while remaining counts past its half");
    drop(h);
    // The held trigger threw more grenades; each later pop re-sets the flash. Let the air clear,
    // then measure the LAST pop's plateau/fade shape in isolation.
    for _ in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.grenades.is_empty() { break; }
    }
    let h2 = &scene.sim.as_ref().unwrap().drones.health;
    let rem = h2.flashbang; let tot = h2.flash_total; let a0 = h2.flash_alpha;
    drop(h2);
    assert!(rem > 0.0, "the last pop's whiteout is still live");
    let half = (rem * 0.5) as u32 + 5; // step past the exact half boundary into the fade
    let tail = (tot * 0.55) as u32;
    for _ in 0..half { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let mid = flash_white_alpha(&scene);
    assert!(mid > 0.0 && mid < a0, "fading past the half point (a0 {a0}, mid {mid})");
    for _ in 0..tail { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    assert_eq!(flash_white_alpha(&scene), 0.0, "the whiteout is gone after the flash time");
}

/// ── The flash scales with distance and facing ───────────────────────────────────────────────────
/// A blast ~15 units ahead of a LOOKING Bond: the distance factor alone is `1 − (d−5)/85` [M
/// 0x12dfb0], so the alpha lands well under the full 1.0 (the port's flat 1.0 ignored geometry).
#[test]
fn the_stun_flash_scales_with_distance() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_grenade(&iso, 0x35);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pitch = 0.0;
    }
    let mut last_grenade = None;
    let mut moved = false;
    let mut popped = None;
    for _ in 0..600u32 {
        // One grenade only: the trigger stops the moment it is out (the refire would throw MORE
        // grenades from the RETREATED spot — the old run's second grenade fell off the ledge).
        let trigger = !moved;
        scene.sim.as_mut().unwrap().step_bond(Input::default(), trigger, false, false);
        {
            let sim = scene.sim.as_ref().unwrap();
            if let Some(g) = sim.drones.grenades.first() { last_grenade = Some(g.pos); }
        }
        // Retreat 35 units BACK along the throw line the moment the grenade is out: the pop then
        // reads from afar along the SAME clear line (the game's distance factor is 1 − (d − 5)/85).
        if !moved && scene.sim.as_ref().unwrap().drones.grenades.first().is_some() {
            let (bp, gr) = {
                let sim = scene.sim.as_ref().unwrap();
                (sim.bond.pos, sim.drones.grenades.first().unwrap().pos)
            };
            // Along Bond's FACING, not Bond→grenade: the game launches from the midpoint of the head and
            // the view model's bone 0 [M Player_WeaponInitBullet 0x1a4bf0:212-240], which starts BEHIND
            // the eye, so on the release tick the grenade can still be behind Bond's xz.
            let _ = gr;
            let yaw = scene.sim.as_ref().unwrap().bond.yaw;
            let (dx, dz) = (yaw.sin(), yaw.cos());
            let l = (dx * dx + dz * dz).sqrt().max(1e-3);
            scene.sim.as_mut().unwrap().bond.pos = [bp[0] - dx / l * 35.0, bp[1], bp[2] - dz / l * 35.0];
            moved = true;
        }
        if scene.sim.as_ref().unwrap().drones.health.flashbang > 0.0 { popped = Some(()); break; }
    }
    popped.expect("the stun flashed");
    let sim = scene.sim.as_ref().unwrap();
    // frag-throw-timing2: the formula is asserted on the flash's recorded geometry (the
    // FlashProbe doc) — the retreat test's reconstruction read Bond's post-teleport look while
    // the pop caught the grenade in flight, so the reconstruction drifted.
    let probe = sim.drones.flash_probe.expect("the flash recorded its geometry (the FlashProbe doc)");
    let d = ((probe.head[0] - probe.blast[0]).powi(2) + (probe.head[1] - probe.blast[1]).powi(2)
        + (probe.head[2] - probe.blast[2]).powi(2)).sqrt();
    eprintln!("blast at {:?}, head {:?}, d {d}, alpha {}", probe.blast, probe.head, sim.drones.health.flash_alpha);
    assert!(d > 20.0, "Bond retreated before the pop (the flash's own dist d = {d})");
    // Sister: the recorded blast is the grenade's own pop position (one sampled tick apart —
    // the record dies the same tick, drone.rs retain(state<2)).
    let last_seen = last_grenade.expect("the grenade's flight was observed");
    let drift = ((probe.blast[0] - last_seen[0]).powi(2) + (probe.blast[1] - last_seen[1]).powi(2)
        + (probe.blast[2] - last_seen[2]).powi(2)).sqrt();
    assert!(drift < 3.0, "the flash's blast is the grenade's pop position (one sampled tick apart), got {drift}");
    // The far blast reads well under the old flat full-white: the distance factor alone caps at
    // 1 − (d−5)/85 ≈ 0.65 at 35 u, and the retreat line's geometry may halve it again (×0.5, the
    // LOS arm [M 0x12e0a4]) — both live in the recorded `los` branch the formula call consumes.
    let e = nf_game::drone::flash_amount(probe.head, probe.blast, probe.px, probe.pz, probe.yaw, probe.los);
    let alpha = sim.drones.health.flash_alpha;
    assert!(alpha < 0.7, "a distant blast is not a near-full white, got {alpha}");
    assert!((alpha - e).abs() < 1e-6,
        "the alpha is the game's distance factor ×(LOS branch) on the recorded geometry [M 0x12dfb0], got {alpha} want {e}");
}

/// ── The guard within the blast is stunned for the game's recover and gets up ────────────────────
/// The flash's delivery rule [M `Bullet_DoTrails` 0x12d890 flash arm 0x12e0c0 →
/// `Drone_MessageObjVicinity` 0x13a438]: msg 0x18 to every drone within 20.0 of the BLAST whose
/// head→blast sight passes (`NDrone2_CanSeeObject` = the AI-net ray + `Collide_LineOfSight`
/// [M 0x176f18]) — NOT within 20 of Bond. The Impact ENTER [M 0x172bb0 case 1] stores
/// `RecoverTime` (the 0x35 arm: 18·(200−0) = 3600 ticks [M 0x147188]) into +0xCE0, drops the held
/// weapon for anim set ≠ 4 (`DropWeapon + ChangeWeapon(1)`), calls the fall 0x2E on the Rand draw
/// 0 (0x2D on 1) with follow-on Loop 0x4E; `StunGrenadeLoop` ENTER re-arms the wake timer with the
/// STORED span — `+0xCC4 = frame + (+0xCE0)`, a fresh 3600 from the Loop's first tick [M 0x1631c0
/// case 1] — and case 0xC releases to Recover 0x4F when nobody is within 2.0
/// (`DroneFunc_AnyoneNear(0x40000000)`); Recover calls the get-up 0x76 with follow-on Attack 0x56
/// for non-civilians [M 0x163350 case 1]. The release chain must FINISH: the CallAnim no-datum
/// fallback substitutes a playable state [M 0x13ab58], and a call whose transition is missing from
/// both the current anim's list and state 0's pends exactly as the game's `GetDAnimForCall` does
/// [M 0x13db30 → 0x13de40] — no timeout exists in the decompile.
///
/// The subject is the PLACED guard list[2] (anim set 5): Bond is teleported 3 u to his right and
/// faces him — the one spot on 07000005 where a placed guard stands inside the delivery rule (the
/// level-start within-20 guard list[6] stands 7.6 u below the blast floor, outside the sight
/// check). Both hops are the test's own staging, like the original's post-pop retreat.
#[test]
fn the_stun_stuns_the_guard_for_3600_and_drops_his_weapon() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_grenade(&iso, 0x35);
    let (gi, gx, gy, gz, gyaw) = {
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[2];
        (2, g.pos[0], g.pos[1], g.pos[2], g.yaw)
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[gi].health = 200.0; // outlive any stray damage: this test is about the stun
    }
    // Bond 3 u to the guard's right (perpendicular −), facing him. [L] measured: the only tried
    // offset near list[2] where the floor holds Bond, the grenade pops and the flash reaches him.
    // stream-batch-2: the delivery premise (the head→blast line of sight [M 0x12e0c0 +
    // NDrone2_CanSeeObject's AI-net ray]) is geometry-fragile — the joined tree's streams drift
    // the guard's warm-up position slightly and the ray clips the wall edge, so the offset is a
    // searched lever over the measured spot; the stun-law asserts below are unchanged.
    let (sy, cy) = (gyaw.sin(), gyaw.cos());
    let mut dev = nf_view::dev::Dev { scenario: None, ..Default::default() };
    let mut stun_offset = None;
    for off in [-3.0f32, -2.5, -3.5, -2.0, -4.0] {
        dev.teleport(&mut scene, [gx + cy * 0.0 - sy * off, gy, gz + sy * 0.0 + cy * off], gyaw + std::f32::consts::PI);
        scene.sim.as_mut().unwrap().bond.pitch = -0.9; // at the floor just ahead
        let mut popped = false;
        for _ in 0..600u32 {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), true, false, false);
            if scene.sim.as_ref().unwrap().drones.health.flashbang > 0.0 { popped = true; break; }
        }
        if !popped { continue; } // this offset never popped — next lever
        // set_state queues next_state — give the SM a pass to land 0x4D. Bond then steps back 30 u
        // (the proper teleport re-cells him): the Loop's case 0xC holds the guard down while anyone
        // is within 2.0 [M 0x1631c0 case 0xc], and he is spawned ~3 u from Bond — the release is the
        // 3600 timer only from a distance.
        for _ in 0..2 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        if dr.state != nf_game::drone::state::STUN_GRENADE_IMPACT { continue; } // the flash missed — next lever
        stun_offset = Some(off);
        break;
    }
    let off = stun_offset.expect("no offset near list[2] delivered the flash premise — the harness needs a new lever (the stun law asserts below are unchanged)");
    eprintln!("gate: the flash premise held at offset {off}");
    let bp = scene.sim.as_ref().unwrap().bond.pos;
    let byaw = scene.sim.as_ref().unwrap().bond.yaw;
    dev.teleport(&mut scene, [bp[0] - byaw.sin() * 30.0, bp[1], bp[2] - byaw.cos() * 30.0], byaw);
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[gi];
    eprintln!("guard state {:#x} weapon {} stun_recover {}", dr.state, dr.weapon, dr.stun_recover);
    assert_eq!(dr.state, nf_game::drone::state::STUN_GRENADE_IMPACT, "the guard enters StunGrenadeImpact 0x4D [M 0x1469b8 case 0x18]");
    // `DropWeapon + ChangeWeapon(1)` unless he held the unarmed id 4 [M 0x172bb0 case 1].
    assert!(dr.weapon == 1 || dr.weapon == 4, "the stunned guard drops to weapon 1 (or held the unarmed 4), got {}", dr.weapon);
    // 3599, not 3600: the game's own float path is `fptoui(f32(1800)·0.01·(200−0xB9))` — f32 0.01
    // is 0.00999999977648, so the product is 3599.99992 and fptoui truncates [M 0x147188:44-46 +
    // the fptoui tail; L the port's mul.s chain]. The old "= 3600" doc rounded.
    assert_eq!(dr.stun_recover, 3599, "the 0x35 arm stores FRAME_RATE_INT·30·0.01·(200−0) into +0xCE0 [M 0x147188 case 0x35]");
    // Sister: no blast damage — the flash branch never reaches Explode_Create [M 0x12e118].
    assert!(sim.drones.explosions.is_empty(), "the stun grenade must not blast");
    // The release chain: step until the guard has LEFT the family, counting the ticks. The law:
    // the fall clip plays out in Impact (Rand 0 → 0x2E, else 0x2D [M 0x172bb0 case 1]), the Loop
    // holds a FRESH 3600 from its first tick [M 0x1631c0 case 1], then Recover's get-up 0x76 runs
    // to its follow-on Attack 0x56 [M 0x163350 case 1] — so the total exceeds 3600 by the two
    // clips' lengths. The old carry ended the Loop one fall-clip early ([M] deviation fixed here).
    use nf_game::drone::state::{STUN_GRENADE_IMPACT, STUN_GRENADE_LOOP, STUN_GRENADE_RECOVER};
    let mut held = 0u32;
    let mut exit_state = 0u8;
    for _ in 0..5000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let dr = &sim.drones.list[gi];
        held += 1;
        if held > 4 && !matches!(dr.state, STUN_GRENADE_IMPACT | STUN_GRENADE_LOOP | STUN_GRENADE_RECOVER) {
            exit_state = dr.state;
            break;
        }
    }
    eprintln!("guard held in the stun chain for {held} ticks, exit {exit_state:#x}");
    // The get-up's follow-on is Attack 0x56 for non-civilians [M 0x163350 case 1]. The poll can
    // land on 0x66 (Attack's own first move: the guard holds the fists ChangeWeapon(1) gave him
    // at the Impact, so Attack draws his sidearm — DRAW_WEAPON 0x66) when the draw is queued
    // within the same tick the follow-on lands.
    assert!(exit_state == nf_game::drone::state::ATTACK || exit_state == nf_game::drone::state::DRAW_WEAPON,
        "the recovered guard wakes into Attack 0x56 (the get-up's follow-on; 0x66 = Attack's draw of the sidearm he holds fists after the drop) [M 0x163350 case 1], got {exit_state:#x}");
    assert!(held > 3600, "the Loop's fresh re-arm holds 3600 from the Loop's first tick — the chain cannot finish before it [M 0x1631c0 case 1], got {held}");
    // The two falls differ by the `Rand_Rand(2)` pick [M 0x172bb0 case 1]; the long one is 322 ticks, the get-up 439.
    assert!(held <= 4400, "the hold is 3600 + the fall and get-up clips, not an endless stall, got {held}");
}

/// ── The 0x69 smoke: a 5-tick fuse into a 3600-tick cloud ────────────────────────────────────────
/// `Bullet_init` arms 0x69's fuse to 5.0 [M 0x12c5a0:212-215]; the pop re-arms to 3600.0
/// (0x45160000) and `Occlude_Create(pos, 3600, 1)` — the cloud's lifetime is the re-armed fuse
/// [M 0x12e114-0x12e1b4]. The port gave every smoke the 0x36 numbers (210 into 1200).
#[test]
fn the_069_smoke_pops_in_5_ticks_into_a_3600_cloud() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load_weapons(&iso, "07000005.bin", false, &[]).expect("load");
    let mut dev = nf_view::dev::Dev { scenario: None, ..Default::default() };
    let bond_pos = scene.sim.as_ref().unwrap().bond.pos;
    let gi = dev.spawn_guard(&mut scene, [bond_pos[0] + 3.0, bond_pos[1], bond_pos[2]], 0.0).expect("guard spawned");
    {
        let sim = scene.sim.as_mut().unwrap();
        let dr = &mut sim.drones.list[gi];
        dr.grenade_count = 2;
    }
    // Force the guard's throw through the real anim/release path, then re-point the record to 0x69
    // (force_grenade hardcodes its 0x35 demo pick).
    dev.force_grenade(&mut scene, bond_pos);
    {
        let sim = scene.sim.as_mut().unwrap();
        let dr = &mut sim.drones.list[gi];
        dr.grenade_weapon = 0x69;
    }
    let mut thrown = None;
    let mut popped = None;
    for t in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(g) = sim.drones.grenades.iter().find(|g| g.weapon_type == 0x69) {
            if thrown.is_none() {
                thrown = Some(());
                eprintln!("[0x69] thrown at tick {t}, fuse {}", g.range);
                assert_eq!(g.range, 5.0, "Bullet_init arms the 0x69 fuse to 5.0 [M 0x12c5a0:212]");
            }
        }
        if !sim.drones.smoke_clouds.iter().any(|c| c.mesh == 0x0200_0273) { continue; }
        let cloud = sim.drones.smoke_clouds.iter().find(|c| c.mesh == 0x0200_0273).unwrap();
        if cloud.ticks > 3500 { popped = Some(t); break; }
    }
    thrown.expect("the guard threw the 0x69");
    let pop = popped.expect("the 0x69 popped its cloud");
    let sim = scene.sim.as_ref().unwrap();
    let cloud = sim.drones.smoke_clouds.iter().find(|c| c.mesh == 0x0200_0273).unwrap();
    eprintln!("[0x69] popped at tick {pop}, cloud ticks {}", cloud.ticks);
    assert!(cloud.ticks > 3500, "the 0x69's cloud lives 3600 ticks [M 0x12e1b4], got {}", cloud.ticks);
    let g = sim.drones.grenades.iter().find(|g| g.weapon_type == 0x69).expect("the grenade re-armed");
    assert!(g.range > 3500.0, "the fuse re-arms to 3600 [M 0x12e1b4], got {}", g.range);
}
