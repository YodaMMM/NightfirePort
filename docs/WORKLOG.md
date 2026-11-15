# WORKLOG

## real-health-00agent-14-landing-4 (started 2026-11-15 ~08:45 UTC, branch s/real-health-00agent-14-landing-4)

- Claimed. Base: origin/master 9f5b13b6e (walker_b2 round_dodge merged: the Kiko hold ends at
  52.5, the second hold dropped, death t35208 at (-13.09, 1.12, -23.10) past the escape touch
  on the Rook's-wake legs to #39 (0x500008c, 0x1a) with Bond limping at 2.5).
- Task: get the 00 Agent walk's test green — reduce the 50.0 bill on the 1039-tick walk east
  to the escape touch (t31000-32000, #20 and #30 15.0 each) and survive the Rook's-wake legs.
- Plan: reproduce with NF_DMG_TRACE, identify the unaccounted 20.0 of the 50.0 bill, measure
  the next lever (round_dodge coverage for the 0x1a rounds on the wake legs is unmeasured).
- Measured (11-15, 5 NF_DMG_TRACE runs): master's baseline re-verified (death t35208, the
  east bill 50.0 itemised); the east lane detour (lenient walk_to (-44.5, 34.0) + aimed
  pass) reaches the touch at 26.6, death t41509 past door #121 to #40/#41's 0x1a posts;
  two other lanes measured worse (t32831 / t37624, closed); the position-keyed post scan
  measured dead (the guards converge, a fight displacement hit the live wire). Next lever
  in the ignore message: the visibility-keyed scan from (-15.5, -22.6).
- walkreal_07000014_live 9 passed / 1 ignored (171.7 s), interactables_07000014_live 10
  passed (13.2 s). Report: docs/reports/real-health-00agent-14-landing-4.md.
- Measured (11-15, 2 more runs): the wake-leg scan levers are dead BOTH ways — the route
  split at (-15.5, -22.6) diverges the nav path into #35's post line (death t40255) and the
  walker's opt-in per-node scan diverges the early timeline (south-loop wedge, panic
  t15313); both reverted, the tree is byte-identical to the tested detour state. Next lever
  in the ignore message: the scan scoped to the nodes past door #121 only.
- walkreal_07000014_live re-run on the final tree: 9 passed / 1 ignored (178.1 s).
