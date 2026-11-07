- [04:05 UTC, Nov 7] script-break-drain started: branch cut from origin/master
  (22d5836b7). Plan per the driver: read the port's obj_type 0x20 arm in nis.rs
  (Objects::pending_breaks, nothing drains it on master) and the queue's
  done/30-port-map-scripts.md record; find how 07000008's 16 gondola windows break on
  master today and keep a single path; add the drain after the scripts tick with
  Break_Create's kind/health/filter from the params and per-tick bounds following the
  stream's posed matrix; re-point an existing same-class stream object instead of adding
  a breakable [M 0x1d8a68-0x1d8a80]; census every disc script that creates obj_type 0x20;
  pin the gondola pane through the real path (ride, shoot while moving, breaks at the
  moved position) + one more level if the census finds one.
- [04:32 UTC, Nov 7] script-break-drain: the finding — master's 995-gondola-windows-08
  commit already drains the 0x20 arm (the per-tick sync at drone.rs:20855, after the scripts
  walk) and the Objects::pending_breaks queue it also added was born dead (pushed, never
  read, comments claimed a drain that lives elsewhere); the queue is REMOVED and the sync
  documented as the single path (zero behaviour delta). The disc census pinned
  (nf-formats script_break_census: 99 archives, 34 obj_type-0x20 creates, exactly two
  levels — 07000008's 16 gondola panes and 07000012's 18 sky-bridge panes, kind 1 on
  streams 15/26). Both levels pinned through the real fire path: the gondola pane shot
  while the ride moves breaks at its moved position (19.20 u from first seen, the
  kill-tick flight crosses the moved bounds); 07000012's panes register 17/18 at level
  start (stream 12 shapeless) and a walked-in real trigger kills. The required binaries
  green: nf-game lib 686/0/5, walkreal_07000008_live whole 6/0, gondola 4/0, skybridge
  1/0, urn 4/0, break_debris 2/0, broken_object_bark 4/0, interact_01 10/0, interact_09
  16/0, interactables_12 12/0, interactables_13 10/0, nis_footsteps 1/0,
  nis_handback_census 1/0/1, nis_skip_handback 2/0, gondola_windows_script 2/0, census
  1/0. The full suite running next (the chunked foreground runner).
