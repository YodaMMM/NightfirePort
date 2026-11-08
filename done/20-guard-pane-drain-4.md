## Your task: `guard-pane-drain-4`, the 07000001 walks stay green with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-3, merge current origin/master, and read its docs/reports/guard-pane-drain-3.md plus the 01 d3 ignore message. Take the d3 walk's next lever from that message: ammo at the door phase, then the armed sustained lure. Then run the sweep levels it could not finish (04, 07, 0c, 0d, 13, 14, 15, 16 and 1b) one walkreal_*_live binary per tool call, building with -j2 because -j4 ran the 4 GB sandbox out of memory. Repair any walk that shifts, gated to its level. The 08 Hind red belongs to the level-08 branches. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the bullet binaries, glass_pass_every_level_live, dead_pane_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T20:47Z

---
## Report (done, 2026-11-08T21:36Z)


---
## Report from the worker (done, 2026-11-08T21:36Z)

# guard-pane-drain-4

## What was done

Task: the 01 d3 walk's next lever (ammo at the door phase, then the armed sustained lure), then
the sweep levels drain-3 could not finish (04 07 0c 0d 13 14 15 16 1b) one walkreal_*_live per
tool call, repairing any shift gated to its level; the bullet binaries,
glass_pass_every_level_live, dead_pane_live and nf-game; the fast runner started in the
background at session start. Branch `s/guard-pane-drain-4` cut from `origin/s/guard-pane-drain-3`
(6a37c12bc) and merged `origin/master` (b059df948, clean).

### The d3 ammo lever: implemented, and the ledger corrected with [L] evidence

The lever from the drain-3 ignore message is in: a d3-gated **walked detour** (`d3_ammo_detour`,
inside walkreal_07000001_live.rs only — no shared-helper change), wired into the named loop's two
dry branches (the dry mid-hall block and the aperture-lure block): when the corridor resupply
returns false, Bond walks the off-path leg to the boxes the ledger named at
(-7.5,5.3,-6.8)/(-3.3,5.8,-9.1) on real input, returns to the hold line, and re-runs the walk's
own resupply (whose trail-based corridor rule now admits them). The helper also dumps every live
pickup when the spots are empty — that dump is what corrected the ledger:

1. **The corridor rule does NOT refuse the named boxes.** The pre-hall stock's own attempts
   consumed them: "d3 pre-hall stock 1: resupply attempt 0 at (-7.5,5.3,-6.8) (d 7.0)" /
   "attempt 1 at (-3.3,5.8,-9.1) (d 5.1)" [L the run logs] — the legs reached pickup radius and
   the boxes went `gone`; resupply's held-gun ammo measure (`clip + spare` of the gun in hand)
   missed the pool they fed, so the attempts looked dry.
2. **At the door phase the whole level holds exactly two live pickups** [L the detour dump, run
   on all three named guards]: the east-wing box (17.4,6.2,-3.2) — the detour to it is forbidden
   by the walk's own ch-0x05 wake law (it wakes WAIT_SWITCH guards 10/16 into ATTACK and stalls
   the contact) — and the west-hall box (-14.4,6.5,16.1), across the aperture. Both are kind 3,
   **weapon 0x47, 50 rounds — a weapon the wheel does not own**.
3. **The wheel is dry end to end**: 12 `switch_to_armed` cycle ticks all print the fists 0x1 —
   Bond holds the FISTS at the door phase; no guard drop remains live either (`from None` on
   both pickups). [M the port drops guard weapons: `DroneWeap_DropWeapon` 0x17a2a0 implemented
   at drone.rs 5561 with the death-path call sites — the drops were taken in earlier phases.]

So the level is **ammo-exhausted** at the door phase under this walk's spending, and the ignore
message now carries that ledger plus the next lever: **the SPEND** — diet the d3 pre-clear loop
and the pre-hall stock passes (a player who knows the hall holds the AND crowd hoards: the lure
draws the ch-0xE crowd across one at a time and the doorway fight kills them there — the same
kills, later, for a third of the rounds), keeping the wheel armed through the named loop so the
sustained lure still has a gun. The detour machinery stays: it fires at every dry branch and
takes any drop that lands near the spots on a future stream.

The d3 walk stays `#[ignore]`d — it is still red at the AND assert (the three named guards alive
and across at tick 22016, the drain-3 shape). No decompile-backed assertion was touched; the
only changed test was already ignored.

### The sweep: all nine remaining levels green, no shifts

| level | binary | result |
|---|---|---|
| 04 | walkreal_07000004_live | 9 passed / 0 failed (1 ignored) |
| 07 | walkreal_07000007_live | 5 / 0 (1 ignored) |
| 0c | walkreal_0700000c_live | 3 / 0 (1 ignored) |
| 0d | walkreal_0700000d_live | 1 / 0 (2 ignored) |
| 13 | walkreal_07000013_live | 3 / 0 (1 ignored) |
| 14 | walkreal_07000014_live | 7 / 0 (3 ignored) |
| 15 | walkreal_07000015_live | 7 / 0 (2 ignored) |
| 16 | walkreal_07000016_live | 15 / 0 (3 ignored) |
| 1b | walkreal_0700001b_live | 16 / 0 (3 ignored) |

One binary per tool call, `-j2` builds (the task's warning is right: `-j4` OOMs this 4 GB
sandbox). Nothing shifted; nothing needed repair. With drain-3's table the whole sweep named by
the two tasks has now run green (03 05 07 0b 0c 0d 11 13 14 15 16 1b 41 4a here or in
drain-3's report).

### Required binaries

- bullet_range_live: **3 / 0**
- fix_foliage_bullets_live: **3 / 0**
- glass_pass_every_level_live: **3 / 0** (3 ignored — the pre-existing, evidence-carrying level
  geometry ignores; untouched here)
- dead_pane_live: **3 / 0**
- nf-game lib: **748 passed / 0 failed / 5 ignored** (same as drain-3 — no src/ files changed on
  this branch; the diff is the 01 walk test file and docs)

### The fast runner

`tools/test/suite.py` was started in the background in the first minutes and restarted on the
final tree (the branch already includes `origin/master`; no src/ files changed after the
restart). Two sandbox notes, for the next runner of this task:

- The committed suite builds with `-j4`, which OOMs this 4 GB box (drain-3 saw the same). This
  session ran it with an **uncommitted local `-j2` tweak** — the committed file is untouched.
- The ALSA headers come from the user prefix `~/alsa`; the link step needs
  `PKG_CONFIG_PATH=$HOME/alsa/usr/lib/x86_64-linux-gnu/pkgconfig` **and**
  `LIBRARY_PATH=$HOME/alsa/usr/lib/x86_64-linux-gnu` (the dev symlink `libasound.so` is only in
  the prefix; cargo does not fingerprint LIBRARY_PATH, so the cache survives it).
- First restart died to an OOM kill during a build I ran concurrently (the nf-view lib test
  compiles clean standalone — verified). The restarted run was still in its build phase at the
  work-stop time; **no suite total line fits the window this session** — every binary the task
  names was run directly instead, with its pass count above. See "What's left".

## How it was verified

Every number above is a direct binary run on the real ISO (`NF_ISO` pointed at
Nightfire-onfoot.iso, `NF_SILENT=1`), `--test-threads=1`, whole binaries. The d3 lever runs are
real input from the level load — the walk's own idiom, no teleports, no health or damage writes;
the detour only walks and takes pickups through the game's own pickup path. The [L] dump lines
quoted in the ledger are eprintln output from those runs, reproducible by running the ignored
test with `--nocapture`.

## What's left

1. **The d3 walk green** — the SPEND lever above (diet the d3 pre-clear + stock passes, keep the
   wheel armed through the named loop), then the un-ignore when the whole walk passes. The
   post-AND d3 phase has never run past the AND assert, so expect escort-phase work after the
   gate fires.
2. **One clean full-suite pass** on the final tree (the restarted runner was still building at
   the work-stop; the next session should find the build warm and get the total line quickly).
   Every binary the task names has been run directly with its count above; what the runner adds
   is the remaining nf-view/nf-formats/nf-audio binaries in one deterministic sweep.
3. The 08 Hind red belongs to the level-08 branches (per the task).

## Live checks for the main agent

Carried over from drain-3 (still the useful ones), now sharpened by this session's dump:

1. The game's own ammo economy at the 01 hall door on 00 Agent: whether a fresh-boot playthrough
   reaching the door phase still has any armed gun (the port's wheel is dry to the fists) — and
   whether the game's named guards (death channels 0x11/0x12/0x13) drop their weapons on death
   where a player can take them (the port drops at the fight floor [M 0x17a2a0; L drops taken
   earlier in the walk]). Expected result if the port is faithful: the game's player is also
   dry unless he hoarded, which would confirm the SPEND lever is the player-faithful move.
2. The pickup list at the door phase in the game ([L] via fieldprobe at the two live boxes):
   whether the game's boxes at (17.4,6.2,-3.2) and (-14.4,6.5,16.1) are the same kind 3 /
   weapon 0x47 / 50 rounds as the port's, i.e. whether the port's inventory mapping of the
   placed boxes matches the disc's.

## Usage

- Context window / tokens / cost: **not visible to me** — no token, credit or context figures
  are exposed anywhere in this environment; I cannot quote numbers I was never shown.
- Wall clock (the only timing figure visible to me, from the session's own `date -u`): the
  driver message arrived ~20:52 UTC; the sweep ran 22:00-22:12 UTC; this report written ~22:35
  UTC, inside the ~22:52 UTC sandbox expiry.
