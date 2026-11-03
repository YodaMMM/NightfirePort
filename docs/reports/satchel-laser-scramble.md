# satchel-laser-scramble — the player's placed satchel under the laser watch

## What was wrong

Lasering the **player-placed** satchel charge showed neither the burn sparks nor the scrambled
countdown: the sparks never spawned and the digits kept counting normally. The merged
satchel-laser work (5d9c783d, the 4a lift charge) pinned the whole burn path for the **shooters'**
charges — but those are drone bullets (`Drones::bullets`). The player's own fired satchel rides the
**thrown-grenade pool** (`scene.rs`'s shot block spawns it as a `crate::grenade::Grenade`),
and `Drones::update_player_bullets`'s 0x4E/0x4F burn walk only scanned the landed bullets —
the player's placed satchel was invisible to it. The `Grenade` also had no burn state at all:
no BU+0x102 counter, no burned-out arm, so even a correct walk could not have scrambled anything.

## The game's path, cited

- `Bullet_init` 0x12c5a0:199-211 [M, work/decomp/ACTION.ELF/0012c5a0_Bullet_init…c]: the 0x3b
  family's fuse `((id − 0x3b)·5 + 5)·60` (300 for the fired 0x3b) **and BU+0x102 = 100** — the
  player's thrown satchel runs the same `Bullet_init`, so its counter starts at 100 too.
- `Bullet_DoTrails` 0x12d890's 0x3b tail [M, work/decomp/ACTION.ELF/0012d890_Bullet_DoTrails…c]:
  - the digits are taken from the fuse FIRST (`sec = fuse·0.016666668`), then — **if
    `BU+0x102 < 1`** — the tail redraws the fuse to `Rand_Rand(0x1518) + 0x3c` EVERY tick and
    jumps past the beep and the contact walk (`goto LAB_0012e41c`, landing at the digit-object
    spawn): the digits show a fresh random number every tick, no more sparks, and the fuse never
    reaches < 0 — the charge never explodes.
  - otherwise the beep (0x447 on the quarter-second change) and the contact walk: each class-5
    contact with weapon id 0x4E takes FRAME_RATE_MUL (1) off BU+0x102, 0x4F takes 10, and every
    contact spawns `Emitter_Create(0xc00000a, entry+0x40, entry+0x10, …)` — the burn sparks at the
    contact's hit point; the contact that drives the counter to ≤ 0 plays `Sound_Play3D(0x5dd)`
    at the charge.
- The walk burns the charge in **every state < 2** (the family gate is the record's +0x02, not a
  landed flag) — the port's grenade pass mirrors that.

## The port changes

- `crates/nf-game/src/grenade.rs`: `Grenade` gains `burn: i16` (BU+0x102) and `defused: bool`
  (fields on their own lines at the struct end). The 0x3b tail now writes the digits every tick
  **before** the burned-out arm (the old code returned early on a beep tick, so the digits lagged
  one tick exactly on beep ticks — the game spawns them every tick), then the burned-out arm
  redraws the fuse to `Rand_Rand(0x1518) + 0x3c` every tick via the shared RNG (one draw per
  burned-out charge per tick, the game's own draw) and returns before the beep. The two explosion
  gates (the resting arm and the shared fuse arm) state the game's outcome directly: no explosion
  while `defused`. 27 existing `Grenade` literals across the tests got the two new fields
  (`burn: 0` except the satchel helpers, which carry 100).
- `crates/nf-game/src/drone.rs`: the laser-burn walk (`update_player_bullets`, weapon 0x4E/0x4F)
  now also scans `self.grenades` — the same decrements, the same `spawn_impact(SATCHEL_BURN_SPARKS)`
  per contact, the same 0x5dd fizzle, the same 0.5 contact-sphere stand-in [S — the charge's exact
  glist bounds are not pinned; see the bullet arm's comment]. `throw_grenade`'s literal initializes
  the counter (this site's `weapon_type` IS the record's +0x02 base).
- `crates/nf-view/src/scene.rs` (the SP player's fired gadget) and
  `crates/nf-game/src/mp/combat.rs` (the MP throw): their literals initialize
  `burn` from the record's base (100 for the 0x3b family), `defused: false`.
- Renderer: no changes needed — the satchel digits already draw from `Grenade.phase.digits`
  (scene.rs kinds 4/5, camera-beam-satchel-draw) and the sparks already draw through the existing
  fx emitter path, so the sim-side fix surfaces visually on its own.

## Verification

- `nf-game --lib`: **656 passed, 0 failed** (the new
  `a_burned_out_satchel_redraws_its_fuse_every_tick_scrambles_the_digits_and_never_explodes`
  unit test pins the redraw range [60, 5459], the integral fuse, the digit pattern = the previous
  redraw after the decrement, the never-explodes outcome and the silent tail — no 0x447 beeps).
- **The new real-input test** `satchel_laser_scramble_4a_live`
  (`crates/nf-view/tests/satchel_laser_scramble_4a_live.rs`): **passed**. It rides 0700004a from
  the load on real input only (no bond writes after the load): burns shooter #286's charge on the
  cab, burns #287's charge when it drops, rides to the park, skips the lobby scene the
  Walker-default way (`scripts.skip_request` — the game's own skip action [M nis.rs,
  Input_Action(-1, 0x19, 1)]), then — via the panel give (the real give path) — draws the satchel,
  throws it at the parked platform, waits for the stick, draws the laser watch and holds the beam.
  It asserts: `burn == 100` and no sparks before the beam; the counter loses exactly 1 per beam
  tick; **sparks (emitter def 0xc00000a) at the charge on every contact tick**; exactly **100
  beam ticks** to the burn-out; the 0x5dd fizzle sound; no new sparks after the burn-out with the
  beam still held; the digits scramble (≥ 6 distinct readings over 12 ticks, each tick's digits =
  the previous redraw after the decrement, the redrawn fuse integral in [60, 5459]); the charge
  survives 400+ ticks past its original fuse, unexploded; Bond alive, no death channel, the
  mission not failed.
- The focused satchel/laser batch, all green: `satchel_laser_4a_live` 2/2, `satchel_digits_4a_live`
  2/2, `interactables_0700004a_live` 6/6, `fix_satchel_tripbomb_live` 6/6,
  `thrown_satchel_pose_live` 3/3, `camera_beam_satchel_wreck_live` 2/2,
  `walkreal_0700004a_satchel_live` 2/2, `sound_coverage_live` 5/5.
- `nf-formats` + `nf-audio` with `--test-threads=1`: green except exactly the **3 `disc_golden`
  failures the slim ISO allows** (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`).

## What's left

- **The full suite did not fit the window.** After the last mechanical fix (27 test-literal
  `Grenade` initializations gaining `burn`/`defused`), the rebuild of every touched test binary
  was still running when the 01:05 UTC stop-new-work mark hit. `nf-game`'s integration binaries
  and `nf-view`'s other live tests are therefore **not run** in this session. The next session
  should run the full suite command from the rules (`NF_ISO=$NF_ISO cargo test --offline
  --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`) on this
  branch — the risk is limited to the grenade-adjacent tests (blast/explode/rattle/walk), whose
  literals were only given the two new fields (`burn: 0` — no behavior change for non-0x3b
  types), and the satchel ones, which ran green above.
- `crates/nf-view/tests/common/` was **not** touched — no walkreal sweep is required by the
  task rules (and `walkreal_0700004a_satchel_live` ran green anyway).

## Render check for the main agent (the visible work)

The sparks and the scrambled digits are player-visible on the parked platform at the bottom of
the 4a lift shaft, exactly where the new test performs them:

- The placed satchel on the platform floor with its two green LED digits counting down from 05
  (the 300-tick fuse).
- The same spot with the watch beam on the charge: the burn sparks (`Emitter_Create(0xc00000a)`)
  at the beam's contact point on the charge, one burst per beam tick while the beam holds.
- After the 100th contact tick: the digits **scramble** — a fresh random value every tick (the
  fuse redraws to 0x3c + Rand_Rand(0x1518)), the charge never explodes, the LED keeps flickering
  random readings for as long as the player watches.

(The test drives these states in-sim; the pose is the test's own park pose — the platform parks
≈147.6 below the board root at the shaft bottom, the lobby side. A `--bond`-pose render aid like
the older NF_LASER_AT can be reused against the player's grenade the same way.)

## Live checks for the main agent

None requested — the decompile's burn arm is unambiguous (the goto structure settles the digit
ordering, and the 4a shooter-charge path was already live-pinned by the merged satchel-laser
round). One optional confirmation, if a PCSX2 session happens anyway: place a satchel
(give 0x3b, throw at a wall), laser it for ~1.7 s, and watch the LED digits — the game should
show fresh random readings every tick after the burn-out, and the sparks should sit at the beam's
contact point.

## Usage

not visible to me — this sandbox exposes no token, cost or context-window figures for the session.
