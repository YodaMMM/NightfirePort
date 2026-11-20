# WORKLOG

(docs/WORKLOG.md does not exist on master; earlier rounds' entries were stripped from master on
merge — their ledgers live in the queue done/ files and the tests' ignore messages.)

## retune-nb-14 (2026-11-19, s/retune-nb-14)

Task: commit 3412a89a3 made every guard run the game spacing scan and the close-range alt attack from the fire states; the level 14 walks `walk_the_infiltration_to_the_delta_exit_real_damage` and its `_00agent` sister now die. Read both ignores and the 14 walk changes in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.

### retune-nb-14 progress

- Branch cut from origin/master 73822d3a0. Both ignore messages read:
  - `_00agent` (d3): Bond dies t47542 at (-48.91,1.13,34.27), drone #33 at 9.6 u (leg 4 east, the Kiko corridor / the east lane above the pocket line).
  - `real_damage` (d1): Bond dies t18133 at (-26.09,0.55,-10.56), a 29.0 world blast (the leg-5 wake-legs nav route).
- The same commit's 14 walk changes: `shoot_the_drop_band_guard` extracted and now called on every walk (the close-range alt attack holds the drop band), `drop_aim_z` corpse band widened, the tripbomb route calls it too.
- d1 `real_damage` GREEN and UN-IGNORED: v4 walked out in 41051 ticks (stuck 0, falls 0, the exit hold drained, 07000015 loaded warm). Levers, each on its own run: (a) the south-loop GL ambush (#12/#13, both 0x14, point-blank 2.0/hit + blasts, the 29.0 world blast death t18133) answered with the d3 fight law's `back_off 4.0` + `finish_fights`, gated `else if !w.tops_up()` (tops-up walks byte-identical); v1's `lane_fights` resupply fetches wedged the descent's node-57 leg (the displacement parked Bond d 4.47 short) so lane_fights stays OFF for d1. (b) the wake legs' 0x1a posts #28/#29 dealt 65.2 before the return-fire fights finished them and the d 15.8 fight burned the gun dry (death t36801 unarmed) — the d1 run takes the d3 chain's node-by-node wake walk + `delta_approach_scan` at node 21 + the loot pass keyed on the dry gun (clip+spare < 10) instead of a fixed node.
- d3 `_00agent` STILL RED: v3 reached the west-door legs at 7.5 health, death t44004; ledger 30 hits/100.0 over 24 fights — the fights kill every shooter only AFTER his first rounds and the first-round tax outpaces them (the descent corridor's four guards 55.0 alone). Measured improvements shipped (gated d3+!tops_up): `switch_to_gun` before the switch-band stands (the Laser-in-hand trap — switch_to_armed reads the 500 beam charge as ammo; the v1/v2 stands burned 49 rounds on east-of-the-door guards, ZERO land through the closed door plane) and the gun-ray gate on the switch-band + Kiko picks (#20 eye-visible muzzle-blocked at d 6.6 landed ZERO in 300 ticks). The measured ledger + next levers (the descent-corridor scan at the leg-2 spot; the door-leg scan after pos01 > 0.5) are in the ignore message.

### retune-nb-14-4 progress

- Branch off master 72a68f828. origin/s/retune-nb-14-3 does NOT exist on the remote (round 3's
  branch was never pushed); its ledger was reconstructed from the queue's done/50-retune-nb-14-3.md
  and the shipped ignore message. Nothing to merge.
- Baseline on this tree: the shipped round-3 code WEDGES loop node 5 (the v15 shape became the
  baseline — the master merges 22d5836b7/e4fb1c512 re-rolled the stream; 6000-tick cap,
  grounded=false on the hover cells, every attacker already dead, 25 hits for 72.5, 19 fights).
- Lever v16 SHIPPED: the fights' feet tied across the south loop (descent_stationary_fights, the
  level-13 stretch idiom, restored before the north leg) — the wedge is gone, the loop crosses,
  death moved to the door's west face t25036 (health 17.5 there, 34 hits for 100.0, 28 fights).
- Lever v17 (the round-3 lever 2, the hall scoop) measured NO-OP: the hall's north-west has zero
  live pickups and the dry gate never fires (the Laser 0x4e clip 499 counts as clip); the door
  face is reached at clip 10 + spare 89 — ammo is NOT binding, the health pool is.
- Levers v18/v19 (the machine-room armour, the task's pickup scan) measured DEAD both axes: the
  north approach latches the mezzanine door (32.0, 2.1, 35.8); the junction approach wedges 2.83
  short on the shelf rim (the v11 shape); both fetches removed, the facts kept as comments. The
  census ([L], NF_PICKUP_CENSUS, read-only): four kind-3 armours on the south half, the reachable
  one at (−41.29, 1.53, 36.48) in the Kiko corridor (live at the door face, d 19.1).
- The round-3 d1-sister "master regression" (walk_the_infiltration_to_the_delta_exit_real_damage_
  agent) PASSES on this master — the newer master commits fixed it; nothing done here.
- Required binaries run whole on the final tree: walkreal_07000014_live 8/0/2 (128 s),
  interactables_07000014_live 10/0/0 (13 s). The fast suite produced no total line (the sandbox
  kills background jobs between tool calls; the foreground four-crate build exceeded the window).
- Still red: the ledger + the next levers (door-face survival, the Kiko-corridor armour, then the
  round-3 lever 3) are in the ignore message. Report: docs/reports/retune-nb-14-4.md.

### retune-nb-14-5 progress

- Merged origin/s/retune-nb-14-4 (the round-4 ledger, v16-v19) and origin/s/retune-nb-14 (the
  Operative sister's green d1 walk, un-ignored) into s/retune-nb-14-5 off master 434826a9b.
- Re-measure on the merged tree [L 2026-11-19]: the 00agent death is UNCHANGED from the round-4
  ledger (t25036, (−60.22,0.73,37.94), 17.5 health, 34 hits/100.0, 28 fights) — the SniperAim
  ENTER merge did not re-roll the d3 stream before the door face. The round-4 ledger stands.
- Lever 1 (the door face at 17.5) shipped in five measured pieces [L, each its own run]: (a) the
  fights' switch_on_first_blood (opt-in, damage_log.rs — the retarget law without the 30-tick
  cadence and the 3.0 margin; the re-measure showed the fight on #33 started t25030 and Bond
  died the tick his 55 damage landed); (b) the west-face leg's return-fire mute + the r(0.3)
  line park (the mid-leg fight parked Bond 1.6 u south of the corridor line where #32/#33 are
  near-collinear 0.74 u and the stands took ~2 dmg/tick); (c) the door-face scans' weaker-first
  rank + the 4-tick slide-off fights (walker_b2 shoot_bounded_fast: the shipped 16-tick cadence
  measured 44 damage over 50 ticks; the slide's 60-tick strafe dragged Bond 3 u off the line
  into the blocked geometry). RESULT [L run 5]: the squad #33/#34 died INSIDE the west-face
  scan (killed true, killed true −22, ZERO volleys) and the walk CROSSED the doorway — the
  death moved t25036 → t25939 (−47.75,1.05,42.58), 34 hits/100.0, 26 fights.
- The new death: the crossing leg ate a point-blank 15.57 at d 2.5 from the near guard
  (−49.6, 40.9) (the scans' gun-ray gate had dropped him — muzzle blocked from the west park,
  eye on Bond) and left 1.93 for #28 (0x14, d 9.3) in the Kiko hold; the corridor's #20 read
  still-alive-after-300 under the shipped shoot_bounded (zero of 18 taps at d 6.5). The
  refreshed ledger + next levers (the eye_ray_stands law d3-scoped to the door-leg scans; the
  fast-tap/eye-ray law for the Kiko picks; then the Kiko-corridor armour fetch) are in the
  ignore message. Still red: the 00agent walk stays ignored.
- The Operative d1 sister: green on s/retune-nb-14's own tree, but the merged master 434826a9b
  RE-ROLLED its stream — the re-measure hit the retune-nb-14-3-era death (t36801 at
  (5.05,1.13,−23.31), the wake legs' #28/#29 0x1a pair, 65.2 of the 100) [L run]. Re-ignored
  with the refreshed ledger; my changes are d3-gated and did not touch the d1 stream.
- Required binaries on the final tree: walkreal_07000014_live 8 passed / 1 ignored (the
  00agent, the refreshed ledger) / 1 ignored (the d1 sister, the refreshed ledger); the two
  non-00agent walk results unchanged. interactables_07000014_live run (see the report). The
  fast suite produced no total line (the sandbox kills background jobs between tool calls; the
  foreground four-crate build exceeds the window — the same note as round 4).

## retune-nb-14-6 — 14 infiltration to the delta exit walks green again (round 6)
Task ID: 1 (whole round)
Agent: main worker (NightfirePort sandbox)
Work Log:
- Fresh sandbox; cloned repo, branch s/retune-nb-14-6 from origin/master (ccd79b5be), merged origin/s/retune-nb-14-5 (not on master) cleanly at 240843c2b, pushed checkpoint. WORKLOG entry opened.
- Round 6 body: the Operative d1 walk green + un-ignored (three runs: run 0 the traced re-measure — same t36801 death as the round-5 ledger; run 1 the named candidates byte-identical — no retarget/no first-blood fired; run 2 green with fire_through_disabled (a dormant guard held the #28 stand's line its whole 300-tick cap; the game's own collide drops Enable(0) objects [M Collide_PickObj 0x1eae80]) + hurting_first (the cooled #28 pick silenced the response ~600 ticks while #29 chewed) — the guards died t35382/t35705, walked out 40206 ticks, 07000015 warm). The 00agent d3 re-measured: the SAME t26979 death as round 5 (the two new master commits re-rolled nothing) — ignore message refreshed with the round-6 ledger, the standing levers (the ray probe from candidate hold spots, then the armour fetch) and the d1-measured fight-law candidates. Binaries: walkreal_07000014_live 9 passed/0 failed/1 ignored (168.63 s), interactables_07000014_live 10 passed/0 failed. The background suite died between tool calls again (rounds 4-5's documented sandbox behaviour); the report carries the detail.
