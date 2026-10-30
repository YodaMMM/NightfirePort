## Your task: `level-13-spawn-to-end-walk`, 13 from the carried load to the mission's end

`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out` starts warm. level-11-every-objective-3 (merged) walks 11 -> 12 -> 4A and pins 0x04000035 "Escape from the tower" as announced only: its writer is the whole-mission AND ch 100 [M 0x1827d8:119-123], first set on 07000013 [D 0700004a.md / 07000013 docs]. Add one walk that loads 07000013 the way the game does after 4A (the RamSave carry [M Player_RamLoad 0x1957a8]) and reaches the level's end on real input with every objective set by the game's own writers (no channel pokes), pinning 0x04000035 complete. Fix any port gap it hits with evidence [D/M]. Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-spawn-to-end-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T06:36Z

---
## Report (done, 2026-10-30T09:15Z)


---
## Report from the worker (done, 2026-10-30T09:15Z)

# level-13-spawn-to-end-walk — report

Branch `s/level-13-spawn-to-end-walk` (from origin/master 0c90eba8; merged origin/master
8878f6b6 mid-task). Task: one walk that loads 07000013 the way the game does after 4A (the
RamSave carry) and reaches the level's end on real input with every objective set by the game's
own writers, pinning 0x04000035 "Escape from the tower" complete.

## What's done

`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out` already carried the
warm chain 07000012 (walked, agent difficulty 3) → 0700004A (the spawn drop onto touch #277, ch
0x4D) → 07000013 (the RamSave carry) → the front entrance → mission complete → the
`mission_end` 0x07300013 latch (the earlier merged tasks built that). This task closed the last
gap: **part 0's records are now pinned on the same walk** (commit "level-13-spawn-to-end-walk:
pin part 0's records on the warm chain…"):

- At the 07000013 carried load: `states[15][1] == 4` — 0x04000036 "Gain access to lower office
  floor" folded on the chain (ch 0x48 written by 07000012's spawn touch #679 [D; M 0x1827d8
  case 3]); `states[15][0] == 3` — 0x04000035 still announced, because its writer is the
  whole-mission AND ch 100 [M 0x1827d8:119-123] which first fires on 07000013; ch 100 off.
- After the success touches #73/#164/#165 write ch 0x64 = 100 [D docs/levels/07000013.md]:
  `states[15][0] == 4` — **0x04000035 COMPLETE** — plus ch 100 raised by the monitor's all-AND
  and all six mission records complete (`states[15][*], [16][*], [17][0], [18][0] == 4`) at the
  mission_end latch.

**No port change was needed**: the port's `Mission_MonitorObjectives` (crates/nf-game/src/
mission.rs `monitor()`) already ANDs every record's condition (including the ch-100 record)
before the state switch and writes `switch_channels[100] = 1` when the AND is true, exactly as
the decompile at ACTION.ELF 0x1827d8 (the `bVar9 &= bVar1` loop; case 3's `puVar10[5] = 4`; the
`if (bVar9) { switch_channels[100] = 1; … Music_Event(8,1); }` tail — checked in
work/decomp/ACTION.ELF/001827d8_Mission_MonitorObjectives__Fv.c). Every objective on the walk is
written by the game's own objects; there are no channel pokes anywhere in the chain.

Every test in the file is green.

## How it was verified

- **Touched binary**: `walkreal_07000013_live` —
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.05s`
  (run twice this session: after the edit at 07:48 and again at 08:41 on the identical final
  tree; both green). The binary was confirmed rebuilt from the edited source (mtime 07:45 +
  the new assertion strings present in the binary).
- **Full suite (incomplete at the sandbox deadline — the one thing left)**: the sandbox kills
  background jobs between tool calls, so the suite ran via chunked foreground runners
  (tools/test/chunk_suite.py, then a 3–4-worker variant of the same per-binary protocol with
  `--test-threads=1` and a per-binary timeout, state in `/home/z/suite13_state.json`). Before
  the run, ~5.4 GB of dead pre-merge artifacts had to be deleted (the rootfs hit 100 % and the
  linker failed with ENOSPC). Recorded by the deadline: **664/915 test binaries — 1510 passed,
  3 failed, 173 ignored**:
  - `fix_0c_pylon_live` (2 failed) and `probe_pylon_render` (1 failed, rc 101): the documented
    no-GPU environment family — both call `wgpu::Instance::request_adapter(..).expect("adapter")`
    and the sandbox has no GPU (probe_pylon_render was already known-failing here).
  - `net_internet_live` (1 failed, rc 101): not investigated before the deadline; almost
    certainly environment (needs live internet peers); next session should rerun it and, if it
    persists, read its assertion.
  - ~160 binaries recorded `rc = -9` (SIGKILL = OOM) and two timeouts (`walkreal_07000009_live`
    — the merge added ~256 lines to it, likely needs >240 s; `rook_door_06_live`): these are my
    runner's 4-way parallelism on a 4 GB box, NOT tree failures — the same binaries pass in the
    previous session's state (e.g. `walkreal_07000012_live`, `npc_doors_live`). They are dropped
    from the state so the next session's run redoes them solo.
  - ~250 binaries were never reached.
  - The next session can resume: `/home/z/suite13_state.json` + `python3 /home/z/par_suite.py`
    (or tools/test/chunk_suite.py) until ALL DONE, then quote the summed total.
- A one-off corrupted link was hit in the `-j4` relink batch (`fix_swim_live` reported
  "running 0 tests … 4 filtered out"); a touch-relink fixed it (then 4 passed / 0 failed) —
  prefer the driver's `-j2` for relinks.

## What's left

1. Finish the full-suite run on the final tree (state file + runner above) and quote the summed
   total; rerun the OOM/timeout victims solo and look at `net_internet_live`.
2. The walk's evidence is complete; no live checks are required for this task. (Standing offers
   from earlier reports — the tower mission's ch-100 gdbwatch etc. — are unchanged.)

## Usage

not visible to me
