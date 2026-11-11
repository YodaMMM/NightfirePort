## Your task: `real-health-00agent-01-hall-2`, 07000001's hall can be crossed on 00 Agent with real health

Branch from origin/s/real-health-00agent-01-hall, merge current origin/master, and read its docs/reports/real-health-00agent-01-hall.md plus the 00 Agent ignore message. Take its next lever: the west-corridor pre-clear before the meet walk. Judge it over several real runs. Do not repeat its measured dead ends. Helpers stay gated to 07000001 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T07:06Z

---
## Report (done, 2026-11-11T09:37Z)


---

# Report (docs/reports/real-health-00agent-01-hall-2.md on s/real-health-00agent-01-hall-2)

# real-health-00agent-01-hall-2

Branch `s/real-health-00agent-01-hall-2` (cut from `origin/s/real-health-00agent-01-hall`
4f76e3fb90, merged with origin/master — already up to date).

## What's done

The task's lever — the west-corridor pre-clear before the meet walk — **landed and is
verified over 8 instrumented real runs** (`NF_DMG_TRACE=1`, per-hit lines with owner,
weapon, position and distance). The 00 Agent double-cross walk is still NOT green: the
corridor is beaten, the failure moved to the door phase's kill path, and the ledger +
next lever are pushed in the test's ignore message.

1. **The corridor pre-clear** (d3-gated, in `walk_to_the_exit_at`, `walkreal_07000001_live.rs`):
   after the intro and the out-of-line step, the walk stages to a measured stance and fights
   the three west-corridor guards (#0/#13/#8 — the load roster's indices [L the probe dump:
   #0 skin 0x5000036, #8/#13 skins 0x5000039 SMG, all ch-0xB enabled]) as CHOSEN fights via
   `d3_corridor_fight` (bounded tap loop, the aim at the live position, the empty-clip reload
   arg [M Player_WeaponFiring 0x1a5460's reload branch], the walk's own shoot as fallback).
   The stances are per guard, from the bullet-lane measurements: #0/#13 from the west band
   (-10.5,-9.0), #8 from the spawn-area south (-7.5,-11.5). Return fire is held OFF for the
   2-u approach leg (the fights must happen from the stance, not wherever the first round
   lands) and the determined pass is OFF for the pre-clear (a failed chosen fight leaves the
   guard to the loop below; the run-1/2 grinds measured the alternative).
2. **The opt-in flank z-hold** (`crates/nf-view/tests/common/walker.rs`):
   `Walker::flank_hold_z: Option<f32>` (defaults `None`; the ladder gate sits next to the
   existing `flank_hold_x` pocket hold). A target whose pos[2] is north of the hold never
   takes a flank nav — run 5's death rode the return-fire fight's flank ladder INTO the hall
   (flank point (3.9,18.0), death at (6.6,13.3) to #6's 2.1-u shotgun). The d3 hall phase
   sets `Some(11.2)` and clears it after the AND. Every other walk unchanged (the field is
   None; the d1/d2 tests pass byte-identical).
3. **The d3 hold-line merge**: the (7.2,8.8) intermediate stop + 60-tick wait are skipped at
   d3 — run 4 died to two #7 volleys during that transition (ticks 3421/3536). The d3 walk
   goes straight to the west-shifted deep line (6.5,7.6). d1/d2 keep the two-stop timeline.
4. **Lures-first at d3**: the hold-line crouch fight is d1/d2 only now — runs 6/7 measured it
   burning the whole 89-round pool on 1.0-per-round body taps before any sustained lure ran
   (the crouched windows expose the lower parts; the head box stays lamp-blocked), and the
   AND crowd never crossed a dry line.

**The measured results** ([L] traces, /tmp/d3_run*.txt on this branch):

- Base repro (before any change): death at tick 13546 at the door phase, 49.0 hp — the
  corridor tax #13 25.2 in 14 + #0 24.0 in 2 + #8 1.8, exactly the prior ledger's numbers.
- Run 4 (the per-guard stances): the corridor phase costs 45.0 (#0 36.0 in 3 — his lane
  reaches every south stance, his opening 24.0 rides the out-of-line step's exposure;
  #13 9.0 in 5 during #0's fight; **#8 0.0** — his lane is walled from the spawn-area south,
  the fight ran free), then **zero damage from the corridor to the door phase** — 55.0 hp at
  the door phase vs the base's 49.0.
- Runs 6/7/8: the walk survives the whole hall phase at 55.0 (lowest health 55.0 — the flank
  hold + the deep line kept the volleys out), but the AND crowd cannot be killed: the
  sustained lures ran with full pools and drew no crossings ("alive true across true" ×3 in
  every run), the fights went dry, the AND never fired (the test fails on its own
  `ch 153` assert).

## How it was verified

- `walkreal_07000001_live` (whole binary, `--test-threads=1`): **7 passed, 1 failed** — the
  only failure is the d3 `walk_..._real_damage_00agent` ledger test; the d1 and d2
  real-damage sisters pass (the shared-walker change is opt-in and the d3 tactics are
  difficulty-gated, so their streams are byte-identical).
- `interactables_07000001_live`: **10 passed, 0 failed**.
- 8 instrumented d3 runs cited above ([L] traces with per-hit owner/weapon/position).
- **The fast suite did not run**: `tools/test/suite.py` was started three times in the first
  minutes (plain, with `CARGO_BUILD_JOBS=1`, via `setsid`); every attempt died between tool
  calls — this sandbox kills all spawned processes when a tool call ends, and the one
  foreground attempt OOM'd at the 4-crate build (4 GB RAM, 2 cores). The explicit batteries
  above are the coverage. **Re-run the suite on this branch before merging.**

## What's left

- The door-phase kill path (the ledger's NEXT LEVER in the ignore message): the sustained
  lures do not draw crossings on this stream — the armed-lure run that drew 5/6/7/11 across
  carried clip 32 spare 16, so the hearing chain's response depends on the weapon/timing mix
  [M DroneFunc_SetNoiseHeard 0x149920 / HandleSoundAlerts 0x148a30]; or kill the crowd
  through the aperture with the crouched flight-lane taps (the pre-clear's lane trigger —
  the game's own flying-round trace is clear where the scoped sight is blocked [M
  drone.rs:16465]), pool metered.
- The suite re-run on the final tree.

## Live checks for the main agent (PCSX2, fresh boot)

1. Guard hearing at the 07000001 hall door line: from the hold line south of the pre-opened
   door, fire a sustained SMG burst into the hall, then a sustained pistol burst — record
   `DroneFunc_SetNoiseHeard` (0x149920) calls and the guards' state transitions per weapon.
   Expected if the lever (a) is right: one of the weapons draws the ch-0xE guards across the
   aperture, the other does not.
2. The hall guards' crouched exposure: with Bond crouched at the deep hold line (6.5,6.4,7.6),
   which body parts of the across guards (5/6/7/11) pass the eye ray — the port measures the
   lower parts only (body taps, hp 24→16 pattern) and never the head box.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~11:10 UTC; this report written ~10:25 UTC), `git` hashes, test outputs, and process
exit codes. No input/output token counters, no credit balance, no dollar cost.
