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

## retune-nb-14-7 — 14 infiltration to the delta exit 00 Agent walk green again (round 7)
Task ID: 1 (whole round)
Agent: main worker (NightfirePort sandbox)
Work Log:
- Fresh sandbox; cloned repo, branch s/retune-nb-14-7 from origin/master (ccd79b5be), merged origin/s/retune-nb-14-6 (not on master) — entry opened, checkpoint pushed.
- Round 7 body: the round-6 ignore message's ordered first step RAN (the NF_KIKO_PROBE read-only
  grid pass at the corridor-hold entry, tick 26224) and measured lever 1 DEAD — #20's boxes read
  exposed with infinite world-stop margin from the hold spot itself and ~30 grid spots (the sight
  gate was never the blocker; the rounds die on a real-flight asymmetry the gate cannot see). The
  lever-1 pocket-line hold (the east lane's west lip, 900 ticks) died the same way (t26386, #20's
  post shot −1.93 at d 3.1, his hp 10 full). The Kiko-corridor armour fetch is measured blocked
  (the north band wedges at (−46.82,1.06,42.38), head_bump, 6000-tick cap, twice). Lever 3 is REAL:
  the round-4 census's untried loop-mouth armour (32.26,0.87,31.14) — the take LANDED on real
  input at the descent foot (d3-gated, r(0.9)), +50 banked, the door-face arrival went 17.5 → ~87.6
  health (the bullets absorb [M Player_HandlePain 0x1902c8]); the walk now dies at the pocket with
  the armour drained (#20's −5.00/hit ×13, health 50.3 → 0.0 at t25389, 41 hits for 100.0). The
  ignore message carries the refreshed ledger + the round-8 levers (the server-room armour, the
  door-face drain law, the live checks). Test still #[ignore]d.
## retune-nb-14-8
Branch s/retune-nb-14-8, from origin/master + merge of s/retune-nb-14-7 (fresh sandbox, round 8).
Goal: walk_the_infiltration_to_the_delta_exit_real_damage_00agent green again.
Ledger from round 7 report to be read next; next lever per ignore message TBD.
### retune-nb-14-8 progress (2026-11-20, fresh sandbox)
Setup redone from scratch (clone, work-slim, ISO 369823203 verified, rustup, user-prefix ALSA).
Lever 1 (round-7 ignore message): the SERVER-ROOM armour (−27.29,0.87,30.96).
- [srv-probe] NF_SRV_PROBE read-only dump: the pickup is kind-3 live at the spot; a per-path nav route exists from the west-corridor end (−48.7,33.2) → armour (11 nodes, the z 27.3-27.8 lane); its middle node (−33.13,27.33) sits d 0.53 from the escape touch #456 — the legs bypass north (−33.1, 28.6).
- v1 run: the west-line take wedged on the (−30.6,29.9) door (ch_open 0, pos01 0.00 — the v10 ledger's door); the pocket is entered from the nav route's north node (−28.00,1.20,33.21) instead [L].
- v2 run: THE TAKE LANDED (armour 35.0 health 87.5 at t~14600 after the loop+fetch fights chewed 65). The bank carried the door face (health 87.5 at the Kiko corridor east — round 7 entered at 17.5) and the pocket #20 died (killed true t27930). Death moved t25389 → t28518: the gun ran DRY and the lane fights' resupply wander dragged Bond into #29/#23 convergence. Next: the wake-legs drop-scoop law re-arms the gun before the lane crossing.
### retune-nb-14-8 close-out
- v3/v4: the corridor-end drop scoop granted DRY guns (gun 0x1 clip 1 spare 0) — removed.
- v5: the Kiko-corridor armour take before the lane fired the resupply wander inside the leg (nav route swings south through the convergence zone) — death t30347, leg removed.
- v6 (final tree): server-room fetch banked, death t28518 (world −30.51 live wire during the dry-gun resupply wander). Still red; ignore message refreshed with the round-8 ledger + next levers.
- Whole-binary runs on the final tree: walkreal_07000014_live 9 passed / 0 failed / 1 ignored (172.24 s); interactables_07000014_live 10 passed / 0 failed (10.27 s). No crates/*/src changes; only walkreal_07000014_live.rs + docs touched.

## retune-nb-14-9
Branch s/retune-nb-14-9, from origin/master + merge of s/retune-nb-14-8 (fresh sandbox, round 9).

### retune-nb-14-9 progress (2026-11-20, fresh sandbox)
- Setup redone from the task message (clone into /tmp/my-project, work-slim + ISO downloads, rustup, ALSA user prefix, cargo fetch). Branch created and pushed with this WORKLOG entry at setup.
- In progress: reading the round-8 ignore message ledger; next levers in order (1) Kiko-corridor armour ledge approach via NF_SRV_PROBE, (2) door-face armour drain law, (3) dry-gun law west of the door.

### retune-nb-14-9 close-out (2026-11-20)
- Fresh sandbox (clone, work-slim + ISO, rustup 1.100, the ALSA user prefix needed PKG_CONFIG_ALLOW_SYSTEM_LIBS=1 + a rewritten alsa.pc prefix + RUSTFLAGS -L — no root this round; the repo moved to local disk, PolarFS small-file writes are ~340x slower). Branch pushed with the WORKLOG at setup; origin/s/retune-nb-14-8 merged clean.
- Lever 1 (the Kiko-corridor armour approach) MEASURED DEAD read-only: the NF_SRV_PROBE extension dumped nav_route lane-end->armour = NO ROUTE and armour->touch = NO ROUTE — the ledge is off the web both ways; never a walk leg spent on it.
- Lever 3 SHIPPED as the opt-in `no_lane_fetch` DamageLog law (the fights' dry/low fetch quieted; set only by this walk's round_dodge stream, cleared after the touch cut-scene). v1 caught the scope INSIDE the lane crossing (the mid-leg return-fire gate fires the LOW fetch); v2 fixed the scope: no fetch, no pocket swing, no wire death.
- Lever 2 landed as the Kiko fast taps REMOVED (v3): the v2 trace measured them zero-kill (#20 hp 10 unchanged ×4 stands) and pure drain (the 0x1 15+61 -> 0+19, the bank eaten standing in the GL line).
- Results: death t25389 -> t28518 (v1) -> t34252 (v2) -> t36527 (v3); the pocket squad killed (#29/#22/#31/#61/#30), the escape touch #456 REACHED, the wake legs loaded the GL armed (c15+s59). New death shape: the wake-legs node-1 ambush (#62, d 3.9, hp 10 full, 28.0 in 11 hits unanswered during the #43/#31 exchange).
- Whole-binary runs: walkreal_07000014_live 9 passed / 0 failed / 1 ignored (173.50 s); interactables_07000014_live 10 passed / 0 failed (15.27 s). The sisters stay green (the gates hold). Still red: the ignore message refreshed with the round-9 ledger + the next levers (the node-1 ambush law, the door-face bank drain, the lane-crossing wedge).
- v4: the round-10 loadout-law variant (switch_to the 0x11 crossbow, c2 dmg 60.0, before the wake legs) measured NET-NEGATIVE on its own run — the 90-tick switch stand re-rolled the deep stream (wake start health 41.7 vs 57.7; a node-0 resupply fetched Bond west, death t36285 vs t36527) — removed; the switch must ride inside a leg's walk. Final tree = v3 shape; whole binary on it: 9 passed / 0 failed / 1 ignored (172.73 s). Report pushed (1779aa6f2); ntfy unreachable (curl 28, python/wget 124 — recorded in the report).

### retune-nb-14-10 start (2026-11-20)
- Sandbox partially reset (repo kept on the leftover s/port-map-scripts branch, no work/, no ISO): branch s/retune-nb-14-10 cut from origin/master ccd79b5be, s/retune-nb-14-9 merged clean first (it was not on master — the round-9 ledger and the levers carried over). Setup redone: work-slim + ISO re-download; suite runner to follow. Round-10 plan: lever 1 the wake-legs node-1 ambush answer point, then the door-face bank drain, then the lane-crossing wedge walk_nav_to — every lever its own real run.

### retune-nb-14-10 close-out (2026-11-20)
- Sandbox partially reset (repo kept on a leftover branch, no work/, no ISO, no rust): branch s/retune-nb-14-10 cut from origin/master ccd79b5be with s/retune-nb-14-9 merged clean first (it was not on master). Setup redone (work-slim 5.6 s, the ISO verified at 369823203 bytes, py7zr for the 7z — p7zip missing this box; the ALSA user prefix survived in env.sh). The fast suite was started in the background in the first minutes as ordered; it was killed between tool calls before suite.out wrote a line (the rounds 4-9 failure mode) — the two required binaries' whole runs are the evidence.
- Lever 1 step 1 (v1): the round-9 answer-point scan at node 0 is a MEASURED NO-OP — the candidate dump read #28 (0x5000087, 0x14, d 1.3, eye AND gun blocked — the corner camp), #31 (d 11.5), #36 (d 15.3) and NO #62: he is not present/enabled at node 0 (the round-9 live-check question answered NO); the scan consumed zero ticks and the stream reproduced the round-9 v3 death tick-exactly (t36527) — stream-neutral when nothing is answerable.
- Lever 1 step 2 (v2, NF_RF_TRACE): the ROOT CAUSE is the finish_fights determined pass — the #43 stand broke on the unseen-60 rule, then the 1800-cap pass on hp-6 #43 held the machinery 752 ticks (no visibility break, no switch) while #62's 0x14 GL raced 28.0 in 11 hits unanswered (fighting=true blocked the return-fire gate the whole stretch).
- Lever SHIPPED as the opt-in pass-break in walker_b2's shoot_gated (switch_on_first_blood && finish_fights — both set only by this level's d1 + d3 wake-legs blocks): break the pass when the target has dealt no damage over the 240-frame window while another shooter HAS; the gate re-picks and the broken target takes the 240-tick cooldown.
- v3 (the lever's own real run): death t36527 -> t37477 (+950, the deepest yet). The break fired twice — the #31 pass broke at +40, the gate re-picked #28 and KILLED him t35326 (the corner camp dead before his second round), the #43 pass broke at +0; the node-1 leg ARRIVED for the first time (52.7 health vs the old stream's 28.0 at the race — +24.7 banked); the wake legs crossed nodes 2-21, the node-21 scan killed #51 AND #41. New death: node 22 at 14.2 health — the world blast (kind 27, (−17.6,1.5,−23.2), d 4.0 = the radius edge, 14.20 dealt) finished a pool the delta approach drained (#39 −3.50 + #40 −31.5 across two fights for his 0.4 hp with the fights DRY clip 4 spare 0 + #41 −7.0 during the scan). The round-11 levers (the near-dry switch to the owned 0x14 c15+s59 riding the leg's resupply gate, the node-21 fast-tap scan variant, the #40 tail law, the blast's trigger) are in the refreshed ignore message.
- Whole-binary runs: walkreal_07000014_live 9 passed / 0 failed / 1 ignored (174.47 s — the sisters green, the d1 walk with its own flag pair green); interactables_07000014_live 10 passed / 0 failed (12.89 s); the walker_b2 spot-checks walkreal_07000009_live 9/0 and walkreal_0700000b_live 8/0 (the other levels byte-identical — the flags short-circuit there). Still red: the 00agent sister (the refreshed ledger in the ignore message). Report pushed; ntfy to follow the three-step fallback.

### retune-nb-14-11 start (2026-11-20)
- Fresh sandbox (full reset: no repo, no work/, no ISO, no rust). Branch s/retune-nb-14-11 cut from origin/master ccd79b5be with s/retune-nb-14-10 merged clean first (it was not on master — the round-10 ledger, reports and levers carried over; WORKLOG appended only). Setup redone: work-slim + ISO to follow. Round-11 plan from the round-10 ignore message: lever 1(b) the node-21 scan's fast-tap variant (shoot_bounded_fast, the door-face 8-tick family) so #41 dies before his second volley — 1(a) the near-dry switch is measured net-negative and stays removed; then 1(c) the #40 tail law, then lever 2 the world blast's trigger. Every lever its own real run; walkreal_07000014_live + interactables_07000014_live required.

### retune-nb-14-11 close-out (2026-11-20)
- Fresh sandbox (full reset: repo, work-slim, ISO, rustup 1.100, ALSA user prefix all rebuilt; py7zr for the 7z again). Branch cut from origin/master ccd79b5be with s/retune-nb-14-10 merged clean first (it was not on master). The v0 baseline reproduced the round-10 death TICK-EXACTLY (t37477, (−13.77,1.04,−24.24), the world blast −14.20 at the radius edge) — the stream carried over clean. The suite.py fast runner was started in the background in the first minutes as ordered; killed between tool calls before suite.out wrote a line (the rounds 4-10 failure mode) — the two required binaries' whole runs are the evidence.
- Lever 1(b) (the node-21 scan's fast-tap cadence) MEASURED NET-NEGATIVE on its own real run (v1): the #51 fast fight sprayed 15 rounds (clip 29 -> 14) with ZERO landed — the no-slide park cannot find the live line the shipped 16-tick+slide fight finds — then the round-10 pass-break broke the dead fight at +120, #51 stayed alive, converged to d 2.9 and dealt 45.7 in 7 hits. Death t37477 -> t37122 (−355). Removed.
- Lever 1(c) (skip the <=1.0-hp tail at d > 10) MEASURED NO-OP — the trigger never exists: the round-10 ledger's "0.4 remaining" was a {:.0}-display misread; the pick-time diagnostic read #40 at 20.00 (first fight) and 11.60 (second fight) — the −21.0 was spent on a real 11.6-hp shooter and the ledger self-corrects. Code reverted.
- NEW trap geometry [L]: the blast is a FUSE — ~507 ticks after the node-21 door E-open; the route nodes 20-24 all sit inside r 4.0 of (−17.6,1.5,−23.2) (node 21's arrival 0.57 u — ON the centre, the scan's own spot); the crossing itself spends ~300 ticks inside, the d3 scans+fights blow the ~200-tick budget left. The d1 sister opens the same door, stands on the same centre, crosses green with no world hit near the zone (v4 sister run). The edge hit deals 14.2 — survivable above ~15 health. Round-12 levers in the refreshed ignore message: (1) the trap budget (probe the pre-door sightlines read-only, move the scan accordingly), (2) the first #40 fight's killed-true semantics (an accurate false walks Bond out of the zone on the existing 240-cooldown), (3) the round-9 levers 2/3.
- Whole-binary runs on the final tree (code identical to the v0 stream — the round's levers reverted): walkreal_07000014_live 9 passed / 0 failed / 1 ignored (169.59 s); interactables_07000014_live 10 passed / 0 failed (15.30 s). Still red: the 00agent sister (the refreshed ledger in the ignore message). Report follows.
- Round-12 lever 1's first step SHIPPED as the read-only NF_DELTA_PROBE hook (env-gated, zero ticks, byte-identical when unset): at node 20's arrival it dumps every live guard's eye/gun visibility. Measured [L]: the posts are #51 (−11.87,−20.11) d 8.2 and #41 (2.92,−23.99) d 22.9 — BOTH eyefalse gunfalse from the pre-door spot, both EAST of the door leaf — the pre-door scan relocation is dead (a no-op scan buys nothing), and the probe run's death reproduced t37477 tick-exactly (stream-neutral proof). The post-crossing scan form (node 24+) stays the round-12 lever 1 remainder. Final-tree binaries re-run: walkreal_07000014_live 9 passed / 0 failed / 1 ignored (172.01 s); interactables_07000014_live 10 passed / 0 failed (15.49 s).
- ntfy: curl 28 (connection timed out), python3 urllib 124 (hung past 20 s), wget exit 0 (delivered on the third fallback; a second wget confirm also exit 0).

### retune-nb-14-12 start (2026-11-21)
- Fresh sandbox (full reset: no repo, no work/, no ISO). Branch s/retune-nb-14-12 cut from origin/master ccd79b5be with s/retune-nb-14-11 merged clean first (it was not on master — the round-11 ledger, probe hook and reports carried over; WORKLOG appended only). Setup redone: work-slim + ISO downloads next. Round-12 plan from the round-11 ignore message: lever 1 remainder is the POST-crossing scan form (node 24+; the pre-door relocation was measured a no-op — both posts eye/gun-false east of the door); then lever 2 (the first #40 fight's killed-true semantics), then the round-9 levers 2/3. Every lever its own real run; walkreal_07000014_live + interactables_07000014_live required whole.

### retune-nb-14-12 close-out (2026-11-21)
- Fresh sandbox (full reset: no repo, no work/, no ISO, no rust). Branch s/retune-nb-14-12 cut from origin/master ccd79b5be with s/retune-nb-14-11 merged clean first (it was not on master — the round-11 ledger, the NF_DELTA_PROBE hook and the reports carried over; WORKLOG appended only). Setup redone: work-slim + the ISO (369823203 bytes verified) via py7zr (no p7zip), rustup stable 1.100, the ALSA user prefix (the dev package's libasound.so symlink is DANGLING on this box — repointed at the system's /usr/lib/x86_64-linux-gnu/libasound.so.2.0.0; the link error persisted through a RUSTFLAGS -L round before the symlink fix, one full rebuild burned), LIBRARY_PATH/RUSTFLAGS in env.sh, cargo fetch. Disk: the rootfs holds deletions from the lower layer (df never reflects them) — ~/.venv and the browser caches were removed to make link room; the full 4-crate test build does NOT fit, so the round's builds were the two required binaries only.
- The v0 baseline reproduced the round-11 death TICK-EXACTLY (t37477, the node-22 blast −14.20 into 14.2 health) — the stream carried over the merge clean. The round-11 prose's trap-budget fight list (two 300-capped fights + the second #40 fight + the resupplies) does NOT match this stream's ledger: the node-21 scan killed #51+#41 in ~150 ticks for −7.0 and the killers were #39/#40's FIRST volleys (−3.50 + −7.00, d 17.8/15.3 — both posts sit BEYOND the scan's 20 u gate at every route node through 24) plus #40's in-flight burst −21.0 while the leg fight answered.
- Lever 1 (the round-11 ignore message's post-crossing scan form) judged on its own real runs: v1 (the scan moved to node 24, the shipped laws) — the fights broke at +0 (stale window blood: #51's t36915 volley sat in the 240-frame window while the silent post #40 was the pick), four zero-tick attempts, the scan a NO-OP — and the FUSE WAS DEFEATED: no world hit near the zone, the walk crossed the wake legs and the touch, death t37477 → t38906 (+1429, the deepest of rounds 4-12) at Rook's first 0x6e volley (−10.70, d 9.8, hp 22 FULL) into the 10.7-health chase endgame — the pool 0.7 short; the shoot stage never started (Rook was never a shooter before that volley). v2 (an opt-in scan-fights-hold-pass-break flag, set by the delta d3 scan only) — the fights RAN and measured DOUBLE-DAMAGE: the in-hand GL (2.5 contact) burned 300 ticks for −3 hp on the hp-20 post at d 17.4 and dried the gun on #42, and the shoot_bounded slide dragged Bond 3.9 u into a NEWLY-MEASURED SECOND kind-27 trap — (−4.4,1.7,−25.3) r 4.0 nominal 50.0 owner −1, route nodes 25/26 INSIDE its radius (d 1.94/2.30) — fuse t37634, −45.70 at d 1.6, death (t38906 → t37634). The flag code REVERTED (the shared files back to HEAD); the scan stays at node 24 on the v1 law (a no-op whose candidates dump is free data; the ignore message warns a future upstream re-roll could wake it).
- The final tree's 00agent single run reproduces the v1 death TICK-EXACTLY (t38906 at (26.99,1.13,−8.62)) — the ignore-message edit is stream-neutral. Whole-binary runs on the FINAL tree (--test-threads=1): walkreal_07000014_live 9 passed / 0 failed / 1 ignored (178.05 s — the sisters green, byte-identical); interactables_07000014_live 10 passed / 0 failed (15.63 s). The fast suite was NOT startable to a total line this round: the 4-crate build does not fit the rootfs alongside the two required binaries (the disk-full link failures are in build3/build4.log — untracked scratch), so the two required binaries' whole runs are the evidence, as in rounds 4-11.
- Still red: the 00agent sister at t38906 (the refreshed ledger + the round-13 levers in the ignore message: the Rook-chase pool (+10.7 needed — the #51 pre-emption at node 23 needs a near-first pick, the crossbow switch re-tested at the room position), the second trap's trigger (a live check), the round-9 levers 2/3).
- ntfy: to follow after the report push (the three-step fallback).
- Round-12 addendum (v3, the round-13 lever 1(a) judged within this round's window): the near-guard pre-emption at node 23 (an 8.0 u radius-gated scan admitting #51 alone, d 3.2 eyetrue guntrue) KILLED #51 before his two-volley −7.0 as designed — and the re-roll drew a WORSE chain: #55 (0x500008c 0x1a, silent and quietly killed at t37285 on the v1 stream) stood at his (8.7,−23.9) post firing EIGHT unanswered −3.50 volleys (t37352-37542; the leg fights never landed on him at d 14.9), death t37542 (t38906 → t37542, −1364). Lever 1(a) MEASURED DEAD; the tree reverted to the v1 state (the scan stays at node 24 only, the shipped laws). The ignore message records the verdict and the round-13 levers stand re-ordered: the crossbow switch at the room position next, then the second trap's trigger, then the round-9 levers 2/3.

### retune-nb-14-13 open (2026-11-21)
- Fresh sandbox again (no repo, no work/, no ISO). Branch s/retune-nb-14-13 cut from origin/master 502f62fae with s/retune-nb-14-12 merged clean first (not on master — the round-12 ledger, the node-24 scan and the reports carry over; WORKLOG appended only). Setup redone; next lever per the round-12 close-out: the crossbow switch at the room position, then the second trap's trigger, then the round-9 levers 2/3.

### retune-nb-14-13 close-out (2026-11-21)
- Fresh sandbox (full reset). Branch s/retune-nb-14-13 cut from origin/master 502f62fae with s/retune-nb-14-12 merged clean first (not on master). Setup redone: work-slim + the ISO (369823203 bytes verified) via py7zr, rustup stable 1.100, the ALSA user prefix (complete this round — the round-12 dangling-symlink trap did not recur), cargo fetch.
- The v0 baseline reproduced the round-12 death TICK-EXACTLY (t38906, Rook's first 0x6e volley −10.70 at d 9.8 into the 10.7-hp pool, 58 hits for 100.0, 48 fights). New facts: the attempt-loop aim prints read Rook at hp 30; the crossbow rides the save at 0x11(c2+0 dmg60.0) — two bolts, no spare.
- Round-13 lever 1(b) (the crossbow switch at the room position, the round-9 design question) MEASURED DEAD SEVEN WAYS on its own real runs (v1 the press never runs — the death is mid-close-on; v2 the stale-snapshot gate; v3 the went-silent pass-break broke the aim at +0 + #57's four standing-target volleys; v4 the 2-tick press does not switch; v5 the held press works but both bolts miss at d 10.9 — AND THE BLIND POCKET discovery: every guard eyefalse/gunfalse at the stop, nobody shot, the pool held 21.2; v6 the 6.5 gate walked into the ~9.8 volley bubble, tick-exact vs v0; v7 the pocket GL bombardment's aim blocked, the slide law dragged Bond into the eyes, death t39025). The tree reverted to the shipped state after each; the whole binaries re-run green on the final tree.
- Whole-binary runs on the FINAL tree (--test-threads=1): walkreal_07000014_live 9 passed / 0 failed / 1 ignored (179.07 s — the sisters green, byte-identical); interactables_07000014_live 10 passed / 0 failed (13.17 s). The final tree's 00agent single run reproduces the v0 death tick-exactly — the ignore-message edit is stream-neutral, certified. The fast suite was started in the background on the final tree (the tail read before the final push; the round-12 disk lesson stands).
- Still red: the 00agent sister at t38906. The ignore message carries the round-14 levers: the pool (#58/#57 pre-emptions, then the blind-pocket bombardment completion — read exposed_aim before standing, bombard where Some, step a leg where None), the second trap's trigger (the live check), the round-9 levers 2/3.

### retune-nb-14-14 close-out (2026-11-21) — THE 00 AGENT WALK IS GREEN
- Fresh sandbox (full reset). Branch s/retune-nb-14-14 cut from origin/master b69bacbc9 with s/retune-nb-14-13 merged clean first (it was not on master; the merge carried the round-4..13 reports, ledgers and WORKLOG entries; WORKLOG appended only).
- The v0 baseline facts re-read from the round-13 logs (untracked scratch, still on disk): the death t38906 = Rook's first 0x6e volley −10.70 at d 9.8 into the 10.7 pool; #58's burst −7.0 (t38808/38812, skin 0x0500008c, hp 10) is the whole deficit; and the round-14 ledger's premise revised — the v0 fight counter (48) exceeds its printed fights (47) by the fight that opened ON #58 only AT his own burst (the end-of-fight print never ran; Bond died inside it). #58 starts the chase at (28.47,1.0,43.15) — the far guards MIGRATE with the chase.
- Round-14 lever 1(a) judged on its own real runs, four forms: v1 (the sight-open break + the shipped fight) — the went-silent pass-break (walker_b2.rs:742, opt-in switch_on_first_blood && finish_fights) broke EVERY pre-empt fight at +0 (a pre-empt's target is bloodless BY DESIGN) and the 3-attempt budget burned with the walk ALIVE at 17.7 — the break spot itself measured BLIND (the pool held 17.7 for 210+ ticks, nobody shot); v2 (the pass-break scoped off, the plain fight) — the GL landed ZERO at d 19.5 in ~280 ticks and the fight's own slide law dragged Bond 4.4 u east into #53's point-blank line (d 1.8, −7.0), death t38939; v3 (the slide-off pocket hold) — parked Bond died to #53's advance (−21.2 in 6 hits, t38781; #53 lives because the break skips his nav-9 free-kill fight); v4 (the nearest-exposed ambush at ≤ 12 u, shoot_bounded_fast, the park held) — KILLED #53 (t38854) AND ROOK HIMSELF (the walked gun path, d 9.9, t38973 — the kill rounds 4-13 could never take) but left #57 (hp 2, the Rook-fight splash) at d 14-18 beyond the band and his two retreat volleys ate the 7.2 pool (death t39216).
- SHIPPED GREEN (v5): the sight-open break (walker_b2's opt-in `preempt_on`/`preempt_hit`, read-only ray tests, d3-gated, None default — the sisters byte-identical) parks Bond at the blind pocket; the ambush loop fights the nearest guard whose boxes expose within 20 u with shoot_bounded_fast (the slide OFF) and, when nothing exposes but a hostile still lives within 20 u, steps ONE nav leg toward the nearest (the leg's own unseen-fallback fights open the line); the went-silent pass-break is scoped OFF for the ambush scope only. Measured chain [L the v5/final-cert runs]: the break at nav 8 +31 ((27.25,−10.13)), #53 killed t38854 (d 4.0), Rook killed t38973 (d 9.9), #57 killed t39013 (d 14.2, ZERO pool cost), #58 killed t39201 (d 17.2, ZERO pool cost — his −7.0 burst never fired), pool 7.2 held through the delta legs, walked out in 40649 ticks, stuck 0, falls 0, 07000015 loaded warm (arrival (94.50,41.37,0.0019)).
- The walk's #[ignore] is REMOVED. Whole-binary evidence: walkreal_07000014_live 10 passed / 0 failed / 0 ignored (210.46 s — the sisters green, the 00agent walk now RUNS), interactables_07000014_live 10 passed / 0 failed (12.57 s).
- Changed files: crates/nf-view/tests/common/walker_b2.rs (the opt-in pre-empt fields + the dead-while-None hook), crates/nf-view/tests/walkreal_07000014_live.rs (the un-ignore, the round-14 ledger, the d3-gated ambush), docs/WORKLOG.md, docs/reports/retune-nb-14-14.md. No file under crates/*/src/ changed.
