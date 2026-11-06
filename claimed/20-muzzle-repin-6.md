# Muzzle: the last reds

Branch from `origin/s/muzzle-repin-5`, merge current `origin/master`, push to `s/muzzle-repin-6`. Game code stays: every draw stays, and fixes are test-side unless a red turns out to be a real fault.

Read docs/reports/muzzle-repin-5.md on that branch. Three binaries were left red:
1. `walkreal_07000015_live`, 2 failed: the mine that never exploded under fire, Bond dry, a door auto-close blocking line of sight.
2. `walkreal_07000008_live`, 1 failed: the shoot-out must kill every end pane.
3. The d2 death assert left red in that report (7 passed / 1 failed).

`walkreal_07000003_live` was not re-run whole, so run it. Do not loosen any pin tagged [L]. Finish with one total line from a full run.

---
claimed-by: sandbox3 at 2026-11-06T09:42Z
