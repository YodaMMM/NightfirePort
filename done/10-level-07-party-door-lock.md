## Your task: `level-07-party-door-lock`, 07000007: the door back to the main party locks after the story points

In the game, after certain points on the party level the door back to the main party room locks; in the port it stays usable.

1. Find the door(s) between the party room and the areas beyond [D the 07000007 door placements, their lock/key channel params] and what locks them [M — the switch channels the level's scripts/triggers/mission records write at those story points, `Door_*` lock handling (the locked state, the "locked" message/sound)]. List the story points that lock (and any that unlock) and which channel each writes.
2. Port any missing writer or lock handling, with real-input tests: before the point the door opens; after it, it refuses (locked state, the game's message/sound); sister: other doors unaffected.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-party-door-lock`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T02:38Z

---
## Report (done, 2026-10-25T03:40Z)

Closed by the driver: the report is on `s/level-07-party-door-lock` (docs/reports/level-07-party-door-lock.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-25T04:23Z)

# level-07-party-door-lock — report

Task: on the party level (07000007) the door back to the main party room locks after the story
points in the game; the report said the port keeps it usable. Find the doors and the channels
that lock them, port any missing writer/lock handling, and pin the behaviour with real-input
tests.

## What was found (the mechanism, all evidenced)

1. **The doors [D placements census, `census_07000007`]**: 15 type-0xDB placements (the level
   doc's old "18" was wrong; §8 already corrected the count). Free E-doors (p3 = 0):
   #242–#248. The ch-200 locked set (p3 = 200 = the lock channel [M Door_Create 0x136100:27 →
   ext+0x68]): #252/#253/#254/#255/#256/#304 (+ the slide #342, hit sound 0xffff) — locked from
   boot, the level-boundary shape (U-044): ch 200 has NO writer on this level [D — the script
   lib's only channel event is below; no trigger's out is 200; no writer family carries 200
   (docs/systems/switch-channel-writers.md)].
2. **The story door**: **#251 "Door_Wooden_8" (85.55, 43.00, −35.12)** — starts-open (p8|p9 =
   1|36 → flags 0x25: spline | starts-open | keep-open), open channel p2 = **130 (0x82)**, lock
   channel p3 = 200, hit sound 320. [M Door_Create 0x136100:96-99] creates it OPEN and forces the
   open channel on — ch 130 reads 1 from the first tick.
3. **The story point that locks it**: the hall walk-in TouchOnce **#412** (94.27, 43.61, −45.92,
   out ch 0x48) fires **SP #420 — script 0x06000073** (the control-taking cut-scene, streamed from
   pack 07800007.bin [M lib.rs's loadable-pack pass; SP_LoadScript 0x1dbc08]). Its **stream 8
   carries the level's ONLY script channel write: event 0x0D [0x82, 0] — ch 130 ← 0** at script
   tick 4326 (the script's exact length tick) [D stream decode; M Script_EventHandler 0x1d94c0
   case 0xd `switch_channels[p0] = p1`]. The writer lives in the PACK, which is why the old
   "no disc writer" scans missed it (they read the level placements only) — the level doc's
   "inert lock" note is disproven and rewritten.
4. **The lock [M]**: the open-channel fall closes #251 (Door_Update 0x1363c8's channel arm);
   Door_SetState 0x135618:26's state mirror bounces ch 130 back to 1 until the leaf arrives at
   CLOSED. The door is then CLOSED with lock channel 200 and nothing ever writes ch 200 — so
   [M Door_Activate 0x135be0 LAB_00135cf8] every later E press is refused with the hit sound
   (320). The locked feedback is the sound only: a locked door gives no text hint at all
   [M Door_DoMsg 0x135f18:20-24's lock gate], and the "Door Unlocked" message (0x2000003) belongs
   to ch 200's rising edge [M 0x1363c8:88-107], which never fires on this level.
5. **Story points that unlock: none on 07000007** — ch 200 never rises [D]. The unlock site
   (rising edge → message + auto-open + the one-shot field clear) is ported and stays covered by
   the existing door tests.
6. The old docs disagreed on whether placement param 3 reaches the lock field: docs/fields/
   door.md's creator lines + the 07000005 [L] ext reads (200 in ext+0x68) settle it — p3 IS the
   lock channel; U-044's "beyond the door's channel fields" phrasing and the
   switch-channel-writers 0700000d row's older mapping are stale on that point.

## What was ported

**Nothing needed porting — the symptom does not reproduce on master `c64b2b80`.** The whole
chain (pack-script load → the scene's event 0x0D write → the channel-arm close → the E refusal →
the no-hint gate) already exists in `nf-game` (door.rs's `ch_lock` handling, `nis.rs`'s event
0xD/0x11, the loadable-pack pass) and works through the real path. The user report presumably
predates the pack-script loading (before it, the cut-scene never ran in the port, so ch 130 never
fell and #251 stayed open — exactly "stays usable"). This branch changes **no product code**: the
deliverables are the census, the real-input pins, and the corrected docs.

## How it was verified

- **`party_door_lock_07000007_live` (new, both green, real input only — no teleports):**
  - `the_party_corridor_door_locks_at_the_control_scene`: #251 created OPEN with ch 130 on [D
    flags; M 0x136100:96-99] → the walkreal route → the plate #412 fires ch 0x48 → the scene
    (0x06000073) runs to its end → ch 130 falls → #251 arrives CLOSED → three E presses refused
    (state stays 0x02; ch 200 stays off; the lock field stays armed — the one-shot unlock edge
    never fired [M 0x1363c8:106 clears it only on the edge]).
  - `sister_the_free_doors_still_open_after_the_lockout`: the same walk, then the free hall door
    #244 (p3 = 0) still answers the use key (state moved 0x04 → 0x10) — a locked door would
    freeze (Door_Activate returns before any state change).
- **The story point is unavoidable on the real route** [D]: the plate #412's box (x 93.31..95.23,
  z −46.43..−45.41) IS the hall→north passage — the probe's plate-free skirt (x 92.4) walks off
  the hall's north ledge and falls (probe_07_route). So the door's pre-story state is pinned
  standing OPEN through the whole walk up to the plate, and the passage itself is never crossed
  before the scene.
- **Crates run**: `nf-view` (the new tests + the census/probes; the whole 07 test family:
  walkreal_07000007_live 2/2, interact_07000007_live in the running suite).
- **Fast suite** (`tools/test/suite.py`, started 02:47 UTC on this branch's tree — the product
  code is identical to the final tree since the branch adds only tests/docs): ran to **779 of 791
  binaries** and was stopped at the session's deadline (the last ~12 binaries are the w-tail
  walk probes). Partial tally: 27 red entries, ALL the sandbox's known environmental families —
  the 3 allowed `disc_golden` (slim work/), `identity_disc` (the onfoot ISO's 518,435 sectors vs
  the full retail 1,191,072 the test pins), the 3 GPU-render tests (no `/dev/dri` — wgpu adapter
  NotFound), and OOM SIGKILLs (exit −9) among the `net_*`/`mp_*` stress binaries and two walk
  probes — the same set a full run of the same tree family produced yesterday (23 reds there; the
  OOM count varies with memory pressure). Zero failures in any binary this branch's files touch:
  walkreal_07000007_live 2/2 green, interact_07000007_live green, and the new
  party_door_lock_07000007_live 2/2 (run directly).

## What's left

- Nothing blocking. If the main agent wants visual proof of the closed door, render
  `--bond 85.5,44.1,-37.5,0,0` (the door #251 leaf shut in the south wall) vs the pre-story
  `--bond 85.5,44.1,-33.0,0,3.14` (the doorway open from the north) — but the state pins above
  already carry the behaviour.

## Live checks for the main agent

None required. Everything pinned is [D] disc bytes / [M] decompile addresses read directly. One
optional confirmation if a fresh-boot PCSX2 session happens to be open: watch `switch_channels+
130` (0x2a38d2+130) while playing through the hall walk-in — expect 1 → 0 at the control-taking
scene's final tick, and door #251 (85.55, 43.0, −35.12) to swing shut and refuse E afterwards.

## Usage

not visible to me
