# level-0b-objective9-writer — 0700000b: objective 9 completes on the warm walk

Branch `s/level-0b-objective9-writer` (from origin/master 7d0091a0; final tree merged with
origin/master c32c53c6 — merge commit d94cf499). Subject: `walkreal_0700000b_live.rs` +
`docs/levels/0700000b.md`. No `nf-game`/`nf-formats`/`nf-audio` source changed.

## Result

**Objective 9 now completes on BOTH walks through the game's own writer** — the warm
walkthrough's `switch_channels[0x08]` poke is DELETED. Both walks walk the proven ring legs
from the Q-Worm desk back to the terminal side and into the dome, and E-fire the **spawned
files use-plate**: the terminal script 06000035 stream 8 spawns at its tick 0x1e5 the type-0xEA
TouchOnce (hash 0x2000ab5, params [6, 4, ..] = out ch 6, mask 4 = use) [D nis dump of
0700000b.bin; M Script_CreateEntity 0x1d8968 case 0xEA → Trigger_TouchOnce] — ch 0x06 →
OR #295 → ch 0x4E. A new sister assert pins objective 9 OPEN at the writer (ch 0x06 and
ch 0x4E off before the E fires).

**The ch 0x75 item resolved as NOT a port gap**: the roof cut-scene's event-6 re-place key
[−14.71, 58.00, −2.06] [D pack 0790000b] is 0.18 m OUTSIDE lift_stop_trigger #245's own box
(z −1.88..2.15 [D trig dump]) — the level doc's "INSIDE #245's volume" claim was wrong per the
disc's own trigger box and is corrected. The port's parsed 12-tri glist matches the box bounds
exactly; a probe on this tree shows `touched_by` = false at the key and true 0.4 m inside the
mesh — the touch geometry works; the key just is not in the volume. #245 is the shaft-top stop
a lift ride crosses, not something the cut-scene's deck placement should fire.

## The writer census (every writer of ch 0x54 / ch 0x06 / ch 0x75)

- **ch 0x06**: exactly ONE writer — the script-spawned TouchOnce above [D nis dump; M 0x1d8968
  case 0xEA]. Sweeps that found nothing else: the trigger census (60 rows, `nf-extract triggers`
  — no out 0x6), every Event-0xD in all of 0700000b.bin's scripts (the writes are 0x77/0x62/
  0x09/0x0a/0x0b only), both pack scripts (060004FA writes 0x57/0x77/0x59/0x2a/0x76; 060004F9
  writes none), and the decompile (no constant `switch_channels[6]` store) [D+M].
- **ch 0x54**: NO writer on 0700000b, either pack, or 0700000a (the same sweeps + 0a's trigger
  and script dumps). OUT #294 mirrors it into ch 0x08 and OR #295 folds ch 0x08 into ch 0x4E,
  but nothing in the level's data ever raises it — the arm is writer-less; the old warm-walk
  ch 0x08 poke modelled a writer that does not exist [D].
- **ch 0x75**: ONE writer — #245 (type 0x2 Touch, mask 8 player, box [−18.90, 56.76, −1.88]..
  [−9.08, 63.80, 2.15], 12 tris) [D trig dump]; reader: the lift SP #232's mode-8 gate [D+M].
  The re-place key is outside it (above), so ch 0x75 staying off at the deck key matches the
  disc's own placement data.

**Which writer the warm route hits**: the spawned use-plate — the warm route is the same
physical route (the shared walkthrough legs), the plate spawns at the keycard chain (terminal
script tick 0x1e5) before the player ever reaches the desk, persists (its stream has no
EntityEnd before the script end [D nis dump]), and is the only ch-0x06 writer.

## The ch 0x75 probe (scratch, deleted before commit)

A probe test loaded 0b, printed #245's 12 world tris (they match the placement box bounds
exactly), and measured `touched_by`: false at the deck key (z −2.06), true at z −1.5 (inside
the mesh). Runtime polling for 3000 ticks with the real 060004FA script played through the SP
machinery: the re-place lands at (−14.71, 58.13, −2.06), ch 0x75 never rises — consistent with
the geometry, and Trigger_Calc is not gated during the NIS (the toucher is built from
player.pos every tick [M drone.rs tick_inner]).

## Test changes (`walkreal_0700000b_live.rs`)

- The obj-9 section is now ONE path for both walks: the proven walked legs from the desk
  (files n26→n25→n22→n21→n20→n16, then n15→n14 — the reverse of the previously proven
  terminal lane), `try_walk_to` the terminal room from n14 and the plate's walk target. Both
  walks print "the files approach: terminal room true plate walked true" — the dome walk to
  the plate WALKS (the old fresh arm's direct desk→plate attempt wedged 37 m short at the
  office doorway; the nav legs fix it, and the run got ~4900 ticks faster: 16676 vs 21536).
- The E-hold stance loop stays as the documented interaction mechanism (≤ 0.6 m around the
  walked spot), the sister assert (ch 0x06/0x4E off) fires before it.
- The roof-gate return simplified to the single terminal lane (both walks end the files
  section at the plate).
- The ch 0x75 observation print re-commented with the [D] box facts (kept observational —
  whether the DISC raises ch 0x75 for a player who walks from the deck key into the shaft-top
  volume is a live check below; nothing in the mission reads it).

## Live checks for the main agent

1. **ch 0x75 at the shaft top** — fresh-boot PCSX2, play 0b to the roof cut-scene (or warp:
   Bond at the helipad deck key [−14.71, 58.00, −2.06] after the cut-scene). Record
   `switch_channels[0x75]` (a) the moment the re-place lands, (b) after walking NORTH ~2.5 m
   into z > −1.88 (into #245's box). Expected if the port is right: (a) stays 0, (b) rises to 1
   (the touch contact once the capsule enters the box). If (a) reads 1 on the disc, the game's
   trigger contact is bounds-sphere-wide (the port's tri/AABB law is the gap).

## Verification

- `walkreal_0700000b_live` **3 passed / 0 failed** (21.33 s) on the final merged tree —
  the warm walkthrough, the fresh-spawn walkthrough, the roof cut-scene sister test; both
  walks complete objective 9 through the real writer and reach MissionState 6 +
  `LevelToEndTo` = 0x0730000b.
- `interactables_0700000b_live` **9 passed / 0 failed** (16.65 s) — the sibling 0b binary,
  regression check.
- Crates touched: **nf-view only** (a test file) + docs. No nf-game/nf-formats/nf-audio source
  changed; their coverage is the suite's.
- Full suite (`tools/test/suite.py`): the sandbox kills background processes between tool
  calls (three starts died with an empty output file), so it ran in foreground chunks; the
  merged tree's rebuild consumed the first chunk. Partial total quoted below.

## Suite total

(filled from suite.out before the finish push — see the final commit)

## What's left / open items

- The ch 0x75 live check above (the only remaining unknown; nothing objective-bearing reads
  the channel on 0b).
- Pre-existing, unchanged: the waiting-room exit transit stand-in and the parachute-stance
  sweep (level-0b-teleports-to-walks' live checks 1/2); the boot-loadout Q-Worm grant gap.

## Usage

not visible to me
