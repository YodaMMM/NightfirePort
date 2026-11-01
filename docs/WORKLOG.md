
## level-1b-platforms-and-drake-scene — 2026-11-01

Task: on 0700001b (1) the breakable platforms break at the slightest touch from Bond, and (2)
Drake is missing from the cut-scene. For (1) find the platforms' break rule [M the class's
damage/contact threshold, what counts as a hit; D placements] and fix the port so only what the
game counts breaks them. For (2) find Drake's cut-scene entity [D the scene's streams and skin;
M how the scene binds the actor] and why the port does not draw him. Pin both (sister checks).
Run every *0700001b* binary and nf-game; quote the totals.

Session 2 (same sandbox, after the corridor task): findings.
- The "breakable platforms": 0700001b carries NO 0x20 breakables [D dump]; the candidates are the
  6 destroyables (0xE1, health 0/10000 [D]) and the 16 SP solar panels (0x06000832, hp 20 [D]).
  The game's rules are bullet-only: Destroy_CollisionHandler 0x12bd48 subtracts only class-5
  contacts; SP_Update 0x1dbda8:283-287 unholds at `+0x14 <= +0x18` — one 20-damage round (the
  start weapon 0x33's damage = 20 [D weapon_data]) breaks a panel BY DESIGN. The port matches
  both rules; an idle/drifting Bond breaks nothing in 900 ticks in the port (pinned). The user's
  "slightest touch" repro needs the exact platform + action — live checks in the report.
- Drake and the cut-scene: the arrival pack scene 0x0600085c (0780001b.bin, 2000 ticks) binds TWO
  anim actors — stream 1 skin 0x0500008f (the station centrepiece = Drake's cut-scene model) and
  stream 2 skin 0x0500007c [D]; both skins on skeleton 0 [D 0700001b.bin]. The port's DATA path
  is complete end-to-end (pinned): the Scene cast rows exist, the Obj::Anim streams tick to 2001,
  both poses resolve (73 bones; Drake's root (2.13, −0.03, 0.44) at the hub). The missing actor
  is therefore a DRAW-layer question — render spots listed in the report.
- Pins: crates/nf-view/tests/platforms_drake_scene_1b.rs (2 tests green). Totals quoted in the
  report: the seven *0700001b* binaries + nf-game 987/0.
