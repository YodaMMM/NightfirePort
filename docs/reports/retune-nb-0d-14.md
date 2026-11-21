# retune-nb-0d-14 — round 14 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-14`, from origin/master ccd79b5be with origin/s/retune-nb-0d-13 merged in
first (0d-13 was not on master; clean merge). The walk `walk_arrival_to_the_c5_door_real_damage_00agent`
stays RED — but the round-14 lever (c) **LANDED ON ITS OWN ROLL**: the plate act killed the roof
shooter at +26 ticks, and the walk reached the **return leg at t24195 — the furthest ever**
(round 13 died t23299 mid roof-act at the trench). Seven real runs this round (14-e..14-j); the
refreshed ledger and the round-15 lever are in the ignore message item (14).

## What was done

1. **The round-14 lever (c) — the plate act — PROVEN** (run 14-e): `NF_0D_PLATE_ROOF_KILL=1`
   fires the certain box act on the roof shooter (138.9, −325.3) y<11 from the PLATE stand
   (135.7, −307.4) — d ~17, the ~13-deg pitch over the open yard — before the trench walk.
   **Down at +26 ticks.** The trench roof act then found it already down, so the degenerate
   trench→roof up-pitch (round 13's cd(snap) 2.086 / 248) never ran. The convergence law this
   pins: an OPEN line at d ~17 lands in ~26 ticks — the 13-c north act's cd(snap) 5.367 refusal
   was the door-corner COVER, not the distance.
2. **The crouch law committed** (the round-13 addendum's own refinement): the trench acts'
   stand press (the 5-tick idiom) now precedes the up-pitch roof act; the trench sleepers
   (#25/#26) stay crouched; the yard legs stand either way. Gated to `NF_0D_STAND_CROUCH`
   (already committed); unset byte-identical.
3. **The photograph crouch measured QUIET** (run 14-h): `NF_0D_PHOTO_CROUCH=1` — the 5-tick
   press around the whole photograph phase (the camera switch, the holster, the shoot_at; the
   display is a LEVEL-pitch stand, so the crouch law allows it). The photograph phase took
   ZERO chips this roll; the death moved to t23536 (the plate arrival zone, the #4+#6 joint
   burst) — neutral on the roll, positive on the phase (the knob stays, default off).
4. **The NW kill measured — #4 is a MOVER** (runs 14-e/14-f/14-g/14-i): #4 (0x500007e, weapon
   0x12) chipped 14.4 (8 × 1.8) across the 14-e legs and killed the return leg at t24195 with a
   5 × 1.8 burst — and it sits 5.5 u from the shooting stand, where the same cadence would ride
   the door kill. But it MOVES: the landing census has it at (133.1, −273.8) and it parks at
   (128.5..129.6, −282.1..−282.5) by its first chip (frame 22935). The fixed boxes missed it at
   all three stands at ZERO ticks (14-f/14-g — the empty-box act returns at once, the timeline
   byte-identical: the free-act proof). The new INDEX-tracking certain act
   (`kill_guard_index_certain`, `NF_0D_NW_KILL=2` — the pick by drone index within a range cap,
   the aim and the line-hit gate read the mover's live boxes each tick) measured **NEGATIVE**:
   run 14-i died mid-act (t22895) — the aim at d 17.3 on the mover never converged in 141 ticks
   while #1 (the yard-east armed, a NEW top chipper at its (164.2, −286.7) park, 12.4 in 7
   hits) punished the exposure. The act aim phases ARE the exposure.
5. **The roll law pinned** (run 14-j): the same shape reproduced the SAME death exactly
   (t24195, (136.50, 4.78, −308.15), 24 hits 80.8) — the sight-event keying re-syncs after
   press-timing shifts within quiet phases, while knob presses that ADD aim phases re-roll
   worse (14-h t23536, 14-i t22895).
6. **The wall restated**: the yard phase spends the whole landing health (~34.8) across the
   legs + acts + photograph (~1000-1900 ticks); the trench is reached at ~10.8; the death is
   the return leg's plate-arrival-zone burst (#4+#6 joint fire, 5 × 1.8).

No guard code changed, no damage/health constants changed, no top-ups, no `crates/*/src/`
change, no `crates/nf-view/tests/common/` change. All the round-14 code is env-gated test-file
knobs (`NF_0D_PLATE_ROOF_KILL` proven, `NF_0D_PHOTO_CROUCH` proven quiet, `NF_0D_NW_KILL` 1 =
the free boxes / 2 = the index act measured negative) plus the crouch-law press reorder under
the already-gated `NF_0D_STAND_CROUCH`; every default is byte-identical, and the knobs-unset
whole binaries reproduce round 13's finals exactly (below).

## How it was verified

- The runs (all [L], the direct test binary, --test-threads=1, --exact --ignored, each its own
  roll, the composed shape STAND_FIRST+YARD_RUN+ANNEX_ARMOUR+CORRIDOR_ARMOUR 1+PIT_KILL+
  TAP_CADENCE 14+EXIT_CROUCH+WIRE_FAST+YARD_FAST+LAND_WAIT 10+SWEEP_PASSES 1+STAND_CROUCH
  unless noted): 14-e (+PLATE_ROOF_KILL) the plate act +26, the trench acts all "already down",
  death t24195 on the return leg (the #4 burst); 14-f (+NW_KILL=1) the early-catch box empty at
  0 ticks, the SAME death t24195; 14-g (+the plate/stand NW slots) the same; 14-h (+PHOTO_
  CROUCH) the photograph quiet, death t23536 (the #4+#6 joint burst, ~7.2 entering); 14-i
  (+NW_KILL=2) the index act burned 141+ ticks at d 17.3 on the mover, death t22895 mid-act to
  #1's 12.4; 14-j (the flagship shape again, no extra knobs) death t24195 — IDENTICAL to 14-e.
- Whole binaries on the knobs-unset final tree: **walkreal_0700000d_live 3 passed / 0 failed /
  2 ignored (100.5 s)** — `walk_arrival_to_the_c5_door`, `walk_arrival_to_the_c5_door_real_
  damage`, `walk_arrival_to_the_c5_door_real_damage_agent` all green; the 00agent walk and the
  hurt_census probe stay ignored — and **interactables_0700000d_live 11 passed / 0 failed
  (30.4 s)**.
- The fast runner (`tools/test/suite.py`) was started once at setup (setsid, per the task) and
  this sandbox killed it between tool calls with `suite.out` empty — the rounds-1-13 signature,
  reproduced again. The four-crate test-binary build also exceeds the rootfs left after the
  ISO + work + toolchain (3.4 GB free vs ~5.5 GB needed). The per-binary foreground runs above
  are the evidence, as in rounds 1-13.
- Lesson-10 grep: the touched file is the 0d test file itself (`grep -l kill_guard_certain_at
  crates/*/tests/*.rs` → only `walkreal_0700000d_live.rs`; no `crates/*/src/` and no
  `tests/common/` file changed), so no other binary is affected by construction; both 0d
  binaries were run whole anyway (above).

## What's left

- The walk: still red — the wall is the yard phase's total chip budget (~34.8 = the whole
  landing health) across ~1000-1900 ticks. The round-15 lever is in the ignore message item
  (14): (a) the return-leg reshape — a DIRECT trench→stand fast leg that skips the plate
  waypoint's arrival zone where the burst lands; (b) the #6 north act re-judged from the
  yard-south-1 stand on a roll where #6 stands in the open (the plate act's +26 proves the
  machinery converges fast on an open line — the 13-c refusal was the door-corner cover);
  (c) the #1 yard-east armed gets an index act slot at the trench (the (154.0, −291.0) box
  missed its (164.2, −286.7) park in 14-i). The proven machinery rides the knobs: the plate
  act, the crouch law, the photo crouch, the free NW boxes.
- The suite: needs a sandbox that keeps background processes alive between tool calls and a
  bigger rootfs; the four-crate build wall stands (3.4 GB free vs ~5.5 GB). Next session should
  run `tools/test/suite.py` whole on the merged tree if its sandbox allows, or per-binary with
  the tests chunked per process.
- Live checks for the main agent: none required this round — the levers were judged on the
  direct test binary's own runs. If the main agent can spare one probe: fieldprobe the
  plate→(129.6, −282.1) line's world-clear state with Bond standing at the plate (135.67,
  5.10, −307.42) watching the segment probe toward (129.6, 5.0, −282.1) — the 14-f/g box acts
  never fired because the target was absent, and a clear-line confirmation would raise the
  round-15 (b)/(c) acts' prior.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).
