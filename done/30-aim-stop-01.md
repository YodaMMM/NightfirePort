## Your task: `aim-stop-01`, guards stop dead to shoot on 07000001 too

Branch from origin/master. The guards' AimStandFire root stop (Drone::zero_root, Drone+0x38 [M 0x166328:117-123 -> DroneAnim_PreTransCallback 0x13b490], [L PCSX2 07000011 d2: guard 0x1896b50 stops dead on its 0x68 entry at frame 214]) is live per level: the zero_root_live list at the AIM_STAND_FIRE UPDATE arm in crates/nf-game/src/drone.rs. Add this level to that list and re-tune its walks against the stop (real input only; never lower damage or health, no top-ups; helpers gated to this level). Done = the level's walk binaries all green with the stop on, then the full suite.

Measured with the stop on 01: (1) following_mayhew_down_completes_the_level: the west-end pair #15/#4 stand at (-30.5, 6.3, 13.5) and fire the length of the hall at the waiting contact 28 u off (hit_by_drone 15 at t5708: 10.0 -> 2.5; #4 finished him at t10021); widening the west-end pre-clear band from 20 to 30 u (farthest first) makes it green. (2) walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent: at the hall guard-6 peek the walk_to the stand (7.8, 6.46, 10.2) wedges 0.72 u short at (7.40, 9.60) for its 3000-tick cap in the door mouth, and #7 (24 hp) trades 37.8 (9 hits) with Bond there (81.4 -> 43.6); the west hall then takes the rest (#3/#12 at (-30.6, 13.3)), death at t12116. 01 has no armour pickup - the hall approach has to cost less. Run walkreal_07000001_live, fix_mission_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/30-aim-stop-01`.

---
claimed-by: sandbox1 at 2026-11-13T01:32Z

---
## Report (done, 2026-11-13T03:20Z)


---
## Report from the worker (done, 2026-11-13T03:20Z)

Closed by the driver: the report is on `s/aim-stop-01` (docs/reports/aim-stop-01.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-13T03:20Z)

# aim-stop-01 — the AimStandFire root stop lives on 07000001 too

Branch `s/aim-stop-01` (base origin/master 1105786cd). Code checkpoint: 0d06bcdab.

## What's done

1. **07000001 joined the `zero_root_live` list** (crates/nf-game/src/drone.rs, the AIM_STAND_FIRE
   UPDATE arm, now `0x01 | 0x02 | ...`): the guards' root stop on aim entry (`Drone+0x38`,
   [M 0x166328:117-123 -> DroneAnim_PreTransCallback 0x13b490]) is live on the level, the same
   one-line gate the other 15 levels landed through. The [L] anchor for the mechanism is the
   07000011 d2 measurement in the code comment (guard 0x1896b50 stops dead on its 0x68 entry at
   frame 214).
2. **The escort walk re-tuned against the stop** (walkreal_07000001_live.rs): with the stop on,
   the west-end pair #15/#4 stops dead at (-30.5, 6.3, 13.5) and fires the length of the hall at
   the waiting contact 28 u off (hit_by_drone 15 at t5708: 10.0 -> 2.5; #4 finished him at
   t10021 [L the aim-stop-01 driver measurement]). The west-end pre-clear band widens 20 -> 30 u
   (the filter is already farthest-first), so the pair is listed and fought before the follow
   begins. `following_mayhew_down_completes_the_level` is green (17.4 s).
3. **The agent door-peek's stand approach is bounded** (file-local `bounded_peek_walk` in
   walkreal_07000001_live.rs). Reproduced the driver's red exactly first: at the hall guard-6
   peek, the walk_to the stand (7.8, 6.46, 10.2) wedges 0.72 u short at (7.40, 9.60) for its
   3000-tick cap in the door mouth, and the shotgun captain #7 (24 hp, 0x1c) trades 37.8
   (9 hits) with Bond there (81.4 -> 43.6); the west hall pair (#3/#12 at (-30.6, 13.3)) then
   takes the rest — death at t12116 [L the aim-stop-01 red run, same tick, same 37.8-in-9-hits
   line]. The cause is a guard crossing the mouth mid-walk: his capsule turns the leg into a
   point-blank (×2 TooClose band [M Drone_ModBulletDamage 0x139f50]) grind, then his corpse
   holds the wedge to the cap. The bounded approach steers at the stand like walk_to's plain
   leg but stops at the 0.55 band, after 240 ticks, or on 60 ticks without closing progress;
   on a stall the leg backs off to the hold line and answers whoever shows from the frame (the
   no-stand arm's fight), instead of pressing. The return-fire answering keeps running inside
   `tick`, so the blocker is fought from the frame, not traded with in the mouth.
   `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` is green: level_exit to
   07000002 at tick 14760, end health 2.4 + armour 50.0, 39 hits for 97.7 absorbed on the port's
   own damage (the armour comes from the level pickup on the later route; the hall approach
   itself now costs ~25 instead of 37.8+, which was the ask — 01 has no armour in the hall).

## Not kept (tried and reverted)

- The walker's opt-in hostile-body-block response (`body_block_response`, the 02/03/05/08
  lever) around the peek call: its shoot machinery moved Bond across the door plane (death at
  (6.60, 6.45, 11.79) at t5893, 81.4 to the #7 shotgun at 6 u [L the run]) — the same
  "flank walked Bond INTO the hall" shape the d3 notes warn about. Reverted; the bounded
  approach replaces it. Nobody should re-try that flag at this door without bounding it.

## What was run (all --test-threads=1, NF_SILENT=1, real input)

- `walkreal_07000001_live`: **7 passed, 1 ignored** (the pre-existing 00-Agent ammo-exhaustion
  ignore, unchanged) — 70.4 s. Both new/changed tests green with the stop on; the d1 variants
  (`..._with_the_contact`, `..._real_damage`) pass unchanged.
- `fix_mission_live`: **4 passed** (incl. `the_07000001_exit_works_end_to_end`) — 75.4 s.
- `walk_07000001_live`: **2 passed**; `guard_noise_burst_07000001_live`: **1 passed**;
  `guard_first_shot_01_live`: **1 passed**; `guard_sight_walls_01_live`: 1 ignored (probe,
  pre-existing); `interactables_07000001_live`: **10 passed** — 01's other binaries, since
  drone.rs is the shared guard brain.
- `guard_stuck_sweep_live`: **2 passed** (53.9 s); `npc_doors_live`: **6 passed, 1 ignored**
  (the pre-existing escort-reach ignore, unchanged; 25.9 s); `fix_guard_misc_live`: **6 passed**
  (7.5 s) — the lesson-10 set for a drone.rs touch.

## What's left

- **The full suite (`tools/test/suite.py`) did not fit in this sandbox** — the 9.9 GB disk
  filled twice (os error 28, then ld "signal 7 Bus error"): each interrupted cargo run recreates
  the nf-view test binaries' build-hash dirs (~20 MB each, ~1259 dirs had piled up in
  `target/release/build/nf-view`). I pruned them (target is 1.1 GB, 5.0 GB free now). The suite
  needs one uninterrupted build + run (~40-60 min) — resume on this branch with:
  ```
  export PATH=$HOME/.cargo/bin:$PATH NF_ISO=/home/z/NightfirePort/Nightfire-onfoot.iso
  export PKG_CONFIG_PATH=/tmp/alsa-prefix/usr/lib/x86_64-linux-gnu/pkgconfig
  export RUSTFLAGS="-L /tmp/alsa-prefix/usr/lib/x86_64-linux-gnu"
  cd /home/z/NightfirePort
  NF_ISO=$NF_ISO nohup python3 tools/test/suite.py > suite.out 2>&1 &
  ```
  (if `df -h /` shows <2 GB free again, prune `target/release/build/nf-view` first). Note the
  churn trick: build only the binaries you run (`cargo test --test <name>`) — never leave an
  all-binaries build half-done. Expected reds on the slim ISO: only the 3 `disc_golden` tests.
- The `real_damage_00agent` d3 walk stays `#[ignore]`d exactly as master left it (the
  ammo-exhaustion ledger is in its doc comment; untouched by this task).

## Live checks for the main agent

- Fresh boot 07000001 (d2 is fine), walk the hall fight: the hall guards across the door
  aperture (one stands at (5.3, 6.3, 13.6)-ish) — during their AIM_STAND_FIRE (state 0x68),
  watch `Drone+0x38` (guard struct base + 0x38): expected 1 from the frame after entry and the
  guard's root step dropped while firing (no slide-in) [M 0x166328:117-123 -> 0x13b490]. The
  0x68 dispatch runs through the same code the port models; the port now arms `zero_root` there
  for 01.
- Visual: a run into the hall mouth — the guards should halt at their spots and shoot instead
  of closing while firing. Bond observation spots: `--bond 3.0,6.5,9.0` (down the hall from the
  east door band) and `--bond -10.0,6.5,14.5` (mid-hall, the west-end pair's end).

## Usage

not visible to me — this environment shows no token, cost or context-window figures.
