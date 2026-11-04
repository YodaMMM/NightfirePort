# WORKLOG (branch running copy)

## real-health-14-agent-00agent (started 2026-11-04 11:50 UTC)

Branch `s/real-health-14-agent-00agent` cut from origin/master (f9f6544c). Task: wrap
walkreal_07000014_live's `walk_the_infiltration_to_the_delta_exit_real_damage_agent` in
`with_real_damage` (its body rode the health top-up while the name says real damage — the
d2-sisters round flagged it) and make it pass on real health; then finish
`..._real_damage_00agent` (#[ignore]d — the tick-26340 wedge: the "1.72 landing" resupply leg
floats Bond 0.41 over the scoped floor at (33.84, 0.82, 30.95), 2.70 short of the drop
(32.50, 0.83, 33.29), the leg cap panics). Levers from
docs/reports/real-health-00agent-12-13-14-15.md (origin/s/real-health-00agent-12-13-14-15):
a d3-gated re-ground before the landing resupply, a direct walk_to the drop's cell, the
stairwell AIM-pair stands. Check why Bond floats first: if the port's grounding differs from
the game, fix the port, with the address. Real input only; no health/ammo/channel writes; no
teleports; never lower damage or health; helpers opt-in only. Done = both green (or the 00
Agent one ignored with a measured reason and the next lever), walkreal_07000014_live +
nf-game + every walkreal_*_live run, the full suite total line.

Progress:
- [11:55 UTC] Sandbox reused (previous task's tree, branch s/drone-block-swaps-g055-3, clean).
  Token refreshed, origin fetched, branch cut from origin/master f9f6544c; docs/reports does
  not exist on master (created fresh with this task's report); docs/WORKLOG.md not on master
  either — created with this entry only (lesson 5: append-only; other tasks' entries live on
  their branches). Stale workspace build artifacts from the previous branch deleted (125 MB)
  to fit the 1.1 GB disk headroom.
- [12:00 UTC] The d2 wrapper fix in crates/nf-view/tests/walkreal_07000014_live.rs: the
  `real_damage_agent` test now runs
  `walker::with_real_damage(|| with_agent_path_clear(|| ...at(2)))` — the same wrap the
  Operative (`..._real_damage`) and 00 Agent (`..._real_damage_00agent`) walks use. Build of
  the binary kicked; the d2 run on real health is next.
- [12:05 UTC] The d2 real-damage run: the walk runs the route (tick 25740, 36 hits for 67.5,
  lowest 32.5, 31 return-fire fights) and dies by the SAME wedge the d3 died in — the "1.72
  landing" leg cap, Bond floating grounded=false over a floor the dump says is in reach.
- [12:15-12:40 UTC] The wedge ROOT-CAUSED with a throwaway probe (zz_probe_wedge, not
  committed) + a temporary band print (removed): the player's floor ray sees the corridor
  floor (hit_y 0.0) but the band fails — cc=0.5500 (the CROUCH root) while crouched=false for
  24198 of 24325 band ticks [L the instrumented run]. The feet band reads cc
  [M Player_FeetOnPoint 0x194188:106 `hit − 0.1 <= cc`]: the crouch reach is 0.65, the
  standing 1.13 — and the slab skirt's ny-0.42 push (NF_CAPS_LOG) holds the float 0.65–0.86
  above the floor, just outside the crouch reach, forever. The desync's writer is THIS test
  file's own stance pokes (lines 242/414/1444 clear `crouched`, 1300 sets it, set_crouch
  toggles it — none touched cc) [M the game's own pair: crouch BL+0x7CC 0.55
  0x194df0:235/248; the stand restores the 1.0347 root — the port's pair player.rs:1637-1638].
  THE PORT IS FAITHFUL; the walk ran the whole route at the crouch root.
- [12:45-12:55 UTC] The fix LANDED (commit 971e480b) and verified through three exposed legs:
  the 1.72-landing float grounds; the SkyBridge crossing release (the standing capsule's
  spine sphere catches the east walkway's west-edge corner x 44.54 — the probe sweep: the gap
  columns x 41.8..44.2 at z 62.0..63.55 read the trench −0.60 first) needs the game's own
  west air-drift, lands x ≈ 42.9; the vent lane lands at the standing rest 3.41–3.47 (the
  assert window's 3.4 cap was a crouch-era artifact, widened to 3.6). The NEXT exposed
  interaction: "the lane south past the mouth" jams on the flush lid's 0.02 lip (the STUCK
  dump at (45.48, 3.42, 42.72), grounded, segment clear, no progress 6000 ticks) — the
  standing spine bottom (root−0.485) drags the floor and the port's live push displacement
  cancels the walk step at the lip, where the game's dead push tail [M Intersect_CylGeom
  0x109da0:447 — the +0x50 DeltaP guard] walks the band alone. Each later leg needs the same
  one-by-one re-verify — more than this window held, so the working tree returned to the
  wrap-only state (master behaviour; the binary 8 passed / 1 failed / 1 ignored) and the
  fix + the three leg fixes live in the branch history (971e480b + the drift diffs in the
  report) for the next session to re-land leg by leg.
- [13:00 UTC] Final: walkreal_07000014_live 8 passed / 1 failed (the d2 agent on real damage
  — the wedge, measured) / 1 ignored (the d3 00agent, unchanged). The report written; the
  full suite not run this window (the final tree's only source diff vs master is this test
  file's wrap + docs).

## real-health-14-00agent-2 (2026-11-04)

- [13:56-15:30 UTC] Cut s/real-health-14-00agent-2 from origin/s/real-health-14-agent-00agent
  + merged origin/master (5918eb5f). Re-landed the prior session's crossing work and measured
  it leg by leg: the cc fix (971e480b cherry-picked as 3c011896), the two SkyBridge crossing
  west-drifts (strafe +1.0 for 15 airborne ticks; the trench landing passes under the STANDING
  capsule — (45.08, 0.51, 62.49) measured), the four vent-lane windows widened to p[1] < 3.6
  (lane true at y 3.468 measured), and the lid leg re-worked: the report's hop-over is refuted
  by measurement (the standing capsule jams at the mouth's lintel — vel (0,0,0) freeze, hop
  arcs squashed at +0.032, and the stand gate's +0.95 LOS passes UNDER the lintel while
  crouched), so the leg crosses on the game's own crouch toggle (real input [M
  Player_HandleJump]) — over true measured to (45.13, 3.05, 40.02).
- [15:00 UTC] The next exposed leg: the crawl jams at the mezzanine's west lip (segment hit
  0.124) and NO stand spot exists along the lane (the headroom LOS blocked z 42.2..40.0 — a
  low duct). Both jams are the live-push vs dead-tail divergence [M 0x109da0:447]; the
  NF_WALK_CLIP=1 / NF_WALK_GHOST=1 experiments both break earlier legs (measured 1.9 s
  panics) — hour-scale to re-land. The standing capsule re-rolls five master-green walks
  (2 passed / 7 failed measured whole), so the tree returned to the wrap-only state at the
  tip (549b5d20); the re-land commits (3c011896 + 71d0651c) stay in the pushed history.
- [15:29 UTC] Final battery: walkreal_07000014_live 8 passed / 1 failed (the d2 agent wedge —
  the task continues from it) / 1 ignored (the d3); 0700000a 12/0/3, 07000012 3/0/0,
  0700001b 17/0/2, 07000002 3/0/1, 07000005 14/0/0, 0700000d 1/0/1,
  fix_guard_first_shot_live 1/0/0 — all green. The full suite did not fit this window.
