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
