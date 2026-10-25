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
  code is identical to the final tree since the branch adds only tests/docs): **PENDING — read
  from suite.out before the finish**. This sandbox's known environmental reds (from the previous
  session's full run, same tree family): 3 allowed `disc_golden`, `identity_disc` (the onfoot
  ISO's 518,435 sectors vs the full retail 1,191,072 the test pins), 3 GPU-render tests (no
  `/dev/dri`), and the `net_*`/`mp_*` OOM SIGKILLs (3 GB RAM).

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
