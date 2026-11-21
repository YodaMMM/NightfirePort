### retune-nb-01 progress

- Branch off master e0adca26e. Previous round's branch `s/retune-nb-01` did not exist on the
  remote at start (checked `git ls-remote`), so nothing to merge; the round ledger lives in the
  tests' ignore messages (walkreal_07000001_live.rs).
- Setup in progress: work-slim + ISO downloads, queue worktree, build tools.
- Levers landed: d3 SMG head-box aim (dy 0.75-first; the part-5 box measured at dy 0.55-0.65)
  WON the door phase (both shotgunners dead, ZERO volleys taken), nearer-shotgunner-first, the
  volley-break windows + quiet walk-ins (d3-gated; d1/d2 one pass, byte-identical); d2
  dry-retry gate (the held-gun measure) killed the t14006 death; d1/d2 pre-plate lane clear
  (real-damage-only via !tops_up) fights the visible pocket guards from the east lane and the
  pocket edge.
- Tried + reverted: the d1 opening of the east-armour fetch (the side door wedged, the escort
  died unattended during the pre-clear grind) — the gate is back at d3, the fact is in the code.
- Still red: d1/d2 die on the plate lane to the pocket guard #17 (parks on the stairwell-head
  lane, every approach inside the 3.0 forced band; the shoot machinery flanks instead of
  standing); d3 dies t75294 in the dry endgame (clip 1 spare 7) trading #3. The refreshed
  ledgers + the named next levers are in the ignore messages.
- Ran whole on the final merged tree: walkreal_07000001_live minus the three ignored = 5/0,
  interactables_07000001_live = 10/0. The fast suite produced no total line (the foreground
  build exceeded the window; background jobs do not survive between tool calls here).
- Report: docs/reports/retune-nb-01.md.

### retune-nb-01-2 progress

- Branch off master a128047d1 with s/retune-nb-01 merged in (it was not on master). Round 2 of
  the 01 spawn-to-exit-with-the-contact real-damage walks; ledger + next levers are in the
  ignore messages of walkreal_07000001_live.rs, report of round 1 at docs/reports/retune-nb-01.md.
- Setup in progress: work-slim + ISO downloads, queue worktree, build tools.
- d1 WON: the plate-lane pocket fight re-shaped on the clear-0 probe (all east stands walled;
  #17's head exposes from the corridor's NORTH edge (-28..-29, 15.5-16.5); the pocket-edge
  transit crosses the guards' advanced spots — parks vary per run, the transit was the v5-v9
  killer). The north-edge band-checked stand + the converged tap loop killed #17 in ONE head
  tap (zero damage), the walled #2/#4 left to the plate walk (the return fire killed #4 in
  590 ticks [L the v2 run]), exit to 07000002 at t29163, end health 46.8 [L the v3 run].
  The v2 lesson: a lure grind on the walled guards drew them onto the unattended contact —
  he died mid-grind and the exit fired as the escort-fail restart; the windows are gone
  (walled = left at once).
- d2 WON: the ledger's levers in order — the lure/hold crouch (in), the pocket tap loop (in),
  the fight-order pass measured a NO-OP (agent_order already put the shotgunners first; the
  sort gate now admits d2, same tick [L the v2 run]). The decisive lever: the d3 volley-race
  FIGHT KIT joined d2 (SMG pre-arm + head-box ladder + loose gates + lane fire, d2/d3-gated,
  d1 byte-identical) — the one-frame 33.6 hall volley is gone. Exit t15479, end health 31.8 +
  armour 50.0 [L the v3 run].
- d3 verified byte-identical to round 1: the same death t75294 at (-25.98,14.46) [L the run].
  Ledger refreshed in the ignore: the named next lever is a level-scoped walker field relaxing
  the no_fetch_z_above zone west of the hall door plane (the west-corridor drop fetch), then
  the #11 window cap, then the #3 trade posture.
- Ran whole on this tree: walkreal_07000001_live = 7 passed / 0 failed / 1 ignored (509 s),
  interactables_07000001_live = 10 passed / 0 failed (25.5 s).
- Round 2 close-out: d1 and d2 GREEN (ignores lifted, proof in the test docs + the report).
  d3 moved t75294 -> t83962 on the ammo x-scope lever; the refreshed ledger names the endgame
  walk posture as the next lever (the post-clear stairwell-head nav's own stuck-flank walked
  Bond into the mouth pack at d 0.8-0.9, inside the x2 band). The recovery tap posture is in
  (d3/real-damage gated) and engages once the walk posture holds. Report:
  docs/reports/retune-nb-01-2.md.

## retune-nb-01-3 (2026-11-21)

Round 3 on the 01 spawn-to-exit-with-the-contact 00 Agent real-damage walk. Branched from
origin/master (d57c3fdac), merged origin/s/retune-nb-01-2 (af1a1dbf6) — one doc-comment conflict
in walkreal_07000001_live.rs resolved keeping both sides' notes. Next: read retune-nb-01-2.md
and the ignore message on walk_..._real_damage_00agent, then real-run levers.

## retune-nb-01-3 (2026-11-21, end of round)

The round-2 ledger's levers landed for d3 and were measured on their own real runs:
flank_point_x_min + fight_hold_west_of + shield_boxes for d3 (the endgame walk posture), the tap
loop joins d3 (plate-lane + pre-stairwell pocket fights), the #11 window burn cap. The d3 death
moved t80171 -> t78420 -> t33826 -> t7568 and the refreshed ledger + next levers are in the ignore
message. d2 green; d1 red on the merged tree by the contact-protection race (tick-identical to the
merged base — every lever is d3-gated; the lever to extend it to the sisters is named). Report:
docs/reports/retune-nb-01-3.md.

## retune-nb-01-4 (2026-11-21)

Round 4 on the 01 spawn-to-exit-with-the-contact 00 Agent real-damage walk. Branched from
origin/master (14126d9e5), merged origin/s/retune-nb-01-3 (2cdd30e3f) — one conflict in
walkreal_07000001_live.rs resolved keeping master's walks-1121 un-ignored state (master's 01
00-agent walk completes at 8.4 health, t63360 per the walks-1121 note) with the round-3 ledger
kept verbatim as history. Next: read retune-nb-01-3.md, verify the walk's current state on real
runs (walkreal_07000001_live + interactables_07000001_live), then levers if still red.
