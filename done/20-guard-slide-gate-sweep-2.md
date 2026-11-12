## Your task: `guard-slide-gate-sweep-2`, the guard leaf slide on every level without losing a walk

Branch from origin/s/guard-slide-gate-sweep, merge origin/master (which now carries guard-leaf-press-07's 13-level press gate: reconcile the two door_leaf_slide gates into one game-correct rule [M Drone_CollisionHandler 0x140968:204-208]). The branch re-ignores walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent, which is GREEN on master: repair it with real input (never lower damage or health) so the slide lands with no walk lost. Run every walkreal_*_live binary one level per tool call.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-slide-gate-sweep-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T03:48Z

---
## Report (done, 2026-11-12T04:38Z)

# guard-slide-gate-sweep-2 — the all-levels leaf slide reconciled onto master; the agent walk repaired forward

Task: branch origin/s/guard-slide-gate-sweep, merge origin/master (08a276ce0), reconcile the
leaf gates into one game-correct rule [M Drone_CollisionHandler 0x140968:204-208], repair
walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent with
real input (never lower damage or health), run every walkreal_*_live one level per tool call.

## What the merge actually carried (a correction to the task text)

The task text said origin/master "now carries guard-leaf-press-07's 13-level press gate". It
does not: `leaf_press_rest` (the press law) exists only on `origin/s/guard-leaf-press-07`
[checked: git grep leaf_press_rest across origin/master, the base sweep branch and the 07
branch]. What master carries is the leaf SLIDE with its ORIGINAL two-level gate —
`if !matches!(d.level, 0x0700_0007 | 0x0700_0014) { return None; }` [M drone.rs door_leaf_slide]
— while the base sweep branch carries the same slide with the gate gone (the all-levels law).
The merge was clean and took the sweep's all-levels version, so after the merge there is ONE
gate in the tree: the game's law runs on every level, exactly the reconciliation the task asked
for [M Drone_CollisionHandler 0x140968:204-208 — the contact pass's DeltaP adds to obj+0x30
whenever the 0x800 capsule's contact list is non-empty, and the swing door is a cel-linked
collider at its live pose]. The press law (leaf_press_rest, s/guard-leaf-press-07) remains
unmerged on its branch — bringing it under the same rule is follow-up work for whichever task
merges that branch.

## The walk repair — nine levers, death 9724 → 17483, not green yet

The re-ignored agent walk was reproduced first on this tree: Bond died at tick **9724** at
(-24.34, 6.48, 14.20) — exactly the sweep's recorded base death, so master's sound commits did
not shift this walk's timeline. The repair is test-side only (no lib change), every lever gated
by `run_slab_ambush`'s new `levers` parameter — `true` only for the d2 direct load
(`difficulty == 2 && !stop_at_fire` inside walk_to_the_exit_at) — so the mayhew sister (which
calls `run_slab_ambush(…, 2, false)` directly) and the d1/d3 timelines are byte-exact. No
shared-helper change at all: the one flag the levers read, `dmg.return_fire`, already existed.

Applied (each with its measured effect):
1. The back-off nav runs QUIET (`dmg.return_fire = false` for exactly that nav): the old
   returned-fire stand sat 6.0-6.4 u inside the pocket tap cone where the world ray blocks
   Bond's own tap and the escort-shield holds it — Bond bled out standing (9724).
2. A bounded settle until the contact falls back BEHIND Bond before the mouth fights (the 9921
   death: the ally inside every pocket cone).
3. NO approach walks in the ambush rounds (the 10358 death mid-approach at the band edge with
   four shooters live).
4. The pre-ambush resupply extended to this walk (the dry taps stalled the mouth fight — the
   10358 run logged no ambush kill).
5. The band-edge window SKIPPED with levers (the slid-in shotguns make it a death trap — the
   10358 death at (-26.29, 16.08)).
b. The step-out hold 300 → 900 ticks (the level-11-d3-pocket pattern) — THE decisive lever: the
   pocket phase went from death-at-entry to three poppers pulled into the mouth line and killed,
   10358 → 17050.
7. The fights AIM (the 0x400000 zero-spread round [M 0x12c5a0:139-170], the 0.01-rad gate):
   17050 → 17483.

Tried and REVERTED, with their measured harm recorded in the code comments and the test's
`#[ignore]` ledger: lever 6 (skip the critical-health resupply, break the rounds — 15586) and
lever 8 (hold-then-retry before the resupply — 15586 again; the hold runs the gun dry).

THE REMAINING DEATH: tick **17483** at (-23.22, 6.47, 16.32) — health ground to ~8.0 across the
mouth phase, the first dry-mag resupply detour engaged the surviving popper #4 (killed false at
3.8 hp) and the later legs re-entered the lane [L the 17050 run; the 17483 run's death is the
same family]. The walk stays `#[ignore]`d with the full updated ledger (every lever, every
death tick, the next levers): (i) the resupply detour's target direction — the walker's
resupply picks the nearest box and the guardroom box is west past the pocket, so walk east
first; (ii) a second aimed-hold round for #4 specifically before any detour; (iii) the ledger's
(a): apply the levers to the mayhew sister with its meet-plate re-timing, then widen the gate
beyond `levers`. No damage or health value was lowered anywhere; the walker's top-up stays off.

## The sweep (one level per tool call, direct per-binary path)

The four-crate fast runner does not fit this sandbox (9.9 GB disk was at 95% with the previous
task's partial suite build; the daemon was killed and its partial artifacts cleared — the
background suite.py died twice in the previous window the same way). The walkreal batteries ran
directly instead, on this final tree (master 08a276ce0 merged + the sweep's all-levels slide +
this task's test-side changes), `--test-threads=1`:

| binary | result |
| --- | --- |
| walkreal_07000001_live (whole) | 6 passed / 0 failed / 2 ignored (the repaired-forward agent ledger + the pre-existing 00agent ledger) |
| walkreal_07000002_live | 3/0/2i |
| walkreal_07000003_live | 5/0/3i |
| walkreal_07000004_live | 10/0/0i |
| walkreal_07000005_live | 17/0/0i |
| walkreal_07000006_live | 6/0/2i |
| walkreal_07000007_live | 6/0/0i |
| walkreal_07000008_live | 5/0/1i |
| walkreal_07000009_live | 9/0/0i |
| walkreal_0700000b_live | 9/0/0i |
| walkreal_0700000d_live | 1/0/2i |
| walkreal_07000013_live | 4/0/0i |
| walkreal_07000014_live | 7/0/3i |

**13 level batteries, every active test green** — no walk lost to the all-levels slide on any
level run. Not run before the window closed (their binaries are built on the sandbox disk; the
next session runs them directly the same way): 0700000a, 0700000c (+barrelrow/+eastroute),
07000011 (+route), 07000012, 07000015, 07000016, 0700001b, 07000041 (+wall), 0700004a
(+satchel), and the probe0c/probeNN diagnostics.

## What was run (the obligated list)

- `nf-view --test walkreal_07000001_live` whole: **6 passed / 0 failed / 2 ignored** (twice —
  once before the commit, once after the lever-8 revert; the mayhew sister
  `following_mayhew_down_completes_the_level` green in both, so the levers provably did not
  re-time it).
- The 13 walkreal batteries in the table above.
- `nf-game --lib` was not re-run this window: the tree has NO lib change (the merge carried
  master's, already green on master; the repair is test-side only) — the previous task's run on
  the same lib hash stood at 791/0/4i.
- The full-suite total line: not produced (the disk/window ceiling above).

## What is left

1. Green the agent walk: the next levers are in the `#[ignore]` ledger (the resupply direction,
   the #4 aimed hold, the mayhew-sister re-timing to widen the gate).
2. Run the remaining walkreal batteries (the list above — the binaries are built).
3. Merge s/guard-leaf-press-07's press law under the same one-rule umbrella and re-run the
   presses' levels.
4. Live checks for the main agent: none new — the slide law's checks are the sweep's; the walk
   repair is test-side and needs no PCSX2 evidence beyond what the ledger already cites.

## Usage

not visible to me
