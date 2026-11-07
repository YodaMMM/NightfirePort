## Your task: `hurt-volume-flight`, the flying hurt volumes on 07000046 follow their path and hurt in flight

`Hurt_Create` 0x17e870 also reads the placement's raw +0x40/+0x42 (u16 count, u16 rate) and +0x44 (path reference) into ext+0x0C, ext+0x04 and ext+0x08, and `Hurt_Update` 0x17e5d0's state 1 flies the volume along that path with `Spline_Interp`, hurting as it goes. docs/reports/unread-params-props.md on origin/s/unread-params-props found the triple non-zero only on 07000046's seven hurt volumes (count 6, rate 0x5A, path words ending 000E/000F/0010) [D], and the port has none of it. First say what these seven volumes are in play on that level, and what starts state 1. Plumb the path reference to the level's path table, port state 1 with the game's rate and end behaviour, and keep crates/nf-game/src/hurt.rs's other states as they are. Pin through the real level: a volume flies its path at the game's pace, Bond in its way takes the damage, and clear of it he does not. Run the hurt binaries (grep tests for `hurt`), the 07000046 binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/hurt-volume-flight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-07T18:02Z

---
## Report (blocked, 2026-11-07T19:41Z)

# hurt-volume-flight — report

Branch `s/hurt-volume-flight` from origin/master (10956badb). Task: the flying hurt volumes
on 07000046 — `Hurt_Create` 0x17e870's placement raw +0x40/+0x42/+0x44 (count/rate/path) and
`Hurt_Update` 0x17e5d0's state-1 spline flight; say what the seven volumes are and what
starts state 1, plumb the path reference, port state 1, pin through the real level.

## What the seven volumes are [D], and what starts state 1 [M]

New probe `crates/nf-view/tests/hurt46_probe.rs` (`--ignored --nocapture`, off the disc):
07000046's seven 0xE4 placements (#1d..#23) are all **"Icicle"** objects hanging in the island
cave — type 1, damage 105, gate 0 (ungated), sound 0xffff, **who 0x2**, one-shot 1, and the
flight triple non-zero on every one: count 6, rate 0x5A (90), path 14..20 (the task's
"…000E/000F/0010" words are the first three of seven [D raw +0x44 low halves; the high halves
are unrelated record data]). The paths are seven 6-key vertical drops, 28.4 m from y ≈ 20.04
(hang) to ≈ −8.36 (cave floor), x/z constant per volume [D path_blocks 14..20].

State 1 starts by `Hurt_Update` 0x17e5d0:51-102's state-0 who-mask law [M]: the volume's
obj+0xD0 hit chain contributes class bits (class 5 → 2, class 3 → 8, class 2 → 0x10), ANDed
with param 4. who 0x2 names **class 5 — a player bullet** (docs/systems/glass-breach.md,
channel-persistence.md): the Icicles are the shoot-down trap — a bullet contact starts the
fall, a body touch never does (0x10 & 0x2 = 0). Gate 0, so no channel is involved.

## The port work (done)

- `crates/nf-game/src/hurt.rs`: new fields (own lines) `count`, `rate`, `path`, `t`,
  `flying`, `local_tris`, `pose`; `create` reads the raw triple (`extra[7]`/`extra[8]`),
  resolves `level.path_blocks()[path]` through `door::parse_keys` when count ≠ 0 (the game's
  loader does the same: `m_paths[raw+0x44]` → level_tag+0x08 → ext+0x08 [M 0x1d1b68:88-92];
  count 0 leaves it null), and keeps the object-space collision + the placement pose.
  `step()` = `(count−1)·(1/60)·(60/rate)` [M 0x17e870 disasm 0x17e968-0x17e998; the constants
  1/60 at 0x30d0dc and 60.0 at 0x30d0d0 [D ACTION.ELF]] — a traversal is `rate` ticks.
  `start_flight()` = the state-0→1 arm (the class-5 starter calls it). `tick_flight()` =
  state 1's body [M 0x17e5d0:104-132]: pose via `door::interp(keys, 0, count−1, t, true)`
  (the pinned `Spline_Interp` port: Catmull-Rom + slerp), the world tris + AABB re-posed from
  obj+0x90's matrix, then `t += step`; past count−1 → t = 0 and a one-shot retires (state 2).
  Other states and the contact/damage law are untouched.
- `crates/nf-view/src/scene.rs`: the flight clock ticks before the contact passes; the class-5
  starter tests this tick's player-bullet flight segments (`drone.rs`'s new read-only
  `bullet_flight_segments()` accessor — the same walk that feeds the game's obj+0xD0 chain
  [M Collide_Update 0x1ea398]) against the volume's tris. The damage itself stays on the
  existing contact arm (`Player_DealWithObjHit` 0x194458's law) — a flying volume hurts what
  it passes through, exactly like a static one.

## The pin (exists, red on old code — NOT green yet)

`crates/nf-view/tests/hurt_07000046_live.rs` (2 tests): free Bond, place him on the cave
floor, shoot Icicle #1d with the walther (the real class-5 starter — `step_bond`'s trigger,
tapped for the semi-auto's press edge), and pin the flight at the game's pace (the ~90-tick
traversal, the pose on the path keys, the one-shot retire) with Bond under the fall taking
the 105 (the 0x46 normal factor is 1.0 [M damage_factor]) and the sister clear of it taking
nothing. On master (no flight) both fail at the start — red ✓. On the branch they still fail:
**the aim harness cannot drive the fire ray onto the icicle yet.** Facts measured while
trying: from directly under (28.5 m of rise) the port's fire ray tops out at ~77° elevation
(the look clamp), so the shot must come from a distance; from the two solid spots tried
(16.5 m south — Bond fell into a pit at y −80.8, so moved; then (−76, −14.2, 34.8)) the
measured fire-ray pitch railed at ~86° regardless of `bond.pitch` — the fire direction comes
from the camera/aim pipeline (scene.rs ~4549's `aim`), not 1:1 from the look pitch, and the
feedback calibration I wrote did not converge inside the window. **Left for the next
session:** fix the test's aim (trace the fire-ray source for Bond's walther — `dir` at
scene.rs 5438's call — and aim via the measured feedback with the correct coupling), place
Bond at a solid mid-elevation spot, then both tests should go green; they are ready to pin.

Live checks for the main agent:
1. On 07000046, shoot an Icicle (a real bullet into the hanging volume) and watch
   `Hurt_Update` 0x17e5d0: expect state 0 → 1 on the bullet tick, the obj+0x90 matrix riding
   the path's 6 keys over 90 ticks (1.5 s), then — one-shot — state 2. Also watch whether the
   wrap ever fires at rate 90: `ext[1] = 5/90` accumulated in f32 may sit at 4.999… forever
   (my port replicates the same arithmetic; the unit test pins the wrap law at rate 60 where
   the sum crosses). If the game's icicle never wraps, the one-shot retire never runs and the
   volume stays armed at the floor pose — the port matches either way, but the report should
   record which.

## What I ran (per-binary)

- `nf-game --lib`: **691 passed, 0 failed, 5 ignored** (the 10 hurt units included — 4 new:
  the step formula, the wrap/retire law, the no-path/no-short-count parks, plus the 6
  pre-existing static-law tests all still green).
- The walkreal regression set (my scene.rs change runs in every level walk): **106
  walkreal_* binaries: 255 passed, 0 failed, 30 ignored.**
- The hurt/07000046 binaries: `hurt_02_live` 1 passed; `hurt_07000008_live` 2 passed;
  `probe_0b_hurts` 1 passed; `hurt46_probe` (the ignored probe, run explicitly) ok;
  `hurt_07000046_live` 2 FAILED (the pin above).
- The full four-crate suite did not fit: this sandbox's rootfs filled (100%) and the target
  dir had to be pruned (test executables deleted, relinked on demand) — the background fast
  runner dies between tool calls here (both this and the previous session). The set above is
  what the changed code touches.

## What's left

1. The live pin's aim harness (above) — the implementation is done and unit-pinned; only the
   test's shooting needs to converge.
2. The rate-90 wrap question (the live check) — cosmetic either way, the arithmetic is shared.
3. `docs/systems/hurt-volumes.md`'s census row could name the seven Icicles (this report has
   them; a doc pass can fold them in).

## Usage

Not visible to me: this session exposes no token, credit or context-window figures (no usage
panel is available in the sandbox or the IM gateway metadata).
