## Your task: `hurt-volume-flight-2`, the Icicles on 07000046 fall and hurt through a real shot

Branch from origin/s/hurt-volume-flight, merge current origin/master, and read the report in the queue's blocked/30-hurt-volume-flight.md. The state-1 flight is implemented and unit-pinned. Only the live pin's aim harness failed to converge, so the test's shooting is what is left. Fix the aim the way the report's harness notes say, using the real input path, then pin through the real level: an Icicle set off by a shot follows its path at the game's pace, Bond under it takes the damage, and clear of it he does not. Remove the ignore. Keep the rate-90 wrap question as a live check for the main agent. Run the hurt binaries, the 07000046 binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/hurt-volume-flight-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T01:24Z

---
## Report (done, 2026-11-08T03:05Z)


---
## Report from the worker (done, 2026-11-08T03:05Z)

# hurt-volume-flight-2 — report

Branch `s/hurt-volume-flight-2` (from `origin/s/hurt-volume-flight`, merged `origin/master` at
39cf38151). Task: the 07000046 Icicle pin's shooting — the state-1 flight was implemented and
unit-pinned by hurt-volume-flight, but the live pin's aim harness could not drive a real bullet
onto the icicle. Fix the aim the harness notes' way, then pin through the real level: a shot
starts the flight, it follows its path at the game's pace, Bond under it takes the damage, clear
of it he does not. Remove the ignore. **Both live pins are green; the ignore never existed on the
file (the tests run unconditionally under NF_ISO).**

## What was wrong, and the fixes

1. **The aim harness (test-side).** The first harness held the aim button. With `aiming: true`
   the port (like the game) routes the look input into the free-aim cursor first
   (`Player_AutoAim`'s aim-mode arm): the cursor clamps at ±0.296/±0.4 and its eased offsets add
   ±13.2° to the fire ray — and past the pitch clamp (bond.pitch 1.0 = 90°) the composed
   direction FOLDS OVER, so the measured fire-ray elevation rails (the old `[cal]` logs: mp
   pinned at 1.504 rad = wp 1.273 + the saturated 0.4·0.57724; myaw off by the saturated
   0.296·0.76966) and no feedback loop can converge. **Fix: hip fire** — with the aim button UP
   the fire ray is Bond's own look direction 1:1 (`run()`'s `dir = aim`, the cross/cursor halving
   to zero outside aim mode), so the harness aligns `bond.yaw`/`bond.pitch` onto the volume and
   taps the semi-auto trigger every 6 ticks. The P2K round (speed 5.0, range 50.0 [D census rec
   2]) carries the ~33 m slant from the shooting spot (−76, −14.18, 34.79 — the ground there
   settles 10 m under the hang spot) to Icicle #1d (−85.5, 20.0, 37.9 [D placements]). Red proof:
   with the old harness both pins die at `started_at.expect(...)`/the duration assert (the
   previous session's report; re-confirmed this session).
2. **The pace clock (test-side).** The fall profile was timed in LOOP iterations; the mid-flight
   `place` steps the sim tens of ticks inside one iteration, so the 90-tick traversal read as
   ~30. Fixed: the profile runs on `drones.frame` (one per `step_bond`). Live profile: frames
   157→247, duration 90 ✓.
3. **The damage arm had a who-mask fold (code fix, `crates/nf-game/src/hurt.rs`).** The port's
   `update_touch` returned no contact when the toucher's class bit missed the volume's who param
   — so the Icicles (who 0x2, the class-5 bullet bit [D params]) could NEVER hurt the player
   (0x2 & 0x10 = 0): the falling icicle passed through Bond and the health bar did not move
   (`[damage] health 81.25 → 81.25`, re-proven red on the old code below). The decompile: the
   who param's only law is the state-0 START mask [M Hurt_Update 0x17e5d0:51-102]; the damage
   flows from the collide contact with NO who read — `Player_DealWithObjHit` 0x194458:94-101
   applies `Hurt_GetDamage` on every class-0x27 hit, `NDrone2_DealWithObjHit` 0x13fff0:308-334
   the same for drones, and the chain walk that reaches them is unconditional
   (`Player_CollisionHandler` 0x1949a8:139-145 deals for every obj+0xD0 entry). The fold is
   removed; the START law stays where it belongs (the class-5 bullet starter's who check +
   `start_flight`'s parks). The two unit pins that asserted the fold ("a mask without bit 8 is
   drone-inert", "who 1 collects nothing, drone included") were TEST bugs under the task rule —
   fixed with the addresses (`the_who_mask_gates_the_start_the_damage_arm_is_who_blind`,
   `the_drone_class_bit_is_8_...` now pin: the touch hurts AND no flight starts).
4. **The flying one-shot retires on the wrap alone (code fix, same file).** The old
   `update_touch` parked ANY one-shot on its first contact — a falling icicle touching Bond
   froze mid-air instead of landing. Per [M 0x17e5d0:129-132] the contact arm never touches the
   state machine; the retire is state 1's wrap. The park now applies only to non-flying volumes
   (the statics' port stand-in [S], unchanged behaviour for 07000008's kill floors).
5. **Doc: ext+0x0C holds count−1.** `Hurt_Create` 0x17e870:28-32 stores
   `*(short *)(param_3 + 2) − 1` (the door's +0x6E convention) — so the Spline last index AND the
   wrap threshold are count−1 and the traversal is `rate` = 90 ticks. The port's step
   `(count−1)·(1/60)·(60/rate)` [M 0x17e870 disasm 0x17e968-0x17e998] and its
   `self.t > count−1` wrap were already right; `tick_flight`'s doc now cites the store. The f32
   sum of 90 steps crosses 5.0 at tick 91 (simulated: step = 0x3D638E3A) — the retire lands one
   tick after the pose parks at the floor key.

## The pin (green, through the real input path)

`crates/nf-view/tests/hurt_07000046_live.rs` (2 tests, no ignore):

- `the_shot_icicle_flies_its_path_at_the_game_pace_and_hurts_bond_in_its_way`: free Bond →
  placed at the shooting spot → real hip-fired P2K shots (press edges through `step_bond`) → the
  class-5 starter arms the flight on the bullet tick → Bond placed under the path mid-fall →
  the falling volume's contact kills him (health 81.25 → 0; the 105 [D param 1] exceeds any
  pool, so the lethal contact is the observable — the amount is pinned by the unit layer). No
  retire assert on this run: the first lethal tick's death NIS pauses the world sections
  [M Game_Run 0x1c9790:30-38] and the wrap lands a tick after the freeze — the game behaves
  identically.
- `the_shot_icicle_never_hurts_bond_clear_of_it`: the same real shot, Bond stays at the shooting
  spot → the flight runs unmolested: **fired on the wrap ✓, the traversal took 90 sim ticks
  (profile 157..247) ✓, the final pose −8.09 < −7.0 (the cave floor) ✓, health byte-identical
  81.25 → 81.25 ✓.**

Red-on-old for the code fix: `git checkout 39cf38151 -- crates/nf-game/src/hurt.rs` (the fold-era
file), rebuilt, ran test 1 → `[damage] health 81.25 → 81.25` → FAILED at the damage assert;
restored, green.

## Live checks for the main agent

1. **The rate-90 wrap question, refined.** On 07000046, shoot an Icicle with nobody under it and
   watch `Hurt_Update` 0x17e5d0: expect state 0 → 1 on the bullet tick, the obj+0x90 matrix
   riding the path's 6 keys, the pose PARKING at the floor key around tick 90, then state 2
   (obj+0xFF = 1) one-ish tick later (the ext[1] f32 sum crosses ext+0x0C = 5.0 at tick 91 in
   the port's operation order; the game's disasm order may differ in the last bit — the crossing
   tick is the question). Confirms the retire is observable in the game and lands ~tick 91.
2. **The death-freeze order.** Stand Bond under a falling Icicle: the first contact tick's 105
   kills; confirm the icicle's flight freezes with the death NIS (the world pause) rather than
   completing its wrap — the port's order (freeze ~1 tick before the wrap) should match.
3. **Visual (lesson 6).** The falling icicle is visible: `--bond -76,-12.6,34.8,5.03,1.35`
   (yaw ≈ 5.03 rad / pitch ≈ 1.35 bond-units points from the shooting spot up at Icicle #1d) —
   the icicle should hang at (−85.5, 20.0, 37.9); after a shot it should drop STRAIGHT down its
   28.4 m path to the cave floor over ~1.5 s and stay there (one-shot). Spot 2:
   `--bond -85.5,-6.4,37.88,0,0` on the cave floor directly under #1d — the icicle overhead, and
   after a shot from anywhere it falls THROUGH this spot (lethal).

## What I ran (per-binary)

- `nf-game --lib`: **695 passed, 0 failed, 5 ignored** (the 10 hurt units included; the two
  rewritten who-law pins green).
- The hurt binaries: `hurt_02_live` **1 passed**; `hurt_07000008_live` **2 passed** (the
  kill-floor harness — untouched by the who/wrap changes); `hurt_07000046_live` **2 passed**
  (the pins); `probe_0b_hurts` **1 passed**.
- The walkreal regression set: this sandbox runs one walkreal binary in 60–90 s (the main
  agent's "~3 minutes for the set" does not hold here — 104 binaries ≈ 2 h), and the sandbox
  kills background jobs between tool calls, so the fast suite runner cannot survive either. Of
  the set, the two binaries whose levels carry the who-1 volumes my damage change could newly
  hurt: `walkreal_07000005_live` **14 passed, 0 failed**; `walkreal_07000006_live` **6 passed,
  0 failed, 1 ignored**. 104 walkreal binaries are BUILT on the branch (the other ~102 not run
  in this window — the next session should run the set, ideally on hardware where the set fits).
- The 07000046 binaries: `hurt_07000046_live` (above) and `hurt46_probe` (assertion-free,
  `--ignored`) — the probe ran green in the previous session's window; not re-run here (no code
  it reads changed: the placements/paths are [D] facts).
- The full four-crate suite did not fit: the background fast runner dies between tool calls in
  this sandbox (all three sessions), the foreground build of all test binaries filled the rootfs
  twice (pruned to fit; 104 walkreal + the hurt set + nf-game finally linked), and the walkreal
  runtime ate the rest of the window.

## What's left

1. The remaining ~102 walkreal binaries + the rest of the four-crate suite on this tree (the
   changed code's surface is the hurt contact/flight pass, exercised by every walk; the two
   who-1 levels and the kill-floor harness are green).
2. The rate-90 wrap live check (above) — cosmetic either way, the arithmetic is shared.
3. `docs/systems/hurt-volumes.md`'s census row could name the seven Icicles (the previous
   report has them; a doc pass can fold them in) and record the who-blind damage law.

## Usage

Not visible to me: this session exposes no token, credit or context-window figures (no usage
panel is available in the sandbox or the IM gateway metadata).
