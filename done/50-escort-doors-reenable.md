## Your task: `escort-doors-reenable`, the 01 escort's interior-doors test, re-enabled

`fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route` is ignored on master ("walker resupply deadlocks door #2 with the escort's DSTATE_OpenDoor (harness, not the port)"). `origin/s/escort-door-harness` (not merged) showed it passes un-ignored on master alone — but with the other changes merged since (the FindOpponent alive filter and LOS budget, the stuck-guard side-step probe along the guard's yaw), it fails at line 249.

1. Un-ignore it on current master and run it; if it fails, find which change moves the escort or the walker and whether the port or the test is wrong [M], fix that, and keep it un-ignored. Report the line and the cause.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/escort-doors-reenable`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox7 at 2026-10-26T12:50Z

---
## Report (done, 2026-10-26T14:04Z)

# Report — `escort-doors-reenable`

Branch `s/escort-doors-reenable` (from origin/master 7e35bd77), 2026-10-26.

## What's done

`fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route` is **un-ignored on
current master and kept un-ignored** — one commit, touching only the test's attribute and
comment (`crates/nf-view/tests/fix_npc_doors_live.rs`). No port code changed.

**The reported failure does not reproduce.** Measured green four times on master 7e35bd77
(three pre-report runs + once on the final tree), tick-identical every time — the sim is
deterministic, so this is the tree's stable behaviour, not luck:

- door #1 OPEN at +9542 ticks with bond→contact 4.7 (the escort's doing, assert 2 holds)
- door #2 OPEN at +54215 ticks with bond→contact 16.3
- the escort settles at (−20.05, 6.34, 14.60) — past x −15 (assert 1 holds), no closed door
  within 2.5 of him (assert 3 holds), 44.5 s wall

## The line and the cause

The driver's "fails at line 249" is `assert!(bond_d1 > 2.5, "door #1 opened with Bond too
close …")` — line 249 in the plain un-ignored file (line 258 on my branch, after the new
comment block). The assert is right as written: it pins the escort's own door handling
(DSTATE_OpenDoor → Door_Activate) by requiring Bond to be outside use reach when door #1
opens. No test change, no port change.

Both changes the driver named are in current master, verified in its history: the
find-opponent-exact merge `b4cdd507` ("the exact alive filter and the shared LOS budget in
the team-2 opponent scan") and the stuck-guard merge `1c5e71f8` ("the combat-move choice
now consults the mover's own test (probe_move)"). They **did move the escort's route**: the
harness branch's green run on master 26543436 recorded door #2 OPEN at +30962 (bond→contact
10.6) and the escort past the doors at +36031, ~21 s — current master takes ~2× the ticks
(door #2 at +54215, 44.5 s). But on 7e35bd77 all three can-fail asserts hold. Between the
driver's observation and 7e35bd77, escort-touching merges landed (e.g. `8a6b9f48` "the
escort's stall recovery gains the player's stick moves", `4ba2c194`'s rook-door work with
"escort cover/follow/hide_findrun: 22 passed") [S — one of these likely accounts for the
difference from the driver's observed state; no bisect was run, see What's left].

## How it was verified

- The test itself, 4× (above), the last one on the exact pushed tree.
- The escort/door/AI neighbourhood on the same final tree, each binary run per the runner's
  protocol (`--test-threads=1`, `NF_SILENT=1`, `NF_ISO` set): **npc_doors_live 6 passed /
  1 ignored; escort_cover_live 3; escort_follow_07000001_live 1; escort_hide_findrun_live 3;
  walkreal_07000001_live 3; find_opponent_live 3; stuck_guards_sweep 1; doors_live 2;
  doors_01_live 2** — 26 passed / 0 failed / 1 ignored, plus this task's own 1. All green.

**The full suite did not complete in this sandbox — disk, not the tree.** The first runner
invocation was given a wrong positional target-dir argument (`target/release`), which made
cargo nest duplicate build trees inside the real one (a 6.8 GB copy, then a further 1.7 GB
one) and filled the 9.9 GB disk twice — the "4 nf-view test binaries failed to compile" it
reported were disk-full symptoms (the same four build clean on the real target). Recovery
(registry re-fetch, relinking) hit the same ceiling. The runner's build step needs ~2 GB of
headroom the disk cannot offer after the ISO and the rebuilt tree. Standing in for it: the
per-binary runs above (the runner's own protocol), covering every family the un-ignore
could plausibly regress — the test's crate nf-view's escort/door/npc/walkreal families, and
nf-game's AI/door cores including `find_opponent_live` and `stuck_guards_sweep` (the two
named merges' own test binaries). nf-formats / nf-audio were not run: their binaries did
not survive the disk recovery and relinking them did not fit; this task's diff (a comment
and an attribute) cannot affect them.

## What's left

- Nothing for this task's goal — the test is un-ignored and green on master.
- [S] If the driver wants the exact commit that ended the line-249 failure, a bisect over
  `26543436..7e35bd77` (harness-green point → current master) with the test un-ignored
  would pin it; each step costs an nf-game/nf-view rebuild + a ~45 s run. Not run here: the
  sandbox disk could not hold the intermediate build artifacts.
- The full four-crate suite on hardware with headroom, as always before a merge.

## Live checks for the main agent

None — no port change to check. The test's asserts already pin the game-facing behaviour
(the escort crosses both interior doors, opens them himself, never settles against a
closed one).

## Usage

Token, credit, context-window and cost figures for this session: not visible to me. The
environment I run in (a CLI worker behind an IM gateway) exposes no token/credit/context or
cost readout anywhere I can read — there is no UI panel in the sandbox and no such fields
in the message metadata. Nothing is estimated here.
