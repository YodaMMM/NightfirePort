## Your task: `fusebox-inverter-2`, create the type-0xF9 inverters without breaking 0c and 41

Start from `origin/s/fusebox-inverter` (merge it, then origin/master). It builds `InverterTrigger_Update` 0x1b30a8 triggers from type-0xF9 placements (p0 input, p1 output) on every level; on 07000014 that makes the fusebox cut's downstream events happen (confirmed in real play). Merged with master these go red (each green on master, confirmed by re-running without the branch):
- fix_guard_misc_live::probe_sniper_direct_bullet, fix_guard_misc_live::the_0c_sniper_takes_damage_when_shot (0700000c)
- interactables_07000041_live::the_searchlight_family_sweeps_as_one_light, ...::the_wrapper_ends_into_the_space_station_after_the_chain (07000041)

1. On 0c and 41, list the type-0xF9 placements and what their new inverters do to the channels those tests depend on. Check the game's create gate for 0xF9 [M parsemap_create_dynamic_objects 0x1d04f0's 0xF9 arm, InverterTrigger_Create — params, flags, any level or flag condition] and the update's exact rule [M 0x1b30a8]. Port the exact create gate and rule; where the game really inverts those channels, fix the tests to the game's behaviour with evidence.
2. All four green plus the 14 inverter test; full suite on the merged tree; report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/fusebox-inverter-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-26T05:13Z

---
## Report (done, 2026-10-26T06:58Z)

Closed by the driver: the report is on `s/fusebox-inverter-2` (docs/reports/fusebox-inverter-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-26T07:18Z)

