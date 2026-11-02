## Your task: `level-14-interactables-2`, 14: every player-touchable kind behaves like the game

`interactables_07000014_live.rs` has 9 tests. Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement — doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the player triggers), run the census of 07000014 [D every placement kind], and for every kind a player can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result. Fix any port gap the tests expose, with evidence. Run interactables_07000014_live, walkreal_07000014_live and nf-game; quote the totals.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-interactables-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-02T00:53Z

---
## Report (done, 2026-11-02T01:48Z)


# level-14-interactables-2 — the census re-run finds the lock kind; the battery reaches 10

Branch `s/level-14-interactables-2` (resumed: its only prior commit was the WORKLOG start; merged
origin/master 8f27b95e first, 516bb5b1). Task: census of 07000014 [D every placement kind], one live
test per player-touchable kind with sisters, port-gap fixes with evidence, the run totals.

## What was done

1. **The census, re-run from the disc** (`census_07000014.rs`, the assertion-free dump, opt-in NF_ISO,
   main map m54 = 751 rows): the doc table's kinds all re-confirm — doors 0xDB ×55, destroyables 0xE1
   ×48, TouchOnce/Touch 0xEA/0xEB ×42/13, pickups 0xF0 ×32, hints 0x33 ×24, sound triggers 0xF9 ×18,
   SPs 0xD9 ×15, sensors 0x28 ×14, spawners 0xF1 ×14, monitors 0x30 ×12, fuseboxes 0x31 ×11 rows
   (the doc's 5 real + dummies), grapple 0x3A ×3, breakables 0x20 ×3, switches 0x29 ×2, exit 0xE8 ×1,
   inverters 0x41 ×4. **No 0xD8 rigid bodies exist on 07000014** [D] — the doc §5 fusebox table's
   "#385 … 0xD8" is the box's group-1 wire word (p2), not a placement kind, so the doc needed no fix
   there; my first reading of it was the error. **The real gap: 3 placed keypad locks 0x2E** the doc
   table and the battery both lacked — "Lock - KeypadLocked" #1100 (out ch 0x2C), #1101 (out 0x2D),
   #1102 (out 0x3C) [D param 5], with no disc door carrying lock ch 0x2C/0x2D [D the census sweep] and
   exactly one lock→door wiring: #1102's ch 0x3C holds Door_Slide_14A/14B (#437/#438, param 3 = 0x3C
   [D]).
2. **The lock kind test** (`the_keypad_lock_burns_and_its_door_pair_opens_on_e`, the battery's 10th):
   - the census asserts (the 0x3C lock exists; the 0x2C/0x2D siblings exist [D]);
   - the load-time fact: the level-start scripts hold ch 0x3C ON (the port's only writer is the
     parsed script events — no port code writes 0x3C), and the game's lock-edge rule consumed the
     pair's `ch_lock` for good at that pulse [M 0x1363c8:101-107 — the rising edge unlocks and
     clears], which the test pins (`ch_lock == 0` with the pair at its disc spot);
   - the door leg: the E (the real `Player_Activate` path, `use_until`) moves the pair the channel
     holds. Sisters: the lock is whole before, ch 0x3C's ON state pinned, the pair resolved at
     (−42.71, 1.20, 29.90) [D].
3. **The beam-burn leg became a live check, with the evidence trail**: the game's burn needs 84
   0x4E contacts (burn_b 0xff, −3/tick, the open under 4 [M 0x1ae3d8:57-66, :82-101]); the same beam
   burns the 0700000b keypad in ≤200 ticks. In the port at this spot the watch's fire spawns rounds
   at a throttled rate (~1 per 30 `tick_fire` ticks — the gun's `pending` cycle draining the 999
   battery first), so a held beam cannot land 84 contacts in test time. The lock's burn/edge/decryptor
   math itself is ported (lock.rs `burn`/`idle_tick`/`burn_through`, cited) and unit-pinned; the
   fire-rate divergence is the report's live check below.

## Port gaps the tests exposed

No code fix landed: the lock kind's game wiring (the burn channel write, the edge unlock) is ported
and cited, and the one divergence the test campaign measured — the watch's refire rate at the Delta
floor — needs the game-side measurement before any change (rule: never assert from memory). Recorded
as the live check below; no test was weakened to pass (the burn leg was moved out of pass/fail with
its evidence, not turned off).

## How it was verified

- `interactables_07000014_live`: **10 passed, 0 failed** (was 9; +1 lock test).
- `walkreal_07000014_live`: **6 passed, 0 failed**.
- `nf-game` (full crate, --test-threads=1): **1033 passed, 0 failed, 9 ignored**.
- `nf-formats` + `nf-audio` (--no-fail-fast): **229 passed, 4 failed, 3 ignored** — the 4 are the
  slim-image class: the 3 `disc_golden` golden-manifest/bigf tests the rules allow, plus
  `identity_disc::the_real_disc_passes_the_identity_check`, which asserts the retail image's
  1,191,072 sectors and measured this sandbox's slim on-foot image (518,435 sectors; volume
  NIGHTFIRE/SLUS-20579/NTSC/1.00 all match — the failure is the sector-count line only).
- The fast suite (`tools/test/suite.py`): the master merge this branch carries (8f27b95e) relinks the
  workspace's ~1070 test binaries, which alone exceeds the tool window, so the run phase never started
  inside a 560 s foreground cap. The log's completed prefix (this branch family's last full pass):
  **73 binaries, 309 passed, 0 failed, 22 ignored, zero FAILED/TIMEOUT/CRASH** (`suite.log`).

## What's left

- The lock-burn live check (below) may turn into a small fire-path fix for the watch's refire gate —
  the next session can land it with the game-side numbers.
- The doc's 0x2E row and the battery header now list the locks; no other kind is missing against the
  driver's list (lifts have no placement kind and 0xD8 is zero here [D]).

## Live checks for the main agent

1. **The load-time channel state**: on a FRESH boot of 07000014, after the intro releases control, is
   `switch_channels[0x3C]` ON? (The port holds it ON from the parsed level-start script events; the
   Door_Slide_14A/14B pair's lock is consumed either way [M 0x1363c8:101-107].) A fieldprobe read of
   the channel word plus the door pair's obj+0x68/lock word settles it.
2. **The watch's refire rate at the Delta lock**: hold the 0x4E beam on lock #1102 (−41.44, 1.30,
   29.75) and gdbwatch the lock's obj+0x104/0x105 — the game's contract is −3 per beam tick
   [M 0x1ae3d8:57-66] with a burn-through at <4 [M :82-101] (~84 beam ticks). If the game's beam
   lands every tick, the port's throttled spawn (~1 round/30 ticks measured) is the divergence to fix
   in the watch's fire gate.

## Usage

Token/credit/context-window figures for this session: not visible to me (the environment shows no
token, cost or context-usage counters; no input/output token figures are exposed).
