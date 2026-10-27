## Your task: `script-events-3-4`, script Event 3 (spawn transform) and Event 4 (player disable)

The script player's Event 3 (the spawn-transform bake, `Script_Event` 0x1d94c0:98-116) and Event 4 (`Player_Disable`, :118-121) fall to the port's generic unused arm (nis.rs and drone.rs). On 07000013 the intro 060006d7 fires Event 4 at key 153 (Bond should not be controllable then) and the shutter script 06000959 fires Event 3 twice, so its Entity ops spawn the shutters from the cel transform instead of the baked one [S]. Port both events exactly, then sweep every level's scripts for Event 3/4 users [D dump] and list them in the report. Tests: during 07000013's intro after key 153 player input does not move Bond [M]; the 06000959 Entity spawns use the Event-3 transform [M + D].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/script-events-3-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T01:18Z

---
## Report (done, 2026-10-27T02:53Z)

Closed by the driver: the report is on `s/script-events-3-4` (docs/reports/script-events-3-4.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-27T02:53Z)

# script-events-3-4 — script Event 3 (spawn-transform bake) and Event 4 (Player_Disable)

Branch `s/script-events-3-4`, based on `origin/master` @ ec917093 (master had not moved when the
tree was finalised, so the branch tree IS the final merged tree).

## What the game does

Both events live in `Script_EventHandler` (ACTION.ELF 0x1d94c0).

**Event 3** (case 3, 0x1d94c0:98-116) [M], two halves:

1. `SCRIPTINFO+0x9C = p0 − 1` (the delay), **skipped while a skip is armed** — the guard byte is
   `+0xAB5` bit 0x1, set by `Script_FFwd` 0x1dac78:56 and cleared by `Script_Update`'s end block
   0x1da998:109 (disasm 0x1d95b8-0x1d95d4: `lbu $v0,0xab5($s2); andi $v0,1; bnel …` skips the
   `sw $v0,0x9c($s2)`). `Script_Update` (0x1da998:22-27 + :113) runs the streams only while the
   delay is 0; a negative delay holds without counting down (disasm 0x1da9c4-0x1da9dc), until
   `SP_Activate` 0x1dc338:84 or `SP_UnPause` 0x1dca58:13 zeroes it.
2. The spawn-transform bake: for every stream index `i < SCRIPTINFO+0xAAA`,
   `Script_GetObj` (0x1db688 — returns the stream's object for kinds 1/2 only) and, when it is
   non-null, `Vec_Copy2(obj+0xC0, obj+0x30, obj+0x40)` (call-site disasm 0x1d95e0-0x1d9614;
   `Vec_Copy2` 0x1e2cf8 disasm: it READS `a0` and WRITES the same 3 words to BOTH `a1` and
   `a2`). With the verified obj_tag layout (docs/fields/obj-tag.md: +0x30 position, +0x40
   previous position, +0xC0 the matrix translation row) that snaps position and previous
   position to the object's current matrix translation. `Script_EntityStart`'s create-time tail
   runs the identical copy (disasm 0x1d8f00-0x1d8f0c) right after its `Script_Interp`, so a
   fresh spawn is born baked.

**Event 4** (case 4, 0x1d94c0:118-121) [M]: `Player_Disable(glb_players, 0)`
(0x195f70 — zeroes `BL+0x94a`, which `Player_Update` 0x193600:80 and
`Player_CollisionHandler` 0x1949a8:24 gate on, so input and the collision pass stop; sets the
player obj's `+0xF0 |= 0x10` (not drawn) and its AnimObject's (`BL+0x7E8`) likewise; clears
`BL+0x7D4`) followed by `PlarStat_LogTimerPause(0)` (0x199520 — pause slot 0 of the mission
timer).

## What the port had and what changed

- Event 3's delay write existed but ran unconditionally. `crates/nf-game/src/nis.rs` now skips
  it while the port's `fade_flags` bit 1 (the port of `+0xAB5` bit 0x1) is set — exactly the
  game's guard. The bake itself is **structural** in the port: a script `Obj::Entity` carries
  only the keyed matrix (its translation row IS the game's +0xC0) and an `Obj::Anim`'s position
  is derived per frame from the same anim data the game's matrix row holds, so
  "position == matrix translation" holds by construction and the bake has no second store to
  make. The full [M] chain and the create-tail equivalence are cited in the arm's comment.
- Event 4 was already wired end-to-end (`Request::PlayerDisable` → the view's `disabled`,
  which zeroes the input every tick and gates the collision pass, plus `log_timer_pause()`).
  The 07000013 audit's GAP-13-3 note ("the port never Player_Disables") is stale. The arm's
  comment now cites the full `Player_Disable` chain.

## What the disc says ([D dump] — the sweep)

The committed probe `crates/nf-view/tests/script_events_dump.rs` (run with `--ignored --nocapture`)
decodes every FILES.BIN `07xxxxxx.bin` archive's kind-7 scripts and prints each Event 3/4 site
with its `05`-wait fire tick, plus the full timed decode of 060006d7 / 06000959 / 0600006f and
06000959's key table. **339 sites total: 303 Event 3, 36 Event 4.**

Corrections to the task brief, from the dump:

- The intro script 060006d7's Event 4 does **not** fire "at key 153": stream 7's command list is
  `05 00` (wait 0) → `12 00 04` — it fires on the script's **first tick**. "153" is the stream's
  `first_key` field (153..152 = a keyless stream); the stream never keys. Same shape for all 36
  Event-4 sites: all fire at the earliest tick their stream runs.
- The [S] shutter hypothesis ("the port's Entity ops spawn from the cel transform instead of the
  baked one") does not produce a difference on 06000959: its key table is **all identity**
  (3 keys, pos (0,0,0), unit quat), so the baked transform, the create-time tail's copy and the
  cel/base-composed transform all coincide at the SP placement translation (−1.4095, 0, 0).
  Event 3's real function in this script is the **delay**: both sites pass `p0 = 0` → delay = −1
  → the script holds at t0 (after the preload spawned SCRIPT_SHUTTER_CLOSE) until the SP's
  E-activate clears it, then runs to t16 (spawning SCRIPT_SHUTTER_OPEN) and Event 3 holds it
  again.
- Event 3 params are 0 at 297/303 sites (the −1 hold). The 6 timed ones: 06000048 s3 [500]×2
  (0700000b), 060008bc s13 [120]×2 (0700000c/16), 06000a1f s1 [60] and 0600091a s12 [10] and
  060007fd s1 [250] (0700001b packs) — positive delays that count down.

### Event 3 users (by script hash; the probe prints every stream/tick site)

| Script | Sites | Levels carrying it |
|---|---|---|
| 0600006f | 62 | 07000001-09, 0a-0d, 11, 12, 13, 14, 15, 16, 1b, 23-29, 41, 46, 4a, 4b (everywhere; the shared lift/door pair) |
| 06000044 / 06000045 | 30 + 30 | 0700000c, 0d, 12, 13, 23-29, 41, 46, 4a, 4b |
| 06000041 / 06000042 | 22 + 22 | 0700000a, 0b, 23-29, 46, 4b |
| 0600003c / 0600003f / 0600003d | 16 + 16 + 8 | 07000023-29, 4b |
| 0600004a / 06000043 | 4 + 2 | 07000046 |
| 06000048 | 2 | 0700000b |
| 0600002f | 2 | 07000001, 04 |
| 0600007d / 0600007e / 0600059b / 06000667 | 2 each | 07000009 |
| 06000530 | 4 | 07000012 |
| 06000664 / 0600066d / 060008bd / 060008c2 / 060008c4 / 060008c6 | 2 each | 0700000a / 0b / 0b / 25 / 25 / 25 |
| 060007e2-060007ea, 06000818, 06000832, 060008fa-06000900, 0600091a, 06000927-0600092e, 06000a1f, 06000a30 | 1-3 each | the 0700001b pack set (0780001b/0790001b) |
| 06000571 | 1 | 0700004a (the Tower-2 lift hold, already modelled by the negative-delay work) |
| 060008af / 060008ba / 0600090a / 060009e9 | 1-2 each | 07000014 / 16 / 16 / 16 |
| 06000951 / 06000958 / 06000959 / 060009f0 | 1-2 each | 0700000a / 12 / 13 (the shutters) / 0a |
| 060009e8 / 060009ee / 060009ef / 060009f9 / 060009fb | 1 each | 0700000d / 0b / 0d / 0b / 0b |

### Event 4 users (36 sites, one per cut-scene script unless noted)

0600005b (07000001), 0600005f/0600007f/06000083 (04), 06000071/0600068f/06000848 (06),
06000080 (03), 06000072/06000087 (41), 06000073/06000077/060008c8/06000952/06000994 (05 —
06000952/0994/098a are the unskippable death NISs), 06000078/0600062f (28), 06000084 (09),
060004f9/06000655 (0d), 060004fa/06000646/060006d7 (13), 06000631/32/33 (15), 060006cf/06000833
(16), 060006d3 (4a), 0600098a (3 sites: 0b, 11, 12), 0600085c/0600085f (01 packs).

## Tests (both green)

`crates/nf-view/tests/script_events_3_4.rs`, through the real input path (the Walker harness →
`Sim::step_bond` + `Input`):

1. `intro_060006d7_event_4_stops_bond_until_the_handback`: after 07000013 loads, the intro's
   Event 4 lands (`disabled`), full-forward input moves Bond < 0.05 units over 90 ticks, and
   after the t349 hand-back the same input moves him (> 0.3 units). [M 0x1d94c0:118-121,
   0x195f70, 0x193600:80, 0x1949a8:24; D the stream-7 dump.]
2. `shutters_06000959_spawn_at_the_event_3_transform_and_hold`: after the load's priming beat
   the primed SP sits in state 1 [M 0x1dbc08], the script is stopped and held
   (`delay < 0`, the Event-3 write), SCRIPT_SHUTTER_CLOSE (0x02000A2C, src (06000959, s0))
   exists at the baked transform (−1.4095, 0, 0) ± 0.01 [D placement × identity keys], and
   SCRIPT_SHUTTER_OPEN (0x02000A2B, the t16 op) does not exist while the hold stands.
   [M 0x1d94c0:98-116, 0x1d8f00-0x1d8f0c.]

## What ran

- The new tests above + the `--ignored` dump probes (the [D] evidence base).
- `cargo test -p nf-game --lib`: **576 passed, 2 failed, 5 ignored** — the 2 failures are
  `sdk::tests::the_disc_has_exactly_one_float_format_string` and
  `sdk::tests::the_kernel_stubs_are_sixteen_byte_syscall_wrappers`, both disc-census tests that
  read `/ACTION.ELF` off the image and die with `Io(NotFound)` on this slim on-foot ISO;
  `crates/nf-game/src/sdk.rs` is untouched by this branch, so they are pre-existing environment
  failures, not regressions.
- The fast suite runner (`tools/test/suite.py`) in foreground chunks (the sandbox kills
  background jobs between tool calls and only has 3 GB of RAM, so the nohup start died): 51 test
  binaries covering the alphabetical head and every script/cutscene/lift/door/trigger-named bin
  plus the four lib bins — **178+ passed, 0 failed, 0 timeouts** in the chunk logs, including
  `nf_view` (the lib unit tests), `walk_tower2_probe` (the Event-3 lift-hold level),
  `cutscene_*`, `data_web_07000013`, `census_07000013`, `fix_script_commands_live`-class bins.
  The rest of the 846 binaries did not fit the window: one binary alone (`campaign_flow_live`)
  takes 510 s here and `walkreal_*` need more RAM than the sandbox has (the audit already noted
  `walkreal_07000013_live` OOMs on a 4 GB box). No failed or timed-out binary in what ran.

## What's left

- A full-suite pass on the final tree on a machine that can run it end to end (nothing in this
  branch touches anything outside the Event 3/4 arms and the two new test files).
- The 07000013 audit's GAP-13-3 live check ("can the player move in the retail during the
  intro?") can be closed for the disable half: the port disables from the intro's first tick
  [D], matching `Player_Disable`'s [M] semantics; the retail-side confirmation remains the main
  agent's to make.

## Usage

not visible to me (this environment shows no token, credit or context-window figures).
