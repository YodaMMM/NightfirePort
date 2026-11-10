
## los-mask-repins-02 — branch cut

Branch cut from origin/master c120d7b49 (contains 823ff190b, the game's guard sight: per-object
masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push,
FellOutMap). Task: the 07000002 walks were ignored because the game-correct sight moved their
fights (walk_the_castle_all_objectives_real_health_00agent and friends). Read each ignore message,
re-route each walk with real input so it passes, and remove its ignore. Never lower damage or
health, no top-ups; helpers gated to 07000002 and the difficulty. Read
docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs for the sight rules. Run
walkreal_07000002_live (and its sister binaries) and interactables_07000002_live.
Status: branch cut + WORKLOG pushed; setup (work/, ISO) starting.

## los-mask-repins-02 — diagnosis of the ignored 00agent walk (checkpoint)

The single ignore is `walkreal_07000002_live::walk_the_castle_all_objectives_real_health_00agent`
(d3): "the mission ends (level_exit) on the stair-bottom leg at tick 12902 and the doorway
sensor's ch 0x81 never rises; cause not traced". Traced on master c120d7b49 (16 s repro):

- The mission FAILS: `level_exit Some(0x07000002) restart=true` = fail_choose_restart
  [M death.rs 0x216618], raised by the scared civilian's death channel **ch 0x60 ON** (FAIL_
  CIVILIANS [M 0x1827d8:98-107]). The Load Point #206 never fired (gone=false, gate 0x00), no
  script LoadLevel event (NF_EXIT_DBG silent).
- The civilian (placement #464 = drone #22, team 3) is a corpse at (-40.6, 6.3, 1.9), her hide
  spot, killed by a GUARD round: `hit_by_player false`, hit (weapon 0x12, part 5, damage 1.5).
- Round trace [L the diagnostic]: t12330/t12586/t12598 `#10 (placement 452) -> #22`; on the
  duel-removed tree the killer is `#11 (placement 453) -> #22` at t12239. Both are the west-end
  rifle guards (0x12): #10 (-54.6, 5.35, -0.95) lower west, #11 roams to (-69.3, 6.27, -3.46)
  upper west. Their lines to a Bond on the UPPER EAST approach (y ~6.37, z < 4) pass 0.05-0.5 u
  from her hide spot — inside the ±1.5-u miss wobble [M DroneWeap_DoBulletAccuracy 0x179740:
  136-148]; from the stair mouth down or the west hall the lines pass 3+ u south of her.
- So the exposure window is the final upper-east stretch itself; ANY sight turn landing there
  can kill her. The d3-only stair-top sniper stand (1200 ticks, the sniper never exposed on this
  tree — clip 32 unchanged [L]) parked Bond exactly there; removing it moved the kill from #10
  t12330 to #11 t12239 — the window is the problem, not the stand.
- No pre-clear lever from the upper hall: probe [L] at "back from the west-wing armour"
  (-46.37, 5.87, 7.15) — #9/#10/#11/#12 all `exposed None`.
- Next levers (in order): (a) compare the d2 run (passes) — does #10/#11 fire across the upper
  stretch there, i.e. is the d3 kill wobble luck or alert-timing?; (b) find when #10/#11 become
  alerted (the alert state at each leg) and whether the walk can beat it; (c) crouch the final
  approach (Act::CrouchOn exists on the walker) — likely insufficient vs the flat line from
  #11; (d) kill them from the lane after the descent is too late (she dies during the descent).

Test file carries temporary diagnostics (stair-bottom d3 branch prints level_exit/channels/
LOAD_LEVEL state/team-3 corpses/live guards/round hits; a visibility probe after the west-wing
armour detour) — remove once the route passes.
