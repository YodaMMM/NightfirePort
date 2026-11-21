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
