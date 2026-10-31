# level-0c-trench-exit — the report

Branch `s/level-0c-trench-exit` (from master 20624c8c). Task: the trench-exit stand-in
`[5.0, 12.35, −112.9]` in `walkreal_0700000c_live` — the crouch exit stalls at z −111.81
(grounded false) while every ray at y ≥ 11.9 there comes back clear; find the block
([D] the cels under z −112.0..−111.6, [M] the capsule contact path), check whether the game's
vent drop lands inside the case (~7.3) rather than in the trench, fix the port gap with
evidence and walk the exit.

## What's done

**The stall is PINNED — and it is not a port gap.** The crouch capsule stops on the disc's own
south-wall slab, exactly where the walk stalled. The full mechanism, all evidence-tagged:

- The "bay/trench" (x 4.78..6.74, z −110.39..−112.34, floor 11.84) is the **display case's vent
  SHAFT**: its floor is the case's glass roof over the jetpack display (the case ground 6.27 and
  the jetpack model 7.55..8.5 sit below it [D this session's block dump]). The shaft is closed
  E/W (x 4.78/6.74, walls to 14.6 and up to 15.86), N (z −110.39) and by the roof
  17.05..17.26 over z −111.97..−110.39 [D]. The only opening is the vent hole 15.03..17.42 at
  the south wall top — 3.2 over the floor: unjumpable (the 0.78 apex [M player.rs]) and
  ladderless (the level's ladders are the yard #164/#495 only [D]). **A one-way drop.**
- The south end z −111.89..−112.34 is a **crawl pocket** (floor 11.84, ceiling 13.34 [D the
  down-facing quad]) under the sill stack (14.81/15.03), closed south by the wall slab
  z −112.34..−112.363 whose outer face spans y 11.62..13.57, x 4.56..6.96 [D].
- The crouch capsule (A = pos + 0.1 − (cc − 0.55), B = pos + 0.45 − cc, r 0.55 [M
  Player_Collision 0x194df0 case 3; cc 0.556 [L 05-truck]]) **fits the pocket** (top =
  floor + 1.251 = 13.09 < 13.34) and walks in — and stops **on the slab**: contact face
  −112.363 + r 0.55 = **−111.813** [deterministic probe: `capsule_push_scoped` returns
  z+push −111.813 from every start z −111.9..−112.2] — the observed stall −111.81.
- **Why the rays came back clear (the old "unpinned" mystery):** at capsule height the slab's
  north face does not exist — its face pieces sit only below the floor (11.62..11.84) and above
  the pocket ceiling (13.34..13.57) [D] — and the outer face points away from a north ray; the
  rays are front-face only [M 0x10a8c8:168]. So "every ray at y ≥ 11.9 clear" was honest, and
  the block is still real: the **capsule touches the slab double-sided** (closest-point contact).
  Nothing is "below ray height"; there is no port-side push-out bug.
- **The game seals it the same way:** [L 2026-10-26 hatch-3, fresh boot] the game HOLDS a
  south-walking body on the bay floor. And no glass exit exists: **0x20 Break placements in
  0700000c = 0** [D census, this session's own dump], and the glass-shatter path runs only
  through `Break_Kill` on 0x20 kind 2 [M 0x12b970; `psiCreateBreakableData` 0x117418's only
  caller is parsemap 0x20 kind 2, 0x1d04f0:70-73].

**The vent-drop question is settled — the port's drop matches the game.** The old comment's
escape story ("the vent duct's own descent into the case interior ~7.3, per the hatch sessions'
map") **misread those sessions**: the ~7.3 was hatch-1's unmapped-leg QUESTION, never a finding.
The hatch sessions' [L]-backed map says the opposite: the fresh-boot drop in bay air rests on the
bay floor 11.84 ([L 2026-10-26 hatch-2] rest (5.42, 12.8773, −111.38), ch 0x1b pulses mid-fall),
the bay never opens into the case, and the case interior (root 7.31 = 6.27 + 1.04) is entered
from the east floor only. The port's drop lands in the trench **because the game's does**.

**Test changes** (`crates/nf-view/tests/walkreal_0700000c_live.rs`, the only code file touched —
no src changes):
- The REAL crouch walk into the pocket replaces the bare stand-in lead-in: crouch once (the
  game's toggle [M Player_SSCrouch 0x1924c0:84-105]), hold south through the pocket mouth, and
  pin the deterministic stall: the body stops at z −111.813 ± 0.02 (the slab contact), stays
  there while still pressing south (t 330 vs t 420), crouched throughout, with the bay floor
  still under it (the prior session's "grounded false" read the walker's push dither, not a
  hole). Then it stands back up so the rest of the walk runs standing, as before.
- New sisters: the pocket-entrance wall (z −111.89, y 13.34..14.6 — the hit the old "lintel"
  sister found is THIS wall's face, not a glass lintel above 13.36), the pocket ceiling 13.34,
  and the ray-blindness pin (the slab has NO ray-visible north face at capsule height — if a
  face ever appears, the exit story must be re-probed).
- The stand-in write STAYS, now documented as crossing the disc's own south seal (the shaft is a
  one-way drop in the disc's data; the game holds the body [L]); the header comment and the
  ~7.3 story are corrected as above.

## How it was verified

- `walkreal_0700000c_live`: **2 passed, 0 failed** (18.1 s, release, single thread) — the whole
  mission walk including the new pocket walk + stall pin.
- The other touched binary: `interactables_0700000c_live`: **5 passed, 0 failed** (10.6 s).
- The walkreal sweep (every walkreal_* binary, release, `--test-threads=1`), totals quoted:
  - walkreal_07000001_live 5 passed / 0 failed; 07000002 1/0; 07000004 7/0; 07000005 10/0;
    07000006 3/0; 07000007 2/0; 07000008 3/0; 07000009 4/0; 0700000a 7 passed / 0 failed /
    2 ignored; 0700000b 3/0; 0700000c 2/0; 0700000d 1/0; 07000011 1/0.
  - **Not run this session (out of time):** walkreal_0700000c_eastroute_live (timed out at a
    300 s probe cap — needs a longer slot), walkreal_07000011_route_live, 07000012, 07000013,
    07000014, 07000015, 07000016, 0700001b, 07000041, 0700004a, 0700004a_satchel,
    walkreal_celprobe_live. No source changed, so these test the same tree the suite already
    covers; the next session should run them plus the suite.
- The full suite: **no total line this session.** The fast runner was started three times in the
  session's first half-hour and died each time at a tool-call boundary: this sandbox kills every
  background job between tool calls (plain `nohup` and even `setsid`-detached), twice before the
  build even finished (first `cargo` off the nohup PATH, then alsa-sys without
  `PKG_CONFIG_PATH` pointing at the user-prefix ALSA). The foreground time went to the
  touched-binary sweep above instead. Next session: run the suite the way the earlier sandboxes
  did — re-invoked foreground chunks (the `par_suite11.py` pattern, per-binary state JSON) — on
  this final tree and quote the total.

## What's left

- The remaining walkreal_* binaries (list above) and the full-suite total line on this tree.
- The live checks below (the main agent).

## Live checks for the main agent

1. **The post-drop flow in the real game.** Fresh boot (no save state), PCSX2 + gdbwatch: burn
   the four hinges, let the vent sweep, drop into the shaft, wait for the rest on the bay floor
   (the [L hatch-2] rest (5.42, 12.8773, −111.38)). Then: (a) hold every direction, crouch-walk
   south into the pocket, jump — record whether the body EVER leaves the shaft or a checkpoint
   reload triggers; (b) with fieldprobe on the player position while south-walking crouched,
   record the exact z the body stalls at — does the real capsule stop at −111.813 (the port's
   slab-contact value)? If the real game soft-locks too, the disc data stands and the stand-in
   stays forever, documented; if the game escapes somewhere, the escape is the route and the
   stand-in can die.
2. **The shaft's south slab at body height** (only if check 1 shows an escape): gdbwatch the
   collision pass (Collide_Pick 0x1eb320) while the crouched body presses south in the pocket —
   confirm the contact record is the z −112.34..−112.363 slab (the port's contact face), so the
   contact model is pinned [L] end to end.

## Usage

not visible to me (no token, context-window or cost figures are shown in this environment).
