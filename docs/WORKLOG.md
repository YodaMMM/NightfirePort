
### level-1b-audit — branch start (2026-10-26 ~09:50 UTC)
Branch start: s/level-1b-audit = origin/master (7e35bd77). Task: the 0700001b (Equinox)
behaviour audit — every placement's creator/update/use chain against parsemap
0x1d04f0/static [M], the port's six-dimension coverage, gaps to docs/levels/0700001b-audit.md;
small gaps fixed with tests, large gaps as report sections. Sandbox note: the slim ISO was
deleted last session for disk; NF_ISO = Nightfire-movies-test.iso (byte-superset, only the 20
injected movie records differ — the pss-content tests fail on it, everything else ran green).

### level-1b-audit — results (2026-10-26 ~11:05 UTC)
The 0700001b audit table (docs/levels/0700001b-audit.md): 304 placements / 19 kinds, 77+2
scripts, the mission records — every kind's creator/update/use chain re-verified against the
0x1d04f0 jump table + control_funcs @0x26f890, the port's coverage with file:line, GAP per row.
Corrections: 0x2a = View_AddSkyObj sky objects (NOT starts, NOT the missiles — the missiles are
0x39); the 129 0x31s are the game's own skips; 74 SPs (not 76). 16/19 kinds gap-free; gaps:
SpaceLaser_Update unported (large), the missile HUD pane, the Apocalypse whiteout consumer, the
rotating sky. FIXED small: the 0x3b gated dynamic object now rides a posed slot that collapses
when the sim deletes it [M 0x18d3b8:26 + the delete pass] — 0700001b's control-room glass
vanishes on ch 10; test dynobj_glass_live 1/1; the four 0x3b-carrying levels' scene-equality
pins re-pinned (the pins' [S] convention). Suite: slots wave 28/28 (84/0), nf-game wave 89/89
(344/0/3) via the wave runner (background suite died in 8 s again; disk cannot hold the full
build). Report: docs/reports/level-1b-audit.md.
