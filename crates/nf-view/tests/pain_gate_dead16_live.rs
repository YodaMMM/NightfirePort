//! pain-generic-gate: the real-input death walk — 07000016's launch-centre approach with the
//! walker's allowances OFF (no top-up, no return fire): the AR guards' unanswered fire kills
//! Bond on the route (the measured 2362-tick death, 47 hits for 100.0 [M
//! walkreal_07000016_live's consider-explosive-gate-3 note]), and the walk then KEEPS STEPPING
//! through the death sequence. Every further guard round that reaches the body runs
//! `Player_DealWithObjHit` → `Player_HandlePain`, which the game gates on health <= 0
//! [M 0x1902c8:18-20 — the return sits before the body's first store, 0x190378].
//!
//! The pin: from the death tick on, the port's damage ledger (`Drones::player_hits`, the
//! (frame, owner, health-lost) record every Bond-damage site pushes) gains NO entry and the
//! pane fade (`pain_alpha`, `BL+0x8bc` [M 0x1902c8:27 — set INSIDE the gated body]) never
//! rises. On the pre-fix code a post-death round pushed a (frame, owner, 0.0) ledger entry
//! (the bullet arm's push sat outside the gate) and a post-death blast popped the pane —
//! changed state on a dead player the game never writes. The bullet arm's own post-HandlePain
//! writes (the +0x967 direction bits [M 0x194458:203-226], the 0x412/0x171 sounds) stay
//! unconditional here, like the game's — `dir_bits` may move on the corpse.
//!
//! Real input only: no `bond.pos` writes after the load; the walker steers with button input,
//! the death arrives under the game's own fire, and the observation loop holds `Input::default()`.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use nf_game::player::RamSave;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The save the game hands 16 at the end of the chain (the walkreal_07000016_live helper): 14's
/// warm exit through 15's. The corridor death the sisters measured (2362 ticks, 47 hits) was on
/// THIS carry — a cold 16 load's AR pair never opens up the same way.
fn chain_save(iso: &str) -> RamSave {
    static SAVE: std::sync::OnceLock<RamSave> = std::sync::OnceLock::new();
    SAVE.get_or_init(|| {
        let s14 = nf_view::warm_exit_save(nf_view::load(iso, "07000014.bin", false).unwrap().sim.as_ref().unwrap());
        let s15 = nf_view::warm_exit_save(nf_view::load_with_carry(iso, "07000015.bin", false, Some(&s14)).unwrap().sim.as_ref().unwrap());
        s15
    }).clone()
}

/// The approach's early legs — the corridor and the ring, where the AR gallery/ring pair
/// engages (the walkreal_07000016_live APPROACH's head; the slab tail is beyond the fight).
const LEGS: &[(&str, walker::Wp)] = &[
    ("the corridor south", walker::Wp { at: [4.3, 0.6, -72.5], act: walker::Act::None, arrive: 1.2 }),
    ("door[7] through", walker::Wp { at: [4.9, -0.1, -78.0], act: walker::Act::None, arrive: 1.2 }),
    ("the lane east", walker::Wp { at: [10.0, 0.0, -82.0], act: walker::Act::None, arrive: 1.3 }),
    ("the lane south", walker::Wp { at: [12.0, 0.0, -88.0], act: walker::Act::None, arrive: 1.3 }),
    ("the ramp foot", walker::Wp { at: [13.8, 0.0, -90.8], act: walker::Act::None, arrive: 1.1 }),
    ("the ramp 1", walker::Wp { at: [14.5, 0.6, -87.5], act: walker::Act::None, arrive: 1.2 }),
    ("the ramp 2", walker::Wp { at: [14.5, 2.4, -84.5], act: walker::Act::None, arrive: 1.2 }),
    ("the ramp 3", walker::Wp { at: [14.5, 4.6, -81.5], act: walker::Act::None, arrive: 1.2 }),
    ("the ring east", walker::Wp { at: [14.5, 5.9, -79.0], act: walker::Act::None, arrive: 1.2 }),
    ("the ring 1", walker::Wp { at: [6.0, 5.9, -78.5], act: walker::Act::None, arrive: 1.4 }),
    ("the ring 2", walker::Wp { at: [-4.0, 5.9, -79.0], act: walker::Act::None, arrive: 1.4 }),
    ("the ring 3", walker::Wp { at: [-11.0, 5.9, -81.0], act: walker::Act::None, arrive: 1.4 }),
    ("the ring 4", walker::Wp { at: [-13.0, 5.9, -86.0], act: walker::Act::None, arrive: 1.4 }),
    ("the station west", walker::Wp { at: [-16.5, 5.9, -88.0], act: walker::Act::None, arrive: 1.3 }),
    ("the station south", walker::Wp { at: [-16.3, 5.9, -93.5], act: walker::Act::None, arrive: 1.2 }),
    ("the station E", walker::Wp { at: [-15.2, 5.9, -94.6], act: walker::Act::Use { use_ticks: 120 }, arrive: 1.2 }),
    ("the return 1", walker::Wp { at: [-16.3, 5.9, -89.0], act: walker::Act::None, arrive: 1.3 }),
    ("the return 2", walker::Wp { at: [-13.0, 5.9, -85.0], act: walker::Act::None, arrive: 1.4 }),
    ("the return 3", walker::Wp { at: [-10.0, 5.9, -81.0], act: walker::Act::None, arrive: 1.4 }),
    ("the return 4", walker::Wp { at: [-4.0, 5.9, -79.0], act: walker::Act::None, arrive: 1.4 }),
    ("the return 5", walker::Wp { at: [6.0, 5.9, -78.5], act: walker::Act::None, arrive: 1.4 }),
    ("the return 6", walker::Wp { at: [14.5, 5.9, -79.0], act: walker::Act::None, arrive: 1.2 }),
    ("the descent 1", walker::Wp { at: [14.5, 4.6, -82.0], act: walker::Act::None, arrive: 1.2 }),
    ("the descent 2", walker::Wp { at: [14.5, 2.4, -85.0], act: walker::Act::None, arrive: 1.2 }),
    ("the descent 3", walker::Wp { at: [14.5, 0.6, -88.0], act: walker::Act::None, arrive: 1.2 }),
    ("the east lane south", walker::Wp { at: [9.5, 0.0, -100.0], act: walker::Act::None, arrive: 1.6 }),
    ("the trench's west end", walker::Wp { at: [4.0, 0.0, -108.0], act: walker::Act::None, arrive: 1.6 }),
    ("door[0]'s west approach", walker::Wp { at: [2.3, 0.0, -110.0], act: walker::Act::None, arrive: 0.7 }),
    ("door[0]'s threshold", walker::Wp { at: [2.3, 0.0, -113.2], act: walker::Act::None, arrive: 0.7 }),
    ("door[0] through", walker::Wp { at: [2.0, -0.1, -119.0], act: walker::Act::None, arrive: 1.2 }),
    ("door[1] through", walker::Wp { at: [2.0, -0.1, -132.0], act: walker::Act::None, arrive: 1.2 }),
    ("door[3]'s panel", walker::Wp { at: [3.4, 0.8, -141.5], act: walker::Act::Use { use_ticks: 90 }, arrive: 0.8 }),
    ("door[3] off panel", walker::Wp { at: [2.0, 0.0, -140.0], act: walker::Act::None, arrive: 1.0 }),
    ("door[3] through", walker::Wp { at: [2.0, -0.1, -147.0], act: walker::Act::None, arrive: 1.2 }),
    ("door[2]'s panel", walker::Wp { at: [3.4, 0.8, -151.9], act: walker::Act::Use { use_ticks: 90 }, arrive: 0.8 }),
    ("door[2] off panel", walker::Wp { at: [2.2, 0.0, -150.4], act: walker::Act::None, arrive: 1.0 }),
    ("door[2] through", walker::Wp { at: [2.2, -0.1, -151.5], act: walker::Act::None, arrive: 1.2 }),
    ("the silo plate", walker::Wp { at: [2.5, 0.3, -153.9], act: walker::Act::None, arrive: 1.0 }),
    ("door[4]'s panel", walker::Wp { at: [3.5, 1.0, -165.7], act: walker::Act::Use { use_ticks: 90 }, arrive: 0.8 }),
    ("door[4] through", walker::Wp { at: [2.5, 1.0, -169.0], act: walker::Act::None, arrive: 1.3 }),
];

fn health(sim: &nf_view::scene::Sim) -> f32 { sim.drones.health.health }

#[test]
fn the_corridor_death_then_the_pain_gate_holds_on_the_body() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000016.bin", false, Some(&chain_save(&iso)), 1).unwrap());
    // The unanswered walk: no top-up (real damage), no return fire (the guards' fire is
    // unanswered — the measured 2362-tick death), and the death does NOT panic the walk:
    // the observation window needs the sim alive past the death tick. All three are this
    // walker instance's own flags — no shared-helper change.
    w.top_up_health = false;
    w.dmg.return_fire = false;
    w.dmg.die_panics = false;
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);

    // The route legs, one at a time, health-gated: a leg that wedges because Bond died mid-leg
    // reports false instead of panicking (try_walk_to's catch), and the loop breaks to the
    // observation window.
    let mut t0 = None;
    for (label, wp) in LEGS {
        let ok = w.try_walk_to(*wp, label);
        if health(w.scene.sim.as_ref().unwrap()) <= 0.0 { t0 = Some(w.total_ticks); break; }
        assert!(ok, "the leg {label} wedged with Bond alive at {}", health(w.scene.sim.as_ref().unwrap()));
    }
    // If the corridor fire never killed him, the slab tail does: the ch-0x81 touch runs the
    // launch chain, the pit floor re-enables Bond, ch 243 raises the 1000-damage shuttle
    // volume, and its contact kills through the real pipeline (the pit-pulse sisters' proven
    // death — real input, no teleport).
    let t0 = match t0 {
        Some(t) => t,
        None => {
            let ok = w.try_walk_to(walker::Wp::new(2.4, 1.0, -172.0).r(0.8), "the slab's north edge");
            assert!(ok || health(w.scene.sim.as_ref().unwrap()) <= 0.0, "the slab edge wedged with Bond alive");
            w.skip_cutscenes = false;
            let mut touched = false;
            for _ in 0..900u32 {
                if w.scene.sim.as_ref().unwrap().drones.channels.on(0x81) { touched = true; break; }
                if health(w.scene.sim.as_ref().unwrap()) <= 0.0 { break; }
                w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
                w.tick(Input { forward: 1.0, ..Default::default() }, false);
            }
            assert!(touched || health(w.scene.sim.as_ref().unwrap()) <= 0.0,
                "the walked step south never touched #255 (ch 0x81) at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
            for t in 0..4200u32 {
                w.tick(Input::default(), false);
                if health(w.scene.sim.as_ref().unwrap()) <= 0.0 { t0 = Some(w.total_ticks); break; }
                assert!(t < 4100, "the pit floor's pulse never killed the idle Bond");
            }
            t0.expect("the pulse's 1000-damage contact kills Bond [M 0x194458 + 0x1902c8]")
        }
    };
    eprintln!("[pain-gate] death at tick {t0}, health {}", health(w.scene.sim.as_ref().unwrap()));
    let sim = w.scene.sim.as_ref().unwrap();
    let bits_at_death = sim.drones.health.dir_bits;
    let alpha_at_death = sim.drones.pain_alpha;
    let hits_at_death = sim.drones.player_hits.len();
    let last_hit_frame = sim.drones.player_hits.last().map(|h| h.0);
    eprintln!("[pain-gate] at death: hits {hits_at_death} (last frame {last_hit_frame:?}), alpha {alpha_at_death}, bits {bits_at_death}");

    // The observation window: hold the death spot, let the guards' later rounds and any blast
    // reach the corpse, and pin the gate. The death sequence runs (the game's own ~450-tick
    // cover); the drones' update keeps ticking — rounds already in flight and later volleys
    // arrive through the real pipeline.
    let (mut hits_after, mut alpha_min) = (0usize, alpha_at_death);
    let (mut round_witness, mut panic_witness) = (false, false);
    for t in 1..=600u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.health.health, 0.0, "the corpse's health stays floored");
        hits_after = sim.drones.player_hits.len();
        if hits_after != hits_at_death {
            let last = sim.drones.player_hits.last().unwrap();
            panic!("a gated pain pushed a ledger entry at death+{t}: {last:?} (was {hits_at_death}) — the game's HandlePain writes nothing on health <= 0 [M 0x1902c8:18-20]");
        }
        alpha_min = alpha_min.min(sim.drones.pain_alpha);
        assert!(sim.drones.pain_alpha <= alpha_min + 1e-6,
            "the pane fade popped back up at death+{t} (alpha {}) — a gated pain wrote BL+0x8bc [M 0x1902c8:27 sits inside the gated body]", sim.drones.pain_alpha);
        if sim.drones.health.dir_bits != bits_at_death { round_witness = true; }
        if hits_after > hits_at_death { panic_witness = true; }
    }
    eprintln!("[pain-gate] death+600: hits {hits_after}, alpha min {alpha_min} (was {alpha_at_death}), dir_bits moved: {round_witness}");
    // The mechanism witness: SOMETHING must have reached the body for the pin to bite — a
    // post-death round moves the bullet arm's own +0x967 direction bits (the game's
    // unconditional post-HandlePain store [M 0x194458:203-226]). If the guards truly ceased
    // fire, print it — the ledger pin still held, but say so.
    if !round_witness {
        eprintln!("[pain-gate] NOTE: no post-death contact moved +0x967 — the pin held vacuously on this run; the ledger/pane pins are still the gate's");
    }
}
