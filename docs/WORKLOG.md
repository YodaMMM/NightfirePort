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
