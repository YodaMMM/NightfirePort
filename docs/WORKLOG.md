# WORKLOG

### level-0a-every-objective-3 (started 17:57 UTC, 2026-10-29)

Task: Tower B office floor (0700000a) — port the vent crawl the game uses to reach the
Q-Worm cubicle's south side [M the crawl/creep path, the vent placements D], walk it on real
input, do the real install (replace the last stand-in), then the mission tail (the ring legs
and the hatch E) the -2 branch re-derived. Un-ignore
`walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` when green. FULL suite.

Base: origin/master (a7eace8e). The -2 report (origin/s/level-0a-every-objective-2) leaves:
walk-reachability of a bullet-clear stand (east stand furniture-blocked, desk-north band
desk-lip-blocked, south stand blocked from the nav route end z ≈ −31.5 [D probes l0a2c]) —
docs/levels/0700000a.md says the vent shortcut is the way in; the port has no vent crawl yet.

Plan: research the game's crawl/creep path + vent placements in the decomp and on disc;
start the fast suite runner early; port the crawl; extend the walkreal test through the
vent; the real install; the tail; un-ignore; full suite on the final tree.
