## level-13-guard-barks

- (16:00 UTC, 10-26) Branch created from origin/master (post-explode-debris-spawn master),
  pushed. Task: do 07000013's lobby guards (skins 0x5000004/06/08) speak? The port's crosshair
  `bond_talk` path exists (drone.rs); idle/patrol barks are voice-type-3 gated. Plan: read each
  guard's voice type from the disc placements + the dtype/voice tables [D], find the game's
  bark gates [M] (crosshair talk + idle/patrol), make the port speak exactly when the game does
  — or prove the game is silent too and pin that. Tests: per guard group, voice type [D] +
  bark requested on crosshair/idle [M].

- (16:55 UTC, 10-26) level-13-guard-barks DONE. Voice types settled: skins 0x5000004/06/08 →
  voice set 3 [M NDrone2_DefaultInit 0x14b300:96-103] — the port already mapped them. Crosshair
  BondTalk: 0x7000007-only + no voice-3 case [M 0x1446b0:30/:40] — the game never barks these
  guards on crosshair; pinned. PatrolTalk: needs PATROL via props 0x24/0x25 [D decoded per
  guard: set on 7 of 12]; at load no guard patrols (the prop-set pair spots Bond → combat), so
  the game is silent at spawn too; pinned. REAL FIX: attack_talk's type-3 arm lacked the
  ever-sighted gate [M 0x13eac0:63-65 + 0x175ef0:82-86] — new ever_seen_3c field (the game's
  +0x228 & 8 exactly: sight ack AND prop 0x3c), gate returns before the Rand_Rand(7). Audit
  corrections: p15=3 on all 12 [D], p4 = spawner group id. Tests: nf-game lib 563/0 (1 new),
  guard_barks_07000013_live 4/4 (new), interactables 11/11 + fix_blast_scripts 3/3 regression.
  Report: docs/reports/level-13-guard-barks.md.
