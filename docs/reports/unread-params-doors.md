# unread-params-doors — report

Task: for `Door_Create` and its variants, lifts, ladders, moving platforms and script-driven
props (`crates/nf-game/src/door.rs` and neighbours), read every `*_Create` in the decomp, list
every placement param and word it stores, find what the port does with it, census the
port-ignored params over every level, close the zero-everywhere ones with the census and port
the real gaps. Branch `s/unread-params-doors`, cut from origin/master 22d5836b7.

## The classes and their stored params ([M] = the decomp files named)

| Placement | Creator | Params stored ([M]) | Port |
|---|---|---|---|
| 0xDB | `Door_Create` 0x136100 | p0 kind, p1 group, p2 open ch, p3 lock ch, p4 portal bit, p5/p6/p7 sounds (0 → 0xFFFF), p8\|p9 flags, p10 approach, p11 no-E, header +2 keys / +4 rate / +8 anim | reads all [M door.rs create]; the p5-7 0→0xFFFF conversion is the known door.md mismatch #2 (see "Left") |
| 0x22 | `Ladder_Create` 0x1966c0 | p1 → inst+0x02 (no reader anywhere [M docs/fields/ladder.md]) | stores it; nothing reads it in the game either |
| 0x20 | `Break_Create` 0x12c230 | p0 kind, p1 health (int→f32), p2 filter (~p2) | reads all [M breakable.rs] |
| 0xE1 | `Destroy_Create` 0x12c318 | p0 out ch, p1 watch ch, p2 blast kind, p3 health, p4 bullet mask (~p4), p5 blast damage ·0.01, **p6 countdown ·−FRAME_RATE** | read p0..p5; **p6 was unread — ported this task** |
| 0xE2 | `SS_Create` 0x1a9c58 | p0 state, p1 wrap, p2 mask, p3 ch, p4 sound, p5 rest, p6 spline, **p7 → +0x2A cel scratch**, header +2/+4/+8 | reads p0..p6 + header; **p7 unread (see census)** |
| 0x3B | `DynamicObject_Create` 0x18dbf8 | p0 gate ch, p1 action ch, p2 fade radius, p3 mode byte | reads all [M worldobj.rs] |
| 0xFD | `CamSubject_Create` 0x1bc5d8 | p0 channel | reads it [M props.rs] |
| 0x34 | `GunImp_Create` 0x131f20 | only tag+0x28 — inside the parser's memset-0 region, never written [M 0x131f20 body; the decomp is `unaff_`-mangled, the read is a dead 0] | reads no params — correct |

Lifts and moving platforms are doors (kind 1/2 sliders with 0x19 paths — `NDrone2_FindElevatorDoor`
0x14a940 treats doors as lifts) and SS objects (the LIFTDOORS 0xE2 rows); both creators' params are
in the table. No driving levels touched, no draw rework.

## The census [D]

`crates/nf-view/tests/placement_param_census_live.rs` walks all 34 on-foot archives of
`nf_view::dev::LEVELS` (68 main maps incl. the shared ones, 11,865 placements) with `Level::parse`
and counts, per creator-stored param, the non-zero rows. Runs in ~1 s. The table (kind, param,
levels-nonzero / rows-nonzero):

| kind | p0 | p1 | p2 | p3 | p4 | p5 | p6 | p7 | p8 | p9 | p10 | p11 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Break | 15/272 | 8/124 | 6/67 | — | — | — | — | — | — | — | — | — |
| Ladder | (1/1 — never read by the creator) | **0/0** | — | — | — | — | — | — | — | — | — | — |
| DynObject | 5/30 | 10/46 | 2/4 | 2/3 | — | — | — | — | — | — | — | — |
| Door | 16/322 | 9/233 | 4/9 | 18/91 | 20/296 | 21/443 | 21/443 | 21/438 | 21/407 | 21/441 | 1/1 | 2/8 |
| Destroy | 5/25 | 6/31 | 17/175 | 8/95 | 7/60 | 18/180 | **1/17 (07000014, max 10)** | — | — | — | — | — |
| SS | 15/109 | 14/68 | 15/106 | 10/51 | 15/109 | 3/33 | 13/71 | **1/1 (07000046#54, value 1)** | — | — | — | — |
| CamSubject | 3/5 | — | — | — | — | — | — | — | — | — | — | — |

(The full per-row table with max values and first placements prints under `--nocapture`.)

Closures carried by the census:
- **Ladder p1 — zero on every level** (and no reader in the game [M docs/fields/ladder.md's
  sweep]). Nothing to port. Ladder p0's single non-zero row (07000041#108 = 3) is never read by
  `Ladder_Create` (its only tag read is +0x30 [M 0x1966c0:19]) — dead by construction.
- Every other class param is non-zero somewhere and is read by the port (table above), except
  the two gaps below.

## Gap 1 (ported): Destroy p6 — the post-smash countdown

- **Reader trace [M]:** `Destroy_Create` 0x12c318:57 stores `tag+0x0C = (int)(p6 · −FRAME_RATE)`;
  `Destroy_Smash` 0x12bb60:31 negates it; `Destroy_Update` 0x12c4f8:17-25 decrements it while it
  reads positive and, landing on 0, writes `switch_channels[p0]` and reclasses the object to 1
  (the watch arm below only sees leaves whose count is ≤ 0). The bullet handler
  `Destroy_CollisionHandler` 0x12bd48:41-49 fires the channel at once when the count reads 0,
  else flips its sign — and the smash's own negation flips it back, so a FIRST bullet death with
  p6 ≠ 0 leaves the count unstarted (the double flip is in the decompile; the next bullet death
  re-arms it, because the latch keeps class 0x21 and the handler keeps running).
- **Timing source [L]:** the load-time FRAME_RATE reads 30 — the same fresh-load reading the
  doors' `CREATE_FRAME_RATE` pins (door.rs, the 1/30 fallback measured in the snapshot). Scale:
  30 ticks per p6 unit, not the play-time 60.
- **Census [D]:** p6 non-zero ONLY on 07000014 — 17 rows, values 1..10 (max 10). The "inert"
  comment in props.rs held only for 0700000d (p6 = 0 there).
- **Port:** `Destroy.latched` (the `Destroy_Smash` +0x17 one-shot latch, previously folded into
  `dead`) and `Destroy.countdown` (new fields, own lines at the struct end); `Props::smash`
  negates the count and fires the channel only when the negated word lands on 0; `channel_kills`
  runs `Destroy_Update`'s countdown arm before its watch arm; `bullet_hit` carries the handler's
  flip. The destroyed-glist draw swap (`dead_destroy_centres`/`wreck_draw_centres`) now gates on
  the SMASH (`latched`) — the game's `Control_SetGList` swap runs inside `Destroy_Smash`
  [M 0x12bb60:26-28] while a p6 leaf stays class 0x21 — identical for every p6 = 0 leaf.
- **Effect:** 07000014's generator-hall cascade now staggers per link: #400 (0x51→0x52, p6 1) →
  #787 (0x52→0x4e, p6 1) → #401 (0x4e→0x53, p6 2) → #826 (0x53→0x5c, p6 1) → #402 (0x5c→0xaf,
  p6 2) → #828 (0xaf→0x5a, p6 2) → #404 (0x5a→0xc9, p6 1) → #827 (0xc9→0x5b, p6 1) → #403
  (0x5b→0x85, p6 6) — about 510 ticks from the seeded root to the door unlock, where the old
  code ran it in ~9. The old "one cascade link per tick" model came from the watch arm only.
- **Tests (red on the old code — the old `smash` wrote the channel in the same call):**
  - `crates/nf-game/src/props.rs::p6_tests` (3): the 3000-tick deferral + reclass, the bullet
    double-flip unstart + re-arm, the p6 = 0 in-call fire unchanged.
  - `crates/nf-game/tests/destroy_p6_countdown_live.rs` (2, real 07000014 placements): all 17
    rows store −p6·30 at create; #923 (kill 13 → death 117) fires 300 ticks after its smash and
    #294 (kill 225 → 79) 30 ticks after, nothing in the smash tick.
  - `crates/nf-view/tests/panel_sequence_07000014_live.rs` — the cascade section rewritten to
    the traced timing: each link's death rises exactly p6·30 ticks after ITS OWN smash (the
    kill-rose-based pin would wobble by the walk's newest-first order), window 700 ticks,
    measured finish 514. The old 60-tick "one link per tick" window contradicted the
    decompile-backed stagger and was changed with the addresses above; the rest of the walk
    (both fuseboxes cut through the real Laser path, the panels, the touch arm, the door
    unlock, the M.Hatch indicator, the escape scene) passes unchanged.

## Gap 2 (not ported — no reader found): SS p7

`SS_Create` 0x1a9c58:50 stores p7 at instance+0x2A; the only access is `SS_Update` 0x1a9948's
tail [M :132-136]: when p7 ≠ 0 and the object sits in a room cel (obj+0x20), it writes
`cel+0x90 = obj+0x84` (one word of the object's gfx-sphere centre into the room-cel struct).
cel+0x90 is an unknown-meaning field (default −500.0, docs/atlas/cel.md; the direct cel_tag scan
`tools/fields/param_offsets.py cel_tag` shows no +0x90 reader, and View_DrawCelGlist's nearest
read is +0x94). The census: exactly one non-zero row on disc — 07000046 #54, value 1 [D]. With no
traceable reader the write has no observable effect to pin, and the task forbids draw changes;
the port does not model room cels' scratch words. Left open, not closed as zero (it isn't).

## Verified

- `nf-game` lib: 689 passed / 0 failed (incl. the 3 new p6 unit tests) — `--test-threads=1`.
- `destroy_p6_countdown_live` 2/2; `0700000d_gunimp_door_live` 5/1 ignored (the C5 smash paths);
  `fix_blast_scripts_live` 3/3; `panel_sequence_07000014_live` 2/2 (rewritten cascade section).
- `walkreal_*_live`: see "Test binaries" below for what ran in the sandbox window.
- No RNG draws added, no wall-clock in Sim, 60 Hz fixed; the countdown is tick-exact.

## Test binaries

| binary | result |
|---|---|
| nf-game lib (whole, --test-threads=1) | 689 passed, 0 failed, 5 ignored |
| nf-game destroy_p6_countdown_live | 2 passed |
| nf-view 0700000d_gunimp_door_live | 5 passed, 1 ignored |
| nf-view fix_blast_scripts_live | 3 passed |
| nf-view panel_sequence_07000014_live | 2 passed |
| nf-view walkreal_*_live (priority set: 14, 08, 01, 05, 0d, 09) | recorded in the WORKLOG tail before the deadline |

(The full four-crate suite did not fit the sandbox disk — 9.9 GB with the ISO and work/ — after
the first runner filled the volume; the binaries above are the ones lesson 10 names for this
change set: every test file that touches `props`/`channel_kills`/`smash`/destroy, plus the
priority walkreal set. The p6 = 0 leaves' behaviour is bit-identical to the old code — the
latch and the death set in the same call — so only 07000014's walks could move, and its
panel/real-data tests pin the new timing.)

## Live checks for the main agent

1. **The 07000014 cascade stagger (the ported p6):** load 07000014, seed the fusebox/touch path
   to ch 0x51 (or watch it in a real playthrough of the generator room). Expected [M 0x12c4f8 +
   D the p6 rows]: #400's wreck appears ~30 ticks after 0x51 rises (death 0x52), then #787 +30,
   #401 +60, #826 +30, #402 +60, #828 +60, #404 +30, #827 +30, #403 +180 — Door_Slide_11A/B's
   lock channel 0x85 rises ~510 ticks after 0x51, and the door unlocks then (not at once).
   Watch `switch_channels[0x85]` (0x26fc90 + 0x85) or the door ext's +0x68.
2. **SS p7 (the open gap):** on 07000046, find the 0xE2 placement #54's live object (class 0x25),
   read its instance+0x2A (expect 1) and watch `cel+0x90` of the cel the object sits in
   (obj+0x20): expect the word to track obj+0x84 (the sphere centre's y bits) every tick while
   the SS animates — then confirm whether ANY code reads cel+0x90 (a gdbwatch sweep over the
   field would settle the "reader" half; the decompile scan found none).
3. **Destroy p6 = 0 regression check:** on 07000008, the GuardRoom SmallExplosionBox #234 (kill
   ch 137) should still smash the same tick the fusebox channel rises (p6 = 0 there [D the
   census]) — the box's wreck visible immediately, death channel in-call.

## Left

- The door sound 0→0xFFFF conversion (door.md "Port mismatches" #2: `Door_Create` 0x136100:28-42
  maps a 0 sound param to 0xFFFF; the port stores it raw and emits `SoundEvent(0, …)` — live on
  07000005's four locked doors when channel 200 unlocks them). Cited, not fixed this task —
  it is a mishandled READ, not an unread param, and the sound-id-0 emission needs a silent-sound
  decision in the audio path first.
- SS p7: needs the cel+0x90 reader question settled (live check 2) before anything can be ported.
- The full suite on a machine with the disk for it (the per-binary results above stand in).

## Usage

not visible to me — this session's environment shows no token, context-window or cost figures.
