// Real-damage bookkeeping shared by both walkers (`include!`d by walker.rs and walker_b2.rs, so
// each module gets its own copy of the type).
//
// `NF_WALK_NO_TOPUP=1` turns the walker's health top-up off for every walk in the binary, the
// tests' own `top_up_health = true` included (`Walker::tops_up`). The log then reads the sim's
// own hit record (`Drones::player_hits` — (frame, owner drone, health lost), pushed by every
// Bond-damage site in nf-game/drone.rs) each tick, keeps the lowest health, and on the first
// tick at 0 health prints the death spot and the last hits (owner skin, weapon, distance).
// `with_real_damage` prints the end summary (the env run prints one every 3000 ticks). No game
// state is written.

thread_local! {
    /// `with_real_damage`'s switch: the top-up is off for every walker on this test's thread.
    static REAL_DAMAGE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_real_damage_no_return_fire`: the walkers made on this thread do not return fire.
    static NO_RETURN_FIRE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_real_damage_head_hunting`: the fights on this thread take the clear head box first
    /// (see `exposed_impl`).
    static HEAD_HUNT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_real_damage_unseen_fights`: the fights on this thread answer guards whose boxes no
    /// eye ray reaches (desk cover) — the hurting-first fallback pick plus the stand's creep,
    /// the `lane_fights` machinery without the lane windows.
    static UNSEEN_FIGHTS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_real_damage_glass_sight`: the sight ray passes INTACT GLASS like the game's own
    /// round flights do (see `exposed_impl`).
    static GLASS_SIGHT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_real_damage_glass_fights`: the return-fire pick fights the shooter who is actually
    /// hurting Bond first and skips cooled-down candidates (see `return_fire`).
    static HURTING_FIRST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_resupply_in_place` (pretrans-arms-4, the 07000011 doorway-leg ledger): while set,
    /// the LEG-level dry resupply (walker_b2's arms while `walk_to` drives a leg) only takes
    /// pickups on the leg's own line — a mid-leg displacement across the room re-routes the
    /// resumed straight leg into furniture [L the 11a run: the resupply carried Bond 15 u east
    /// to the (24.85,22.21) box and the resumed line to the (16,6.4) doorway clipped the
    /// (21.5,15.9) desk corner, 6000 ticks]. walker.rs's master resupply never reads this.
    static RESUPPLY_IN_PLACE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The leg walker_b2::walk_to is currently driving (its straight-line target), read by the
    /// gate above; `None` outside walk_to (a walk file's own resupply stops stay master's).
    static IN_LEG_WP: std::cell::Cell<Option<[f32; 3]>> = const { std::cell::Cell::new(None) };
    /// The latest real-damage summary on this thread (refreshed every 60 ticks and at a death).
    static LAST_SUMMARY: std::cell::RefCell<Option<(u32, String)>> = const { std::cell::RefCell::new(None) };
}

/// Keep the walker's latest summary; under `NF_WALK_NO_TOPUP` print one every 3000 ticks.
pub fn note_summary(tick: u32, s: String) {
    if tick % 3000 == 0 && std::env::var_os("NF_WALK_NO_TOPUP").is_some() { eprintln!("[real-damage] at tick {tick}: {s}"); }
    LAST_SUMMARY.with(|c| *c.borrow_mut() = Some((tick, s)));
}

/// Print and clear this thread's latest summary (the end of a real-damage walk).
pub fn print_summary() {
    if let Some((tick, s)) = LAST_SUMMARY.with(|c| c.borrow_mut().take()) {
        eprintln!("[real-damage] at tick {tick}: {s}");
    }
}

/// The top-up is off: `NF_WALK_NO_TOPUP` for the whole binary, or `with_real_damage` for this
/// test's thread (libtest runs each test on its own thread, so a sister test is untouched).
pub fn no_topup_env() -> bool {
    REAL_DAMAGE.with(|c| c.get()) || std::env::var_os("NF_WALK_NO_TOPUP").is_some()
}

/// Run a full walk on real damage: no health top-up for any walker the closure makes (the
/// tests' own `top_up_health = true` included), and Bond's death panics with the death line.
#[allow(dead_code)]
pub fn with_real_damage<R>(f: impl FnOnce() -> R) -> R {
    // Reset on the way out, a panic included (libtest may run the next test on this thread).
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { REAL_DAMAGE.with(|c| c.set(false)); print_summary(); } }
    REAL_DAMAGE.with(|c| c.set(true));
    let _reset = Reset;
    f()
}

/// The leg-level resupply gate's reader (walker_b2's `resupply`): the opt-in is set and the
/// walker is inside a `walk_to` leg — the pick must stay on that leg's own line.
pub fn resupply_gate_active() -> Option<[f32; 3]> {
    if RESUPPLY_IN_PLACE.with(|c| c.get()) { IN_LEG_WP.with(|c| c.get()) } else { None }
}

/// walker_b2::walk_to brackets each leg with these (the gate reads the leg's own target).
/// Replace (not set/clear): the resupply fetch's own nav legs nest walk_to inside a leg, and
/// the outer leg's line must survive them.
pub fn walk_leg_replace(wp: [f32; 3]) -> Option<[f32; 3]> {
    let prev = IN_LEG_WP.with(|c| c.get());
    IN_LEG_WP.with(|c| c.set(Some(wp)));
    prev
}
pub fn walk_leg_restore(prev: Option<[f32; 3]>) { IN_LEG_WP.with(|c| c.set(prev)); }

/// OPT-IN (pretrans-arms-4, the 07000011 doorway-leg ledger): while set, the leg-level dry
/// resupply only takes pickups on the leg's own line (see RESUPPLY_IN_PLACE). The 07000011
/// real-damage walk sets it; every other walk keeps master's resupply behaviour.
#[allow(dead_code)]
pub fn with_resupply_in_place<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { RESUPPLY_IN_PLACE.with(|c| c.set(false)); walk_leg_restore(None); } }
    RESUPPLY_IN_PLACE.with(|c| c.set(true));
    let _reset = Reset;
    f()
}

/// `with_real_damage_head_hunting` plus the fights' glass sight (glass-pierce-2): the sight
/// rays drop the intact mat-0x10 static glass — see the GLASS_SIGHT field doc. Opt-in per test:
/// the walks whose fights must answer shooters behind intact station windows (the 08 real-damage
/// Agent sisters); every other walk's fight timelines stay byte-exact.
#[allow(dead_code)]
pub fn with_glass_sight<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { GLASS_SIGHT.with(|c| c.set(false)); } }
    GLASS_SIGHT.with(|c| c.set(true));
    let _reset = Reset;
    f()
}

/// `with_real_damage` plus the fights' head-hunting pick: the real-health runs whose fights
/// burn Bond down by attrition (the return fire's torso-nearest taps take 3-5 rounds a
/// 12-health guard while the guard's own volley lands) take the clear head box first — one
/// head hit drops a 12-health guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head; the
/// P2K's 3.5 → 35]. Opt-in per test: the master-green real-damage walks keep the
/// torso-nearest pick (their fight timelines are pinned; level-13-lobby-rifles-2).
#[allow(dead_code)]
pub fn with_real_damage_head_hunting<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { HEAD_HUNT.with(|c| c.set(false)); } }
    HEAD_HUNT.with(|c| c.set(true));
    let _reset = Reset;
    with_real_damage(f)
}

/// `with_real_damage` plus the unseen-cover fights: the plain pick only answers a shooter
/// whose boxes one eye ray reaches, and a guard behind desk cover empties volley after
/// volley while no fight ever starts (the 07000012 bend death: 64 health gone while no fight
/// ever started [L the level-13 NF_RF_GEOM dump — the Office-C_Obj9_computer mesh eats every
/// ray]). With this opt-in the fight takes the shooter who is actually HURTING Bond and the
/// stand creeps toward the muzzle until the line opens over the desk lip — the
/// `lane_fights` machinery's pick fallback and creep WITHOUT the lane windows (the
/// 20 u / 6.0 u shooters window, the plain eye-ray visibility and the torso-nearest pick
/// stay). Opt-in per test: the master-green real-damage walks keep the plain pick.
#[allow(dead_code)]
pub fn with_real_damage_unseen_fights<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { UNSEEN_FIGHTS.with(|c| c.set(false)); } }
    UNSEEN_FIGHTS.with(|c| c.set(true));
    let _reset = Reset;
    with_real_damage(f)
}

/// The 07000008 Agent-sisters' fight tactic (guard-glass-finish): `with_real_damage` + the
/// head-hunting pick + the GLASS-SIGHT sight rule (see `exposed_impl`) + the HURTING-FIRST
/// return-fire retargeting (see `return_fire`). The three laws answer the merged-master tree's
/// measured deaths: the mouth's solid wall band made every foot stand on the burst pair blind
/// (eleven 61-tick stands, aim None) while the pair walked 8.5 u to point-blank and the
/// stairwell-top pair landed free hits into the `fighting=true` window (d2, tick 25036); a
/// failed stand cooled the pick's first candidate and bailed the whole response, and the next
/// stand fought a zero-damage guard at 11 u while the 59-damage burst rifle chewed 4.4 u away
/// (d3, tick 25116). Opt-in per test: the master-green real-damage walks keep every pinned
/// timeline (none of them sets it).
#[allow(dead_code)]
pub fn with_real_damage_glass_fights<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) {
        GLASS_SIGHT.with(|c| c.set(false));
        HURTING_FIRST.with(|c| c.set(false));
    } }
    GLASS_SIGHT.with(|c| c.set(true));
    HURTING_FIRST.with(|c| c.set(true));
    let _reset = Reset;
    with_real_damage_head_hunting(f)
}

/// `with_real_damage` for a route whose legs were laid for an unanswered walk (a return-fire
/// stop on a climb or ahead of a scripted rescue breaks the leg): real damage, no return fire.
#[allow(dead_code)]
pub fn with_real_damage_no_return_fire<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { NO_RETURN_FIRE.with(|c| c.set(false)); } }
    NO_RETURN_FIRE.with(|c| c.set(true));
    let _reset = Reset;
    with_real_damage(f)
}

#[derive(Default)]
pub struct DamageLog {
    /// Entries of `player_hits` already read.
    pub hits_seen: usize,
    /// capsule-join-5 (OPT-IN, the 01 escort): the friendly-fire shield for the RETURN-FIRE
    /// stands — `Some((drone, bloom))` holds a stand's tap whose cone would cross that drone,
    /// the same cone `Walker::shoot` applies for `Walker::shield` (+ `shield_bloom`). The game's
    /// bullets hit ANY drone [M Drone_CollisionHandler 0x140968:85-95, no team gate]; the 01
    /// real-damage run's return fire on guard #15 killed the escort contact standing past him
    /// (by_player, weapon 0x1c, head part 5, t6620 [L the NF_HEALTH_TRACE run on this branch])
    /// and the mission failed (exit 0x07000001 restart). `None` = every other walk unchanged.
    pub fire_shield: Option<(usize, bool)>,
    /// Lowest health seen.
    pub min_health: f32,
    /// Health lost to recorded hits, by owner (drone index, or usize::MAX for the world).
    pub by_owner: Vec<(usize, f32, u32)>,
    /// Health lost outside the hit record (falls, drowning, hurt volumes).
    pub other: f32,
    /// The death line, once.
    pub died: Option<String>,
    /// Panic on Bond's death (the walk cannot go on; the real-damage variants and the env run).
    pub die_panics: bool,
    /// The log is live (top-up off at construction or later).
    pub active: bool,
    /// Return fire on real damage (`shooters`); a test that scripts its own fight turns it off.
    pub return_fire: bool,
    /// OPT-IN (drone-block-swaps-g055-3): when the return-fire stand's cap expires without the
    /// drop, run ONE determined `shoot_bounded` pass before the 240-tick cooldown. Under the
    /// flush [M 0x148010:100-131] the un-dropped guard keeps hunting from his first attack —
    /// the measured d1 07000016 route bled ~80 health across 8 such fights before the pit —
    /// and a player finishes the guard he is fighting. The route has no hard window, so the
    /// kill's time is free; the guards are hp 20, so the kill's rounds are cheaper than the
    /// bleed. False everywhere else — no other walk changes.
    pub finish_fights: bool,
    /// Inside a return-fire fight (the fight's own walks must not start another).
    pub fighting: bool,
    /// Return-fire fights started, for the summary.
    pub fights: u32,
    /// (drone, tick) — a guard a fight did not drop is not re-fought before the tick.
    pub cooldown: Vec<(usize, u32)>,
    /// OPT-IN for `stand_and_fire`: give the target up after this many taps without a round
    /// landing (a player switches to the next guard instead of emptying the clip into cover the
    /// world trace does not see). None keeps the 300-tick rule alone.
    pub dry_taps: Option<u32>,
    /// OPT-IN for `stand_and_fire`: walk on past the 300-tick no-progress give-up. The deck
    /// sentry's HIP stand fires the game's own 0.14 rad cone [M `Bullet_init` 0x12c5a0:139-170 —
    /// `mul = 0` only with the aim bit; the 0x1e record's +0x24 = 100.0 [L ee_start_05.bin
    /// @ 0x2c10b8]] — its legal misses never move the target's hp, so the default give-up (a
    /// guard in cover the stand cannot reach) walks off after ~3 rounds and the clip math never
    /// completes. False everywhere else — no other walk changes.
    pub dry_walk_on: bool,
    /// OPT-IN: record this stand's PLAYER round landings on drones (`PlayerImpact::Drone` —
    /// the per-tick impact gather `player_impacts`, visible at the after-step note). The flight
    /// Vec retires dead rounds and `BU.first` is consumed inside the tick, so the impact gather
    /// is the only post-tick view of where the fired rounds went. For the deck-sentry hip test
    /// only; False everywhere else.
    pub record_round_dirs: bool,
    /// The recorded landings (drone index, part) (see `record_round_dirs`).
    pub round_dirs: Vec<(usize, i16)>,
    /// OPT-IN (`shoot_blockers`, set by the 07000011 real-health walk): when the gun line to the
    /// fight's target is blocked by a LIVE HOSTILE drone, retarget the blocker. A player whose
    /// line another guard walks into shoots that guard — the hold-fire rule alone stood Bond on
    /// west flight B while the blocker (1.6 u, weapon 0x19) landed 15 rounds for 51 health
    /// [L the 2026-11-02 level-11 real-damage run]. Off = master behaviour (hold fire; the
    /// 03 servants stand among the guards and must not be shot through).
    pub shoot_blockers: bool,
    /// OPT-IN (`fire_through_disabled`, set by the 07000003 00 Agent walk): a drone the game has
    /// disabled (WAIT_SWITCH, +0xF4 enable cleared) never blocks the gun line — the game's own
    /// collide drops Enable(0) objects [M Collide_PickObj 0x1eae80: the +0xfd&1 arm returns the
    /// obj before the hit test; NDrone2_DSTATE_WaitSwitch 0x15b960 enter → NDrone2_Enable(0)
    /// 0x149dd0 obj+0xF0 |= 0x20 — the pin the smg-close-range-hits report cites for the dormant
    /// ch-0x5 guard taking nothing], and the bullets and the auto-aim skip it [M Check_AutoAim
    /// 0x1a73f8 skips +0xF4 == 1]. The plain blocked hold reads present/alive only, so a
    /// not-yet-enabled guard standing on the line held every tap ("killed false" x5, the
    /// 00agent ledger) while the shooter chewed Bond down. Off = master behaviour (the hold).
    pub fire_through_disabled: bool,
    /// OPT-IN (`switch_dry`, set by the 02/03 00 Agent walks): the in-hand mag ran dry mid-fight
    /// — the game's own weapon cycle to the next firearm with rounds (one press + the draw
    /// wait), instead of the master give-up. A player whose mag ran dry swaps guns mid-fight,
    /// not after it: the hp-5 rifle chaser finished a clip-0 Bond whose P99 still carried 15
    /// rounds [L the real-health-00agent-02-03-2 ledger, the courtyard-lane run]. Off = master
    /// behaviour (the fight is given up).
    pub switch_dry: bool,
    /// OPT-IN for `stand_and_fire`: Bond steps back (aim held) while any live guard stands
    /// inside this range. Under 3.0 u every drone round hits [M DroneWeap_DoBulletAccuracy
    /// 0x179740:113-115] and a shotgun guard's 8 pellets all land; None keeps Bond still.
    pub back_off: Option<f32>,
    /// Inside a return-fire resupply pass (`resupply`'s nav walks run the leg walks, whose
    /// return-fire gate would call `resupply` again — the recursion overflowed the stack).
    pub resupplying: bool,
    /// OPT-IN lane tactics for a long open-floor crossing (the 07000013 lobby probe): the wide
    /// shooters window (48 u / 7.5 u instead of 20 u / 6.0 u), the damage-weighted target pick,
    /// and the floor-drop resupply passes around a fight. Off = the plain return fire every
    /// other walk is pinned against: with the three on for every walk, the 07000001 contact,
    /// 07000002 castle and 07000007 party real-damage walks went red (a death at the 01 stairwell,
    /// 6000-tick stalls on the 02 geisha and 07 east-stair legs) and they pass with them off.
    pub lane_fights: bool,
    /// OPT-IN (`lane_floor_drops`, set by the 07000011 00 Agent walk): the lane resupply's
    /// on-floor band relaxes to the master 1.5. This level's dropped guns rest with their centre
    /// ON the floor ray [L drop-stand-height probe 2026-10-22 — the 07000001 drops' centre
    /// 5.3452 vs the floor 5.3452096], 1.09 under Bond's standing pos y, outside the lane 0.5
    /// band — the lane pass could not see this level's own fight drops and every pass came back
    /// dry while the fights ran the 7-round pistol empty. Off = the lane 0.5 band everywhere
    /// else (no other walk sets it).
    pub lane_floor_drops: bool,
    /// The fight about to run picked a shooter whose boxes are NOT reachable from here (the
    /// unseen-cover fallback): a short stand cap, the creep has to open the line fast.
    pub unseen_fight: bool,
    /// OPT-IN (drone-block-swaps-g055-4, `level_fights`, set by the 07000016 real-damage walk):
    /// the return-fire pick keeps only the shooters at Bond's LEVEL (pos y within ±2.0 of
    /// Bond's) — a guard a level above (the route's ring guards at y 6.71 over the floor-0
    /// approach) or below (the trench guard under the ring) is not answerable from here. The
    /// 07000016 route's ambushers are the ring guards — placements 186-191 spawn at y 6.71 on
    /// the upper ring [L probe_16_guards] over the floor-0 approach — and a floor-level
    /// Bond cannot answer them: the measured reactive stands fought the overhead fire (the
    /// flank nav dragged Bond up the ramp wall, head_bump; the fights' displacement pushed him
    /// off the lane into the trench — two deaths at y −4.65/−5.04, 71 hits for 100 each). A
    /// player under overhead fire keeps walking the legs and fights the ring AT LEVEL (the
    /// route's own ring legs, hp 10 each). Off = master behaviour (the plain pick) everywhere
    /// else — no other walk changes.
    pub level_fights: bool,
    /// OPT-IN (`tranq_fights`, the 0700000b walk): Bond's only conventional weapon on 0700000b is
    /// the Korsakov K5 sleep dart (0x43 — the silent-infiltration kit [M 0x1a7ef8 case
    /// 0x7000009/0a/0b the boot loadout; L the death run's bullet probe — wpn 0x43, dmg 0.10,
    /// range 45]), and the game's own dart law drops msg 0x19 for a guard in full combat (`DSTATE_Attack`
    /// 0x164210 has no case 0x19; the state gate is `DroneFunc_HandleImpact` 0x1469b8:315-361 →
    /// `Drone::dart_impact_state_ok`). So a fight against an attacking guard is UNWINNABLE — a
    /// player walks on instead of standing in the open eating volleys — and a fight's WIN is the
    /// guard's sleep, not a health drop (the dart's HitDamage is never called [M 0x13fff0:269-306];
    /// hp never moves). The pick answers only guards whose state carries case 0x19, and
    /// `stand_and_fire` reads a dart landing (the target entering the dart family) as progress and
    /// the under states as the win.
    pub tranq_fights: bool,
    /// OPT-IN (`hurting_first`, set by the 07000008 Agent sisters, `with_real_damage_glass_fights`):
    /// the return-fire pick orders the VISIBLE shooters by the damage they dealt Bond over the
    /// same 240-frame window (nearest on ties — the lane pick's order, the plain eye rays and
    /// windows), and candidates a failed fight just put on cooldown are dropped instead of
    /// bailing the whole response. Measured d3 death (tick 25116): the pick fought the
    /// zero-damage #23 at 11 u while #20 (59.0 in 7 hits, 4.4 u) chewed, after the failed #20
    /// stand's cooldown silenced the response outright. Off = master behaviour (nearest visible
    /// first, cooldown bails) everywhere else.
    pub hurting_first: bool,
    /// OPT-IN (`volley_carrier_first`, set by the 07000001 real-damage escort walks only):
    /// the same two laws as `hurting_first` (the damage-weighted pick and the stand's 30-tick
    /// retarget) WITHOUT the cooldown drop, scoped to this walk. Measured mechanism
    /// (pretrans-arms-3/4, [L the NF_RF_TRACE run on s/pretrans-arms-4]): the hall flank's
    /// deeper nav leg wedges one step early on master's solid guard bodies ("flank 1 nav 2"
    /// vs the -2 tree's "flank 1 nav 3"), the 24-hp shotgun captain #7 is answered LAST from
    /// the wedged node, and his planted 8-pellet volley — the weapon datum reads 0x0 at the
    /// fight, the volley rides the script side [L the death ledger], so neither `agent_order`'s
    /// weapon key nor a loop-order class can see him — lands 48.0 (8 × 6.0) in one window on
    /// Bond at 76.0 while the stand is on #11 (health 76.0 → 28.0 between fight#11 t0 and
    /// fight#7 t0 [L the same run]); death at tick 10682 (46 hits for 100.0). On the -2 tree
    /// the captain engaged FIRST at full hp and cost 7.5 [L the -2 report's walk list]. The
    /// volley carrier's fingerprint IS the damage window: while his pellets land, his 240-frame
    /// damage leads every other shooter's, so the damage-weighted pick answers him first and
    /// the stand's retarget switches to him mid-volley (the pellets stop with him). A player
    /// under a planted volley answers the shotgun, not the further pistol. Off = master
    /// behaviour (nearest visible) everywhere else — no other walk sets it.
    pub volley_carrier_first: bool,
    /// OPT-IN (`shooters_window`, the 07000005 00 Agent walk): the shooters list's (horizontal,
    /// vertical) reach in u, replacing the plain (20.0, 6.0) / lane (48.0, 7.5) windows. The 05
    /// castle-approach spine is fired on from 16.6 u BELOW (the gallery riflemen #8/#9 stand at y
    /// 22.0 while the spine runs y 38.6 [L the 2026-11-03 00agent repro a05c.out: 18 rounds for
    /// 48.0 on the spine, no fight — the dy filter dropped them before the pick]); their rounds
    /// reach Bond, so the eye ray reaches them — a player pinned on a ledge shoots DOWN at the
    /// riflemen. None keeps the master windows (default).
    pub shooters_window: Option<(f32, f32)>,
    /// OPT-IN (`aimed_fights`, the 0700000c yard-deck-sentry stand, player-upward-shot): the stand
    /// holds the AIM button through the whole stand — `Input { aiming: true }` every tick — so a
    /// 0x400000 record (the snipers) fires its ZERO spread [M `Bullet_init` 0x12c5a0:139-170 —
    /// `mul = 0` only with the aim bit; the 0x1e record carries 0x400000 [L ee_start_05.bin
    /// @ 0x2c10b8 flags68 0x408050, +0x24 = 100.0]]. Without it the same stand fires the HIP cone —
    /// 0.14 rad = 4.9 u of scatter at the deck sentry's 35 u — and a clip dries with the target's
    /// hp untouched (the game's own hip behaviour; [L] the blocked real-health run's three dry
    /// clips). Off = the plain stand every other walk is pinned against.
    pub aimed_fights: bool,
    /// OPT-IN (`precise_fights`, the 07000011 real-health walk): the aimed TAP GATE (0.01 rad)
    /// WITHOUT the aim button. The 2026-11-03 level-11 experiment measured the split: with the
    /// button held the port's free-aim path reinterprets the fight's `turn`/`look` as CURSOR moves
    /// (scene.rs free-aim: `want = cursor − turn/0.76966`, clamped ±0.296) and the fired round's
    /// direction is the view PLUS the cursor offset — the 07000011 top-floor fight tapped 7 rounds
    /// at err 0.0000 into a 4-u stationary guard for zero boxes (repro11c) — the fight machinery
    /// drives the VIEW and cannot drive the cursor window. Gate only, every fight dropped its
    /// target (repro11d: the stairwell fights all killed, 100.0 → 56.2 through the descent). The
    /// 0c snipers keep the button (`aimed_fights`) — their zero spread needs the bit
    /// [M Bullet_init 0x12c5a0:139-170].
    pub precise_fights: bool,
    /// OPT-IN (`sniper_stands`, set by the 0700000c real-health walks): the return-fire stand
    /// against the FAR snipers (d 61-90, the 0x1e family) runs the level's PROVEN still-gate
    /// aimed stand (walkreal0c.rs's `yard_stand_aimed`, the deck-sentry proof): the aim button
    /// held from a settled gun (the raise queues ONLY on the aim-PRESSED edge from the idle
    /// machine [M 0x1a24c8:113-152]), the CURSOR driven to the position that puts the EFFECTIVE
    /// aim (the view + the eased offsets [M scene.rs free-aim]) on the part, and the tap gated
    /// on the aim BIT plus the effective error 0.01 plus the target's root holding still 18
    /// ticks — a guard that steps during the round's flight is not where the view was
    /// [M Bullet_update 0x12e7b0 — the round steps its speed per tick]. Measured on 0c (the
    /// 2026-11-06 window): the plain stands' converged VIEW taps (err 0.000, perr 0.000,
    /// blocked false) drained clips on the d 61-90 snipers without one landing — the hip cone
    /// (the 0x1e spread base 100.0 [L ee_start_05.bin @ 0x2c10b8 +0x24]) scatters multiples of
    /// the chest box at that range, and the wiggle-step ±0.1 u beats the flight. Off = the
    /// plain stand everywhere else — no other walk changes.
    pub sniper_stands: bool,
    /// OPT-IN (`escalating_cooldowns`, set by the 0700000c Agent walk, real-health-agent-0c-0d):
    /// a fight the stand did not drop doubles its re-fight wait each further failure on the same
    /// drone — 240, 480, 960, 1920 (capped). The master 240 keeps re-fighting a shooter whose
    /// boxes the geometry eats on every leg, and each failed stand is standing exposure in the
    /// open (the first d2 run re-fought the display-ground guard #27 at ticks 2707 and 3637 for
    /// 26.6 + 29.6 health — the drains that killed the walk). A player who could not answer a
    /// shooter from a spot does not re-take the same spot; the leg's own walk carries Bond out of
    /// the sight line. Off = master behaviour (the flat 240).
    pub escalating_cooldowns: bool,
    /// (drone, tick) per failed fight — the escalation's counter (the flat law keeps no history:
    /// `cooldown` holds only the latest entry per drone).
    pub fight_fails: Vec<(usize, u32)>,
    /// OPT-IN (`eye_ray_stands`, set by the 07000011 00 Agent walk): under `lane_fights` the
    /// stands aim through the plain eye ray (`exposed_part`) instead of the muzzle stand-ins
    /// (`exposed_part_gun_rays`). This level's strip guards fire across the flights while the
    /// stand-in rays clip the flight lips — the measured crossings drained the armour 43→0 with
    /// NO fight ever triggered (the pick returned None, the volleys kept landing), and the 23.0
    /// strip fights met Bond with the armour gone. A player answers what he SEES. Off = master
    /// behaviour (the lane gun-ray pick) everywhere else — no other walk sets it.
    pub eye_ray_stands: bool,
    /// OPT-IN (`pre_throw_stands`, set by the 07000015 real-damage Agent walk): a
    /// spawner-released hostile pre-empts the return-fire pick (nearest first, visible only,
    /// inside `PRE_THROW_WINDOW_TICKS` of the game's own birth stamp), and `stand_and_fire`
    /// runs the 0c `sniper_stands` pattern minus the aim bit — the settle (the raise/switch
    /// anims drained before the first tap [M 0x1a24c8:113-152's pressed-edge law]) and the
    /// tap gated on the exposed part at the 0.01-rad gate — so the kill lands the tick after
    /// the release, BEFORE the guard's first throw. The base ledger's kills always came AFTER
    /// the throw: #11 was killed at 6524 and its grenade still landed 6730 for −15.0; #13 was
    /// killed at 9248 and its grenade −17.5 landed at the death tick 9422 [L the
    /// aim-last-seven §3 NF_DMG_TRACE ledger] — the guards' throw cycle beats the ~90-tick
    /// plain kill from every measured stance (the four position levers all measured worse).
    /// The birth stamp is the game's own: the spawner release writes `Drone::spawn_frame` =
    /// the sim frame alongside `present`/`enabled` [M nf-game drone.rs spawner arm], and
    /// placement drones carry 0 for life, so a non-zero stamp inside the window IS a release.
    /// Off = master behaviour everywhere else — no other walk sets it.
    pub pre_throw_stands: bool,
    last_health: f32,
    last_hits: Vec<(u32, usize, f32)>,
    /// The live-hostile drones this log has already seen alive, with the walker tick of the
    /// first sighting — `pre_throw_stands`'s release detector. A drone's first alive tick IS
    /// its release: spawner templates are written `present` the frame the spawner releases
    /// them [M nf-game drone.rs — the spawner arm also stamps `Drone::spawn_frame`], and a
    /// channel-held placement sits present-but-disabled until its channel fires [M drone.rs
    /// switch_channel/channel_on] — either way the first poll that sees it alive is within
    /// 10 walker ticks of the release (the poll cadence). Seeded on the first poll after the
    /// walk's own `wait_enabled`+`wait` prologue, so the level's standing garrison is born
    /// outside every window that matters.
    alive_seen: Vec<(usize, u32)>,
}

impl DamageLog {
    pub fn new() -> DamageLog {
        DamageLog { min_health: 100.0, last_health: 100.0, die_panics: no_topup_env(), active: no_topup_env(), return_fire: !NO_RETURN_FIRE.with(|c| c.get()) && std::env::var_os("NF_WALK_NO_RETURN_FIRE").is_none(), hurting_first: HURTING_FIRST.with(|c| c.get()), ..Default::default() }
    }

    fn owner_desc(sim: &nf_view::scene::Sim, owner: usize) -> String {
        let b = sim.bond.pos;
        match sim.drones.list.get(owner) {
            Some(d) => format!("drone #{owner} skin {:#x} weapon {:#x} at ({:.1},{:.1},{:.1}) d {:.1} hp {:.0}",
                d.skin, d.weapon, d.pos[0], d.pos[1], d.pos[2],
                ((d.pos[0] - b[0]).powi(2) + (d.pos[1] - b[1]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt(), d.health),
            None => "the world (live wire / blast / hurt volume)".into(),
        }
    }

    /// Read this tick's damage (before the walker's top-up, when on).
    pub fn note(&mut self, sim: &nf_view::scene::Sim, tick: u32) {
        if self.record_round_dirs {
            for imp in &sim.drones.player_impacts {
                if let nf_game::drone::PlayerImpact::Drone { drone, part, .. } = imp {
                    self.round_dirs.push((*drone, *part));
                }
            }
        }
        let h = sim.drones.health.health;
        let hits = &sim.drones.player_hits;
        let mut recorded = 0.0;
        if self.hits_seen > hits.len() { self.hits_seen = 0; }
        for &(f, o, dmg) in &hits[self.hits_seen..] {
            recorded += dmg;
            match self.by_owner.iter_mut().find(|e| e.0 == o) {
                Some(e) => { e.1 += dmg; e.2 += 1; }
                None => self.by_owner.push((o, dmg, 1)),
            }
            if std::env::var_os("NF_DMG_TRACE").is_some() && dmg != 0.0 {
                let p = sim.bond.pos;
                eprintln!("[dmg] tick {tick} frame {f}: -{dmg:.2} (health {:.1}) Bond ({:.1},{:.1},{:.1}) from {}",
                    sim.drones.health.health, p[0], p[1], p[2], Self::owner_desc(sim, o));
                if o >= sim.drones.list.len() {
                    for e in sim.drones.explosions.iter().rev().take(3) {
                        let d = ((e.0[0] - p[0]).powi(2) + (e.0[1] - p[1]).powi(2) + (e.0[2] - p[2]).powi(2)).sqrt();
                        eprintln!("[dmg]   explosion at ({:.1},{:.1},{:.1}) d {d:.1} radius {:.1} damage {:.1} owner {} kind {}",
                            e.0[0], e.0[1], e.0[2], e.1, e.2, e.3 as isize, e.4);
                    }
                    // scratch-16-diag: grenades in flight (owner, weapon, pos) around a world hit.
                    for (gi, g) in sim.drones.grenades.iter().enumerate() {
                        let d = ((g.pos[0] - p[0]).powi(2) + (g.pos[1] - p[1]).powi(2) + (g.pos[2] - p[2]).powi(2)).sqrt();
                        eprintln!("[dmg]   grenade #{gi} at ({:.1},{:.1},{:.1}) d {d:.1} owner {} weapon {:#x} state {}",
                            g.pos[0], g.pos[1], g.pos[2], g.owner as isize, g.weapon_type, g.state);
                    }
                }
            }
            self.last_hits.push((f, o, dmg));
            if self.last_hits.len() > 6 { self.last_hits.remove(0); }
        }
        self.hits_seen = hits.len();
        if !self.active { self.last_health = h; return; }
        let lost = self.last_health - h;
        if lost > recorded + 0.01 { self.other += lost - recorded; }
        if h < self.min_health { self.min_health = h; }
        self.last_health = h;
        if h <= 0.0 && self.died.is_none() {
            let p = sim.bond.pos;
            let mut s = format!("Bond died at tick {tick} (frame {}) at ({:.2},{:.2},{:.2}) armour {:.1}; last hits:",
                sim.drones.frame, p[0], p[1], p[2], sim.drones.health.armour);
            for &(f, o, dmg) in &self.last_hits {
                s.push_str(&format!("\n  frame {f}: -{dmg:.2} from {}", Self::owner_desc(sim, o)));
            }
            if lost > recorded + 0.01 { s.push_str(&format!("\n  -{:.2} this tick outside the hit record (fall / drown / hurt volume)", lost - recorded)); }
            s.push_str(&['\n'].iter().collect::<String>());
            s.push_str(&self.summary(sim));
            if std::env::var_os("NF_RF_TRACE").is_some() {
                s.push_str("\n  recent player_hits:");
                for h in sim.drones.player_hits.iter().rev().take(20) {
                    s.push_str(&format!("\n    frame {} owner {} -{:.2}", h.0, h.1, h.2));
                }
            }
            eprintln!("[real-damage] {s}");
            self.died = Some(s);
        }
    }

    /// Return fire (real damage only): the hostiles shooting AT Bond right now, nearest first — a
    /// guard (team 1, alive, enabled) within 48 u that hit him in the last 240 frames, or fired
    /// in the last 90 with his aim point on Bond (`opp_target` within 1.5 u). A player answers
    /// whoever shoots him; a guard that has not opened fire (a party guest's guard, an unalerted
    /// patrol) is left alone, so stealth and no-kill legs are unchanged. The caller keeps the
    /// ones it can see a part of (`exposed_aim`). The window is 20 u / 6.0 u; with `lane_fights`
    /// it widens. The wide vertical window is 7.5: the 07000013 tower
    /// lobby's mezzanine deck sits 6.05 above the floor (the sniper at y −362.36 vs −368.43 [D
    /// the drone dump]) and the level-13-chain-difficulty real-damage run died under his fire
    /// with the old 6.0 window. The wide horizontal window is 48, not 20: the same sniper works the
    /// lobby's north lane from d ≈ 43 [L the level-13 real-damage death log] — the fight is a
    /// STAND, no chase, and `exposed_part`'s world trace still decides reachability, so a far
    /// guard the stand cannot hit just times the fight out.
    pub fn shooters(&self, sim: &nf_view::scene::Sim) -> Vec<usize> {
        let b = sim.bond.pos;
        let now = sim.drones.frame;
        let dxz = |p: [f32; 3]| ((p[0] - b[0]).powi(2) + (p[2] - b[2]).powi(2)).sqrt();
        let mut c: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
            .filter(|(i, d)| {
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { return false; }
                let (wxz, wy) = self.shooters_window.unwrap_or(if self.lane_fights { (48.0, 7.5) } else { (20.0, 6.0) });
                if dxz(d.pos) > wxz || (d.pos[1] - b[1]).abs() > wy { return false; }
                let hit = sim.drones.player_hits.iter().rev()
                    .take_while(|h| now.saturating_sub(h.0) <= 240)
                    .any(|h| h.1 == *i);
                let aiming = d.shots.last().is_some_and(|&f| now.saturating_sub(f) <= 90)
                    && ((d.opp_target[0] - b[0]).powi(2) + (d.opp_target[2] - b[2]).powi(2)).sqrt() < 1.5;
                hit || aiming
            })
            .map(|(i, d)| (dxz(d.pos), i)).collect();
        c.sort_by(|a, b| a.0.total_cmp(&b.0));
        c.into_iter().map(|(_, i)| i).collect()
    }

    /// The exposed part of drone `i` Bond's bullet can reach: the hit box whose centre ray enters
    /// the box before the static world's bullet trace stops it (walker.rs `exposed_aim`'s rule —
    /// `Collide_Jointy` 0x1ec180's first box vs the flight's world stop [M 0x1ea398:216-229]),
    /// the one nearest the torso. None = nothing of him is reachable from here. The ray set is
    /// the single EYE ray — the shared rule every walk fights with; the stricter gun-ray set
    /// below is opt-in (`lane_fights`, the 07000013 walk only), because the master-green
    /// real-damage walks fight on this rule (level-13-lobby-rifles-2).
    pub fn exposed_part(sim: &nf_view::scene::Sim, i: usize) -> Option<[f32; 3]> {
        Self::exposed_impl(sim, i, false)
    }

    /// The lane-fight ray set: the EYE and the gun's stand-in offsets. The round leaves the view
    /// model low-right of the eye, and at a wall edge the eye ray clears by a hair while the gun
    /// ray clips the corner — the 07000013 security-room stand held aim Some (eye ray clear) while
    /// every round died on the door jamb (tgt hp pinned at 6.0, clip draining, [L the fight#9
    /// rf-trace]). Both lateral offsets and the low one must be clear to call the box exposed.
    /// OPT-IN: set only by the 07000013 walk (`lane_fights`) — on the shared rule the other
    /// real-damage walks pick fights master-green (level-13-lobby-rifles-2).
    pub fn exposed_part_gun_rays(sim: &nf_view::scene::Sim, i: usize) -> Option<[f32; 3]> {
        Self::exposed_impl(sim, i, true)
    }

    fn exposed_impl(sim: &nf_view::scene::Sim, i: usize, gun_rays: bool) -> Option<[f32; 3]> {
        let d = &sim.drones.list[i];
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
        let right = [cy, 0.0, -fy];
        let gun_spots = [eye, [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
            [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]];
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
            ai_points: &sim.ai_points,
        };
        let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
        let mut best: Option<(f32, [f32; 3])> = None;
        let mut head: Option<[f32; 3]> = None;
        for b in d.hit_boxes(&data) {
            let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
            let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
            let Some(t_box) = nf_game::damage::ray_box(eye, ray, &b) else { continue };
            // GLASS_SIGHT (opt-in): the world trace runs the flight's GLASS PASS — a mat-0x10
            // pane contact is a PASS [M Collide_Update 0x1ea398:216-229], the same law the
            // guards' and Bond's own gun rounds fly under ([L weapon_data] bit 0x800 clear).
            let glass = GLASS_SIGHT.with(|c| c.get());
            let sight = |g: [f32; 3]| if glass {
                sim.world.segment_flight_scoped(sim.world.find_cel(g), g, c, true)
            } else {
                sim.world.segment_bullet_scoped(sim.world.find_cel(g), g, c)
            };
            if gun_rays {
                let mut clear = true;
                for g in &gun_spots {
                    let gr = [c[0] - g[0], c[1] - g[1], c[2] - g[2]];
                    let tw = sight(*g);
                    if tw.is_some_and(|w| w < t_box) { clear = false; break; }
                }
                if !clear { continue; }
            } else {
                let tw = sight(eye);
                if tw.is_some_and(|w| w < t_box) { continue; }
            }
            let dev = ((c[0] - torso[0]).powi(2) + (c[1] - torso[1]).powi(2) + (c[2] - torso[2]).powi(2)).sqrt();
            if best.map_or(true, |(bd, _)| dev < bd) { best = Some((dev, c)); }
            // HEAD-HUNT (opt-in, `with_real_damage_head_hunting`): the clear head box (part 5)
            // within 6 u of the eye — the same window walker.rs `exposed_aim`'s `prefer_head`
            // uses — wins over the torso-nearest pick.
            if head.is_none() && HEAD_HUNT.with(|c| c.get()) && b.part == 5 && ((c[0] - eye[0]).powi(2) + (c[2] - eye[2]).powi(2)).sqrt() <= 6.0 {
                head = Some(c);
            }
        }
        if let Some(h) = head { return Some(h); }
        best.map(|(_, c)| c)
    }

    /// Bond can put a round into drone `i` from where he stands (`exposed_part`).
    pub fn visible(sim: &nf_view::scene::Sim, i: usize) -> bool { Self::exposed_part(sim, i).is_some() }

    /// `visible` under the lane-fight ray set (`exposed_part_gun_rays`).
    pub fn visible_gun_rays(sim: &nf_view::scene::Sim, i: usize) -> bool { Self::exposed_part_gun_rays(sim, i).is_some() }

    /// Why `exposed_part` cannot see drone `i`: every hit box's centre ray with its world stop and
    /// the stopping owner's run, plus the guard's own muzzle→Bond path. NF_RF_GEOM diagnostic.
    pub fn geom_dump(sim: &nf_view::scene::Sim, i: usize) {
        let d = &sim.drones.list[i];
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        eprintln!("[geom] guard #{i}: pos ({:.2},{:.2},{:.2}) weapon {:#x} hp {:.1} down {} corpse {} present {} enabled {} team {} los {} seen {} dist {:.2} state {:#x} danim {:#x} anim_class {} anim_state {:#x}",
            d.pos[0], d.pos[1], d.pos[2], d.weapon, d.health, d.down, d.corpse, d.present, d.enabled, d.team, d.los, d.seen, d.dist,
            d.state, d.anim.danim, d.anim.class, d.anim.state);
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
            ai_points: &sim.ai_points,
        };
        let boxes = d.hit_boxes(&data);
        let cell_bond = sim.world.find_cel(eye);
        eprintln!("[geom] guard #{i}: {} boxes, bond cel {cell_bond:?} eye ({:.2},{:.2},{:.2})",
            boxes.len(), eye[0], eye[1], eye[2]);
        let mut stop_named = false;
        for (k, b) in boxes.iter().enumerate() {
            let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
            let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
            let t_box = nf_game::damage::ray_box(eye, ray, b);
            let tw = sim.world.segment_bullet_scoped(cell_bond, eye, c);
            let twf = sim.world.segment_bullet(eye, c);
            eprintln!("[geom]   box {k} part {} centre ({:.2},{:.2},{:.2}) half {:?} t_box {t_box:?} tw_scoped {tw:?} tw_flat {twf:?}",
                b.part, c[0], c[1], c[2], b.half);
            // Name the blocking runs once: the runs whose bounds hold the first blocked ray's stop point.
            if !stop_named && t_box.is_some() && tw.is_some_and(|w| w < t_box.unwrap()) {
                stop_named = true;
                let s = [eye[0] + ray[0] * tw.unwrap(), eye[1] + ray[1] * tw.unwrap(), eye[2] + ray[2] * tw.unwrap()];
                if let Some(runs) = cell_bond.and_then(|cel| sim.world.scoped_run_bounds(cel, eye, c)) {
                    for (r, owner, lo, hi) in &runs {
                        let inside = (0..3).all(|a| s[a] >= lo[a] - 0.05 && s[a] <= hi[a] + 0.05);
                        if inside {
                            eprintln!("[geom]   stop ({:.2},{:.2},{:.2}) inside run {r} owner {owner} bounds lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})",
                                s[0], s[1], s[2], lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
                        }
                    }
                }
            }
        }
        // The guard's own lane: muzzle → Bond's chest, scoped from the muzzle's cel and flat.
        let m = d.muzzle(&data);
        let chest = [bp[0], bp[1] + 0.6, bp[2]];
        let cell_g = sim.world.find_cel(m);
        let tw_g = sim.world.segment_bullet_scoped(cell_g, m, chest);
        let twf_g = sim.world.segment_bullet(m, chest);
        let owners_g = cell_g.and_then(|cel| sim.world.scoped_run_owners(cel, m, chest));
        eprintln!("[geom] guard #{i}: muzzle ({:.2},{:.2},{:.2}) cel {cell_g:?} → chest tw_scoped {tw_g:?} tw_flat {twf_g:?} owners {:?} bond cel {cell_bond:?}",
            m[0], m[1], m[2], owners_g.as_ref().map(|o| &o[..o.len().min(6)]));
    }


    pub fn summary(&self, sim: &nf_view::scene::Sim) -> String {
        let mut top = self.by_owner.clone();
        top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut s = format!("end health {:.1} armour {:.1}, lowest {:.1}, {} hits for {:.1}, other {:.1}{}",
            sim.drones.health.health, sim.drones.health.armour, self.min_health,
            top.iter().map(|e| e.2).sum::<u32>(), top.iter().map(|e| e.1).sum::<f32>(), self.other,
            if self.died.is_some() { ", DIED" } else { "" });
        s.push_str(&format!(", {} return-fire fights", self.fights));
        for e in top.iter().take(5) {
            s.push_str(&format!("\n  {:.1} in {} hits: {}", e.1, e.2, Self::owner_desc(sim, e.0)));
        }
        s
    }
}

/// `pre_throw_stands`'s window: a spawner release is answered for this many sim frames after
/// its `spawn_frame` (the return-fire poll runs every 10 walker ticks, so the window must
/// cover a poll plus the converge; 1200 = 20 s at the fixed 60 Hz). The measured release→sight
/// gap on 07000015 is the reason it is not tighter: the west-room pair releases at the
/// door[32] threshold leg (~28 u away, BEHIND the west wall — the door[29] doorway's z band
/// blocks every ray from there [L the 2026-11-07 run-1 trace: no PRE-THROW line for #10/#11])
/// and rounds the corner into sight only during the route_c legs, ~1000 frames later — a
/// 120 window expired before the first sight and the pair was left to the jog sweep's flank
/// machinery (21.0). A fight started inside 1200 still precedes the guard's SECOND throw
/// cycle; a failed fight's 240-tick cooldown lets a release be fought at most ~5 times by
/// this path, and the plain shooters take over from there.
const PRE_THROW_WINDOW_TICKS: u32 = 1200;

// The return-fire response, shared by both walkers (this file is `include!`d into each walker
// module, so `Walker` is that module's walker; both carry `tick`, `tick_fire`, `switch_to_gun`,
// `tops_up`, `scene`, `total_ticks` and `dmg`).
#[allow(dead_code)]
impl Walker {
    /// Real damage only: answer the guard shooting Bond (`DamageLog::shooters`, the nearest one
    /// Bond can see) from where he stands, then let the leg walk on. Returns true when a fight
    /// ran. Off while the top-up is on, so the top-up walks are unchanged.
    pub fn return_fire(&mut self, label: &str) -> bool {
        let trace = std::env::var_os("NF_RF_TRACE").is_some();
        if self.tops_up() || !self.dmg.return_fire || self.dmg.fighting { return false; }
        let mut picked: Option<usize> = None;
        let di = {
            let sim = self.scene.sim.as_ref().unwrap();
            if sim.disabled || !sim.bond.grounded { return false; }
            let cands = self.dmg.shooters(sim);
            // OPT-IN `level_fights` (drone-block-swaps-g055-4, the field doc): the shooters a
            // level above Bond are not answers — the pick drops them and the leg walks on.
            let cands = if self.dmg.level_fights {
                let by = sim.bond.pos[1];
                cands.into_iter()
                    .filter(|&i| { let y = sim.drones.list[i].pos[1]; y <= by + 2.0 && y >= by - 2.0 })
                    .collect::<Vec<usize>>()
            } else { cands };

            // The plain pick: the nearest shooter Bond can see (`shooters` is nearest first).
            // With `lane_fights`, fight the shooter who is actually HURTING Bond first: the 07000013 crossing mixed
            // 0-damage desk guards with two rifle guards (weapon 0x18, 7.0 a hit) at 25-45 u, and
            // nearest-first spent the whole crossing on the desks while the rifles chewed 78
            // health [L the level-13 real-damage death logs]. Damage over the same 240-frame
            // window, nearest first on ties; the visibility gate still applies.
            let now = sim.drones.frame;
            let dmg_of = |i: usize| -> f32 {
                sim.drones.player_hits.iter().rev()
                    .take_while(|h| now.saturating_sub(h.0) <= 240)
                    .filter(|h| h.1 == i)
                    .map(|h| h.2).sum::<f32>()
            };
            let dist_of = |i: usize| -> f32 {
                let d = &sim.drones.list[i];
                ((d.pos[0] - sim.bond.pos[0]).powi(2) + (d.pos[2] - sim.bond.pos[2]).powi(2)).sqrt()
            };
            let mut unseen_fight = false;
            // tranq_fights (the 0700000b walk): the K5's dart lands only on a guard whose state
            // carries `DroneFunc_HandleImpact` case 0x19 [M 0x164210 — Attack has no case 0x19;
            // 0x1469b8:315-361] — fighting an attacking guard cannot win, so the player walks on
            // (the death run's fight#1/#3 stood 833 ticks in the ring open for 21 health against
            // two ATTACK-state guards the darts could never touch [L]).
            let cands = if self.dmg.tranq_fights {
                cands.into_iter()
                    .filter(|&i| nf_game::drone::Drone::dart_impact_state_ok(sim.drones.list[i].state))
                    .collect::<Vec<usize>>()
            } else { cands };
            // HURTING-FIRST (opt-in, `hurting_first`, the 07000008 Agent sisters, guard-glass-finish):
            // two retarget laws the plain pick lacks, both measured in the same d3 death (tick 25116):
            // 1. A guard a failed fight just put on cooldown does not silence the whole response: the
            //    failed #20 stand (he ducked behind the mouth's wall band mid-fight, aim None at t60)
            //    cooled #20 for 240 ticks and the pick returned #20 first and BAILED — #19 (fightable,
            //    hp 10, 6 u) was never considered, and the pair burst Bond down unanswereded. The
            //    filter drops cooled candidates so the response moves to the next answerable shooter.
            // 2. The pick fights the shooter who is actually HURTING Bond first (the lane pick's
            //    damage-weighted order, the PLAIN eye-ray visibility and the plain windows — see the
            //    pick arm below): the same run's later stand fought the zero-damage stairwell-top
            //    guard #23 at 11 u while the burst rifle #20 (59.0 in 7 hits, 4.4 u, visible) chewed
            //    health 68 → 0 in 107 ticks inside the `fighting=true` window.
            let cands = if self.dmg.hurting_first {
                let tick = self.total_ticks;
                let now = sim.drones.frame;
                cands.into_iter()
                    .filter(|&i| !self.dmg.cooldown.iter().any(|&(ci, until)| ci == i && tick < until)
                        // The lethality override: a guard the stand CAN see right now who dealt
                        // real damage over the last 240 frames is answered even inside his
                        // cooldown — the cooldown exists to stop re-standing on a guard the world
                        // trace cannot see, and visibility is re-tested here every pick. The d3
                        // death run's #20 sat cooled until tick 25061 (the failed fight on his
                        // mid-fight duck behind the mouth's wall band) while his bursts took
                        // health 68 → 5 — every retarget and re-pick skipped him BECAUSE he was
                        // cooled, and Bond died at 25116 with #20 at full hp.
                        || dmg_of(i) > 15.0)
                    .collect::<Vec<usize>>()
            } else { cands };
            // OPT-IN pre_throw_stands (the field doc): the spawner-released hostile inside
            // the pre-throw window pre-empts the plain pick — nearest first, VISIBLE only (an
            // unseen release is left to the plain path: a 60-tick unseen stand would burn the
            // window while the throw cycle runs), within the plain shooters' own 20 u reach
            // (every ledger fight sat at ≤ 14.4 u [L the aim-last-seven §3 ledger]). The
            // release detector is the first-seen-alive tick (`alive_seen`): it catches BOTH
            // the spawner templates (present written at the release) and the channel-held
            // placements (enabled flips at the release, no spawn stamp — the run-2 trace saw
            // no PRE-THROW for the west-room pair off the spawn_frame check alone).
            let newborn = if self.dmg.pre_throw_stands {
                let bp = sim.bond.pos;
                let now = self.total_ticks;
                let mut nb: Vec<(f32, usize)> = Vec::new();
                for (i, d) in sim.drones.list.iter().enumerate() {
                    if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { continue; }
                    let born = match self.dmg.alive_seen.iter().find(|e| e.0 == i) {
                        Some(e) => e.1,
                        None => { self.dmg.alive_seen.push((i, now)); now }
                    };
                    if now.saturating_sub(born) <= PRE_THROW_WINDOW_TICKS
                        && ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)).sqrt() <= 20.0 {
                        nb.push((((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)) as f32, i));
                    }
                }
                nb.sort_by(|a, c| a.0.total_cmp(&c.0));
                nb.into_iter().map(|(_, i)| i).find(|&i| DamageLog::visible(sim, i))
            } else { None };
            if trace && newborn.is_some() {
                let d = &sim.drones.list[newborn.unwrap()];
                eprintln!("[rf-trace] {label} tick {}: PRE-THROW fight on drone #{} (weapon {:#x}, hp {:.1}, at ({:.1},{:.1},{:.1}); bond gun {:#x} clip {} spare {})",
                    self.total_ticks, newborn.unwrap(), d.weapon, d.health,
                    d.pos[0], d.pos[1], d.pos[2], sim.gun.weapon, sim.gun.clip, sim.gun.spare);
            }
            let pick = newborn.or_else(|| if self.dmg.hurting_first || self.dmg.volley_carrier_first {
                cands.iter().copied()
                    .filter(|&i| DamageLog::visible(sim, i))
                    .min_by(|&a, &b| {
                        dmg_of(b).total_cmp(&dmg_of(a))
                            .then(dist_of(a).total_cmp(&dist_of(b)))
                    })
            } else if !self.dmg.lane_fights {
                match cands.iter().copied().find(|&i| DamageLog::visible(sim, i)) {
                    Some(i) => Some(i),
                    // The unseen-cover fallback, opt-in (`with_real_damage_unseen_fights`): the
                    // same hurting-first pick the lane fights run, so a desk guard the eye rays
                    // cannot reach is still answered (the creep opens the line).
                    None if UNSEEN_FIGHTS.with(|c| c.get()) => {
                        unseen_fight = true;
                        cands.iter().copied().find(|&i| dmg_of(i) > 0.0)
                            .or_else(|| cands.first().copied())
                    }
                    None => None,
                }
            } else {
                let vis = cands.iter().copied()
                    .filter(|&i| if self.dmg.eye_ray_stands { DamageLog::visible(sim, i) } else { DamageLog::visible_gun_rays(sim, i) })
                    .min_by(|&a, &b| {
                        dmg_of(b).total_cmp(&dmg_of(a))
                            .then(dist_of(a).total_cmp(&dist_of(b)))
                    });
                match vis {
                    Some(i) => Some(i),
                    // Nobody's boxes are reachable from here (a crouched guard behind the lobby
                    // desks: the centre rays stop at the desk top [L the NF_RF_GEOM dump — the
                    // Office-C_Obj9_computer mesh at (15.0,0.9,-2.1)..(15.7,1.6,-1.3) eats every
                    // ray]). A player under fire from unseen cover pushes TOWARD the muzzle until
                    // the line opens over the desk lip — the stand's creep does exactly that — so
                    // fight the nearest shooter who is actually hurting Bond instead of standing
                    // in the open chewing (the 07000012 bend death: 64 health gone while no fight
                    // ever started).
                    None => {
                        unseen_fight = true;
                        // Prefer whoever is actually hurting Bond; else the nearest shooter —
                        // `shooters` already guarantees he hit Bond in the last 240 frames or is
                        // firing at him this second (the 0.00 grazes still mean the muzzle is
                        // live and the pellets reach).
                        cands.iter().copied().find(|&i| dmg_of(i) > 0.0)
                            .or_else(|| cands.first().copied())
                    }
                }
            });
            self.dmg.unseen_fight = unseen_fight;
            if trace && pick.is_none() && !cands.is_empty() {
                eprintln!("[rf-trace] {label} tick {}: shooters {:?} none visible (bond {:?}, health {:.1}, clip {} spare {})",
                    self.total_ticks, cands, sim.bond.pos, sim.drones.health.health, sim.gun.clip, sim.gun.spare);
                // NF_RF_GEOM: full ray analysis of every shooter the world trace cannot see —
                // each hit box's centre ray (t_box, the world stop, the stopping owner) and the
                // guard's own muzzle→Bond path. Investigation tool (level-13-lobby-rifles).
                if std::env::var_os("NF_RF_GEOM").is_some() && self.total_ticks % 60 == 0 {
                    for &i in &cands { DamageLog::geom_dump(sim, i); }
                }
            }
            if std::env::var_os("NF_RF_TRACE").is_some() && (self.dmg.hurting_first || self.dmg.volley_carrier_first) {
                let dmgs: Vec<(usize, f32, f32)> = cands.iter().copied()
                    .map(|i| (i, dmg_of(i), dist_of(i))).collect();
                eprintln!("[hf] {label} tick {}: cands (i,dmg,dist) {dmgs:?} pick {pick:?} cooling {:?}", self.total_ticks,
                    self.dmg.cooldown.iter().filter(|&&(i, u)| u > self.total_ticks).map(|e| e.0).collect::<Vec<_>>());
            }
            picked = pick;
            pick
        };
        // NF_RF_GEOM at a fight START too: the pick's own rays plus every other candidate's —
        // the 2026-11-02 level-11 investigation needed the 1.6 u shooter's boxes while a farther
        // shooter won the pick. Diagnostic only (no behaviour change).
        if trace && picked.is_some() && std::env::var_os("NF_RF_GEOM").is_some() {
            let sim = self.scene.sim.as_ref().unwrap();
            for &i in self.dmg.shooters(sim).iter() { DamageLog::geom_dump(sim, i); }
        }
        let Some(di) = di else { return false };
        if self.dmg.cooldown.iter().any(|&(i, until)| i == di && self.total_ticks < until) { return false; }
        // tranq_fights: the dart is slow and the win is a sleep, not a wound — the caller's
        // "killed" bookkeeping below keys on stand_and_fire's return, which reads the dart
        // family, so no extra cooldown plumbing is needed here.
        // Only with a gun in hand: a player without rounds walks on (the 16 arrival carries
        // the fists alone — the weapon cycle stood Bond 730 ticks at the ramp foot under fire).
        // With `lane_fights`, dry NEAR a fight is not the old walk's dry: the guards Bond dropped litter the
        // floor with their guns, and a player picks one up. One resupply pass before giving up
        // (level-13-chain-difficulty: the fight at the security-room door ran on clip 2 spare 0
        // and the mezzanine sniper outlived it).
        let armed = |w: &Self| {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            let in_hand = (2..=0x47).contains(&g.weapon) && g.clip + g.spare > 0;
            let inv = &sim.drones.inventory;
            in_hand || (2..=0x47usize).any(|w| inv.owned.get(w).copied().unwrap_or(false) && {
                let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w);
                inv.clip.get(slot).copied().unwrap_or(0) > 0
                    || inv.carried.get(sim.weapons.ammo_type(w)).copied().unwrap_or(0) > 0
            })
        };
        // The unseen-fights mode shares the lane fights' resupply gates: a player who runs dry
        // picks up the guards' own drops instead of walking on (the 12 corridor: clip 1 spare 0
        // at the shotgun guard's door — the fights' silent `!armed` return left Bond standing
        // in the lane [L the walkreal_07000012_real_damage run, 2026-11-02]).
        let lane = self.dmg.lane_fights || UNSEEN_FIGHTS.with(|c| c.get());
        if !armed(self) {
            if lane && !self.dmg.resupplying {
                self.dmg.resupplying = true;
                self.resupply(label);
                self.dmg.resupplying = false;
            }
            if !armed(self) { return false; }
        } else if lane {
            // A nearly-dry gun starts a fight it cannot finish (the door fight ran on clip 2
            // spare 0 and the 24 hp sniper outlived it): restock first when the floor has drops.
            // INSIDE a resupply pass the fights still run (a pass with no answer leaves Bond
            // walking the nav legs under crossfire — the spawn-lobby chew on the skipped-fight
            // run): the dry stand's own resupply-after covers the empty gun.
            let low = { let sim = self.scene.sim.as_ref().unwrap(); sim.gun.clip + sim.gun.spare < 10 };
            if low && !self.dmg.resupplying {
                self.dmg.resupplying = true;
                self.resupply(label);
                self.dmg.resupplying = false;
            }
        }
        self.dmg.fighting = true;
        self.dmg.fights += 1;
        let h0 = self.scene.sim.as_ref().unwrap().drones.health.health;
        // An unseen fight gets a short cap: the creep either opens the line quickly or Bond is
        // standing in a lane he cannot answer — walk on (the leg's own return-fire gate re-fights
        // from the next spot).
        let cap = if self.dmg.unseen_fight { 300 } else { 900 };
        let mut killed = self.switch_to_gun(label) && self.stand_and_fire(di, cap);
        if !killed && self.dmg.finish_fights {
            let alive = {
                let sim = self.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                d.present && !d.corpse && d.health > 0.0
            };
            if alive {
                let h_mid = self.scene.sim.as_ref().unwrap().drones.health.health;
                killed = self.shoot_bounded(di, label, 1800);
                let sim = self.scene.sim.as_ref().unwrap();
                eprintln!("[real-damage] {label}: the determined pass on drone #{di} (killed {killed}) health {h_mid:.1} -> {:.1}", sim.drones.health.health);
            }
        }
        // The stand may have emptied the gun (stand_and_fire gives up when clip+spare hit 0):
        // restock from the floor drops so the NEXT fight does not run dry (same run, the sniper).
        if lane && !armed(self) && !self.dmg.resupplying {
            self.dmg.resupplying = true;
            self.resupply(label);
            self.dmg.resupplying = false;
        }
        // Level the view again before walking on (the walk's E presses and plate legs were
        // written for a level view; a fight leaves the pitch on the guard).
        for _ in 0..90 {
            let p = self.scene.sim.as_ref().unwrap().bond.pitch;
            if p.abs() < 0.01 { break; }
            self.tick(Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
        }
        self.dmg.fighting = false;
        // A guard the stand did not drop (out of sight, the cap) is left for 240 ticks: the
        // leg walks on instead of standing in the open re-fighting him.
        if !killed {
            // The failed-fight cooldown: master 240 ticks flat. The escalation (opt-in
            // `escalating_cooldowns`) doubles the wait per further failure on the same drone —
            // the re-fights of a geometry-blocked shooter are the exposure that killed the
            // first d2 run (see the field doc).
            let fails = self.dmg.fight_fails.iter().filter(|&&(i, _)| i == di).count() as u32;
            self.dmg.fight_fails.push((di, self.total_ticks));
            let wait = if self.dmg.escalating_cooldowns { 240u32 << fails.min(3) } else { 240 };
            let until = self.total_ticks + wait;
            self.dmg.cooldown.retain(|e| e.0 != di);
            self.dmg.cooldown.push((di, until));
        }
        let sim = self.scene.sim.as_ref().unwrap();
        eprintln!("[real-damage] {label}: returned fire on drone #{di} (killed {killed}) at tick {}, health {h0:.1} -> {:.1}",
            self.total_ticks, sim.drones.health.health);
        true
    }

    /// Aim at drone `di` (his exposed part, `DamageLog::exposed_part`) with the view and tap the
    /// trigger when the view is on him; reload on an empty clip. Bond does not move: a player
    /// returning fire on a ledge or a stair does not sidestep off it (the walker's `shoot`
    /// flanks, which walked Bond off the 05 tower lane and the 14 stairwell). Stops when the
    /// guard dies (true), is out of sight for 60 ticks, takes no round for 300, the gun is dry,
    /// or after `cap` ticks. A
    /// tap whose line passes within 0.6 u of another live drone short of the target is held
    /// (the game's bullets hit any drone — the 03 servants stand among the guards).
    pub fn stand_and_fire(&mut self, di: usize, cap: u32) -> bool {
        let trace = std::env::var_os("NF_RF_TRACE").is_some();
        let mut di = di;
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
        let mut aim: Option<[f32; 3]> = None;
        let mut hp0 = self.scene.sim.as_ref().unwrap().drones.list[di].health;
        let mut last_hit = 0u32;
        let mut dry = 0u32;
        let mut blocked_runs = 0u32;
        // switch_dry: the cycle press + draw wait (see the field doc).
        let mut cycle_wait = 0u32;
        // sniper_stands: the still-gate state (the target's root, per tick — see the field doc).
        let mut still = 0u32;
        let mut gate_pos = [0.0f32; 3];
        let mut have_gate_pos = false;
        // sniper_stands: the raise queues ONLY on the aim-PRESSED edge from the idle machine
        // [M 0x1a24c8:113-152] — a press fired into the switch anim or a live fire script never
        // queues it (the level-0c-yard-climb-real-health-4 measurement), so a player lets the
        // gun settle before the first press; the loop's aim hold then runs the raise.
        // pre_throw_stands runs the same settle: a tap fired into a live switch anim or fire
        // script is a wasted round from the pre-throw window (the walk legs hold the gun idle,
        // so the loop breaks on its first tick there).
        if self.dmg.sniper_stands || self.dmg.pre_throw_stands {
            for _ in 0..90 {
                let s = self.scene.sim.as_ref().unwrap();
                if s.gun.switch_state == 0 && !s.gun.fire_state { break; }
                self.tick(Input::default(), false);
            }
        }
        for t in 0..cap {
            if self.dmg.switch_dry {
                let clip_dry = { let s = self.scene.sim.as_ref().unwrap(); s.gun.clip == 0 && s.gun.spare == 0 };
                if clip_dry {
                    if cycle_wait >= 80 { return false; }
                    let press = cycle_wait == 0;
                    cycle_wait += 1;
                    self.tick(Input { weapon_cycle: i8::from(press), ..Default::default() }, false);
                    continue;
                }
                cycle_wait = 0;
            }
            let (input, fire, reload) = {
                let sim = self.scene.sim.as_ref().unwrap();
                // RETARGET (opt-in `hurting_first`, the 07000008 Agent sisters): every 30 ticks the
                // fight re-reads who is actually hurting Bond and switches to the worst VISIBLE,
                // off-cooldown shooter when his 240-frame damage beats the current target's by
                // more than 3.0. The pick's blind spot is the fight's first seconds: the d3 run's
                // fight#23 opened at tick 24850 when the zero-damage pistol guard #23 (14 u) was
                // the ONLY candidate, and ten ticks later the burst rifle #20 (4-7 u) opened up —
                // 59.0 in 7 hits while the stand stayed on #23 (his ledger total: 0.0) and chewed
                // health 68 → 0 (death tick 25116). A player under new, closer fire switches
                // targets; the shared fight never did. The switch resets the fight's own progress
                // trackers (hp0/last_hit/aim/unseen) for the new target.
                if (self.dmg.hurting_first || self.dmg.volley_carrier_first) && t > 0 && t % 30 == 0 {
                    let now = sim.drones.frame;
                    let dmg_of = |i: usize| -> f32 {
                        sim.drones.player_hits.iter().rev()
                            .take_while(|h| now.saturating_sub(h.0) <= 240)
                            .filter(|h| h.1 == i)
                            .map(|h| h.2).sum::<f32>()
                    };
                    let cur = dmg_of(di);
                    let better = self.dmg.shooters(sim).into_iter()
                        .filter(|&i| i != di)
                        .filter(|&i| DamageLog::visible(sim, i))
                        // The cooldown bends to active lethality (see the pick-side note): a
                        // guard visibly dealing > 15 over the window is switchable even inside
                        // his cooldown — the cooldown guards against INVISIBLE re-stands.
                        .filter(|&i| !self.dmg.cooldown.iter().any(|&(ci, until)| ci == i && self.total_ticks < until)
                            || dmg_of(i) > 15.0)
                        .max_by(|&a, &b| dmg_of(a).total_cmp(&dmg_of(b))
                            .then(b.cmp(&a)));
                    if let Some(i) = better {
                        let nd = dmg_of(i);
                        if nd > cur + 3.0 {
                            if trace {
                                eprintln!("[rf-trace] fight#{di} t{t}: retarget to #{i} (dmg {nd:.1} vs {cur:.1})");
                            }
                            di = i;
                            hp0 = sim.drones.list[di].health;
                            last_hit = t;
                            aim = None;
                            unseen = 0;
                        }
                    }
                }
                let d = &sim.drones.list[di];
                if d.corpse || d.health <= 0.0 { return true; }
                // tranq_fights: the K5's win is the sleep itself — the fall 0x50 and the asleep
                // loop 0x51 [M 0x163430/0x163570]; the Recover state 0x52 is the wake, and the
                // fight stays on him (the next dart re-enters the impact [M 0x163700 case 0x19]).
                if self.dmg.tranq_fights && nf_game::drone::Drone::dart_asleep_state(d.state) { return true; }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 { return false; }
                // sniper_stands: the still gate — root motion only (the box sway rides the anim;
                // the root is what carries the chest out of the round's path mid-flight
                // [M Bullet_update 0x12e7b0 — the round steps its speed per tick]). The tap
                // waits for STILL_NEED still ticks (the level file's own still-gate note;
                // walkreal0c.rs's yard_stand_aimed is the proof).
                if self.dmg.sniper_stands {
                    let moved = have_gate_pos && ((d.pos[0] - gate_pos[0]).powi(2)
                        + (d.pos[1] - gate_pos[1]).powi(2) + (d.pos[2] - gate_pos[2]).powi(2)).sqrt() > 0.03;
                    have_gate_pos = true;
                    gate_pos = d.pos;
                    still = if moved { 0 } else { still.saturating_add(1) };
                }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                // Re-pick the exposed part every 10 ticks (the guard animates, cover buries a part).
                // The ray set follows the fight's pick: sniper stands use the strict muzzle
                // stand-ins (the yard stand's idiom — the round leaves low-right of the eye and
                // the predictor gap is the measured miss), lane fights use the gun stand-ins, the
                // shared walks the plain eye ray (master behaviour; level-13-lobby-rifles-2).
                if t % 10 == 0 { aim = if self.dmg.sniper_stands || (self.dmg.lane_fights && !self.dmg.eye_ray_stands) { DamageLog::exposed_part_gun_rays(sim, di) } else { DamageLog::exposed_part(sim, di) }; }
                if aim.is_none() {
                    unseen += 1;
                    // NF_RF_GEOM: name the world stop that buries the target on the stand's first
                    // blind tick (the 08 mouth investigation — the diagnostic return_fire already
                    // runs, wired to the direct stands too).
                    if unseen == 1 && std::env::var_os("NF_RF_GEOM").is_some() { DamageLog::geom_dump(sim, di); }
                    if unseen > if self.dmg.lane_fights { 120 } else { 60 } { return false; }
                } else { unseen = 0; }
                // No round landed for 300 ticks: walk on (a guard in cover the stand cannot reach;
                // the 13 lobby stand took 40 health that way). tranq_fights: the dart's HitDamage
                // is never called [M 0x13fff0:269-306] — progress is a dart LANDING (the target
                // entering the dart family), not a health drop.
                if d.health < hp0 { hp0 = d.health; last_hit = t; dry = 0; }
                if self.dmg.tranq_fights && nf_game::drone::Drone::dart_family_state(d.state) { last_hit = t; dry = 0; }
                if t - last_hit > 300 && !self.dmg.dry_walk_on { return false; }
                if self.dmg.dry_taps.is_some_and(|n| dry >= n) && t - last_tap > 20 { return false; }
                let at = aim.unwrap_or(torso);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let err = { let mut a = dx.atan2(dz) - sim.bond.yaw;
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let perr = (dy / h).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
                // sniper_stands: with the aim bit held the fired round obeys the VIEW PLUS the
                // eased cursor offsets [M scene.rs free-aim], so the drive targets the CURSOR
                // position that puts the effective aim on the part and the gate reads the
                // EFFECTIVE error (the yard stand's measured rounds steered −12° off on the
                // view-only converge — walkreal0c.rs's a04k trace). The drive uses the UNCLAMPED
                // target: at the window edge the overflow becomes the free-aim spill and turns
                // the VIEW (the clamped drive fed zero there and the aim stalled — a04m).
                let (drive_turn, drive_look, gate_err, gate_perr) = if self.dmg.sniper_stands {
                    let desired_yaw = dx.atan2(dz);
                    let desired_pitch = (dy / h).atan();
                    let yaw_err = { let mut a = desired_yaw - sim.bond.yaw;
                        while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                        while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                    let pitch_err = desired_pitch - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
                    let cx_want_raw = -yaw_err / 0.76966;
                    let cy_want_raw = pitch_err / 0.57724;
                    let cur = sim.cursor;
                    let turn = ((cur[0] - cx_want_raw) * 0.76966).clamp(-0.08, 0.08);
                    let look = ((cy_want_raw - cur[1]) * 0.57724 / std::f32::consts::FRAC_PI_2).clamp(-0.08, 0.08);
                    let aoff = sim.drones.aim_offset;
                    let eff = { let mut a = yaw_err - aoff[0];
                        while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                        while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                    (turn, look, eff, pitch_err - aoff[1])
                } else {
                    (err.clamp(-0.08, 0.08), perr.clamp(-0.08, 0.08), err, perr)
                };
                // Another drone on the line, short of the target: hold the tap. With the opt-in
                // `shoot_blockers`, a blocker that is itself a LIVE HOSTILE (team 1, enabled,
                // alive) becomes the new target — a player whose line a guard walks into shoots
                // that guard (the 07000011 flight-B stand held fire ~143 ticks while the blocker
                // at 1.6 u landed 15 rounds for 51 health [L the 2026-11-02 run]; see
                // DamageLog::shoot_blockers). Servant/ally walks and the master behaviour keep
                // the hold (the flag is off).
                let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                let u = [dx / len, dy / len, dz / len];
                let mut blocker: Option<usize> = None;
                let blocked = sim.drones.list.iter().enumerate().any(|(k, o)| {
                    if k == di || !o.present || o.corpse || o.health <= 0.0 { return false; }
                    // fire_through_disabled (opt-in): the game's own collide drops Enable(0)
                    // objects before the hit test [M Collide_PickObj 0x1eae80] — a disabled
                    // drone never stops a round, so it never blocks the line.
                    if self.dmg.fire_through_disabled && !o.enabled { return false; }
                    let c = [o.pos[0] - eye[0], o.pos[1] + 0.6 - eye[1], o.pos[2] - eye[2]];
                    let s = c[0] * u[0] + c[1] * u[1] + c[2] * u[2];
                    if s <= 0.0 || s >= len - 0.3 { return false; }
                    let q = [c[0] - u[0] * s, c[1] - u[1] * s, c[2] - u[2] * s];
                    let on_line = (q[0] * q[0] + q[2] * q[2]).sqrt() < 0.6 && q[1].abs() < 1.2;
                    if on_line && blocker.is_none() { blocker = Some(k); }
                    on_line
                });
                if blocked && self.dmg.shoot_blockers {
                    if let Some(k) = blocker {
                        let o = &sim.drones.list[k];
                        if o.team == 1 && o.enabled && o.present && !o.corpse && o.health > 0.0 && k != di {
                            if trace { eprintln!("[rf-trace] fight#{di} t{t}: line blocked by live hostile #{k} (hp {:.1}, {:.1} u) — retarget",
                                o.health, ((o.pos[0] - b[0]).powi(2) + (o.pos[2] - b[2]).powi(2)).sqrt()); }
                            di = k;
                            hp0 = o.health;
                            last_hit = t;
                            aim = None;
                            unseen = 0;
                            have_gate_pos = false;
                            still = 0;
                        }
                    }
                }
                // fire_shield (OPT-IN, see the field): the shielded drone inside the tap's cone —
                // the aim line extended to his depth, widened by 0.9 + 0.05·ds (and the spread
                // cap 0.2464·ds with bloom, [M bullet::player_spread]) — holds the tap.
                let shield_line = self.dmg.fire_shield.is_some_and(|(s, bloom)| {
                    let Some(sd) = sim.drones.list.get(s) else { return false };
                    if s == di || sd.corpse || sd.health <= 0.0 { return false; }
                    let ds = ((sd.pos[0] - eye[0]).powi(2) + (sd.pos[2] - eye[2]).powi(2)).sqrt();
                    let dtg = (dx * dx + dz * dz).sqrt().max(1e-3);
                    if ds <= 1.0 || (sd.pos[0] - eye[0]) * dx + (sd.pos[2] - eye[2]) * dz <= 0.0 { return false; }
                    let k = ds / dtg;
                    let (ex, ey, ez) = (eye[0] + dx * k, eye[1] + dy * k, eye[2] + dz * k);
                    let miss2 = (ex - sd.pos[0]).powi(2) + (ez - sd.pos[2]).powi(2);
                    let reach = 0.9 + 0.05 * ds;
                    let reach = if bloom { reach.max(0.2464 * ds) } else { reach };
                    miss2 < reach * reach && (sd.pos[1] - 1.2) < ey && ey < (sd.pos[1] + 2.2)
                });
                if shield_line && trace && t % 30 == 0 { eprintln!("[rf-trace] fight#{di} t{t}: the shielded drone is in the tap cone — holding"); }
                // The friendly-fire shield (`Walker::shield`, the 01 escort): a round that would
                // cross the shielded contact past the target is not taken here either — the
                // return-fire stand shot the contact standing behind guard #3 at 12 u (by the
                // player, "Killed Civilians") once the guards' aim stands stopped drifting
                // (pretrans-arms-2: the +0x38 root zero). A player holds the tap.
                let blocked = blocked || self.return_fire_shield_blocks(sim, eye, at, di);
                let reload = sim.gun.clip == 0;
                // aimed_fights (OPT-IN): the tap gate TIGHTENS with the aim held — the stand's
                // plain tolerances (0.03 rad yaw / 0.04 norm pitch = ±1.05 u / ±2.2 u at the deck
                // sentry's 35 u) are wider than the chest box, and the zero-spread aimed round
                // [M 0x12c5a0:139-170] goes exactly along the view — the diagnostic trace hit with
                // an exact view and the loose stand dried clips on the same box. 0.01 / 0.01 =
                // ±0.35 u / ±0.55 u at 35 u — inside the box, still reachable by the ±0.08/tick
                // converge.
                // pre_throw_stands: the tap gated on the exposed part at the same 0.01-rad
                // gate (the 0c pattern minus the aim bit — the walk's P2K/AR are not 0x400000
                // records, and the aim-bit hold would hand the drive to the cursor law
                // [M scene.rs free-aim, the precise_fights measurement]).
                let (err_gate, perr_gate) = if self.dmg.aimed_fights || self.dmg.precise_fights || self.dmg.sniper_stands || self.dmg.pre_throw_stands { (0.01, 0.01) } else { (0.03, 0.04) };
                let fire = !reload && unseen == 0 && !blocked && !shield_line && gate_err.abs() < err_gate && gate_perr.abs() < perr_gate
                    && t.saturating_sub(last_tap) >= 8
                    // sniper_stands: the aim BIT (the raise finished — a tap before it rides the
                    // hip cone [M Player_SetWeaponAnimObj case 0xf, the level-0c-yard-climb-
                    // real-health-4 shot trace]) and the STILL gate (the root held still — the
                    // wiggle-step beats the flight).
                    && (!self.dmg.sniper_stands || (sim.aiming && still >= 18));
                // NF_RF_TAPS: every tap's aim state (the 2026-11-03 level-11 investigation needed
                // the per-tap error against the recorded hits — the t%60 lines only sample).
                if fire && std::env::var_os("NF_RF_TAPS").is_some() {
                    eprintln!("[rf-tap] fight#{di} t{t}: err {err:.4} perr {perr:.4} aim {} clip {} spare {}",
                        aim.map(|a| (a[0], a[2])).map_or("none".into(), |a| format!("({:.2},{:.2})", a.0, a.1)),
                        sim.gun.clip, sim.gun.spare);
                }
                let crowd = self.dmg.back_off.is_some_and(|r| sim.drones.list.iter().any(|o| o.team == 1 && o.enabled && o.present
                    && !o.corpse && o.health > 0.0 && ((o.pos[0] - b[0]).powi(2) + (o.pos[2] - b[2]).powi(2)).sqrt() < r));
                // The unseen-cover creep: a player pushing toward a guard crouched behind a desk
                // walks up to the lip until his own eye line clears it over the top (the ray to
                // the boxes steepens as the stand-off shrinks). Creep in bursts along the aim,
                // never while a guard stands inside the back-off range. OPT-IN (`lane_fights`,
                // the 07000013 walk): the shared walks' stands never move forward (master
                // behaviour; level-13-lobby-rifles-2).
                let creep = (self.dmg.lane_fights || self.dmg.unseen_fight) && aim.is_none() && !crowd
                    && (self.dmg.back_off.is_some() || self.dmg.unseen_fight) && unseen % 12 < 7;
                if trace && t % 60 == 0 {
                    eprintln!("[rf-trace] fight#{di} t{t}: tgt hp {:.1} at ({:.1},{:.1}) bond ({:.1},{:.1}) hp {:.1} aim {:?} blocked {blocked} err {err:.3} perr {perr:.3} clip {} spare {} creep {creep}",
                        sim.drones.list[di].health, sim.drones.list[di].pos[0], sim.drones.list[di].pos[2],
                        b[0], b[2], sim.drones.health.health, aim.map(|a| [a[0], a[2]]), sim.gun.clip, sim.gun.spare);
                }
                // aimed_fights (OPT-IN, the 0700000c deck-sentry stand): the aim button HELD every
                // tick — the raise completes, the aim bit stays set, and a 0x400000 record (the
                // snipers [L ee_start_05.bin @ 0x2c10b8 flags68 0x408050]) fires its zero spread
                // [M Bullet_init 0x12c5a0:139-170]. Off (every other walk) the bit never sets and
                // the pull keeps the record's hip cone [M 0x12c5a0:139-170 mul = 1].
                (Input { turn: drive_turn, look: drive_look, forward: if crowd { -1.0 } else if creep { 0.5 } else { 0.0 }, aiming: self.dmg.aimed_fights || self.dmg.sniper_stands, ..Default::default() }, fire, reload)
            };
            if reload {
                self.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false);
                self.total_ticks += 1;
                self.after_step();
                for _ in 0..60 { self.tick(Input::default(), false); }
            } else if fire {
                last_tap = t;
                dry += 1;
                self.tick_fire(input);
            } else {
                self.tick(input, false);
            }
        }
        false
    }
}
