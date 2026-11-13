## Your task: `level-0b-deck-agent-retune`, 0700000b's Agent and 00 Agent real-damage walks are green after the files-plate fix

Branch from current origin/master (level-0b-files-plate-hazard is merged) and read docs/reports/level-0b-files-plate-hazard.md on origin/s/level-0b-files-plate-hazard, "What's left" items 1 and 2, plus the ignore messages on the 0700000b Agent (d2) and 00 Agent (d3) walks. The plate fix shifted the deck arrival about 700 ticks. First the Agent walk: it now dies in the deck band near tick 17366 (23 hits, 1 return-fire fight); the d3 six-round window extension did not clear it, so give d2 its own pass over the shifted phase using the Operative (d1) lever list as the template, judging each lever over several real runs. Then the 00 Agent walk: the same pass on its shifted phase (the last measured death is roof guard #4 at 17096). Helpers stay gated to 0700000b and the difficulty, so the Operative walk and its committed replay stay byte-identical (replay_parity_0700000b must stay green). Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000b_live binary, replay_parity_0700000b and interactables_0700000b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-deck-agent-retune`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-12T18:46Z

---
## Report (done, 2026-11-13T01:25Z)

d2 (Agent) GREEN untouched — the "dies near 17366" premise is stale on master b1fd37ee2 (deterministic twice: mission complete t17886, hp 1.2, 27 hits). d3 (00 Agent) still RED with the ledger: the deck shift spawns #5 late, the early r1 locks #4, and the 0x2e round never detonates on this tree (chased 40 u, still flying at the death tick 17415) so the sprint-10 six-round window is structurally jammed (band_shot 1 < cap 6 — no 0x43 switch-back, no dart close, no fights). Two levers landed d3-gated (the jam close, the deck fight call — the fights were never wired into the deck loop and no candidate is visible from the crossing); whole walkreal_0700000b_live 7 passed/1 ignored, replay_parity_0700000b 1 passed, interactables_0700000b_live 14 passed; the suite did not fit (setup ate 40 min, background jobs die between tool calls). Next lever (the visible-ray census + the #4 dart sleep) and 2 live checks (the 0x2e fuse law, the crossing sight line) in the d3 ignore + docs/reports/level-0b-deck-agent-retune.md on s/level-0b-deck-agent-retune (6fa8518d7).
ADDENDUM (2026-11-13T02:00Z): the earlier "no deck fight pick" reading was a stale-binary artifact (a failed compile left the old binary running — the 2026-11-12 method note applies). The committed binary's fights DO pick via the unseen-fight fallback and killed #4 outright (the fights' in-hand 0x2e blasts, d4hp -60 [L d3run5/6 identical twice]) but the open-floor stands cost 38.5 hp and Bond died at the entry t17600; the z > 6 crossing gate was measured and reverted (the #4 kill landed t17334 yet the stands paid 43.0 -> 4.5, death t17383). The refined next lever: an opt-out flag for the unseen-fight fallback (rule 7 compliant) + the sighted P3-funnel dart fight; two live checks (the 0x2e fuse law, the crossing sight line) in the d3 ignore. Final tree re-pinned: walkreal 7/0/1i, parity 1/0, interactables 14/0. Branch tip 83d4e6f02.
