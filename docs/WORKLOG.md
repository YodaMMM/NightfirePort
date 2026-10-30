
## level-0b-every-objective-3 (started 02:58 UTC 2026-10-30)

Task: walkreal_0700000b_live still carries position stand-ins on the 0700000b exec floor —
the two bond.pos transit writes (Q-Worm C desk, terminal room), the terminal-room fallback
into plate #163, and the keycard lock driven with set_channel(9) when the walked E fails.
Replace each with real step_bond walks + real E presses. First a probe test that walks every
exec-floor nav route and dumps where each dies, then the walks re-attached with per-leg caps.
Where a walk cannot pass: [M collision / Switch_Activate 0x1b02a8 reach / D furniture] — fix
the port, or pin the stand-in with evidence. Full suite, quote the total line.

Plan: branch from origin/master (21fbeed3); next: push, read the test + the -2 report, probe
the nav routes, then the walks.

Update (04:40 UTC): the nav probe walked every exec-floor route (one ring path, 89 legs OK) and
mapped the waiting-room trap; the test's two transit stand-ins, the plate-#163 fallback and the
ch-9 drive are replaced with real walks + the real walked E (ch 9 green [M 0x1b02a8]); ONE
stand-in remains (the waiting-room exit transit) with the wedge/fall map as evidence.
walkreal_0700000b_live 2/0 (22.76 s). Report: docs/reports/level-0b-every-objective-3.md.
