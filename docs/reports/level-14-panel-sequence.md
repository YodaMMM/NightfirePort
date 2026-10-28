# level-14-panel-sequence — the Countdown Part 1 sabotage sequence after the two panels

Branch `s/level-14-panel-sequence` (from origin/master fcf6f334). The task: after 07000014's
capture scene, destroying the two panels must run the game's scripted sequence and the door out
must unlock when the game unlocks it.

## The chain, with evidence

- **The panels** are the two DESTROY_BoxSatellite destroyables beside the two west
  generator-room fuseboxes: #714 (kill ch **0x71**, death **0xdb**, kind 0x32, blast 10.0) at
  (-90.41, 0.90, 34.61) and #713 (kill **0xd8**, death **0xdb**) at (-90.40, 0.90, 49.46), next
  to fuseboxes #709 (words 0x87/0x71) and #711 (0x88/0xd8) [D the placement census,
  probe14_destroyables]. Cutting each box's BOTH wire groups writes its words — group 0 → p1+p3,
  group 1 → p2+p4, one-shot [M `FuseBox_Update` 0x1adfe8:110-123].
- **The panels blow up by remote**: `Destroy_Update` 0x12c4f8's second arm smashes a destroyable
  whose +0x10 word (param 1) names an ON channel and clears the word [M 0x12c4f8:30-35]; the
  smash is `Destroy_Smash` 0x12bb60 — glist flip, the dead mark, `switch_channels[+0x12] = 1`
  (the death channel) [M 0x12bb60:32-45] and `Explode_Create` with the kind-byte-routed debris
  script (1→0x06000027 ×4, 2→0x0600004f ×2, else 0x06000052 ×3) [M 0x12bb60:58-76]. Both panels
  die writing **0xdb**; SoundTrigger #718 (on 0xdb, id 0x398, GlobalVoice claim p5=1) plays at
  the south station [D #718].
- **0x8C** = AND #717 [0x87, 0x88] [D]. Downstream: the Electricity console swap (#1119 active /
  #1132 inactive on 0x8C [D 0x3b pairs]), the guard spawners #720/#1333 (start ch 0x8C [D 0xf1]),
  the hint plates #1224/#1225 [D].
- **The cascade to the door**: SEQ_AND #719 [0x8C, 0xA0] → **0x51** [D]; 0x51 is satellite box
  #726's kill channel, and the kill→death pairs chain box→generator→box across the hall:
  #726 (0x51→0x52) → GENERATORS_Intact #1113 (0x52→0x4e) → #727 (0x4e→0x53) → generator #1152
  (0x53→0x5c) → #728 (0x5c→0xaf) → generator #1154 (0xaf→0x5a) → #730 (0x5a→0xc9) → generator
  #1153 (0xc9→0x5b) → **#729 (0x5b→0x85)** [D the census; the tags' kill/death pairs are unique].
  The remote kills run in the control walk's newest-first object order [M 0x1336f8:54-68], one
  cascade link per tick.
- **The door**: ch 0x85 is Door_Slide_11A/B's lock channel (p3 → DOORINFO+0x68 [M
  `Door_Create` 0x136100:31]); `Door_Update` 0x1363c8:88-107 unlocks on the rising edge, clears
  the field, and suppresses the "locked" text on this level (0x7000014 special case [M
  0x1363c8:101-104]). With 0x85: SoundTrigger #1220 (id 0x5e3) at the door [D] and the M.Hatch
  indicator ScriptPlayer #1273 (script 0x06000708, trigger 0x85 [D]; the port replays it — the
  test saw it playing).
- **The escape scene**: Touch #782 (out 0x0C) + SEQ_AND #771 [0x8C, 0x0C] → 0x09 → ScriptPlayer
  #733's script 0x06000848 (950 ticks): t0 event 0x0b [0x1a] (the previous-stream anim object's
  +0x62 byte [M 0x1d94c0:152-158]) and event 0x05 (`Drone_EnableAll`), the camera keys at
  (27.82, 0.97, -20.38)→(27.94, 1.64, -21.83) world-frame, Rook's anim (0x0600085a) with sound
  0x4c6, t949 events 0x07/0x06 (the hand-back, prev stream 6 has no object → the player resumes
  where he was [M 0x1d94c0:125-134]), t949 **[0x11, 1]**, t950 **[0x4A, 1]** [D decode,
  dump_07000014_escape_scene]. 0x4A completes objective 3 [L c07000014.rs].

## What was broken and what changed

1. **`Props::channel_kills` never wrote the death channel** (crates/nf-game/src/props.rs). The
   arm was added for 07000008's SmallExplosionBox #234 whose death channel is 0, so the omission
   was invisible there; on 07000014 the whole cascade runs through those writes. Fixed: the
   remote arm now runs the same `smash()` death half as the bullet path (dead mark + the
   `switch_channels[+0x12] = 1` write [M 0x12c4f8:35 → 0x12bb60:40] + the blast), clears the
   kill word [M 0x12c4f8:32], and walks the tags newest-first to match the control walk's order
   [M 0x1336f8:54-68] — one link per tick. Call site: drone.rs (the same pending_blasts queue).
2. **Tests**: props.rs unit tests (the death-channel write; the newest-first one-link-per-tick
   cascade); `crates/nf-view/tests/panel_sequence_07000014_live.rs` — the real-input sequence:
   the capture hand-back (the marker position pinned), the locked-door sister, both boxes cut
   through the real Laser path, 0x87/0x88/0x71/0xd8/0x8c/0xdb asserted, the cascade seeded at
   its root and pinned link-by-link to 0x85, Door_Slide_11A/B unlocked, the escape touch → 0x09
   → the scene → 0x4A → the objective state 4; the sister test pins the one-box gating (no 0x8C
   → no cascade → the door stays locked).

## The open root: how the game writes 0x51

Per the disc and the decompile, the ONLY 0xA0 writer is touch #725 (out 0xA0), and its contact
mask is **0x4** [M the 0xEB wrapper `Trigger_Touch` 0x1b2aa8 re-homes old p1 into the mask slot
+0x50; D #725 p1 = 0x4]. `Trigger_Hit` 0x1b1fc8 builds contact bits {2 bullet (class 5), 8
player (class 3), 0x10 drone (class 2/0x4a)} and tests `mask & 0x18`, `mask & 2`, then the raw
`mask & contacts` — **a mask of 0x4 matches nothing**, so the touch cannot fire by contact and
SEQ_AND #719 keeps 0x51 low ([M]; the port reproduces this faithfully — the test pins Bond
standing in the volume without 0xA0). The port therefore cannot reach the cascade's root through
the real inputs yet, and the test seeds ch 0x51 to pin the mechanics below it. The candidates
the data offers and rules out: no other 0xA0/0x51 writer exists in the level's placements
(muxes, spawners, drones' param-12 death channels, sensors, scripts all checked [D]);
`DroneSpawner_Control`'s 0x700000C/0D quirk is power-station-only. This needs the live game.

## What's left

- The 0x51 root (above) — the live check decides whether the game's own data path fires it
  (then the port needs whatever that reveals) or the cascade root is reached another way.
- The full suite tally on this merged tree (see Verification).

## Live checks for the main agent

1. Fresh boot 07000014, play to the capture hand-back, cut both west fuseboxes (both wires per
   box), and watch channels **0x87, 0x88, 0x71, 0xd8, 0xdb, 0x8C, 0xA0, 0x51** with
   `tools/trace/gdbwatch.py` on `switch_channels` (0x2d7a40) — expected per the port: the first
   six rise; the question is **whether 0xA0 and 0x51 ever rise in the real game** and by what
   (a guard crossing touch #725 at (-60.30, 0.61, 38.37)? a scripted write?). If 0x51 rises,
   note the tick order of the box/generator deaths.
2. If 0x51 rises in the game: watch `switch_channels[0x85]` and Door_Slide_11A/B's DOORINFO+0x68
   (expected: cleared on the rising edge [M 0x1363c8:106]) and whether the door then opens on E.
3. The escape scene's t949 order: does the port's missing **[0x11, 1]** write appear in the game
   (stream 8 t949 vs stream 7's t949 hand-back — which runs first)? Watch switch_channels[0x11].
4. Draw checks (`--bond` spots): the panel explosions at (-88.7, 0.9, 32.6) and (-88.7, 0.9,
   47.5) (the bounds centres through the placement matrix [D probe14_destroyables]) — the intact
   → smashed art swap (the glist flip [M 0x12bb60:28-31] is not modelled in the port's draw);
   the cascade blasts at the five hall boxes (-85..-70, -0.3..0.9, 33-47); the Electricity
   console swap at (-62.43, 1.19, 38.59); the M.Hatch indicator at (-52.40, 2.64, 40.14).

## Verification

What ran (foreground; this sandbox kills background jobs between tool calls, so the fast runner
could not survive — `suite.out` holds no total):

- `cargo test -p nf-game --lib` (release): **579 passed / 0 failed / 5 ignored** — includes the
  two new props tests.
- `cargo test -p nf-view --test panel_sequence_07000014_live`: **2 passed / 0 failed**.
- Regression families (all green): walk_07000014_live 2/0, capture_placement_07000014_live 1/0,
  panels_07000014_live 2/0, fix_fusebox_cut_live 4/0 (includes the 07000008 fusebox→box chain —
  its death channel is 0, so the new write is a no-op there), fusebox_slots_07000014_live 2/0,
  walkreal_07000014_live 2/0 + 1 ignored (the pre-existing leg-2 blocker).
- Not run: the remaining ~840 integration binaries (no disc_golden/identity bins were touched by
  the change; nf-game --lib covers the props/door/fusebox units).

## Usage

not visible to me
