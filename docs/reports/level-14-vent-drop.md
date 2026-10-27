# level-14-vent-drop — the vent-drop hand-back chain is decoded; the fix is scoped and ready to land

**Status: BLOCKED by a sandbox reset at 20:48 UTC (the window's second reset).** The branch
carries the plan only; every finding below is in this report. The next window can implement
in ~20 minutes from the "The fix" section — all the evidence work is done.

## What the task needed

`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` stops at leg 2: the SP #276
vent-drop cut-scene (0x06000833, 1400 ticks) runs and writes ch 0x47, but the port leaves Bond
on the mezzanine. Find how the game moves Bond into the facility and port it.

## The decode (all verified this window)

**The chain that starts the scene [D/M]:**
1. Touch #262 (placement kind 0xea = the port's TOUCH_ONCE, at (40.72, 3.10, 39.01), out ch
   0xd9, param 1 = 8) [D 07000014 map 54 placements dump].
2. OR gate #276 (kind 0xef = kind::OR): `Trigger_MultiplexOrIn` 0x1b2d38 shifts params —
   **out = param 0 = 0x7e, inputs = params 1.. = {0xda, 0xd9}** [M 0x1b2d38:16-24]. So the
   touch's 0xd9 fires the OR → ch 0x7e.
3. SP #268 (kind 0xd9, script 0x06000833, switch = param 6 = 0x7e, mask = param 5 = 1, stay =
   param 4 = 1) starts when its switch channel is on: `SP_Hit` 0x1dc470 makes
   `switch_channels[si+0x30]` **bit 1** of the start flags, and mask 1 selects it [M
   0x1dc470:19-22 + SP_Update 0x1dbda8 state 0]. (The port's `Player.out = g(6)` field is
   actually the SP's *switch* channel per SP_CreateScriptPlayer 0x1db9a8's param map:
   +0x2e mask = tag+0x40 = p5, +0x30 switch = tag+0x44 = p6, +0x32 gate = tag+0x50 = p9.)

**The player-transport machinery [M — instruction-level verified, incl. the EE's rd-carrying
`mult` (Toshiba 3-operand multiply; the decompile's `*0x28` terms are correct)]:**
- `Script_EventHandler` 0x1d94c0 case 6: sets SCRIPTINFO+0xaa8 |= 0x100, then **captures a
  matrix INTO SCRIPTINFO+0x40**: `Mat_Copy(src=player|object+0x90, dst=si+0x40)` —
  `Mat_Copy(src,dst)` per `amemcpy(dst=param_2, src=param_1)` 0x1e3348/0x1f5c0. The source is
  the record at `si + (+0xab3)*0x28 + 0x80` when its kind byte ∈ {1,2} (obj at +0x10), else
  the player's own matrix `glb_players+0x90`. The record base +0x80 with the current stream
  index = **the previous stream's record** — `Script_Update` 0x1da998 increments +0xab3 only
  after `Script_Run` returns (0x1daa10-0x1daa28; `Script_Run` runs record +0xa8+idx*0x28 =
  stream idx).
- `Script_Free` 0x1da880:44-48: flag 0x100 → `Player_Enable(glb_players, si+0x40, si+0xab6)`;
  `Player_Enable` 0x195498 copies matrix+0x30 (translation) into the player obj+0x30 and the
  spherical angles from the matrix direction. SP_Delete 0x1dc8b0 calls Script_Free — with
  stay=1 the SP hits state 5 (delete) on script end, so the hand-back fires [M 0x1dbda8 state
  2 tail]. Only ONE write to glb_players+0x90 exists in the whole decompile (the case-6
  capture itself); nothing else moves the player during a scene; `Player_Update` 0x193600:80
  returns immediately while BL+0x94a == 0.

**The Kiko precedent the port already has [M+D, master's nis.rs level-14-capture-placement
arm]:** in 0x0600062F the marker **entity stream (s10, hash 0, kind 0x31) directly precedes
the event-6 stream (s11)**, so the game's capture reads it, and the port captures it the same
way. The port composes the base for entities but the Kiko scene keys world-space (scoped arm).

**The vent pack 0x06000833 [D — full stream/key dump, nf-formats example
zzz_dump_06000833.rs]:**
- 25 streams, 296 keys, length 1400. s0 camera (keys 0..166); s1/s3/s5 guard anims
  (0x06000854/06000856/06000855, skins 0x05000087/0x05000090/0x05000093, weapon event 0xB
  [14]); s10/s11 the door leaves (0x02000891/92, type 0xff, keys 173..227/228..293); **s14:
  event 0x05 at t0 then event 0x06 [0] at t1399** (prev = s13 = Wait/Marker only, no object);
  s15: event 0x04/0x07; s16: 0x0d [0x47,1]; s17: 0x0d [0x0e,1]; **s18: the Bond body — Anim
  script=0x06000857, skin=0x0500005e**; s21-24 the drop sounds (attached to s5/s18).
- **NO marker entity stream exists in this pack** (unlike Kiko's s10) — streams 7/8/9/12/13/
  19/20 are empty. So the game's case-6 capture hits the `glb_players+0x90` **fallback**.
- **The Bond body's root track [D — zzz_roots_0857.rs via the view's seqs]**: anim 0x06000857
  seq 04000728, root ch0/1/2 from f1 (27.837, 0.940, 49.956) to f1401 (27.598, 0.979, 43.563)
  — **inside the facility at floor height (y ≈ 1), world-frame** (the Kiko scene is the same
  world-root pattern; the SP base (52.16, 2.64, 46.62) composed would throw it outside the
  map). The guards' roots are likewise world-frame; none of them leave y ≈ 1.16.
- The scene keys 294/295 (stream 18's own keys) are the origin — the hand-back pose is NOT
  keyed; it lives in the anim root.

**Conclusion:** the only mover in the game is the case-6 capture → Script_Free →
Player_Enable(si+0x40). For THIS pack the previous stream is empty, so the decompile-strict
capture source is the player's own matrix — yet the game carries Bond in (no other entry into
the facility exists: the z 38.21 wall [D walk-4], nav None [D walk-4]). The one candidate the
disc data offers is the scripted Bond body (s18, skin 0x0500005e = the player's skin) whose
root ends inside the facility. **A live PCSX2 trace is the missing discriminator** (see Live
checks).

## The fix (ready to implement — ~20 min)

In `crates/nf-game/src/nis.rs`, mirroring the existing scoped-arm style (the Kiko 0x0600062f
arm, the ride pack's world-alone arm):

1. `Obj::Anim` gains a field `root: Mat4` (own line at the end). The one constructor
   (Script::update's AnimStart arm, nis.rs:840) seeds it `IDENT`.
2. `interp` case 2 (has `w: &dyn World`): after advancing the frame, store the body's current
   world root matrix into the object: `if let Some(m) = w.discrete0(*skin, *script, *frame) {
   *root = m; }` — `discrete0`'s translation is already the port's anim-object position
   (drone.rs:12833 uses exactly that as `script_obj_pos_yaw`).
3. Event 6 (the `6 =>` arm): after the existing `prev` capture, if `player_at.is_none()` and
   `self.nis.hashcode == 0x0600_0833` [S unverified — the game-side discriminator; see Live
   checks], scan `objs.list` rev for `Obj::Anim { script: 0x0600_0857, root, .. }` and set
   `player_at = Some(*root)`. The synthetic matrix's rotation gives Bond::stand_at its yaw —
   the body walks −z at the end, so its discrete0 rotation carries the facing.
4. Walk test: after the ch 0x47 wait + the disabled tail, assert Bond stands near the body's
   end root (27.60, 0.98, 43.56) ± 1.5 [D root track], then legs 2..6 run unchanged
   (follow_nav_route([-49.0, 1.0, 38.4]) → … → the Delta exit #944). Un-ignore when green.

## What ran (before the reset killed the tree)

- The full stream/key decode of 0x06000833 and 0x0600062F (scratch nf-formats examples,
  removed with the tree); the placements dump (07000014 map 54: the #262 touch, the #276 OR,
  the #268/#315/#263 SPs) [D].
- The anim-root probe for 0x06000857/0x0600083c/0x06000854-56 (zzz_roots_0857.rs): **1 passed,
  0 failed** — the numbers quoted above [D].
- The full suite did NOT run: the sandbox killed every background process between tool calls
  this window (walk-4 hit the same), and the second reset (20:48) destroyed the tree before a
  foreground run could fit. `tools/test/suite.py` was started twice (19:02, 19:32); both died
  at the next tool-call boundary with 0-byte output. The touched-crate runs did not get to
  execute either — the reset hit mid-implementation.
- Instruction-level verification done by hand (scripts/mdis.py + raw-word decoding): the EE
  `mult rd, rs, rt` 3-operand form (0x1daa64 `mult $2,$5 (rd=3)` → `addu $2,$3,$6`) resolves
  the apparent "missing mflo"; Mat_Copy's arg order; the EventHandler's +0x80 record base; the
  +0xab3 loop; SP_Update's state machine; SP_Hit's switch-channel bit.

## What is left

1. Implement "The fix" above (the port change + the walk-test legs 2..6), un-ignore on green.
2. Run the touched crates (nf-game, nf-view: walkreal_07000014_live, ride_cutscene_objects_
   live, cutscene_door_07000014_live, the nf-game nis unit tests) and the full suite on the
   final tree; report the total line.
3. If a live check disproves the body-capture reading (below), re-derive from its answer.

## Live checks for the main agent

Fresh boot 07000014, walk west into the #262 band (x < 41.75 at z ≈ 39) on the mezzanine:

1. **The discriminator**: gdbwatch the player object's pos (obj+0x30) and matrix (+0x90)
   through SP #268's 1400 ticks, and `tools/trace/fieldprobe.py` SCRIPTINFO+0x40 of the
   active script player at event-6 time (t1399): does si+0x40 hold the mezzanine pose (the
   strict-fallback reading) or the Bond body's root pose (≈ (27.6, 0.98, 43.56))? Also watch
   obj+0x90 for player 0 DURING the scene — if anything writes it, that is the mover.
2. Where does control return (the player pos one second after the scene)? If ≈ (27.6, 0.98,
   43.56) the fix's capture source is proven; if the mezzanine, the game moves Bond some other
   way and the pack's s18 body is a red herring.
3. Does the game's Bond ever walk/jump the mezzanine's south edge himself (top 3.5-3.7 at z
   38.21 [D walk-4]), or is the drop exclusively cut-scene-carried?

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the
session.
