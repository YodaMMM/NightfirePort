# WORKLOG

(docs/WORKLOG.md does not exist on master; earlier rounds' entries were stripped from master on
merge — their ledgers live in the queue done/ files and the tests' ignore messages.)

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
