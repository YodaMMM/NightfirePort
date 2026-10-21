# Unreached guard-AI functions (call-tree audit outcomes)

Functions the call-tree flags that the shipped game never reaches on an on-foot level, the front
end or MP — with the evidence. Ported/cited functions are not listed here; each has its citation at
the Rust code that does the work.

## NDrone2_DSTATE_CastleChatGuard1 0x1612e0 (state 0x36)

- Handler of `NDrone2_StateFuncs[0x36]` [L 0x29b300 table read]. In-code entries: the DTYPE-0x18
  home state (`NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8 case 0x18 → +0x5a2 = 0x36 [M]) and the
  PlayScript redirect (`NDrone2_DSTATE_PlayScript` 0x15ba68 case 1, `*(char*)(drone+0xc5) == 0x18` →
  SetState(0x36) [M]).
- The disc census over all 99 `.bin` level archives (555 kind-0x0f drone placements) shows **no
  placement carries DTYPE (placement parameter 15) 0x18** [D census; docs/reports/
  unported-guard-ai-other-3b.md]. The port's `dtype_of` (`NDrone2_GetDTYPENEW` 0x14c938) never
  returns 0x18 either, so the PlayScript redirect's `+0xc5 == 0x18` precondition has no disc producer
  [M]. The mode-keyed start-state arm the port carries (`start_state`: mode 0x18 → 0x36) mirrors the
  game table for completeness; no disc placement produces mode 0x18 [D census of parameter 16: only
  0x00/0x02/0x03/0x04/0x05 occur].
- Remaining theoretical entry: a level script's Global msg-0x1d SetState (0x15b830 case 0x1d [M])
  naming state 0x36 — script-data-driven, not exercised by any placement or start script found in
  the census. Recorded unreached on the shipped placement data; a script-driven entry would be a new
  finding.

## NDrone2_DSTATE_InterogateWalk 0x162338 (state 0x3d)

- Handler of `NDrone2_StateFuncs[0x3d]` [L 0x29b300]. **No in-code entry exists**: an exhaustive scan
  of the decompile (SetState constant 2nd args, direct state-field stores, `return 0x3d/61` in state
  mappers) finds none — the interrogation family's own transitions go to 0x3c/0x3a/0x56/0x60 [M
  0x161a40/0x1616c0/0x161860; the scan is in docs/reports/unported-guard-ai-other-3b.md].
- Entry is data-driven only: `DroneTypeSettings` rows whose start/alert state is 0x3d, or the
  obj+0xf4 coder initial state. The interrogator DTYPEs 0x1b/0x1c come from the start-script
  specials 0x600089c/0x6000892 [M 0x14b300:613-627]; **no placement carries DTYPE 0x1b/0x1c as its
  parameter 15** [D census] and their DroneTypeSettings rows do not start in 0x3d
  (docs/fields/dronetype-tables.md [D ELF data]).
- The same Global msg-0x1d script caveat as CastleChatGuard1 applies.

## Data-driven states deliberately not proven unreached

- `NDrone2_DSTATE_EnemyMission` 0x15e1e8 (0x1b): entered from Idle/Alert ENTER when the mission-path
  route has pending nodes (`+0x9f2 != 0` after `AssignAIPath`) [M 0015bf18:36-60]. Prop 0x1d is set
  on 484/555 drone placements [D census], so the state is data-reachable; the port's route-walking
  machinery (`update_mission_route`, 0x154a80) exists and the state's port is future work.
- `NDrone2_DSTATE_CivilianMissionWait` 0x15dbb0 (0x17): entered from CivilianMission's msg-0x22 hold
  command [M 0x15d860:130-136]; whether any shipped script sends 0x22 to a CivilianMission drone is
  not censused — left open rather than claimed unreached.
