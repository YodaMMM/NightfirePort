## level-13-guard-barks

- (16:00 UTC, 10-26) Branch created from origin/master (post-explode-debris-spawn master),
  pushed. Task: do 07000013's lobby guards (skins 0x5000004/06/08) speak? The port's crosshair
  `bond_talk` path exists (drone.rs); idle/patrol barks are voice-type-3 gated. Plan: read each
  guard's voice type from the disc placements + the dtype/voice tables [D], find the game's
  bark gates [M] (crosshair talk + idle/patrol), make the port speak exactly when the game does
  — or prove the game is silent too and pin that. Tests: per guard group, voice type [D] +
  bark requested on crosshair/idle [M].
