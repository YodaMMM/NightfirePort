# level-0a-office-hatch-2 — the 0700000a office walk, round 2 (blocked: root-caused, un-fix placed, not landed)

## Status

`walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` is still `#[ignore]`. The walk now
reaches ring leg 6 ("down past the west door") and wedges there — the previous round's blocker
position is confirmed, and this session instrumented the wedge to the triangle. Finishing
**blocked**: the collision-core fix that un-wedges it is not landed, and the remaining piece needs
either a live PCSX2 number or a longer decompile pass than this window allowed.

## The repro (this session, on master a67f1a27e + this branch)

Running the ignored test with the real input path only:

```
walk_to ring: down past the west door: 6000-tick cap
STUCK at ring: down past the west door: bond=(-17.87,1.18,4.87) target=(-17.50,1.30,2.50) d=2.40
grounded=false substate=0 ... floor_ny=1.00 ... cel=Some(182) ...
doors nearby: door@(-19.5,1.1,4.6) kind=0 state=0x4 pos01=0.00 ...
floor_below: Some(2.3841858e-7)
```

The file's other two tests pass (`test result: ok. 2 passed; 0 failed; 1 ignored`) — the ring's
west legs walk clean; the wedge is one corner.

## The root cause chain (per-tick probe; scratch probes removed, the numbers live in the WORKLOG)

1. **The glide** — Bond leaves the lift cab at y 1.1336 and *glides* over the y-0 office floor the
   whole way west: the band [M `Player_FeetOnPoint` 0x194188:106 — `HITDATA+0xC − 0.1 <= BLData+0xCC`]
   keeps him grounded while the floor is within cc + 0.1 = 1.1347 below, so no gravity fires and
   he never drops the 1.13 to the y-0 rest (1.0347). His rest y 1.1336 sits **0.0011 under the
   band's unhook edge** — the ignore message's "~1.5 cm margin at rest".
2. **The corner** — the leg's straight line presses him into a corner of [D] geometry at
   (−17.87, 4.87): thin 1.1-tall plates at z ≈ 4.5–4.6 spanning x −19.5..−17.5 (the stair
   stringer — the west door sits at the top of a [D] staircase, pivot y 1.09986) whose north face
   pushes +z, and a 0.734..0.838 step block at x −17.5..−16.59, z 4.43..4.79 whose west face
   pushes −x. The per-tick push sums to (−0.0156, +0.0165, +0.0988) and exactly cancels the walk's
   south-east press — x and z freeze.
3. **The lift** — the below-B edge contacts (q under the capsule's bottom end) add +0.0165 y per
   tick. The y dithers 1.1336 ↔ 1.2019, and every excursion past 1.1347 unhooks the band →
   `grounded=false` → gravity pulls him back → re-ground → dither forever. The walker's stuck
   mitigations can't clear a corner (both sidesteps feed him back into it on re-aim), so the leg
   burns the 6000-tick cap.
4. **The game's side** — the game's own push tail [M `Intersect_CylGeom` 0x109da0 ~:412-470:
   `v = q − HITTEST+0x28` (the A arm at BLData+0x760), push `= v/|v| · (+0x4c − |v|)`, DeltaP and
   HITTEST+0x10/+0x18 accumulate] produces the **same negative-magnitude upward flip** for
   below-span contacts (for a face at |q − A| > radius the magnitude goes negative and the
   direction reverses). So the lift is game-shaped. What the game does with it — the handler-side
   application (`Player_CollisionHandler` 0x1949a8:136-140 lands +0x750 on obj+0x30; whether it
   clamps), or simply that the real player's slide takes a different line — is the un-pinned piece
   this task stopped at.

The ignore message's "stuck→E-toggle closes the starts-open door #4" premise is **stale on this
master**: door #4 reads state 0x4 pos01 0.000 (open at its anim start) the whole wedge, and the
walker's `use_door` sees open/opening and never toggles. The wedge needs no door state change.

## What's left (the next message's work)

1. Decide the fix from the game's side. The two candidates, in evidence order:
   - **The push application** [M 0x1949a8:136-140]: port the handler's exact DeltaP landing (any
     clamp/component rule) — the port currently applies the raw accumulated push per tick.
   - **The glide height**: whether the game's player really glides out of the cab at y 1.1336 or
     drops to the y-0 rest earlier on the route (the disc's own floor profile under the corridor —
     the walk-off glide's span). If the game drops him at the cab mouth, the corner is never
     pressed at the band edge and the whole wedge evaporates.
2. Un-ignore the test, with the sister.
3. The full suite on the final tree.

## Live checks for the main agent

Fresh boot of 0700000a (a save-state resume skips the boot path — don't):

1. **The glide height**: walk Bond out of the lift cab toward the west offices (no stick presses
   beyond the walk), and read the player object's +0x30 (position) once he is in the west hall
   (≈ −10, 16): does his y read ≈ 1.13 (gliding at the cab height) or ≈ 1.03 (dropped to the y-0
   floor rest)? This decides fix candidate 2 in one number.
2. **The corner behaviour**: place Bond at (−17.87, ?, 4.87) fresh-boot (fieldprobe poke), hold
   forward toward (−17.5, 2.5) for 3 s, and record obj+0x30's y over the window: does the real
   game dither 1.13↔1.20 like the port, glide up and over the 0.84 step block, or slide west along
   the stringer plates? A gdbwatch on the `Player_FeetOnPoint` 0x194188 outputs (BLData+0x110 / the
   band's normal store) during the window shows whether the game's band also unhooks there.
3. **The push landing**: with the same wedge setup, gdbwatch the write to obj+0x30 from
   `Player_CollisionHandler` 0x1949a8:136-140 — record whether the landed DeltaP keeps the +y
   component the push tail computes (the negative-magnitude flip) or arrives clamped
   (e.g. horizontal only). That decides fix candidate 1 without a decompile pass.

## Run report

- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live` — the file's 2
  non-ignored tests pass, the task test still ignored (the repro above from `--ignored`).
- No source change landed this session (WORKLOG + this report only), so no other crates re-run;
  the previous session's suite state files are not valid for this branch.
- Scratch probes were used and deleted (never committed); the raw numbers are transcribed into
  `docs/WORKLOG.md` under this task's entry.

## Usage

not visible to me — this sandbox exposes no token, cost or context-window figures to the session.
