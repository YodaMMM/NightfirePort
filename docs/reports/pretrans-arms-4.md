# pretrans-arms-4 — the six interaction reds, one level per tool call

Branch s/pretrans-arms-4 = origin/s/pretrans-arms-3 525153962 (merge origin/master 0a9d39435 was
already up to date). Task: repair the six walkreal reds the PreTransCallback root zero left on the
rebased tree (the -3 report's list), the -2 round's way — per-level walk repairs with the trace, no
sim changes. Real input only; no damage, health, pin or top-up touched.

## Round 1 (2026-11-09): 4 of 7 red tests fixed and green; 3 ledgered ignores

1. **01 escort real-damage pair** — `DamageLog::volley_carrier_first` + `Walker::legs_keep_off`
   (both opt-in, the 01 real-damage walks only). d1 lowest health 46.0 (was death), d2 lowest 37.7.
2. **05 d2 door-band exit** — the fight-broken-leg re-walk un-gated from d3 + the shelf y band
   28.0 < y < 35.0. Passed, health 38.4, other 0.0.
3. **09 alarm switch** — the press-reach check before the approach beat + the final node's
   post-arrival beat skipped in reach. Passed in 3.4 s.
4. Ledgered: 01 mayhew (the contact is KILLED by guard crossfire, state 0x47), 0c handoff +
   eastroute (the west-ledge diagonal fall, corpse-solid theory [S]), 11 (the dry-resupply
   displacement wedge; two repairs tried and reverted).

## Round 2 (2026-11-10, this sandbox): the ledgered reds attacked — 11 GREEN, the 0c crossing REPAIRED

### 11 real damage — FIXED, UN-IGNORED, GREEN

The wedge's own arm is gated. The leg-level dry resupply (walker_b2's walk_to arms) displaced Bond
mid-leg — the fetch carried him 15 u east to the (24.85,22.21) box and the resumed straight line to
the (16,6.4) doorway clipped the (21.5,15.9) desk corner [L the 11a run]. The player's way is a grab
that does not abandon the leg:

- `with_resupply_in_place` (damage_log.rs thread-local, OPT-IN — this test only) +
  walker_b2::resupply's `on_leg_line` filter: the leg-level pick must sit ON the leg's own line
  (within 1.6 u of the bond→wp segment, or within 2.5 u of Bond's feet), so the fetch walk advances
  the leg and the resume stays on it.
- `walk_to` brackets each leg with `walk_leg_replace`/`walk_leg_restore` (nesting-safe: the
  resupply fetch's own nav legs are walk_to calls inside the outer leg).
- walker.rs's master resupply never reads the flag; every other walk is untouched (lesson 7).

Result: `walk_spawn_to_the_lower_floor_load_point_real_damage` GREEN; the whole binary
walkreal_07000011_live **4 passed, 0 failed, 0 ignored** (53.9 s serial) — the plain, Agent and
00 Agent sisters stay green.

### 0c handoff + 0c_eastroute — the ledgered crossing REPAIRED and verified; still ignored at the NEXT blocker

The [0c-diag] dump (drones + the scoped lip heightmap at the leg-b start) settled the round-1
theory: **no corpse** — the ALIVE west sniper #35 (skin 0x500087, hp 10.0, team 1) stands at
(-7.07,12.24,-218.60), 0.2-0.3 u off the -3 tree's single push line (B-drop (-8.98,-217.44) →
(-6.0,-219.6)). Pre-0a9d39435 guards and corpses were non-solid and the -2 tree walked THROUGH him
[M master 0a9d39435]. On this tree the solid capsule either slides Bond off the lip's south-east
edge (r 1.5) or wedges the leg (a west-lane pair: the second leg's line passes 0.7 u and the slide
puts Bond east of it with the target line still blocked) [L the runs].

The repair (walkreal0c.rs, the shared walk fn — only the handoff and eastroute walks reach it):
the shelf's west lane leg (arrived (-8.36,12.37,-217.82), ON lip), then the brush-past leg to
(-7.3,-219.6) with r 1.2 — its 0.7-u clip slides Bond around the capsule's east side onto the
holding shelf, the slide stands at (-6.32,12.37,-218.94), INSIDE the arrive radius, so the slide IS
the arrival — then the -3 leg, the diagonals 2/3, icon #6, the wire #282 grab and the ride east:
**ALL GREEN on real input** [L the run: "wire #282 ridden east — landed (4.98,13.16,-223.29)"].
The real-damage handoff sister (green, shares the fn) stays green with the new legs (13.2 s).

STILL RED past the crossing (the walks stay #[ignore]d — the existing ignores, updated with the
measured mechanism; no new ignores): the SW arc approach. This timeline's ride lands Bond on a
13.16-high tier at (4.98,-223.29) whose east side overhangs the yard — the walkable spine the
[0c-arc] dump mapped (z −223 floor x 7-8 … z −218 x 12-14) sits 2 u below at 11.2. "The ledge to
the arc" falls to the yard floor (8.99,2.01,-222.09) and "the arc west end" wedges. Evidence: NO
drones within 10 u before AND after the fall (pure geometry, not a solid body); the arc-landing
stands sweep (opt-in `with_arc_landing_stands`, the handoff test only) was a no-op; a band-tip leg
(7.2,-223.3) measured worse — the landing patch is isolated: standing holds, walking east falls
[L the runs]. The -3 tree walked the same legs green from its own ride phase (the repaired crossing
moved the grab approach and with it the drop point). NEXT LEVER: shift the ride's release point
(the jump disc's d<0.9 phase — the ride loop is in walkreal0c.rs, gate it per-test before touching
the green sister) or map the landing tier fine and route from what holds.

### 01 mayhew — not reached this round

The 0c cycles filled the window. The round-1 ledger stands (the contact dies to guard crossfire
during the stairwell-head clear; next lever: clear the gallery guards before the contact crosses
under them).

## What ran (this tree, release, --test-threads=1)

- walkreal_07000011_live: **4 passed, 0 failed, 0 ignored** (53.9 s) — the round-2 fix's binary.
- walkreal_07000001_live: **6 passed, 0 failed, 2 ignored** (86.3 s).
- walkreal_07000005_live: **16 passed, 0 failed, 1 ignored** (74.9 s).
- walkreal_07000009_live: **9 passed, 0 failed, 0 ignored** (17.1 s).
- walkreal_07000014_live: **7 passed, 0 failed, 3 ignored** (105.0 s).
- walkreal_0700000c_live: the handoff still fails at the arc (the ledger above); the real-damage
  sister **1 passed** (13.2 s); the agent sister stays ignored (pre-existing, round-1 ledger).
- walkreal_0700000c_barrelrow_live: **1 passed** (7.2 s).
- guard1_rounds_0700000c_live **2 passed**, comms_chatter_0c_live **1 passed**,
  upward_shot_0700000c_live **2 passed** — the three other walkreal0c.rs consumers (they stop at
  the ladder base / ring diagonal, before the changed legs; run as the recompile insurance).
- The walkreal_0700000c_eastroute binary compile-checks clean; its walk shares the handoff's
  changed legs and fails at the same arc blocker (the ledger above).
- The full four-crate suite did not run this round either: the sandbox kills background jobs
  between tool calls, and the foreground per-binary runs filled the window. The round-1 report's
  caveat stands: the suite is the first thing the next round should start.

## Live checks for the main agent

1. [S] In the real game at 0700000c, after the wire #282 ride east (the drop inside 0.9 of A at
   (5.85,14.11,-223.15)): where does Bond land, and can a player walk EAST off that landing tier
   onto the SW arc ledge (the spine z −223 floor x 7-8 … z −218 x 12-14 at y 11.2)? gdbwatch on
   Bond's y while a player walks east from the landing settles whether the port's landing tier
   (y 13.16 at (4.98,-223.29)) matches the game's.
2. [S] gdbwatch on the escort contact (skin 0x4F) during the 01 stairwell-head clear (carried over
   from round 1): which guard's round kills him, and does the game's own AllyLeadHide run keep him
   alive through the same crossfire.
3. [S] In the real game at 0700000c's west ledge, watch the west sniper (0x500087) at
   (-7.07,12.24,-218.60): confirm a player brushes his capsule walking the diagonal (the port's
   brush-past slide is built on the assumption the game's collision pushes Bond around him the
   same way).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
