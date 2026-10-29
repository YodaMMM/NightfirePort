# level-14-skip-handback — the skipped vent cut-scene handed Bond back at the frozen scripted body; the event-6 capture is now [M]-strict

Branch `s/level-14-skip-handback` = origin/master (b25b5f34, already up to date when merged
again). Window 18:18–20:00 UTC. The task: on 07000014, skipping the early cut-scene (SP #276,
0x06000833 — the one a real play reaches ~frame 1371 of a fresh load and that ends ~frame 3046
naturally) left Bond inside the wall beside the control-room window — (26.70, 1.03, 36.27,
yaw 2.122) @frame 3046 in the user's run, and an earlier run put him outside the map.

## Root cause (all [M], with the address)

The port's event-6 hand-back carried a level-14-vent-drop **[S] stand-in arm**: when the strict
capture found nothing on pack 0x06000833, it scanned the script objects for the scripted Bond
body (anim 0x06000857, Bond's own skin) and captured its matrix as the hand-back pose. The
decompile has no third arm:

- `Script_EventHandler` 0x1d94c0 case 6 [M 0x1d94c0:73-84,125-134]: the capture is the
  PREVIOUS stream's object (`si + (idx·0x28) + 0x80` = stream idx−1's record) gated on its kind
  byte ∈ {1, 2} (`+0x24 − 1 < 2`), reading that object's `obj+0x90` — ELSE `glb_players+0x90`,
  Bond's own matrix. For 0x06000833 the event 6 sits on stream 14 and stream 13 is Wait/Marker
  only (kind 0 [D the pack dump; the port's NF_SCRIPTLOG prints "prev kind Some(0)"]), so the
  game captures **Bond's own matrix**.
- `Script_Free` 0x1da880:44-48 Player_Enables `si+0x40` verbatim; `Player_Enable` 0x195498:38-52
  copies the matrix translation and `Mat_GetDir` yaw into the player (event-6 param 0 = 0 →
  +0xab6 = 0 → the collide arm is skipped, so the position applies exactly).
- On a SKIP the capture is the same: `SP_Update` 0x1dbda8:129-141 takes `Input_Action(-1,0x19,1)`
  → `Script_FFwd` 0x1dac78 arms the fade (min(remaining,45)), `Script_Update` 0x1da998:19-100's
  do-while catches the remaining script up (every op 0x12 still runs — ch 0x47 and the objective
  still land), and the case-2/4 freeze (`Script_Interp` 0x1d9a08:146,205, armed when the fade
  passes its length) freezes the anim/camera interps.

The [S] arm's failure mode on skip: the freeze stops `anim_obj_matrix` updates at the fade-end
tick, so the arm captured the body's FROZEN matrix — for a press before the body's stream even
starts (t294 [D the pack dump]) that is the body's spawn pose by the control-room window. The
port's probe (before the fix) released Bond at **(27.84, 1.10, 49.96)** for a press at scene
t3 — the user's in-wall (26.70, 1.03, 36.27, yaw 2.122) is the same mechanism at their press
tick. The arm was also unfalsifiable in principle: its natural-end capture drifted with every
composition change — (30.09, 38.73) at vent-drop-2 → (27.60, 0.98, 43.56) after the
camera-base reset — the signature of a stand-in, not a rule. Its own comments marked it
"[S] unverified — the game-side discriminator is the report's live check 1", and that live
check was never run.

## The fix

`crates/nf-game/src/nis.rs`, event 6: the 0x06000833 body-root scan is **removed** (the
comment block cites the addresses above). `player_at = None` → the PlayerEnable drain leaves
Bond where he is — exactly the game's `glb_players+0x90` fallback. Verified with a four-point
sweep (a real-input walk to the #301 touch, then the skip pressed at scene t+2, t+400, t+1300,
and no skip): the release lands at **exactly the grab spot (42.15, 2.82, 39.05) yaw 4.680** in
all four, the skip compresses the scene (release 47 ticks after the press vs ~1400 natural),
and ch 0x47 + objective 1 (state 4) land on every path [M the catch-up runs every op 0x12].

## Tests

- **`the_skipped_vent_cutscene_hands_bond_back_at_his_grab_spot`** (new, appended to
  `walkreal_07000014_live.rs`): the full real-input infiltration route to the grab, the game's
  own skip action pressed two ticks into the scene, then asserts the release position/yaw ==
  the grab spot (±0.35 / ±0.05), the compression (< 150 ticks), ch 0x47, objective 1, and Bond
  standing settled 120 ticks later. GREEN.
- **`the_vent_drop_hands_bond_into_the_facility`** (re-pinned): the old assert pinned the [S]
  arm's inside-the-facility landing — per the task's rule the TEST was wrong (it pinned an
  unverified stand-in against [M 0x1d94c0 case 6], and its own comment admitted the
  discriminator was open). It now pins the [M] hand-back: Bond back at the #301 grab volume on
  the mezzanine (39.0..43.0, y 2.0..4.14, z 37.5..40.5). GREEN. The address story is in the
  test's comment.
- `walk_the_infiltration_to_the_delta_exit` is **`#[ignore]`d with the exact blocker**: the
  [M] hand-back stands Bond on the mezzanine where the scene grabbed him, and this walk's
  route into the facility WAS the removed [S] teleport. The mezzanine descent has no proven
  real-input path yet: the mezzanine's west face is a static wall (x ≈ 31.5; west rays blocked
  knee-to-head for z 38.4..40.8; clear only at z ≤ 38.2, where the floor edge at x 32..34 has
  no walkable strip [D ray probes]) and every steer — west, south-west, the walker's own
  avoidance — wedges at (32.06, 2.70, 39.2..39.7). The nav web has no route from the mezzanine
  to the facility floor ("no nav route — the direct chain"). NOTE: the old route's
  "(30.09, 1.07, 38.73)" was reached FROM the [S] arm's (27.60, 43.56) — never by a fall, so
  the descent was never actually proven; the pre-fix route rode the teleport.
- The other 07000014 binaries re-run green after the fix: capture_placement 1/1,
  cutscene_door 3/3, countdown_cutscene_cameras 2/2, invisible_guard 2/2, and the whole
  `walkreal_07000014_live` binary 5 passed / 0 failed / 1 ignored.

## The suite (partial — the sandbox constraint)

`tools/test/chunk_suite.py` (foreground chunks, the background runner dies between tool calls
here): **201/906 binaries — 709 passed, 1 failed, 55 ignored** before the window closed.
- The 1 failure is `probe_pylon_render::probe_render_pylon_only` — it panics in
  `request_adapter(...).expect("adapter")` [crates/nf-view/tests/probe_pylon_render.rs:25]:
  **this sandbox has no GPU** (lesson 6); environmental, pre-existing, unrelated to the change.
- The chunk log also flags `mp_stress_explosives_live` — recorded `rc: -9` (SIGKILL 4 s in,
  memory pressure under the chunk's `-j4`), not an assertion failure; it needs a re-run.
- nf-game's own lib tests: 588 passed / 0 failed / 5 ignored. nf-formats / nf-audio binaries in
  the chunks so far: green.

## Live checks for the main agent

1. **The skip hand-back in the real game** (the discriminator vent-drop-2 never got): fresh
   boot of 07000014, walk into the #301 touch (39.68..41.75, 2.07..4.14, 37.62..40.40), SKIP
   the scene immediately. Watch `glb_players+0x30` (the player position) at the
   `Player_Enable` write [M 0x195498:44-48]. Expected: Bond stays at his grab spot (wherever
   the touch caught him, e.g. ≈ (42, 2.8, 39)) — NOT the scripted body's poses (27.60, 0.98,
   43.56) natural / (27.84, 0.94, 49.96) skip. gdbwatch on glb_players+0x30, write access, one
   hit at the scene end.
2. **The same watch on the natural end** (no skip): expected — the same grab-spot result (the
   capture mechanism is identical [M 0x1d94c0 case 6; the freeze cannot change a
   no-prev-entity capture]). If the real game instead moves Bond to the body root
   (27.60, 43.56), then the game has a mover outside 0x1d94c0/0x1da880/0x195498 and the
   mezzanine-descent question dissolves — re-pin accordingly.
3. **The mezzanine descent**: fresh boot, let or skip the vent scene, then find the player's
   real route from the mezzanine (≈ (42, 2.8, 39)) to the facility floor (y ≈ 0.1-1.1). The
   west face is a wall at x ≈ 31.5 (z 38.4..40.8); the south strip at x 32..34 has no floor
   edge below z ≈ 38.4. A screenshot at the mezzanine's west end plus a walk-through would
   settle whether the intended exit is the unlocked door at (32.0, 2.1, 35.8) or a corner drop
   the port's walker refuses.

## What's left

- The remaining 705 suite binaries (the chunk state is not a repo file — rerun
  `python3 tools/test/chunk_suite.py` until ALL DONE and quote the summed tally).
- The `#[ignore]`d full delta walk: find the real mezzanine descent (live check 3), un-ignore.
- Re-run `mp_stress_explosives_live` (killed, not failed).

## Usage

not visible to me — this environment shows no token, credit or context-window figures (no
status line or /usage output available); nothing to copy, and per the task's rule I did not
estimate.
