# gas-rng-draws-3 — finish landing the gas RNG draws (the open reds)

Branch from origin/s/gas-rng-draws-2 (merge origin/master into it first). That branch re-lands the game's gas-init draws on the SHARED RNG stream (proven [M 0x1aacc8:72-80; Rand_FRand 0x1e3780]) and fixes goggles_render_live, mp_perf_16_live, hold_npcs_live, mp_bot_armour_online_live and doublecross_transitions_live. Read its report (docs/reports/gas-rng-draws-2.md) first.

Still red on that branch (each diagnosed in the report):
sight_frustum_live (2), walkreal_07000001_live (3), walkreal_07000002_live (1), walkreal_07000003_live (2), walkreal_07000014_live (4), walkreal_07000016_live (2), walkreal_0700001b_live (1), fix_blast_scripts_live (1), fix_finale_live (1), hostage_killer_07000002_live (1), enemy_accuracy_live (1).

Do:
1. Walks: a walk that no longer finishes its level after the stream shift must be re-routed the way a player would adapt (approach, kill order, wait for a door), never by forcing state. The 14 corridor door (30.3, 35.4) and the 03 east-lane wedge need the door/route chains checked against [M].
2. sight_frustum: re-derive the census onto a guard that stays present (261 is deleted on arrival [M 0x165640:47-82]).
3. hostage_killer_07000002: re-check the opener law [M 0x144340:34-40] before touching the pin.
4. enemy_accuracy / fix_blast_scripts / fix_finale: re-pin only values the stream legitimately moves, citing [M]; a level that can no longer be finished is a regression to fix.
5. Every file above green, plus every walkreal_*_live; then the full suite total line.

---
claimed-by: sandbox3 at 2026-11-04T11:14Z
