
## retune-nb-13 (2026-11-18, s/retune-nb-13)

Task: commit 3412a89a3 made every guard run the game spacing scan and the close-range alt attack from the fire states; the level 13 walk `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` now dies. Read the ignore and the interactables_07000013 changes in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove the ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000013_live binary and interactables_07000013_live.

### retune-nb-13 progress

retune-nb-13 progress (2026-11-18): repro'd the recorded death byte-identical (t3774, the
mezzanine #5 at d 25.7 outside the re-anchor sweep's 24, entering 13 at 20 hp because the
12-side melt cost 59.5 around a failed first fight). Landed levers (all d3-gated): the tower2
flank fires one leg earlier (FLANK_LOBBY_EAST-gated) — 12 now exits at 54 hp + armour 50; the
file-local mezz_far_stand pre-emptive (exposed dy>2 pick) at the re-anchor and west-mid ends —
#4 killed zero-cost at the west-mid. Measured dead and recorded: the pocket reposition+resupply
(t993, worse), finish_fights (t13833, earlier), the aim-walk flag (null). Still red: the pool
runs dry mid-lobby and the group-1 respawner stream (max 4 alive, 32000 waves) kills at the
north-west leg (t4849) — the full ledger + the next levers (the 12-side spare / the safe-fetch
lane) are in the test's ignore message. Sisters byte-identical (verified: d2/d1 walks green).
Binaries at the final state: walkreal_07000013_live 3 passed/1 ignored, interactables_07000013_live
10 passed, walkreal_07000012_live 4 passed, walkreal_07000011_live 4 passed, campaign_flow_live
(running at push time — result in the report). The fast suite could not build (ENOSPC, the
1112-binary nf-view set needs ~2 GB past this sandbox's disk); the report documents it.
