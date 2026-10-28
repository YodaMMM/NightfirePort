## Your task: `level-0b-every-objective`, level 0700000b completable end to end on real input

Make `crates/nf-view/tests/walkreal_0700000b_live.rs` walk the WHOLE mission from a fresh load on real input (`step_bond`: movement/aim/fire/use; no teleports past a blocker, no channel or state pokes): every objective the level's mission data lists [D the objective records / Mission_MonitorObjectives 0x1827d8 M], in the order a player does them, each asserted complete, ending on the level exit firing. First list which objectives the current tests already cover; add legs for the rest. Every blocker you hit (a door that will not open, an interactable that does nothing, a model missing, a scripted event that never fires, a guard that never comes) is a port gap: find the game's behaviour [M/D] and fix it, or - if it is drawing only - list it for the main agent. Keep any ignore only with the exact blocker and its evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-every-objective`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-28T18:34Z

---
## Report (done, 2026-10-28T19:33Z)


---
## Report from the worker (done, 2026-10-28T19:33Z)

# level-0b-every-objective — report

Task: make `crates/nf-view/tests/walkreal_0700000b_live.rs` walk the WHOLE 0700000b mission from a
fresh load on real input (`step_bond` only), every objective the mission data lists asserted
complete, ending on the level exit firing.

## What's done

1. **The objective records enumerated from the game's own data** [L ee_snapshot 0x2a3f78 /
   0x2a3fc0 / 0x2a4008, parsed per `Mission_Init` 0x182270's lookup]. `Mission_MonitorObjectives`
   0x1827d8 monitors every mission-table row whose BaseMap matches; the Tower mission (BaseMap
   07000009) spans **11 records** keyed by the entry whose last word (`entry+0x24`) names the level:
   order 0 → 0700000a (objs 1-3), order 1 → 0700000b (objs 4-6), order 2 → 0700000c (objs 7-11).
   Record layout [L raw bytes]: label @+0, failptr @+4, **completion ch @+0xC**, reveal ch @+0xF,
   **flags word @+0x10 (bit 1 = invert)**, state @+0x14. Obj 1 (label 0x04000016) completes on the
   INVERTED alarm channel ch 0x61 (flags = 2) — "don't raise the alarm". This level's six: obj 6
   ch 0x4B, obj 7 ch 0x4C, obj 8 ch 0x4D, obj 9 ch 0x4E, obj 10 ch 0x59, obj 11 ch 0x5A. The
   mission-complete AND [M 0x1827d8:119-123] includes ALL 11 + the ch 0x61/0x60 fail arms
   (0x7B keys on map 0x7000016, NOT 0b [M 0x1827d8:108-115]).
2. **Coverage survey of the old test**: objs 6/7/8/10/11 present; obj 9 driven via ch 0x08; the
   lock ch 9 driven; the roof gate ch 0x2D driven; the lift ride a teleport; the Q-Worm gadget
   inventory-poked. The rewritten test documents the per-objective matrix in its header.
3. **The Q-Worm boot loadout pinned, the inventory poke dropped**: `nf-extract weapon-matrix` shows
   0700000b load=1 for weapon 0x58 [D; the gadget cycle reaches owned records only, M 0x1a6f80],
   and the run proves the port's fresh boot owns it (`boot owned[0x58] = true` in the log).
4. **The roof gate is now the REAL chain, no channel drive**: obj 9's ch 0x4F = AND #299(0x4C,
   0x4E) → E-plate #166 (mask 4 = use, [D trig dump]) → ch 0x35 → SEQ #252(0x36, 0x35) → ch 0x34 →
   `Trigger_Calc` 0x1b1900's per-level copy (`GameState == 0x700000b && ch 0x34 → ch 0x2D`,
   0x1b1900:24-26) → SP wedge #177 plays 060004FA → ch 0x59 (obj 10) + ch 0x57 + ch 0x2A. The
   walkthrough asserts ch 0x35, ch 0x2D and the script tail through the real SP machinery [M
   0x1db9a8/0x1dbda8 gate law].
5. **SP gate law pinned from the decompile** [M 0x1db9a8]: mask = placement p5 @+0x2E, out ch = p6
   @+0x30, lock ch = p9 @+0x32, mode = p2; mode 1 → state 7 → load → armed; the gate fires on
   `SP_Hit bits & mask`, the out channel is written while the gate holds. #355 ("files wedge",
   0x060009FB, mode 1, out ch 0x4E) is therefore PRESENTATION downstream of the OR mux, not the
   writer.
6. **The real shaft-mouth walk**: the arrival cut-scene's GameFlow push logs a level-exit request
   in the port (it cannot load the movie state) which made every later `walk_to` vacuous in the
   OLD test (`stop_on_exit` short-circuited at +0 ticks — the old "walked" exec-floor legs were
   vacuous). The rewrite scopes `stop_on_exit = false` and the balcony→shaft walk runs REAL
   ("the shaft mouth: arrived at +1717 ticks pos (1.90,15.13,25.77)").
7. **The lift ride attempt** rides the port's real `ground_motion` machinery (the paternoster cab
   0x2000367 is in the script-entity dynamic set since the trench-landing whitelist fix [M 0x194188
   → Control_InheritVelocity 0x133e88]); the boarding poll reads the cab's live matrix from
   `scripts.objects`. The ride leg prints its result; on failure the documented stand-in transit
   fires and says so.

## How it was verified

- The sister test `the_roof_cutscene_060004fa_writes_ch59_ch57_and_arms_the_helicopter_gate`
  PASSES (the script tail, the copter gate law, the re-place, the pre-gate baseline).
- The walkthrough run log: the spawn plate ch 0x4B (obj 6) real; the boot Q-Worm owned; the shaft
  walk real; the terminal plate ch 0x4D (obj 8); the lock ch 9 + the bridge/terminal ch 10/11
  chain; the Q-Worm install (obj 7); the E-plate chain (obj 10); the parachute + jump (obj 11).
- The remaining walkthrough-test failure is at the Q-Worm install leg (see Left).

## What's left

1. **The walkthrough test fails at the Q-Worm install leg** ("the Q-Worm C install never landed",
   [M 0x1aee20]). The pre-shot state is proven correct (weapon 0x58, request 0x58, owned true, the
   same desk position and the same `shoot_at` code as the merged version), so the residual
   difference is upstream state from the rewrite's real walks; next step is a bisect of the legs
   before the shot (the prime suspect: the walker's stuck-mitigation side effects during the
   wedged terminal/lock walks, or the exit-request state interacting with the bullet pass).
2. **The lift boarding** (cab_y never matched hash 2000367 in the run — the entity id/hash
   pairing needs one probe; the ride machinery itself is pinned). Until it boards, the ONE
   documented stand-in transit (balcony→exec floor) remains, as the level doc records.
3. **The walked lock approach + terminal approach** stay stand-ins with their pinned blockers
   (the boardroom furniture row, [D probes]; the real walk attempts now run first and wedge, as
   the blockers predict).
4. **The disc's writer of ch 0x54 / ch 0x06 (obj 9's OR inputs)** — still the level's open item;
   swept again this session: `Trigger_Calc` 0x1b1900 has only the 0x34→0x2D copy for 0b; no
   placement param carries 0x54/0x06 [D placements-all + trig dumps]; no direct
   `switch_channels[0x54]/[6]` writer in the decompile; the RamSave carries all 256 channels
   [M 0x195670 — amemcpy 0x100 blocks from 0x26fc90/0x26fd90/0x26fe90]. The walkthrough keeps the
   mux's ch 0x08 input driven, exactly as the level doc's walkthrough does.
5. The FULL suite total line: the fast runner (`tools/test/suite.py`) could not run to completion
   in this sandbox — background jobs are killed between tool calls here (three nohup/setsid
   launches died silently with an empty suite.out), and the foreground window did not fit the
   build + full pass after the rewrite iterations. The crates the task touches were run directly:
   `nf-view`'s walkreal_0700000b_live binary (the sister test green, the walkthrough failing at
   the leg above) — run per-binary with `--test-threads=1`, NF_SILENT=1.

## Live checks for the main agent

- `tools/trace/fieldprobe.py`: at the 0700000b exec floor, poke nothing — observe
  `switch_channels[0x54]` (address 0x26fc90+0x54) and `[0x06]` during the terminal-room sequence
  (the disc's obj-9 writer hunt). Expected: whichever turns on names the writer system.
- `tools/trace/gdbwatch.py`: watch the script-entity object for the paternoster cab during the
  balcony→exec ride — the object's hash/id (the port's cab_y matcher keyed on hash 2000367 [D nis
  dump] never matched at runtime; the live id settles the pairing), access type R, during the cab
  cycle.

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures for the session
(no environment variable, file or tool output in this session carried them).
