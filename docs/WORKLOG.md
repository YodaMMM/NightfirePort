# WORKLOG

One section per task, appended at the end. Never rewrite or drop another task's entry.

### fusebox-wire-geometry-2 (started 21:00 UTC, 2026-10-25)

Branch from origin/master (24084ef2), merging origin/s/fusebox-wire-geometry (a1aefc0c) next.
Task: land the fusebox wire contact on the real cable geometry — aim the four red-on-master
tests (fix_fusebox_cut_live sabotage, walk_07000014_live facility chain, fusebox_06_live alarm
bundle + searchlight bundle) at the wire object's visible world bounds (the branch's `wire_geo`)
the way a player would, keep every assert; check the 07000006 fuseboxes resolve their wire
geometry too ([D] scripts/hashes); all four green plus fusebox_geometry_07000014_live; full
suite on the merged tree.

Plan: push branch start; start the fast suite in the background early; read the base branch's
wire_geo change (a1aefc0c) and the four red tests; re-aim each input at the real geometry;
verify 07000006 boxes; suite; report.
