
## difficulty3-07000001-pins (started 2026-11-03 16:30 UTC)

Branch from origin/master (cbed0983). Task: pin everything difficulty 3 changes on 07000001 against
difficulties 1 and 2, each with its address — the extra Drone_Create 0x136be0:49 placements [D the
param per placement], the pain scale, the accuracy row, locks/pickups/ammo deltas [M] — in a new test
file difficulty3_07000001_live.rs (spawned set + stats at load, per difficulty). Fix any reader the
port is missing. walkreal_07000001_live untouched (level-01-agent-hall-fight-2 owns it). The escort-fight
and hall-fight reports read first. Runs: the new test, interactables_07000001_live, nf-game; the fast
runner in the background from the start.

Progress (17:00 UTC): difficulty3_07000001_live.rs written and GREEN 4/4 — the load-time pins per
difficulty: the spawned set is identical at 1/2/3 (all 20 created-map kind-0x0F placements carry
param 2 = 1 [D]; the param-2=2 record on map 12 is dead data — the dynamic-create pass parses only
the archive's last map [M 0x1d1020 case 0x1a / 0x1d1cf0 / 0x1d6360:146]); the ONLY roster change is
the captain criterion [M 0x14b300:504-535]: #167 captains on Operative (prop 9), #163 joins on Agent
(prop 5), #165 joins on 00 Agent (prop 8) — each reskin 0x36/0x39 → 0x05000038, health 12 → 24 (×
DroneCaptain_Mod_Health 2.0 [L ee_start_07000001.bin 0x30cb2c]), accuracy stat 8 → 4 (× 1/2.0); the
pain rows 0.5/0.7/1.2, accuracy rows 0.5/0.7/1.0, DroneDamage rows 1.5/1.0/0.8 [L the snapshot
0x30cc50-58 / 0x30caf0-f8 / 0x30cab0-b8] with the three reader selections pinned through
damage_factor / hit_chance (the d3 TooClose product 300) / hit_damage (10 torso → 15/10/8); no
lock/pickup/ammo/hint/soundtrigger delta (no 0x2E [D census], Pickup_Create reads no difficulty [M
grep], hint #330 stage 3 and trigger #319 param 6 = 0 create everywhere); Bond loads identically
(100 hp, hands held + pistol switch request 2, the Autoaim_*Mul rows [L 0x30cd5c-64]). Runs:
difficulty3_07000001_live 4/4, interactables_07000001_live 10/10, nf-game --lib 656/0/5. The
background fast runner is killed between tool calls in this sandbox (canary verified) — the
per-binary foreground runs stand in, as documented in the escort-fight report. Remaining: the
report (the 00 Agent route changes), the final suite attempt, the finish steps.
