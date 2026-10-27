# level-0a-band-settle-7 — the walked-dy mechanism bounds pinned: the walk substate has NO vertical writer, the +0x750 add is per-tick, and the climb is real ([L]) but its writer narrows to the walk matrix's pitch-y — one live check from the port

## What was asked

Port the feet-band walked-dy growth (the hall rest 1.1285 [L 2026-10-26] vs the port's 1.1201)
from the game's ground/step rule [M Player_FeetOnPoint 0x194188, the walk's vertical], un-ignore
`walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game`, keep every walkreal green.

## What this round established

The branch is `origin/s/level-0a-band-settle-6-c` (= 6-b + master) merged with origin/master
(1d2651c0) = a64264ba. **No src change landed** — every candidate writer the decompile offers
was read, and the remaining one needs a live pin before it can be ported without guessing. The
findings, each load-bearing:

1. **The walk substate carries NO vertical writer** [M Player_SSWalk 0x192310 = Player_Move +
   AnimSetUpdate + HandleJump + the matrix rebuild; Player_Move 0x191dd0 writes only the
   smoothed local slots (+0x10/+0x18) and the turn (+0x34)]. A flat walk is y-neutral in the
   game's own move slots.
2. **The handler's `obj+0x30 += BL+0x750` add is per-tick, not a growing accumulator**
   [M Player_Init 0x1910f8:40 sets `obj+0xD4 = BL+0x740`; Collide_Update 0x1ea398:50 runs
   `Vec_Zero(obj+0xD4 + 0x10)` every tick]. So the port's `pos += push` model of the add is
   correct, and the "lifetime +0x750 growth" reading of the climbed dy is refuted.
3. **The grounded zero-skip is jump-gated, not a walk lift**: the handler skips the
   `BL+0x14 = 0; Vec_Zero(BL+0x50)` arms only while `BL+0x94c == 1` [M 0x1949a8 tail], and
   `BL+0x94c` is Player_HandleJump's jump state (0 none / 1 rising / 2 falling
   [M 0x1916b0:181/:69/:193]). The FeetOnPoint hitdata copy (the 0x60-byte `amemcpy` into
   BL+0x00 [M 0x194188 tail], gated on `surfaced` — dry land, per Player_MonitorAir 0x191a70's
   swim monitor) overlaps the move slots, but the grounded arm zeroes them every walking tick.
4. **The floors along the whole glide route are flat** [D band_settle_probe, the port's world
   tris]: the spawn, the lobby, the doorway mouth, the jamb west, the hall mid/deep all read
   floor y ≈ 2e-5 — there is no floor step; the climb is not a geometry step-up.
5. **The port's dynamics measured** [L-port band_settle_probe + NF_CAPS_LOG]: the flat
   approach does not climb at all (1.0891 rock-steady for 13 ticks); the jamb press (root
   x −7.754 — the SAME 0.55-radius standoff the game's f5/f13 frames read [L]) climbs
   +0.0025..0.0106/tick off the big edge face (cent (−8.304, 1.094, 15.601), the radial dir
   tilting up as the query rides above the sinking Q) to a peak 1.1720, then settles 1.1130.
   The game's press: peak 1.1465, settle 1.1285 [L band-settle-2 brief].
6. **The game's flat-walk climb is real**: the [L] frames show +0.02 in the f0-5 window
   (placed 1.12 → 1.1402 @f5) BEFORE the press — walking on the flat floor lifts Bond; the
   port's flat walk never lifts. This is "band-settle finding 1" confirmed with the frames.
7. **The one candidate left standing — the walk matrix's pitch-y** [M]: Player_SSWalk rebuilds
   the walk matrix `RotMatrix(−pitch·π/2, yaw, 0)` [M 0x192310 tail's RotMatrix/Concat; the
   same call the port models for zero-g], and the integration at 0x193600:284-295 multiplies
   the whole matrix by the local move — so the forward row's y component
   (`sin(pitch·π/2)·vf`) rides into obj+0x30 every walking tick. **The port's walk integration
   is flat** (crates/nf-game/src/player.rs, the walk branch: `pos[0] += c·vs + s·vf;
   pos[2] += −s·vs + c·vf` — no pos[1] term; only the SWIM branch rides the look-y
   `vel[1] = ld[1]·vf·60`). The [L] climb rate fits: +0.004/tick at vf ≈ 0.1 needs pitch ≈
   −0.025 (looking up ~1.5° — the probe placement's look). **[S] unverified**: whether the
   [L] probe's look pitch was nonzero, and what stops the climb at the 1.1285 rest (the pitch
   recentring vs the press descent), cannot be pinned statically — porting the pitch-y without
   that pin would shift every walked route on a guess, so it was NOT landed this window.

## Why the test stays ignored

The pin (settled 1.1285 [L 2026-10-26, fresh boot]) is not met: the port reads 1.1201. The
ignore note now carries this round's bounds (the flat floors, the no-vertical-writer verdict,
the per-tick +0x750, the press-y residue, the pitch-y candidate) so the next round starts from
the shortlist. No decompile-backed test was weakened; no pass/fail was changed.

## How it was verified

- `walkreal_0700000a_live --test-threads=1` — **5 passed / 0 failed / 3 ignored** (the note
  change only; the battery is code-identical to the 6-c base).
- `nf-game --lib --test-threads=1` — **578 passed / 0 failed / 5 ignored**.
- No src file changed this round, so the walkreal batteries are byte-identical to the base
  branch's green record (6-b: 01 3/0, 0a 5/0+3i; door-state-collision-2's window: all 24
  walkreal binaries green on the same player/world code).
- New probe `crates/nf-view/tests/band_settle_probe.rs` (assertion-free, --nocapture): the
  raw floor hits along the route + the per-tick glide trace through the doorway window.

## What's left

1. **The live check below** pins the probe's look pitch and the y together; if the climb tracks
   `sin(pitch·π/2)·vf`, port the pitch-y into the walk integration
   (`pos[1] += the fwd row's y·vf` from `RotMatrix(−pitch·π/2, yaw, 0)`), re-run the full
   walkreal battery (the change touches every walk), and re-derive the glide pin if the game's
   rest embeds the probe's own pitch.
2. If the pitch is pinned at 0 during the [L] walks, the pitch-y candidate dies and the
   remaining suspect is the press-side radial-y (the big edge face's dir tilt) — the
   face-level [L] push table (settle-5's live check 1, extended with q/p y per tick) decides.
3. The trio's other two (`the_corner_crosses_like_the_game`,
   `walk_office_floor_qworm_keyboard_hatch`) ride the same mechanisms.

## Live checks for the main agent

1. **The glide's pitch/y pair** [the decisive one]: fresh boot 0700000a, probe-place Bond at
   (−8.0, 1.12, 16.29) and walk west exactly as the 2026-10-26 glide; per frame for ~40 frames
   dump **obj+0x50 (the view pitch, BL+0x8a8's source) and obj+0x30.y**. Expected if the
   pitch-y candidate is right: `Δy/frame ≈ sin(pitch·π/2)·vf` with vf the smoothed forward —
   the +0.02 climb over f0-5 needs pitch ≈ −0.02..−0.03 at vf 0.1, and the f13→f21 descent
   needs the pitch crossing zero/negative. If pitch reads 0.0 the whole glide, the candidate
   is dead and the press-side radial-y is the suspect (check 2).
2. **The press's per-tick DeltaP y**: at the same glide, dump HITTEST+0x10 (BL+0x750) per tick
   through the press (the settle-5 check 1 protocol, extended): the y component per tick, with
   the contact's q and the query p (HITDATA+0x40 and the spine point). The game's press climbs
   +0.002/tick where the port climbs +0.008/tick — the q/p geometry decides whether the port's
   edge-face radial tilts too steeply.

## Usage

not visible to me — this session's driver surface shows no token, cost or context-window
figures (the IM gateway metadata carries only session/chat/trace ids).
