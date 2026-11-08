# WORKLOG

One entry per task, newest at the bottom. Append only — never rewrite or reorder.

## level-0d-wire-jump-grab (s/level-0d-wire-jump-grab)

- 2026-11-08 20:07 UTC: started. The ignored real-damage Operative walk on 0700000d
  (walkreal_0700000d_live.rs ~415) fails at the wire jump-grab — Bond does not catch the
  wire. Plan: read the ignored test + the port's grab scan, reproduce on the real path with
  a per-tick Bond-vs-wire log, find the game's grab rule in the decompile (Player_Wire
  0x197658's grab arm / the pick), fix the port if it differs (test red on old code), else
  fix the route as a player would and un-ignore what passes.
