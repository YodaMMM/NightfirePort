# level-14-delta-exit-walk — Countdown Part 1 walked from the vent drop to the Delta exit

Branch `s/level-14-delta-exit-walk` (from origin/master ae4d2894). The task: un-ignore
`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` — legs 2-3 west on real input,
the two fusebox cuts on the real laser path, the E-use switch, then legs 4-6 east through the
opened Door_Slide_11A/B to the Delta exit #944.

## Done — the walk is green, un-ignored

`walk_the_infiltration_to_the_delta_exit` passes end to end on real input (70993 ticks, 0 stuck
events, 0 falls, 66.8 s): the vent start → the SkyBridge Q-Claw crossing → the trench/corridor/
stairwell → the shaft reel → the vent lane → the mezzanine → the #301 drop → the SP #276
cut-scene (ch 0x47) → the hand-back (30.09, 1.06, 38.73) → the south loop west → the #381 touch
→ the SP #382 cut-scene (ch 0x48, objective 2) → the west generator hall → both fuseboxes cut
through the REAL Laser path (ch 0x87 at +78 ticks, ch 0x88 at +1) → AND #391 raises 0x8C → back
east up the north ramp → the E-use switch pressed on foot (0xA0 → the 0x51 pulse → the kill
cascade → 0x85) → the door auto-opens (pos01 > 0.5 asserted) → east through the opened doorway
→ the escape touch #456 → SP #407 (ch 0x4A, objective 3) → Rook's wake touch #476 (0x0D, Rook
started) → the walked gun path kills him (ch 0x54, objective 4) → the walk to the exit #944 →
E → ch 0xCE (objective 5) and `level_exit == Some(0x0700_0015)`.

## The three discoveries that unblocked it

1. **The 0x85 door seals completely — the old "gap past the leaf" premise was wrong.** The
   crossing probe walked Bond at the closed leaves from both sides: the walk east stops at
   (−52.79, 1.20, 39.42) on the leaves' collision plane (x ≈ −52.35) across z 39.2..41.3, and
   the walk west never leaves the east frame jamb (tris x −53.5..−52.5 and −51.7..−51.0 [D
   world tris] seal z 38.6..38.9; the static rays at y 0.3..2.2 are clear exactly where the
   walk is not — the leaves are the dynamic colliders [M Door_Update 0x1363c8:88-107: the lock
   channel's rising edge is the only opener]). The west-door report's live check 2 is answered:
   the player CANNOT reach the switch with the door closed.
2. **The Kiko cut-scene itself delivers Bond to the west wing.** The TouchOnce #381 (out 0x08
   [D placement 381]) fires as Bond nears the band, and the SP #382 pack's (0x0600062F)
   scripted body carries Bond WEST past the closed door — the sweep's mid-scene pose
   (−56.18, 40.59), the release pose (−55.49, 1.10, 42.02) — the same event-6 body mechanism as
   the vent hand-back [M Script_Free 0x1da880 → Player_Enable 0x195498]. The original leg-2
   wedge was the walk resuming steering after the scene and dragging Bond back east into the
   sealed doorway (the E-press loop on the locked leaf, the 6000-tick cap). The test now stops
   steering the moment the touch fires and lets the scene deliver the west side. Game-side
   scene-end pose: [S] unverified — live check 1 below.
3. **The Laser left in hand from the fusebox cuts cannot kill Rook.** The first full run's
   trace: 198 taps at point-blank (dist 2.9), Rook's health 10.0 constant — the Laser (0x4e)
   is a tool; its rounds never drew blood. The kill switches back to the hand weapon 0x2
   first (owned from the loadout [L]); Rook (10 hp [D drone]) then dies to the walked taps.

Also fixed in the test: the objective-banner lag (the monitor completes the objective the tick
after the channel — the 2-tick settle before each states[20][n] == 4 assert, matching
walk_07000014_live's fix-objective-banner notes), and `cut_fusebox` now picks the group-0 wire
(the p1 word's group [D placements 383/385]) and aims at the VISIBLE cable slab with the
miss-feedback loop (the entity translation sits 1.5 m below the cables [D
OBJECT_GreenWire1/BlueWire1 local y 1.49..1.72]; the straight-at-translation aim never touched
the slab the bullet pass tests, `Collide_PickObj` 0x1eae80 → obj+0xD0 → `FuseBox_Update`
0x1adfe8:88-123). The route legs cite the game's own nav lanes ([D nav path 0/5/6 nodes]) and
probe-measured floors ([D probe: the ramp −0.6/−1.4/−2.2 at x −63.7/−65/−66.2, z 32.8; the
fb1/fb2 stance floors −2.2]).

## What the player would need that the port does not draw

- **The switch wall art at oDroneTrigger1b** (−60.30, 0.61, 38.37): the E-use touch has no drawn
  prop — the player stands in the band and sees nothing to press. (Carried over from the
  west-door report's live check 3.)
- **The M.Hatch indicator** above the exit hatch: ScriptPlayers #947/#950/#952 (script
  0x06000708, triggers ch 0x85/0x54 [D]) flip the locked/unlocked light — the SPs mount, but
  whether the light's frames draw through the port's script player is unverified.
- The Kiko scene's scripted walk (through the closed door's plane) plays from the pack's body
  stream — the port carries Bond correctly; the drawn Kiko body along that path is GPU-check
  territory.

## Live checks for the main agent

1. Fresh boot 07000014, play to the Kiko scene (walk into the SCC band at (−49.2, 38.4)):
   after the 0x0600062F scene releases, note Bond's position — the port releases him at
   (−55.49, 1.10, 42.02), the west corridor north lane. Expected: the same west-of-the-door
   area (the scene walks Bond and Kiko west together).
2. `--bond -60.3,0.9,38.4,π/2,0`: the switch wall art at the touch band (the game draws a
   hatch/wall panel there; the port's draw state unknown), and `--bond -52.2,1.2,40.1` before
   and after the cascade: the M.Hatch indicator's locked → unlocked light flip (SPs
   #947/#950/#952).

## Verification

What ran (release, NF_ISO set):

- `walkreal_07000014_live` whole binary: **5 passed / 0 failed / 0 ignored** — the un-ignored
  delta-exit walk, the east-route, the hand-back, the grill and the E-use tests.
- The full fast suite (tools/test/suite.py) on this exact tree: see the total line below.

## Notes on this session's constraints

- The early-start fast runner failed three times before it could build (cargo off PATH; the
  ALSA env missing; the sandbox disk full twice — a duplicate artifact set from a
  RUSTFLAGS-less partial build had to be removed by mtime). The baseline suite run was
  therefore skipped; ONE full suite ran on the final tree.

## Usage

Token/credit/cost figures: not visible to me. Context window: not visible to me (the session
runs from a continuation summary; no usage readout is exposed).
