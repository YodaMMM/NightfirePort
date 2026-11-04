
## real-health-14-00agent-3 (2026-11-04, branch s/real-health-14-00agent-3)

- [15:55 UTC] Branch cut from origin/s/real-health-14-00agent-2 and merged origin/master
  (fast-forward: the -2 tip is already in master's batch-1104j merge b57ad91a8). Setup
  (clone ok; downloads next). Plan: cherry-pick 3c011896+71d0651c, settle the live-push vs
  dead-tail divergence [M Intersect_CylGeom 0x109da0:447] first, then the connector/descent
  legs, wrap the d2 agent walk in with_real_damage, un-ignore the d3 when green.
- [19:07 UTC] The guard question SETTLED with the subagent's decompile trace: the walk push's
  gate is the REF-GUARD (|q − HITTEST+0x50| < radius [M 0x109da0:431-447]), not the +0x51
  flag gate — the player's HITTEST+0x9a is 0x0000 forever (Player_Init 0x1910f8:40 bases the
  HITTEST at BLData+0x740; no 0x7da writer exists; Collide_Update 0x1ea398:94 passes the SELF
  hittest's +0x9a), pick candidates arrive memset-clean (Collide_Pick 0x1eb320:199/303/348),
  so the push arm RUNS structurally — but +0x50 boots (0,0,0) [L ee_start] and its ONLY
  writer is the DeltaP add-back [M 0x1ebaa8:353-373] which needs a prior push: structurally
  dead for the walk far from the origin (the 0a [L] walked contacts sat 10-25 u from ref).
  The position move is the CALLER's: Player_CollisionHandler 0x1949a8:131-140 adds
  BLData+0x750 (HITTEST+0x10, zeroed per frame [M 0x1ea398:50]) to obj+0x30 when the pick
  list is non-empty. The port's spine through-push fallback (world.rs:801-805) is the
  divergence: at the 14 mezzanine lip it fires where the game's guard would not — measured:
  raw forward input holds x 45.1338 for 600 ticks (the push cancels the move), the walker
  STUCKs (45.13,3.05,39.87) seg 0.124, all 6 non-agent tests die on it under the re-land.
- [19:08 UTC] The coupled chain measured state by state: A (wrap-only) 9/0/1 green incl the
  d2 unwrapped; +cc fix (bd007c18f) 1/8/1 — the honest standing capsule catches the east
  walkway edge at the trench release; +strafe drift needs the 3.6 lane window (2/7 at the
  lane catch, release 3.468); the full re-land 2/7 — the lid crosses crouched (stood=false,
  the stand gate blocked z 42.2→40.0) and the connector jams at the lip. The d2 wrapped in
  with_real_damage SURVIVES the damage (32.5 left, 36 hits for 67.5, 31 fights) and wedges
  at walker_b2.rs:641 tick 25740 (the 1.72-landing float — the cc fix's target); back_off
  6.0 measured WORSE (died tick 5594, 45 hits for 100.0 — the corner LOS dies at range).
  The tip returns to the wrap-only state (tree byte-identical to a3b83fcae, verified); the
  re-land + the wrap + the cc fix ride the history one cherry-pick away. The principled
  fix is the lib push-tail port (the guard rule at world.rs:801-805) + the full walkreal
  battery (rule 10) — the next window's first move.

## audit-text-lines

- [21:08 UTC] Branch cut from origin/master 690e8fefb; WORKLOG restored from the latest
  running copy and entry started. Sandbox was reset (symlink target /home/z/nf gone) but
  the full state survived at /home/z/my-project/nf: work/ extracted (decomp, elf,
  action-symtab.tsv, ee_snapshot.bin), Nightfire-onfoot.iso 1061754880 bytes, nf-queue
  worktree present. Re-set the remote token, fetched origin (master now 690e8fefb).
  Task: sweep every Text_AddMsg 0x1b0c30 call in ACTION.ELF (23 functions per the task
  brief): type, id, label text, condition, display frames, port site; type rules at
  0x1b0c30:81-92 (4→5 outside a script camera, 6→1 in multiplayer, 4/5 follow Subtitles
  unless id 0xff). Fix missing/wrong lines with a clear port site, pin each with a test,
  report the rest as a table.
