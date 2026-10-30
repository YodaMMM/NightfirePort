# level-0c-real-crossings-2 — the real crossings on top of the green walk

Task: put `origin/s/level-0c-real-crossings`'s hinge-burn leg onto master+blockers-2, add the
remaining wire crossings (#284, #282) and descents on real input, keep blockers-2's #283 leg,
and keep the whole 0c walk green. FULL suite, quote its total line.

## What's done

1. **The branch carries blockers-2 + the real-crossings work, merged**: `s/level-0c-real-crossings-2`
   = origin/master 21fbeed3 + origin/s/level-0c-every-objective-blockers-2 (fast-forward,
   6082a67a — the #283 ride re-derived, the walk green) + origin/s/level-0c-real-crossings
   (merged at 21831436). The merge conflicts resolved by hand: the test file takes the
   crossings' legs everywhere EXCEPT the #283 ride/drop/settle, which keeps blockers-2's green
   version (the natural-stop jump + the air-then-ground settle); `docs/WORKLOG.md` got both
   branches' entries concatenated (append-only).
2. **The hinge-burn leg is REAL and green on the merged tree**: the walkway north (east of the
   raised block x 5..10 [D probe0c-lane]) to the inner balcony repro spot (7.61, 15.71, −113.39
   [L 991]), the Laser (0x4E) equipped through the real switch input, all four hinge locks
   #139-142 burned through the held-fire path (+144/+88/+90/+85 ticks; one contact per held
   tick, −3 off both burn bytes [M Lock_Update 0x1ae3d8:82-101 + Player_WeaponFiring
   0x1a5460:653-658]), the AND #505 → ch 18 [D], SS #138/#179 swept and pinned by the END POSE
   (the y-16.3 ray crossing the open vent [M the SS end poses]), the walk through the vent and
   the **trench drop firing plate #274 mid-fall at +37 ticks** → ch 0x1b, objective 1 complete
   on the game's own route ([D #274; L hatch-2]). The vent/pulley stand-in for objective 1 is
   gone.
3. **The wire crossings are real and green** (keeping blockers-2's drop mechanism): #284's
   jump grab at icon #4 + strafe ride + the jump press inside 0.9 of B landed at
   (−8.21, 12.82, −215.66) inside the west-ledge band; #282's contact grab + ride east landed
   at (4.97, 13.18, −223.31) inside the (4..8, −221..−226) band; **#283 is blockers-2's leg
   unchanged** — the ride to the natural stop (the mode-7 anim overshoots to d_B ≈ 1.1 past B
   [M 0x1977c0's d_b <= 0.5 + the anim root motion]), the jump there, the air-then-ground
   settle [M wire_tick 0x197758's stale grounded], landed (37.26, 12.14, −208.47) on the ring
   SE corner.
4. **The fire-variant finding (the one real divergence from real-crossings)**: 29b43fa9 moved
   the wire drops to the FIRE action (`Input_Action(9,1)` [M Player_SSGrapple 0x192860's arm])
   but was NEVER run on that branch ("the run of this variant is the next session's first
   step"). This session ran it: the #284 ride rode the full 2400 ticks to the natural stop
   d_B ≈ 1.05 past B WITHOUT releasing — the body hung at (−8.98, 12.65, −217.44), the y 12.65
   hang root = wire 13.67 − 1.02 [M hang 1.0207]. The wire release does not answer the trigger
   on this tree; the jump press does (blockers-2's three green drops). The #284/#282 drops
   therefore use blockers-2's `jump: d < 0.9`, with the observation recorded in the test
   comments; the FIRE variant stays documented here as the open question (live check 2).
5. **The walk is GREEN end to end**: `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`
   — 8347 ticks walked, 0 stuck, 0 falls, objectives 1-3 complete through the 0d handoff.
   Binary result: **2 passed, 0 failed**.

## The stand-ins that remain (each pinned, from both branches)

The case's trench exit (the glass lintel z −111.89 at y 13.2+ [D probe rays]; glass is
body-solid [M hatch-4]), the display eye + the plates reach (the unported 0xE2 vent/pulley;
the pad riser capsule-blocked [D probe]), the yard descent's east return (the unprobed face at
x ≈ 55.55 [this branch's stand-in pin]), and the gate-band crossing x 55.5..67 (the disc's own
collision is void there [D probe door4 pit map]). All carry sister rays/position pins in the
test so they cannot silently become walks.

## Verified

- `cargo test --offline --release -p nf-view --test walkreal_0700000c_live -- --test-threads=1`:
  **2 passed, 0 failed** (the whole-mission walk + the ladder495 sister).
- No `crates/` src changed vs origin/master (`git diff origin/master HEAD -- crates/` is empty
  outside test files: the merge brings test + docs only).

## FULL suite — PARTIAL (the window closed first)

`python3 tools/test/chunk_suite.py /home/z/suite_0crc2_state.json --budget 540`, repeated
(the serial chunked runner; the environment kills background jobs, so the chunks ran in the
foreground):

**446 of 911 test binaries recorded: 1409 passed, 2 failed, 82 ignored.**

The one failed binary is `fix_0c_pylon_live` (2 tests) — the **no-GPU environment**: both
tests panic at `wgpu`'s `adapter: NotFound` (the sandbox has no GPU; the same family as
probe_pylon_render's known environmental FAIL). Not from this branch: the branch's `crates/`
src is byte-identical to origin/master, and the failure reproduces on master's own binary.

What's left: the remaining ~465 binaries (resume the same chunk_suite invocation; the build
phase is done, chunks average 45-100 binaries each) and the final re-run merged with
origin/master. Risk is low — the only test file changed is walkreal_0700000c_live.rs (green)
plus docs.

## Live checks for the main agent

1. **The wire-drop button**: in the real game, riding a zipline/wire (e.g. `wire_firstsilo2mainsilos`
   #284, grab segment A (−4.63, 13.67, −209.88) B (−8.47, 13.67, −216.52) [D probe blk]) —
   which input releases the ride? Watch `Player_Wire` 0x197658 / `wire_tick` 0x197758's action
   reads (the access type: read of the pad/action word per tick during the ride) and whether
   the release is the JUMP action, the FIRE action (9), or the use key. The port's jump press
   drops (three green drops), the fire trigger does not answer — if the game drops on FIRE,
   the port's wire release has a gap; if on JUMP, the port is right and the test comments
   stand.
2. **The gate-band crossing x 55.5..67** (carried blocker): the game's own nav path17 links
   (49.69, 10.74, −207.29)↔(63.06, 10.74, −207.54) across the void [D probe door4] — a live
   walk of the band's east end would show what carries the player (an unprobed mover, a
   zipline, or a collision the port mis-reads).

## Usage

not visible to me
