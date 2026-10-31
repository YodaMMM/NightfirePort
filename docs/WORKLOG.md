## fe-pages-rebase (2026-10-31, s/fe-pages-rebase)

Branch from origin/master (81da6c47, batch-1031l). Task: land the credits/menu pages on top of
the stats page — merge origin/s/fix-stats-page, then origin/s/fe-pages-2, resolving the
conflicts in crates/nf-formats/src/hud.rs, crates/nf-view/src/lib.rs and
crates/nf-view/src/menu.rs so BOTH features survive (one FONTMedium load; both pages' draw
code); then merge origin/master again. Every page must still render as the tests pin
(fe_pages_draw.rs, fe_pages_draw2.rs, stats_page_elements.rs). Run every menu/front-end/stats
test binary and the full suite (tools/test/suite.py); quote every total.

## Progress (2026-10-31, ~19:40 UTC)

- Branch created from origin/master (81da6c47). Origin state checked: both source branches
  diverge from e4550f2e (already in master's history); s/fix-stats-page adds 2 commits
  (580f8697 stats/results pages, 4e123355 corner stamp clip), s/fe-pages-2 adds 8 commits
  (credits roll / 007 Bonus / win page, FONTMedium face 3, Screen Adjust, P_PARISENUM card,
  cheat-medal page, menudef SET ids). Merges start next.
