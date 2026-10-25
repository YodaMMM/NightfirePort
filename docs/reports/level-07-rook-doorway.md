# level-07-rook-doorway — the undrawn slide is the game's raw add with the game's boundary clamp

Branch `s/level-07-rook-doorway` off origin/master (62f70bcc), three commits (234e2dac, d53ac81c,
db988c57) + the worklog. `docs/WORKLOG.md` carries the full entry; this report is the plain-words
summary.

## What's done

1. **The undrawn slide is now the game's raw position add** — `obj+0x30/0x38 += sinf(yaw [+ π/2]) ·
   Drone+0x58c` directly, no `move_by`, no probe [M Drone_Control 0x139268:75-96; `+0x58c` =
   `Drone_AnimInfo[danim]+4` per SetDAnimInternal 0x13aee8:50-51; the gate = the danim's info
   flag 4 per NDrone2_DoAnimation 0x149f88:11-18]. `move_by`'s 0.4-overshoot rays were what
   wedged drone 22 in the 07000007 west doorway (994 consecutive rejects on the frozen walker,
   round-3's trace) and every other off-screen walker near jamb geometry. Not landed (per the
   task's scope): the player-push slide.
2. **The game's boundary clamp landed on the same arm** [M 0x139268:76-83]: while `+0x4F8 & 1`
   arms AND no door is bound (`+0xBA8 == 0`, the port's `hit_door`), a blocked local +z
   (`+0x320` bit 2) zeroes a positive slide and bit 4 a negative one.
3. **`DroneMove_SetBoundryFlags` 0x14f140 ported live**: drone+0x320 zeroed each control tick,
   the four local-axis feet probes (±0.5, the game's MoveTest semantics — the distance/slope
   gates then `AINetwork_BoundsTest` ONLY, no collision LOS [M 0x155330:86-108]) behind the
   `+0x4F8 & 1` gate, the movement-locked arm (`+0x4F8 & 0x10`) setting all four bits without
   probing. Called from the control tick behind the +0x23 air/rope flag [M ControlSTANDARD
   0x1490f0:202-206]. The port's `move_test` LOS stand-in deliberately NOT used — its rays graze
   the 07000007 doorway throat.
4. **The decisive decompile find**: the fold doc's "the `+0x4F8` bit-1 setter is unaccounted" was
   WRONG — `NDrone2_DefaultInit` 0x14b300:60's composite init OR `0x801e2` arms bit 1 at
   creation for every drone (the only clearer: `DSTATE_AstronautLaunch` 0x16dd28:19). The clamp
   is live on the shipped paths; `docs/systems/boundary-flags-fold.md` is corrected. This is why
   round 4's three clamp variants could not scope: the game's stopper for a route-stalled
   undrawn walker is BOTH the clamp (boundary bits live) and the state machine (blocked 0x63 →
   CallAnim 0x21 [M 0x165640 case 3], the walk danim — flag 4 — replaced, the slide stops). The
   port's blocked arm (`move_to_goal` 0xc → face_and_aim) already swaps the anim, so with the
   probes live both mechanisms run as the game's.
5. The walker harness gained round-3's pocket back-off (walker.rs: stuck 3/4 arms a 60-tick
   straight reverse) — the undrawn guards' game-correct raw-add positions shift the walkreal_01
   flanking lines into the level's known swung-panel pockets.

## How it was verified

- **Reproduce-before-fix, twice**: (a) the raw add alone turned `npc_doors_live`'s
  `the_01_escort_driven_through_the_shoji_opens_it_and_crosses` red — the driven Mayhew crossed
  the CLOSED shoji (the trace: 0x63 at (6.4,8.8) → through the panel → exits 0x63 to 0x1c at
  (6.7,12.4), door state 0x2 throughout); (b) the shoji's faithful forward probe hits the 0x800
  door wall at d 0.416 (scratch probe, untracked example, deleted) — with the live clamp the
  same test is green again.
- Green on the final tree (foreground slices, `--test-threads=1`, NF_ISO the slim ISO):
  `npc_doors_live` 6 passed / 1 ignored (the pre-existing escort ignore) — including the shoji
  re-pin; `walkreal_07000007_live` 2 passed / 1 ignored — `walk_the_party_start_to_finish` and
  the wake test both green with the raw add + live clamp; `walkreal_07000001_live` 2 of 3.
- `cargo build --release -p nf-game` clean (only the pre-existing warnings).

## What's left

1. **The 07 doorway-leg real-input test** (`the_mission_walker_completes_his_doorway_leg`,
   landed in walkreal_07000007_live) is IGNORED with the diagnosis: under the live clamp
   walker22's start pose (80.15, −50.46) re-freezes — the port's ONE-CEL boundary flattening
   probes wall #684/#685 across the west doorway, while the game's `AINetwork_BoundsTest` walks
   only the STRADDLED CELS' BLINK lists [M 0x157d80:113-158] and evidently does not carry this
   doorway's wall into the probed cels' lists (the real Rook crosses; round-3's raw-add-only
   tree passed this leg at t256). Next round: port the per-cel filter (or the door-state wall
   handling) into `set_boundary_flags`'s probe, then un-ignore.
2. **walkreal_07000001_live's contact walk fails at the mission-monitor assert**
   (`states01[0] == 4`, line 451) AFTER progressing past the ch-0x99 guard kills. Full trace
   diff vs master: the walks are IDENTICAL through the meet scene and the stage; the first
   divergence is the guard-7 flank target (master (−8.2,9.8) vs mine (−8.2,10.0)) — the
   guards' game-correct raw-add movement shifts the harness choreography; the backoff let the
   walk advance past the kills. Needs either the harness pocket handling finished (round-3's
   full dodge set) or a live check on where the real game's guards park.
3. **No full-suite total line this session** — the sandbox kills background jobs between tool
   calls here (two detached runs died), and the foreground window could not fit the whole
   four-crate run. The touched binaries are reported above; run
   `NF_ISO=<abs> python3 tools/test/suite.py` on the merged tree before merging.

## Live checks for the main agent

1. 07000007 fresh boot, gdbwatch drone[22] +0x4f8 (expect bit 0x1 SET — DefaultInit 0x14b300:60)
   and +0x320 while the walker approaches the west doorway: which of the four bits arm, and does
   the walking-direction bit stay CLEAR while he crosses? That settles whether the game's
   per-cel BLINK lists exclude the doorway wall (the ignore's diagnosis) or the wall's absence
   in the level data.
2. 07000001 fresh boot, gdbwatch the driven Mayhew's +0x320 bits at the closed courtyard shoji
   (expect bit 2 set, the slide zeroed at ~0.4 from the wall — the 01d test's pinned stop).

## Usage

not visible to me — this session's interface exposes no token, context-window or cost figures.
