# level-0a-walk-clip-flip — the clip's dead move-vector found and fixed; the model corrected to the [L] r-skin response; two of the three flip-red tests reconciled; the flip not landed

## What was asked

Reconcile the three tests that go red under `NF_WALK_CLIP=1`
(`the_pick_ref_accumulator_tracks_the_game_ref`, `walk_office_floor_qworm_keyboard_hatch`,
`walk_office_floor_to_the_security_keyboard` — fix the port or the test, with cites), make the
clip the walk default (remove the env gate), and run every walkreal_* binary plus the
collision/player crate tests, quoting every total.

## What this session found (the face-plane clip never engaged at all)

1. **The clip never fired on any wall press — Bond walked THROUGH the west end wall.** Running the
   accumulator's press under the flag with a contact dump: zero clip applications; Bond integrated
   from x −40 through the wall plane (−46.41) to x −48 and fell off the hall's end into the void.
   The cause is in `physics_tick` [M crates/nf-game/src/player.rs]: `pre_move_pos` was captured
   AFTER the walk's matrix·(vs, vf) x/z add (the walk's actual horizontal move), so the clip's
   `move_v = pos − pre_move_pos` read **(0, gravity, 0) on every walked tick** — the wall-contact
   dump shows `mv=(0.00,0.00,0.00)` at the interior face contact. The clip (any variant) can never
   see the move it is supposed to clip. The capture now reads `physics_tick`'s own `prev`
   (the pre-move pos), so `move_v` is the whole tick's positional delta.
2. **The face-plane retraction is also the wrong model.** With `move_v` fixed, a face-plane clip
   (retract to the contact face's plane) still fails the [L] press: the press never crosses the
   face plane within r, so nothing clips and Bond walks through. The game's own press response —
   [L 2026-10-26 fresh boot, pinned by the accumulator test] — is the **r-skin**: the spine rests
   at face+0.45/+0.55 (two values 0.1 apart = the pre/post phases of one retraction: the tick's
   0.1 step presses dist to r − 0.1, the response restores r) and HITTEST+0x50 grows +0.1 per
   contact tick (the clipped step), constant on the push-free walk. Master's push log on the same
   wall (tri centroid (−46.406, 0.912, 16.903), n (1,0,0), q interior, dist 0.45, push (0.1,0,0))
   reproduces exactly those two phases — the west-wall [L] is a radial-skin response, not a
   face-plane rest.
3. **The corrected model: the radial r-skin clip.** `capsule_tri_clip` now retracts the move's
   approach along the contact radial v̂ = (p − q)/|p − q| so the post-clip distance to the contact
   is r; the tangential component carries untouched (the corner slide); per-triangle sequential
   (the shifted-segment flow [M 0x109da0's A/B += push]); the head-on cull and the walkable/pick
   signals keep the raw face normal ([M 0x109da0:321-322; the up-facing dot1 gate]); edge contacts
   never displace (the door[19] slab-poke choice stands). Marked [S] as code — the game's clip
   writer is unpinned (the decoded tail's ref = the lifetime accumulator is dead on walked lines
   [L ee_snapshot ≈15 m drift]) — with the [L] press as the model's evidence.
4. **The accumulator feed is unconditional.** The [M] add-back (Collide_Intersect 0x1ebaa8:353-373)
   accumulates whatever displacement the collision pass applies; under the clip that displacement
   is the retraction. `pick_ref += push` now runs on both paths (identical to master's behaviour
   when the flag is off).

## How it was verified (the totals quoted)

- `walkreal_0700000a_live` under `NF_WALK_CLIP=1` (--test-threads=1, slim ISO): **6 passed /
  1 failed / 2 ignored** (the corner report's baseline under the flag was 4/3/2):
  - `the_pick_ref_accumulator_tracks_the_game_ref` — **GREEN** (the [L] press accumulation, the
    free-walk constancy and the boot zero all reproduce under the radial clip);
  - `walk_office_floor_to_the_security_keyboard` — **GREEN** (the office-floor legs' divergence
    was the frozen accumulator feeding the pick/ref-aim queries; with the feed restored and the
    r-skin rest ≈ master's push rest, the walked line is master's again);
  - `walk_office_floor_qworm_keyboard_hatch` — still **RED** (below).
- The same file with NO env (the shipped default): **7 passed / 0 failed / 2 ignored** — the
  default behaviour is untouched by this session's changes (the clip path, the feed's clip arm and
  `capsule_tri_clip` are all behind the flag).
- `nf-game` lib tests: **588 passed / 0 failed / 5 ignored**.
- The fast suite runner (`tools/test/suite.py`) could not run in this sandbox: background
  processes are killed between tool calls here (verified four times: nohup, setsid+disown, warm
  build and cold — same finding as the corner-walk-speed session's report), and the foreground
  window did not fit the whole four-crate battery after the research. The touched-crate runs above
  are what ran. **The full-suite total line is therefore NOT quoted this session** — the next
  session should start the runner first thing.

## The qworm ring, honestly

`walk_office_floor_qworm_keyboard_hatch` (the 44-leg marathon with the alarm-runner watches) still
fails under the flag. Two distinct stalls were met and one fixed:

- **jink 4 (fixed)**: the leg east passes the door @(−20.7, 1.1, −5.4)'s open leaf; under the clip
  the leaf's face contact is a dead-on press (cp = −mv, zero tangential) and the spine holds at
  the leaf's skin — master's push leaked through the thin-leaf poke by alternating side flips. The
  route now bends a half-step south of the leaf's swing (the helper route is a port-choice
  stand-in, not a game pin; noted at the leg).
- **the alarm (left)**: later in the south corridor the mission fails on ch 0x63 (fail_label
  0x0400_0032, the alarm) — a guard's cone caught the walked line/timing under the clip and the
  per-leg watch missed the runner. Reconciling it needs the ring's watch pattern extended or the
  line re-derived — route/helper work, not model work. NOT done in the window.

**The flip is therefore not landed**: removing the env gate with the qworm ring red would push a
red test. The gate stays; the flip is one line once the ring is reconciled.

## What's left

1. The qworm ring under the flag (the alarm leg).
2. The flip: `walk_clip()` → true (remove `NF_WALK_CLIP`), delete the dead push-walk branch.
3. Every walkreal_* binary + the collision/player crate tests with every total quoted, and the
   full suite on the final tree.

## Live checks for the main agent

1. **The glide wall's rest geometry** (the one [L] the radial model does not pin): fresh boot
   0700000a, walk Bond into the west hall's south wall at the z 16.29 line and hold 5 s; record
   obj+0x30.z minus the wall's face plane. The corner report quotes ~0.011 INSIDE the plane [L
   fresh boot]; the west-wall press [L] reads face+0.45/+0.55. If both are right the response keys
   on something per-contact (face vs edge/shape class) — the number decides.
2. **The open-door leaf press**: fresh boot, walk Bond dead-on into the open leaf of the
   (−20.7, 1.1, −5.4) door and hold 3 s. EXPECTED if the r-skin response is general: the spine
   rests at leaf+r (no leak-through). This decides whether the qworm jink-4 wedge is game-true
   (and the route must bend) or a port artifact.

## Usage

Not visible to me — this environment shows no token, credit, context-window or cost figures for
the session. No sub-agents were spawned this session, so there are no per-call footers to quote
either.
