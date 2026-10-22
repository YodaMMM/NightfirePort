//! mp-death-effects: the local player's death presentation in MP — the game runs the SP death
//! sequence's sim core in MP too and the view reads it through `Sim::local_death_fx` /
//! `Death::phase` / `Death::pane_px` (docs/systems/mp-death-effects.md).
//!
//! Evidence base:
//! - The death tick [M `Player_CheckForDeath` 0x190810]: the stamp `obj+0xec = GameState+0x52`
//!   (:59-60), cam mode 0 (:64), substate 0xd/0xe (:89-94) and the death sound
//!   `Sound_Play3D(100.0, 1.0, 1.0, 0x89, obj+0x30, …)` (:88) are COMMON — 0x89 plays in MP.
//!   SP-only [M :39-42, the `_384_4_ == 0` arm]: ch 0x5e + `Music_Event(6,1)`. MP-only
//!   [M :67-87]: the kill message.
//! - The dead ticks [M `Player_HandleDeath` 0x190bb0]: the SP arm hides the whole HUD for the
//!   blood pane (`HUD_DisableAll` + `HUD_Enable(pane 12, 1, 0)` :23-25) and waits for the cover
//!   or the ✕ before ch 0x62 (:29-33); **the MP arm enables pane 12 WITHOUT the DisableAll —
//!   the HUD stays up (:38-40) — and respawns after `FRAME_RATE_INT * 5` = 300 ticks from the
//!   stamp (:41-54)** via `MP_ReSpawn`. No countdown or prompt is drawn in MP.
//! - The pane's growth [M `HUD_UpdateBloodPane` 0x19a6a8:23-34]: 1 px per tick
//!   (`FRAME_RATE_MUL`) to the 448 cover [M 0x19f4e8:33; 512-wide templates [L BloodSprInfo
//!   0x2bcfd8]] — **the MP respawn at 300 < 448 ends the sequence before the cover**.
//! - The respawn [M `MP_ReSpawn` 0x185368:65-91]: pane 12 OFF (:67), cam mode 0 (:68), health
//!   `MPSettings[slot*0x30+0x2c] + 100.0` (:71-73), `MP_EquipPlayer` (:78), the viewer snapped
//!   to the spawn (:79-80), `obj+0xff = 3` (:93).
//!
//! Opt-in via NF_ISO.

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const ARENA: &str = "07000023.bin";

/// The offline Quick-Game shape (mp-host-match-logic's harness): 1 bot, no frag/time end.
fn offline_settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.frag_limit = 100;
    s.bot_count = 1;
    s.time_limit = -1;
    s
}

fn two_human_settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 0;
    s.total_players = 2;
    s.seed = 0x0bed_0bad;
    s.time_limit = 10 * 60;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let mut scene = nf_view::load_mp_carry(path, ARENA, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

fn neutral_row() -> [nf_net::Input; nf_net::MAX_AGENTS] {
    std::array::from_fn(|_| NEUTRAL)
}

fn audio_starts(sim: &mut nf_view::scene::Sim) -> Vec<nf_game::sound::Cmd> {
    let l = sim.bond.head();
    sim.drones.audio.update(sim.sfx.as_ref(), (l, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), &|_| false);
    sim.drones.audio.take_commands()
}

/// Offline MP: the local player's death runs the phases with the game's numbers — the pane
/// grows 1 px/tick from 0 [M 0x19a6a8:27-30], the HUD stays up [M 0x190bb0:39], the 5 s gate
/// counts down [M 0x190bb0:41-54], the death sound 0x89 queues [M 0x190810:88], and the
/// respawn wipes the sequence (pane off [M 0x185368:67]) with the MP_ReSpawn health.
#[test]
fn mp_local_death_runs_the_pane_growth_and_the_5s_respawn() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load_mp_carry(&iso, ARENA, false, None, offline_settings())
        .expect("the arena loads through the app's MP carry path");
    let sim = scene.sim.as_mut().unwrap();
    assert!(sim.local_death_fx().is_none(), "no death view while alive");

    // The kill (mp-host-match-logic's seam: the hit registers, the raw damage lands).
    {
        let m = sim.mp.as_mut().unwrap();
        assert!(m.register_bullet_hit(0, 1), "the FFA hit is allowed [M 0x1857c0]");
    }
    sim.drones.health.health = 0.0;
    sim.step_bond(Default::default(), false, false, false);

    // The death tick: the sequence starts at elapsed 0 and the death sound queues.
    let fx = sim.local_death_fx().expect("the death view is up on the death tick");
    assert_eq!(fx.phase, 1, "the sequence is running [M 0x190810:66 obj+0xf4 = 2]");
    assert_eq!(fx.pane_px, 0, "elapsed 0 on the stamp tick [M 0x190810:59-60]");
    assert!(!fx.hud_hidden, "MP enables pane 12 WITHOUT the HUD DisableAll [M 0x190bb0:39]");
    assert_eq!(fx.respawn_ticks_left, Some(Sim_MP_RESPAWN), "the 5 s gate [M 0x190bb0:41-54]");
    assert!(fx.camera_frozen, "cam mode 0 [M 0x190810:64]");
    assert!(audio_starts(sim).iter().any(|c| matches!(c, nf_game::sound::Cmd::Start { id: 0x89, .. })),
        "the death sound 0x89 queues in MP too [M 0x190810:88 — it sits after the SP/MP arms]");

    // 150 dead ticks: the pane is 150 px in (1 px/tick [M 0x19a6a8:27-30]), the gate counts down.
    for _ in 0..150 { sim.step_bond(Default::default(), false, false, false); }
    let fx = sim.local_death_fx().unwrap();
    assert_eq!(fx.pane_px, 150, "1 px per dead tick [M 0x19a6a8:27-30]");
    assert!((fx.fade_alpha - 150.0 / 448.0).abs() < 1e-6, "the alpha rides the px clock");
    assert_eq!(fx.respawn_ticks_left, Some(Sim_MP_RESPAWN - 150));
    assert_eq!(fx.phase, 1);

    // 299 ticks in: still dead — and the MP pane never reaches the 448 cover.
    for _ in 0..149 { sim.step_bond(Default::default(), false, false, false); }
    let fx = sim.local_death_fx().unwrap();
    assert_eq!(fx.pane_px, 299);
    assert_eq!(fx.phase, 1, "300 < 448: the respawn ends the sequence before the cover [M 0x190bb0:41 vs RED_FADE_TICKS]");
    assert!(sim.drones.death.active, "still inside the gate (the 299th dead tick)");

    // The 300th dead tick respawns: the sequence is gone (pane off [M 0x185368:67]), the health
    // is armor_bonus + 100 [M 0x185368:71-73].
    sim.step_bond(Default::default(), false, false, false);
    assert!(sim.local_death_fx().is_none(), "the respawn wiped the sequence [M 0x185368:93 obj+0xff = 3]");
    assert!(!sim.drones.death.active);
    assert!((sim.drones.health.health - 100.0).abs() < 1e-5, "armor_bonus 0 + 100.0 [M 0x185368:71-73]");
}

/// Online MP, two machines: the kill enters through the shared hurt path on both machines;
/// every tick's net hash agrees across the death and the respawn (the other machine's view of
/// that player is unchanged — the hash sister), and the dying machine's getters run the same
/// phases on the same clock.
#[test]
fn mp_online_death_view_agrees_across_machines() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = two_human_settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    const K: u32 = 10; // the kill tick
    let row = neutral_row();

    // The shared blast that kills slot 0 — applied identically on both machines (the shared
    // hurt path's shape: `mp_hurt` is what the combat step feeds).
    let blast = nf_game::mp::combat::Hurt::Blast { victim: 0, attacker: 1, amount: 500.0, weapon: 0, at: [0.0; 3] };
    a.sim.as_mut().unwrap().mp_hurt(blast, 0);
    b.sim.as_mut().unwrap().mp_hurt(blast, 1);

    // The kill tick, before the net step: the slot is marked dead, the sequence has not started
    // (the port's online order: the hurt pass, then the health mirror + Drones::tick).
    {
        let sa = a.sim.as_mut().unwrap();
        let fx = sa.local_death_fx().expect("the dead slot surfaces on the kill tick");
        assert_eq!(fx.phase, 0, "drones.death starts the tick after the kill (the port's order)");
        assert_eq!(fx.pane_px, 0, "elapsed 0");
        assert!(fx.camera_frozen, "net_dead freezes the camera on the kill tick");
        assert_eq!(fx.respawn_ticks_left, Some(Sim_MP_RESPAWN - 1));
        assert!(b.sim.as_mut().unwrap().local_death_fx().is_none(), "the OTHER machine's local player is alive — no view");
    }

    let mut pane_at_50 = 0u32;
    for t in K..=315u32 {
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the other machine's view of the death is unchanged (the hash sister)");
        let sa = a.sim.as_mut().unwrap();
        let fx = sa.local_death_fx();
        match t {
            10 => {
                let fx = fx.unwrap();
                assert_eq!(fx.phase, 0, "the sequence starts next tick");
                assert_eq!(fx.pane_px, 1, "the kill tick's dead-wait increment counts as elapsed 1");
            }
            11 => {
                let fx = fx.unwrap();
                assert_eq!(fx.phase, 1, "drones.death is up [M 0x190810]");
                assert!(!fx.hud_hidden, "the HUD stays up in MP [M 0x190bb0:39]");
                assert!(audio_starts(sa).iter().any(|c| matches!(c, nf_game::sound::Cmd::Start { id: 0x89, .. })),
                    "the death sound 0x89 [M 0x190810:88]");
            }
            60 => {
                let fx = fx.unwrap();
                pane_at_50 = fx.pane_px; // mp_dead_ticks = 2 + (60 - 10) = 52 → elapsed 51
                assert_eq!(fx.pane_px, 51);
                assert_eq!(fx.respawn_ticks_left, Some(Sim_MP_RESPAWN - 52));
            }
            307 => {
                let fx = fx.unwrap();
                assert_eq!(fx.pane_px, 298);
                assert_eq!(fx.phase, 1, "the online pane never covers either [M 0x190bb0:41]");
            }
            308 => {
                // mp_dead_ticks hit 300 in this tick's dead wait: the respawn ran on BOTH
                // machines (the shared state).
                assert!(fx.is_none(), "the respawn wiped the sequence [M 0x185368:67,93]");
                assert!((sa.drones.health.health - 100.0).abs() < 1e-5, "armor_bonus + 100 [M 0x185368:71-73]");
            }
            _ => { let _ = fx; }
        }
    }
    assert_eq!(pane_at_50, 51, "the pane clock rides the dead-tick counter");
    // The respawn put slot 0 at the same point on both machines (the policy's pick is shared).
    let pa = a.sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[0].pos;
    let pb = b.sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[0].pos;
    assert_eq!(pa, pb, "the other machine sees the respawned player at the same spot [M 0x185368:23-54]");
    assert!(pa.is_some(), "slot 0 is placed again");
}

/// SP sister: the same getters on an SP death — the whole HUD hides for the pane
/// [M 0x190bb0:24], there is no respawn gate (None), and the growth reaches the 448 cover and
/// phase 2 — what the MP respawn never lets happen [M 0x190bb0:41 vs RED_FADE_TICKS].
#[test]
fn sp_death_sister_hud_hidden_no_gate_cover_reached() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 05");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.local_death_fx().is_none());

    // The kill through the real pain path (death-animation-sp's harness).
    {
        let (h, r) = (&mut sim.drones.health, &mut sim.drones.rng);
        h.pain_generic(r, 500.0, 1, 0x0700_0005, 0);
    }
    sim.step_bond(Default::default(), false, false, false);
    let fx = sim.local_death_fx().expect("the death view is up");
    assert_eq!(fx.phase, 1);
    assert_eq!(fx.pane_px, 0);
    assert!(fx.hud_hidden, "SP: HUD_DisableAll for the pane [M 0x190bb0:24]");
    assert_eq!(fx.respawn_ticks_left, None, "SP has no respawn gate [M 0x190bb0:20-35]");
    assert!(fx.camera_frozen, "cam mode 0 [M 0x190810:64]");
    assert!(audio_starts(sim).iter().any(|c| matches!(c, nf_game::sound::Cmd::Start { id: 0x89, .. })),
        "the death sound 0x89 [M 0x190810:88]");

    // 150 dead ticks: 150 px — the same growth rule as MP [M 0x19a6a8:27-30].
    for _ in 0..150 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.local_death_fx().unwrap().pane_px, 150);

    // To the cover: 448 px, phase 2 — SP-only (the MP gate ends the sequence at 300).
    for _ in 0..(448 - 150) { sim.step_bond(Default::default(), false, false, false); }
    let fx = sim.local_death_fx().unwrap();
    assert_eq!(fx.pane_px, 448, "the cover [M 0x19a6a8:23-25, viewer.h 448 0x19f4e8:33]");
    assert_eq!(fx.phase, 2, "the covered phase — MP never reaches it [M 0x190bb0:41]");
}

use nf_view::scene::Sim;
const Sim_MP_RESPAWN: u32 = nf_view::scene::Sim::MP_RESPAWN_DELAY_TICKS;
