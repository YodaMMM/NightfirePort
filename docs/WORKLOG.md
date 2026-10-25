### level-4a-rappel-vanish (started 20:17 UTC, 2026-10-25)

Task: 0700004a lift ride — "rappel guards disappear after shot": a guard on a
rope vanishes instead of dying/falling when hit. (1) Find the 4a abseil guards
[D], drive the real ride with real input, shoot one, log state/anim/visibility
per tick. (2) Compare with the game [M — NDrone2_DSTATE_AbseilSlide 0x169730 /
AbseilHang 0x169940 damage arms, the death on a rope (the fall, the corpse),
abseil_death_live on 11]. Fix the port so a shot rope guard does what the
game's does; real-input test; headless renders named in the report. Branch from
master (24084ef2). Sandbox deadline ~22:14 UTC (stop new work 21:54).

