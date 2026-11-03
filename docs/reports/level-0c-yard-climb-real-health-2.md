# level-0c-yard-climb-real-health-2 — the aimed-yard attempt, the measurements, and what is left

Branch `s/level-0c-yard-climb-real-health-2`, cut from origin/master 63e76f06 (the task's premise held:
`s/player-upward-shot` was already fully merged into master — `DamageLog::aimed_fights` and the two
`upward_shot_0700000c_live` proofs were on the cut point, so no merge was needed).

## What is done

**The opt-in aimed machinery is in and verified harmless to every other test.** All in
`crates/nf-view/tests/common/walkreal0c.rs` (included only by the three 0c test binaries) + the sister
test in `walkreal_0700000c_live.rs`:

- `with_yard_aimed_stands(f)` — a thread-local opt-in (libtest isolates threads), mirroring the
  `damage_log.rs` wrapper pattern. When set, `walk_spawn_to_the_yard_ladder_base` sets
  `w.dmg.aimed_fights = true` on its walker, so **the whole walk's return-fire fights hold the aim
  button** — the driver's "opt the 0c real-health walk into the aimed fights". Flag off ⇒ byte-identical
  master behaviour.
- `yard_stand_aimed(w, di, label, cap)` — one aimed-0x1e stand: the direct number-key switch through the
  real input path [M Player_WeaponChange mode 0], `stand_and_fire` under the aimed flag, the view levelled
  after (the fights' own idiom). Never writes health, armour or ammo.
- `yard_sniper_stands(w, label, max)` — the deck-emplacement sweep (the PAIR #18/#36 at (36.4, 12.06,
  -208.8) [L the predecessor census], re-running on the survivor until both drop) for the yard-floor
  labels, then the recent-hitter pick, then (pre-climb) the suppression pass over every other alive
  weapon-0x1e within 48 u (the patroller #15 and the walkway #1 acquire Bond late and never pass the
  recent-hit window). Call sites: **at-the-door** (after the secret door, before the yard crossing),
  **pre-climb** (the ladder base, before the climb), **ladder-top** (after the mantle, max 3).
- The sister `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage` runs the whole route
  on real input under the opt-in — un-ignored mid-session, re-ignored at the end with the measured
  reason (below): it must not ship red.

## How it was verified

- **The whole-walk aimed fights work**: the same run that died at the yard in the hip configuration
  (run 1: 71 hits for 100, dead at tick 9115) now kills the en-route fights (#33/#27/#24/#20 killed) and
  takes 54 hits instead of 71 — the aimed gate change is real on the whole route.
- **The emplacement sweep works on the reachable twin**: the at-the-door sweep killed #36 in one aimed
  round (run 5/6: the alive list after the sweep is `[18]`).
- **The upward-shot proof stays green**: `upward_shot_0700000c_live` 2 passed (the hip dry-clip pin + the
  aimed kill) on this branch's tree — the flag-off paths keep the pinned behaviour.
- Runs (all on real input, no teleports; `NF_RF_TRACE` for the per-tick stand state):
  - run 1 (per-stand aimed only): dead at tick 9115 at the ladder base; the auto-fight burned the
    sniper's clip hip-first, my stand started mid-reload. → led to the whole-walk opt-in.
  - run 4 (whole-walk aimed): dead at tick 9522; the trace shows my own converged aimed rounds (err
    0.000, perr 0.000, blocked false) at #18 never moving its hp 10.0 while #36 stood beside it.
  - run 5/6 (door sweep + suppression): #36 dies to the sweep; #18's hp never moves from the door spot
    or the ladder base (7+ converged aimed rounds across two spots); Bond dies at tick 9877 at the base
    with the fatal graze from the patroller #15 (18.4, 12.2, -216.7) at 1.6 hp.

## The two measured walls (what is left)

1. **#18 is unreachable from the yard floor.** The twin #36's body — alive, and then its corpse — meets
   every round first [M Drone_CollisionHandler 0x140968: the game's bullets hit any drone], and the
   stand's own blocked-check deliberately ignores drones within 0.3 u of the target's distance. The
   master proof landed because its walk matched only ONE live twin at the spot. A real player at these
   two spots cannot snipe #18 either; the deck fight needs a spot whose angle clears the twin's body, or
   the corpse must be considered.
2. **The health budget through the yard is negative.** Bond arrives at the yard at ~33.6 hp (54 hits for
   100 en route — the aimed fights' stands vs the patrolling guards trade: #30/#37 ran 300-tick "killed
   false" stands), the crossing + base stands cost 2 × -16 (#18's beat), and #15's late-acquiring beat
   lands the fatal graze. The predecessor's measurement stands: the climb itself is survivable at ~19.8
   (Bond mantled at 3.8); the kill is #1's close beat within ~110 ticks of the mantle. Next lever: cut
   the en-route fight damage (the aimed 0.01 gate vs moving guards lengthens the stands), or suppress
   #15/#1 before the crossing, not just before the climb.

The ignore reason on the sister carries all of this with the run evidence.

## Runs performed on this branch's tree (all green)

- `walkreal_0700000c_live`: top-up walk + ladder495 **2 passed** (the sister ignored).
- `upward_shot_0700000c_live` **2 passed**; `upward_shot_trace_live` **1 passed**.
- The climb/mantle group: `fix_ladder_live` **4 passed 1 ignored**, `fix_ladder2_live` **4 passed**,
  `fix_ladder_water_live` **4 passed**, `ladder_02_live` **3 passed**, `ladder_underwater_02_live`
  **6 passed**.
- `nf-game`: **655 passed, 0 failed, 5 ignored**.
- The fast runner (`tools/test/suite.py`) did not run this session: the sandbox was reset (setup from
  scratch: clone, work-slim + ISO, rustup), and the window went into the yard measurements above; the
  crates touched (nf-view's 0c binaries + nf-game) were run explicitly as listed.

## Live checks for the main agent

1. From the yard floor at the ladder base (11.6, 1.2, -186.6) on a fresh boot of 0700000c: does a real
   aimed sniper round (0x1e, aim held) at the deck emplacement (36.4, 12.06, -208.8) ever damage BOTH
   sentries, or does the front twin's body (and then its corpse) eat every round the way the port does?
   Expected: the game shows the same corpse-shield behaviour [M Drone_CollisionHandler 0x140968].
2. Does the game's own #15 patroller (18.4, 12.2, -216.7 area, weapon 0x1e) open fire on Bond during the
   yard crossing on real health, and does its beat stop while being fired at (the suppression the
   predecessor measured [L])?
3. Where does the game expect the deck fight from — is there a spot on the yard floor/strip where the
   back twin's chest box clears the front twin's body? A screenshot from (12.7, 1.1, -186.4) looking at
   (36.4, 12.1, -208.8) would settle the angle.

## Usage

not visible to me (no token, credit or context-window figures are exposed to this environment; the only
counters visible are process wall-clock times in the tool outputs above).
