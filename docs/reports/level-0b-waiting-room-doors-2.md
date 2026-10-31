# level-0b-waiting-room-doors-2 — the pick law reverted; the doors walk through the swing

Branch `s/level-0b-waiting-room-doors-2` (from `origin/s/level-0b-waiting-room-doors`
5aea1206; merged origin/master 20624c8c clean). Subject: the prior branch's pick fix (the
door's radius alone in `Doors::collide_step`) was the wrong arm, per the driver's read of
`Collide_PickObj` 0x1eae80 — confirm, revert, find the real difference, walk the transit.

## The revert ([M] code, `crates/nf-game/src/door.rs`)

Confirmed from the decompile: the player's hittest mode word (`BLData+0x7D4` = hittest+0x94)
is 0x800 on every `Player_Collision` 0x194df0 arm, and the 0x800 arm of `Collide_PickObj`
([M] 0x1eae80:152-168) is `sqdist(door centre, closest point on the pick capsule A..B) <
(1.5·r_door + r_pick)²` with r_pick = hittest+0x8C [M :160 the ×1.5, :164 the sum, :166 the
gate] — the radius-alone law is the 0x200/0x201 arm [M :91-107]. The port's r_pick mapping was
already right (walk/default arm 0.55 [M 0x194df0:316], climb 0/6 arm 0.275 [M :204]; the port's
mover radius carries exactly that). `collide_step` is back to master's `r = sph.1 * 1.5 +
m.radius`, with the citation corrected.

## The real difference (the diagnosis the task asked for)

The port's anti-crush matches the decompile line for line — `Door_ReactToHit` 0x135980: the
live-matrix inverse (Mat_Inverse stores −t unrotated [M 0x1e3578], RotPreTransVec = R·(v − t)
[M 0x1e49e8], so the local frame is the true hinge frame), `f = local.z·(+0x5C)` [M :81], the
negation past half travel [M :82-84], the ±0.0002 gates and the SetState law [M :85-97]. With
the reverted (game) pick radius, Bond at the E spot IS a permanent contact — and the trace
shows what the game's own law then does: **Bond's live-frame lz crosses zero mid-swing**
(the NF_DOORTRACE/[doorreact] capture: lz falls +0.353 → −0.028 as pos01 runs 0.48 → 0.34,
swing −1), so the away-swing reverses at the crossing (`f > 0.0002` while OPENING), the
closing arm reverses back (`f < −0.0002`), and the leaf locks into a **3-tick limit cycle
around Bond's lz = 0 leaf angle** (pos01 0.3448 ↔ 0.3966) — never clearing the doorway. The
prior branch's oscillation was this cycle, not a wrong pick constant; the radius-alone "fix"
only hid it by dropping the contact (and with it the game's anti-crush entirely).

**Why the game works**: the player never stands at the E spot waiting for the leaf — they
press E and WALK THROUGH the swinging doorway; the anti-crush yields (the closing branch
re-opens) and the leaf completes to its open end once Bond is through. `Door_Activate`
0x135be0 moves nobody [M :136-146 + Player_Activate 0x1a5170:128 — no stand law], the
activation's 2-tick busy timer [M 0x135be0:172] locks the react out only 2 ticks.

**The fix is the test's flow, not the law**: the walker presses E and immediately walks
through. `probe_0b_doors` (committed) proves both doorways pass: door #3 +70 ticks, door #0
+73 ticks, both leaves completing to their open ends (pos01 1.000 / 0.000, state 4) once Bond
is through.

## The walked transit (the stand-in deleted)

`walkreal_0700000b_live.rs`'s waiting-room exit is now walked with real input: E on door #3 →
through its opening (+62 ticks) → the east room on the game's nav lane (n53/n52/n50/n49/n19
[D]) → E on door #0 → through (+67 ticks, n17) → ring n11 — the old teleport stand-in (the
whole transit) is GONE. Kept, minimal and evidenced: the ~6 m dock→east-approach hop. The
waiting room's floor is split: the dock band (y 51.8) is an island — the pit x 3-8 z 23-25.5
(the SW rim line descends into the bowl and its east rim walls the walker at body height —
walked and wedged), the x 2-3.5 hole, the dome hole x 7-9.5 z 26.5+ [D the floor grid in
probe_0b_doors] — and the main floor's x 4.5-6.5 barrier (the N-S slot at x 5.0 z 17.5-23,
the furniture 48.2-48.8 at x 6-9.5 z 18-21.5, the wall band x 8-14 z 16-17) leaves only the
z 17.0 seam, which the wall band closes at x 9.5+. The nav lane's own n41→n55 hop rides
furniture tops the AI ignores (n55 sits on the 48.7 band [D the grid]).

## Verification (what I ran)

- `walkreal_0700000b_live`: **3 passed / 0 failed** (41.5 s) on the final tree (merged with
  origin/master 20624c8c).
- `interactables_0700000b_live`: **9 passed / 0 failed** (16.3 s) — including
  `waiting_room_door_opens_on_use` on the reverted law.
- Every `*door*` binary, both crates (33): **56 passed, 0 failed, 3 ignored** — nf-view:
  0700000d_gunimp_door 3/0, cutscene_door_07000014 3/0, door_aim_ray 2/0, door_glist0_e_dead
  2/0, door_guards_locked_door_07000014 2/0/1, door_hint_boot_count 2/0, door_portal_links
  3/0, door_portal 2/0, door_pose_probe 2/0, door_text_labels 4/0, door_wait_07000014 1/0,
  doors_remainder 4/0, fix_door_react_path 1/0, fix_npc_doors 1/0, npc_doors 6/0/1,
  party_door_lock_07000007 2/0, probe_07_door_lock 1/0, probe_0b_doors (ignored probe),
  rook_door_06 1/0, rook_door_06_walk 1/0, walkreal_probe0c_door2-6 1/0 each,
  walkreal_probe0c_door 1/0, walkreal_probe_door 1/0; nf-game: door_glist_census 1/0/1,
  door_guard 5/0, door_research_07000001 2/0, doors_01 2/0, doors 2/0, misc3_door_wipe 1/0.
- Build: `cargo test --offline --release --no-run` for the four crates on the final tree
  (977 test binaries).

## Suite total

The sandbox kills every background job between tool calls (the nohup'd `suite.py` cannot
survive; the prior three sessions hit the same wall), so the suite ran on the resumable
foreground protocol: per-binary `--test-threads=1` under NF_SILENT=1, 240 s cap, results
accumulated in a state JSON across invocations. State at report time (05:18 UTC):

- **465 of 976 test binaries completed: 1496 passed, 7 failed, 84 ignored, 0 CRASH.**
- The 7 failures are pre-existing/environmental, none touched by this diff (the revert
  restores master's exact law; the rest is test files): `disc_golden` (3 — the slim ISO's
  known allowed failures), `fix_0c_pylon_live` (2 — the GPU-less wgpu `adapter: NotFound`
  pair), `campaign_flow_live` (1 — the 240 s wall cap), `identity_disc` (1 — the slim ISO's
  identity bytes; a disc-dependent test, same environmental class as disc_golden).
- The remaining ~500 binaries are resumable (the state JSON lives in the sandbox home).

## What's left / open items

- The suite's remaining ~500 binaries (resumable protocol; the touched binaries and the whole
  door family are green above).
- The dock→east-approach hop (the kept stand-in) — settles only with a live look at how the
  disc's player descends from the cab dock band; the floor-maze evidence is in probe_0b_doors.
- The parachute section of this branch still carries the pre-parachute-task stance sweep (that
  fix lives on `s/level-0b-parachute-step-up`, not yet merged to master — untouched here).

## Live checks for the main agent

1. **The waiting-room doors on the disc** — PCSX2, fresh boot of 0700000b, Bond at the exec
   floor's lift landing (8.0, 46.8, 24.8). E door #3 (13.71, 46.78, 19.44) and WALK through
   while the leaf swings; then the east room south to door #0 (13.82, 46.78, 7.09), E and
   walk through to the ring. Expected: both crossings pass without waiting for the open state,
   and a Bond who STANDS at the E spot sees the leaf hesitate/reverse near mid-swing (the
   anti-crush limit cycle the port reproduces). Record whether the disc's standing-Bond door
   ever completes its swing (if it does, the port's anti-crush inputs differ — dump the door's
   +0x50/+0x58/+0x5C and the contact's local.z).
2. **The dock descent** — same boot: after the lift cut-scene, how does the player get from
   the cab dock band (y ≈ 51.8) down to the waiting-room floor (46.8)? The port's floor data
   offers no walked descent (the grid in probe_0b_doors). If the disc has a route (stairs/
   slope the grid missed), dump the cel faces along it.

## Usage

not visible to me
