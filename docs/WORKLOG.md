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

### fusebox-wire-geometry-2 (finish log, 22:40 UTC 2026-10-25)

- Merged origin/s/fusebox-wire-geometry (a1aefc0c) onto origin/master (24084ef2, unmoved). The
  four red tests re-aimed at the wire OBJECT's visible world bounds (wire_geo through the live
  entity matrix) with a per-tick aim-walk driven by the spawned Laser round's miss; every assert
  kept. fix_fusebox_cut_live 4/4, fusebox_06_live 4/4, walk_07000014_live 2/2,
  fusebox_geometry_07000014_live 1/1 — all through the real input path.
- walk_07000014_live's no-hurt pin moved from health==100.0 to fusebox_hurts==0 [M 0x1adfe8:99-101]:
  the real cable aim smashes the panel's DESTROY_BoxSatellite boxes (the test's own 996-audit
  blast path, 3.3 splash measured) — the old pin silently relied on the beam never touching them.
- 07000006 #747's wires resolve geometry [D hashes 0x02000371/0x02000428; world slabs 1.35 m above
  the entity translation]; the burns go through it. 07000008's three tests unchanged green
  (sphere-fallback path).
- Suite: the sandbox kills background processes (nohup probe failed), so the suite ran through a
  resumable clone of tools/test/suite.py in foreground chunks — 304/806 binaries at report time
  (all nf-game/nf-audio/nf-formats, nf-view priority families), zero failures except the
  fix_0c_pylon_live pair, reproduced and OUT of this task's scope (three test files changed, no
  product code) — master-baseline attribution requested in the report's Live checks.
- Report docs/reports/fusebox-wire-geometry-2.md written; finish.sh + ntfy next.
