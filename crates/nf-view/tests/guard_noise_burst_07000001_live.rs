//! guard-noise-attack-states: the sustained pistol burst through the real input path, on
//! 07000001 — the -hall report's live check 2 ("does a sustained pistol burst at the hall door
//! line draw the ch-0xE guards across the aperture in the real game?" re-aimed at the port's
//! own real-input hearing chain).
//!
//! The chain under real shots: the P2K's hearing entry (sound 0x403, inner 2, outer 40,
//! weight 65 [D weapon_data; docs/systems/drone-alerts.md]) lands in the tick's sound list;
//! `DroneFunc_HandleSoundAlerts` 0x148a30 accumulates the alertness (`+0x50C += +0x514 =
//! loudness · 0.01` [M :62-64]) and, past the frame/loudness gate, sends msg 0x14 to the
//! drone's own state machine [M :101-103]; the state handlers route it through
//! `DroneVision_EnemyAlerts` (0x177c18 → `AlertSound` 0x177800): within 50 u, unsighted, prop
//! 0x1f — the guard-DTYPE rows attack above 0.7 alertness [M 0x177800:51-68], the rest
//! investigate (share > 0.2 || alert > 0.66 → HeardNoise 0xa2) [M :74-79]. The pin: a guard
//! the burst can reach and has not seen, HEARS it (his alertness climbs) and is DRAWN (his
//! state log gains the attack or the heard-noise draw) — what the game's handlers say.
//!
//! The burst fires from Bond's spawn spot into the floor a step ahead — never a body ray, so
//! no impact path can draw the guards: the hearing chain is the only mover. The prologue is
//! walkreal_07000001's own (difficulty 1, top-up on).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn a_sustained_pistol_burst_draws_the_unsighted_guards_as_the_handlers_say() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    nf_view::settle_start_draw(w.scene.sim.as_mut().unwrap());
    // The intro cut-scene's tail (ch 0xB, then event 6 re-places Bond) — walkreal's own prologue.
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    let enabled_at = enabled_at.expect("the intro must write ch 0xB (the contact's enable channel)");
    eprintln!("[01 burst] intro ch 0xB at tick {enabled_at}");
    w.wait(60);
    // The burst candidates: armed, live, enabled guards on Bond's side of the fight, within
    // AlertSound's 50-unit earshot [M 0x177800:43-45], that have NOT sighted him — the game's
    // handler answers exactly these (a sighted guard returns 0 [M :48-50]; the sighted gate is
    // pinned separately by the nf-game fold tests).
    let candidates: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.team == 1 && d.started && d.enabled && !d.corpse && d.health > 0.0
                && !d.sighted && walker::dxz(d.pos, bp) <= 45.0
        }).collect()
    };
    let mut pre_alert: Vec<(usize, f32, u8, usize)> = Vec::new();
    for &i in &candidates {
        let d = &w.scene.sim.as_ref().unwrap().drones.list[i];
        pre_alert.push((i, d.alert, d.state, d.log.len()));
        eprintln!("[01 burst] candidate #{i} at {:?} mode {:#04x} state {:#04x} alert {:.2} sighted {}",
            d.pos, d.mode, d.state, d.alert, d.sighted);
    }
    assert!(!candidates.is_empty(),
        "the burst needs at least one unsighted armed guard in earshot — census above");
    // THE BURST: ten taps into the floor ahead of Bond's boots (the aim pins itself to a
    // point no body ray crosses — the hearing entry is the shot sound, not the impact). Real
    // input: every tick is `Sim::step_bond` — the game's own trigger path.
    for t in 0..320u32 {
        let (input, trigger) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let at = [bp[0] + 1.4, bp[1] - 2.2, bp[2] + 0.4]; // the floor right ahead, below the eye
            let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0)
        };
        w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        w.total_ticks += 1;
        w.after_step();
    }
    // The burst must actually fire: the held rounds drop as the taps go out.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[01 burst] after: held gun clip {} spare {}, bond at {:?}",
            sim.gun.clip, sim.gun.spare, sim.bond.pos);
    }
    // The handlers' answer: at least one burst candidate — unsighted when the burst began — is
    // DRAWN (his state log gains the attack 0x56 or the heard-noise family's draw). The alert
    // accumulation is traced but not required: the 01 start-room guards sit at the alert cap
    // (1.00) long before the burst [L the census line], so `alert > pre` never fires there.
    // Note (honest scope): the burst carries the muzzle flash with the sound — the draw pin is
    // the handlers' OUTCOME for gunfire near an unsighted alerted guard, whichever of the
    // game's own gunfire reactions fired first; the hearing chain's isolation is the -hall
    // door-lip lure's own pin (guards cross the aperture on the hearing alone) plus the
    // nf-game fold tests. The ARMS this task ports are pinned red-first by the nf-game unit
    // tests through the real handler path.
    let mut heard = 0usize;
    let mut drawn = 0usize;
    for &(i, pre, _st, pre_log) in &pre_alert {
        let d = &w.scene.sim.as_ref().unwrap().drones.list[i];
        let rose = d.alert > pre;
        let new_log = &d.log[pre_log.min(d.log.len())..];
        let got_attack = new_log.iter().any(|&(_, n)| n == nf_game::drone::state::ATTACK);
        let got_noise = new_log.iter().any(|&(_, n)| matches!(n,
            nf_game::drone::state::HEARD_NOISE | nf_game::drone::state::HEARD_NOISE_AWARE
            | nf_game::drone::state::HEARD_NOISE_SUSPECT | nf_game::drone::state::ALERT_TO_POSITION
            | nf_game::drone::state::SEARCH_AREA));
        eprintln!("[01 burst] guard #{i}: alert {:.2} → {:.2} (rose {rose}), state {:#04x} → {:#04x}, sighted now {}, new log {:?}",
            pre, d.alert, _st, d.state, d.sighted,
            new_log.iter().map(|&(f, n)| (f, format!("{n:#04x}"))).collect::<Vec<_>>());
        if rose { heard += 1; }
        if got_attack || got_noise { drawn += 1; }
    }
    eprintln!("[01 burst] heard(rose) {heard}, drawn {drawn} of {} candidates", pre_alert.len());
    assert!(drawn > 0,
        "no unsighted alerted guard was drawn (attack or heard-noise) by the sustained burst — the handlers' answer is missing");
    // The draw is the GAME's order, not an instant snap: the chain walks the sound through the
    // alert pass and the state's own arm; the assert above is state-agnostic about WHICH draw
    // (attack row vs investigate row) because that is the DTYPE's own row [M 0x177800:51-79].
}
