## level-0a-ignored-checks (2026-10-31, s/level-0a-ignored-checks)

Branch from origin/master (81da6c47, batch-1031l). Task: `walkreal_0700000a_live` has two
#[ignore]d tests (a doorway's walking height and a corner crossing). Read their ignore notes and
the merged capsule work's notes. Find what the game does at each spot [D collision; M the capsule
contact rules — head-on cull on the raw denom 0x10a1f0-0x10a2e8, Dist2Tri flags 0/99 0x10a330 —
and Player_FeetOnPoint 0x194188], fix the port gap with evidence or re-derive the expectation
where it was port-derived, and un-ignore what can be un-ignored; name the live check for anything
left. Run walkreal_0700000a_live and interactables_0700000a_live. Run the full suite with
tools/test/suite.py and quote every total.

## Progress (2026-10-31, ~21:08 UTC)

- Branch created from origin/master (81da6c47). Sandbox persisted from an earlier session: ISO,
  work/, queue worktree, cargo target dir all present. Suite start next.
