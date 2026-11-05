# regression-read-player-move — Player movement and pickups: read the recent commits for player-visible breaks

Read every commit since 2026-10-20 touching: crates/nf-game/src/player.rs crates/nf-game/src/pickup.rs. For each, read the diff and ask what a player would see change. Check the claim against the game [M], then prove any suspect by running it through the real App path (HeadlessApp keys, at 1/240 as well as 1/60), not by reading alone. Name the commit that broke each thing and its first tag. Fix what you prove, with a test that fails first. Not casings, driving or gamepad. Report every commit range read and what was clean.

---
claimed-by: sandbox4 at 2026-11-05T15:08Z

---
## Report (done, 2026-11-05T16:29Z)


---
## Report from the worker (done, 2026-11-05T16:30Z)

# regression-read-player-move — the report

Branch `s/regression-read-player-move` (cut from origin/master 82405afdb). Task: read every commit
since 2026-10-20 touching `crates/nf-game/src/player.rs` and `crates/nf-game/src/pickup.rs`, judge
each diff for player-visible change, check the claims against the game [M], prove suspects through
the real App path (HeadlessApp keys, at 1/240 as well as 1/60), name the breaking commit and its
first tag, fix what is proven with a test that fails first. Out of scope: casings, driving,
gamepad.

## The audit

Pathspec log on the two files for 2026-10-20..HEAD = **67 commits** (each with its first tag
computed via `git tag --contains`). Every first-parent diff was read; the three batches (oldest
23, middle 16, newest 23 — the two Nov 4–5 commits I read personally) were also cross-checked by
an independent reviewer pass. Verdicts: **42 clean intended visible changes**, **9 doc-only**,
**8 not-visible at that commit** (additive fields with no reader, visibility renames, test-only),
**8 merges whose net in-scope change is covered by their branch commit** — no message-vs-diff
mismatch anywhere except the one proven break below.

Ranges read (all clean unless named): 10-20 hatch-crouch-0c3/0c4 (fb5e19524, c81312481 — the
pick-bit grounding experiments, the second reverting the first's grounded arm), 10-21/22
front-end/standins/options/goggles batches (8b79e076a, cb4438077, e06a9626a, 67e8a915c, 0df471687,
3b6592800, 26a1c7560, aa408b1c6, c751bfaa5, 39ede9453, c5b6143d1, a29826941, e685b83ef, e3e933afd),
10-22 scrubs (272cea833, 0fdae9614, 117b077bc — verified comment-only), 10-22/23
drop-stand-height/capsule-redo/audit-fixes (78f6bab30, abc582730, 00f5508de, a028ea46b, 7fbec2ccb,
d5e947b00), 10-24 office-hatch/sleeper-push/tally/cite-take batch (d9d07b532, 915b4923a, 25b303bc9,
074e18b9e, ebe55e00f, 3ea172ed8, 90e1eaea6, 25d8d14b5, 4772dcb6d, cc1cb19e3, 1ed1da934 — the
settle-hold narrowing verified line-by-line), 10-26/27 medal-gate/band-settle/placement/14-walk
(a35f55db6, 882187b6c, b55121af2, 2a305e510, 52f9092ee), 10-29 wire/clip (f40be7601, f9e7b2565,
bd9580cc9 — NF_WALK_CLIP default OFF verified), 10-31 weapon-handling/zero-g pair (0617ba3e1,
85735a9e4, 1e6c464ba, c3ed2d764 + its exact revert c03027ebc, 621de7c4c, 55b63bfba, 917fd0e65,
ed0b1f78f), 11-01 moving-accuracy cycle (70e0a3607 → 43752669e revert → 933065298/566f1d298
re-land — traced commit-by-commit, nothing lost between revert and re-land; 7312bb13c test-only;
6a1c75cb1/d75ea0424 comment-only; 757e6af67/c5ac13d3d difficulty carry; 3d97ed08c exact revert of
237f1f744), 11-03 6d1ef8e95 (the break, below), 11-04/05 63be85966 and e95ea7ca4 (below).

Suspects raised and cleared:
- **e685b83ef** (v0.1.18) inverted the stand-up gate for default-derived inputs (`crouch_toggle`
  unread in the pad/derived path ⇒ Hold ⇒ auto-stand). Real at that commit, but the repo's own
  next fix (aa408b1c6) reworked the gate to `crouch_hold` with the derive meaning the game's
  Toggle, and HEAD pins it: `the_crouch_option_picks_the_stand_condition` + the HeadlessApp
  "Crouch defaults Toggle [M 0x17edd8 +0x04 = 1]" pin. Nothing to fix today.
- **63be85966** (v0.1.31, camera case-0 eye fixes): re-verified in the decompile this session —
  the eye's surface ease is case-0/0xe-only in `Player_PositionCamera` [M 0x18f5d8 — the
  `surfaced ? 0.7 : 0.0` lerp 0.1/tick], the crouch-drop ramp is substate-4-only [M 0x18f5d8:55-70],
  and the air/drown head is the ANIM head bone (`AnimGetBoneWorldTrans`), not the camera eye
  [M 0x191a70:50-53] — the port's decouple is the right approximation. Clean.
- **e95ea7ca4** (v0.1.32, pickup respawn one tick later): re-verified [M 0x18e148:183-185 —
  `hidden while elapsed <= FRAME_RATE_INT·respawn·10`], code (drone.rs both tails) and unit test
  agree. Clean.

## The proven break: the grapple lost its own quad memset (6d1ef8e95, first tag v0.1.30)

**The game.** `Player_SSGrapple` 0x192860 ends every call with the move-quad zero: disasm
0x192a24-0x192a34 — `addiu $a0,$s2,0x10` / `$a1=0` / `addiu $a2,$zero,0x10`, `jal memset`
(symtab 0x23653c), the delay-slot `sqc2` copying the old quad into the +0x20 wish block; only the
phase-4 exit branch (`beq` at 0x192a18) skips it, and by then the quad is already zero from the
previous tick [M disasm work/elf/ACTION.ELF]. So a grapple entered WHILE WALKING reads STILL to
the guards from the first grapple tick, through the reel and the hang alike.

**The regression.** Two commits were each individually decompile-cited and still broke the pair:
566f1d298 (v0.1.28) documented "the move quad stays ZERO through the whole grapple [M 0x192860]"
when the tick zeroed the quad every on-foot tick; 6d1ef8e95 (v0.1.30) narrowed that tick zero to
CLIMB-only (correct per [M Player_Update 0x193600:129-134 — the Vec_Zero pair sits inside case 1])
under the comment "Every other substate RETAINS the previous tick's lanes" — false for grapple,
whose memset lives in its own substate body. Nothing re-zeroed the grapple side, so the last
walk's lanes rode the whole hang and the guards' accuracy moving test read a hanging Bond as
moving [M DroneWeap_DoOpponentTargetting 0x179a48:34-41 → DroneWeap_DoBulletAccuracy
0x179740:71-73's TargetMoving_Accuracy]. Player-visible: walk, fire the Q-Claw mid-walk, hang —
the port's guards lead their shots at the hanging Bond, the game's hold aim.

**The fix** (one line + citation): `grapple_tick` zeroes `move_local` on entry, mirroring the
memset's every-call scope. The hang drift is untouched (it rides `grapple_drift`, not the quad).

**Fail-first proofs** (both ran red before the fix, green after):
- Unit (nf-game lib): `grapple_entered_from_a_walk_does_not_carry_the_walk_lanes` — 10 strafe
  ticks, grapple, 30 hang ticks; failed with `move_local = [0.06694694, 0.0]` riding the hang.
- Real App path (nf-view lib, HeadlessApp keys): `the_grapple_hang_reads_still_after_a_walk_at_
  both_frame_steps` — mission-load 07000014, the window's own input build with KeyW held, the
  Q-Claw switch through the real switch lane, steer+fire latches the SkyBridge grapple point
  [D placement 45.29, 5.10, 62.94], reel to the phase-2 hang, W released, pin `move_local == 0` —
  run at BOTH `frame(1/60)` and `frame(1/240)` (four frames per tick). Failed without the fix,
  passes with it.

## Verification (each binary whole, --test-threads=1)

| Binary | Result |
|---|---|
| nf-game lib (684) | 679 passed, 5 ignored, 0 failed |
| nf-view lib (138) | 138 passed, 0 failed (150 s; includes the new two-step pin) |
| qclaw_live | 3/3 |
| qclaw_ride_07000012_live | 1/1 (a real Q-Claw ride) |
| gadget_grapple_live | 1/1 |
| rope_guard_hitbox_07000011_live | 6/6 (guards vs a hanging Bond) |
| probe14_grapple | 0 run, 1 ignored (probe) |
| walkreal_07000014_live | 7 ok incl. `the_east_duct_route_reaches_the_stairwell_slot` (the real SkyBridge grapple crossing) and both `walk_the_infiltration_to_the_delta_exit{,_real_damage}`; 1 pre-existing ignore (00agent); `real_damage_agent` SIGKILLed by the 3 GB sandbox (environment kill — the same walk without the agent flag passed whole) |
| nf-formats + nf-audio (all targets) | 125 passed, 0 failed |

The full four-crate suite command did not fit the session window; everything that uses the touched
code ran whole above. Not casings/driving/gamepad.

## What's left

- The full suite's remaining integration binaries (nf-view's non-grapple walks) did not run this
  session; the next session can run the standard command.
- Optional live check for the main agent (confirmatory only): in PCSX2 on 07000014, stand at the
  vent start, walk forward ~1 s, Q-Claw the SkyBridge point, hang — a guard firing at the hanging
  Bond should NOT lead him (the TargetMoving term 0 while hanging). Address to watch: BL+0x10/+0x14
  (the move quad) reads 0 on every SSGrapple tick [M 0x192860 memset].

## Usage

not visible to me — this session exposes no token, cost or context-window figures; no numbers to
copy.
