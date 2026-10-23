# The under-bathhouse guards' voices (07000002)

Build: ACTION.ELF `e7ffdcb3…`. Tags: [M] code (function@addr), [D] disc bytes, [L] EE snapshot, [S] suggested.
Symptom: under the bathhouse (through the tunnels), none of the guards' voice lines could be heard. The speech machinery itself is
docs/systems/speech.md; this note is the 07000002 census and the level's rule.

## Census — the guards around/under the bathhouse's south side

All 25 drones live on the main map (the other 51 maps carry one origin dummy, skin 0x05000000, no channel) [D
07000002.bin `Level::parse`]. The south-yard groups (the tunnel exits land here):

| group | guards (placements) | skins → voice type [M 0x14b300] | wake channel | the plate that raises it [D] |
|---|---|---|---|---|
| the tunnel-pond trio | 458/459/460 at (−11..−18, 6.4, −38..−41) | 0x39/0x36/0x3a → 1 | 0x19 | TouchOnce #349 (−26.9, 6.0, −20.3) |
| the "bunker" trio | 455/456/457 at (−33/−42/−24, 6.4, −20) | 0x39/0x39/0x36 → 1 | 0x81 | TouchOnce #389 (−47.0, 6.0, −23.3) |
| the ch-0x18 five | 450..454 at z −7..−31 | 0x36/0x3a → 1 | 0x18 | plates #347 (−49.7, 2.6, 5.6) / #348 (−32.1, 5.7, 7.9) |
| the ch-0x15 five (bathhouse floor, north) | 442..446 | 0x36/0x3a/0x39 → 1 | 0x15 | plates #341/#383 |

No guard is placed underground (all y 5.4–6.6; the tunnels/pool floor hold only the civilian 0x0500004f,
voice type 9) [D]. The ally females 0x05000034 (voice type 8) stand with the trios (#464/465/466).

## The bunker plate chain — and why the tunnel route is the one that arms the trio

[D placements; M Trigger_TouchOnce 0x1b2b88 (the TmpParam reshuffle: ext out = old p0, mask = old p1, lock =
old p7, hide = old p8) + Trigger_Update 0x1b1a78's pre-gate (a trigger with a lock channel set is skipped
while that channel is ON)]:

- The three wire-gap plates #385/386/387 at the cable gap (−59..−60, 5.6–7.7, −25..−29), outs 0x7F/0x7E/0x80,
  **lock 0x81** — they fire while the bunker channel is still clear.
- AND #388 (out 0x66 = AND(0x7E, 0x7F, 0x80)) drives 0x66 continuously [M 0x1b1a78 case 3].
- Plate #389 (out 0x81, **lock 0x66**): fires only while 0x66 is CLEAR. So the trio arms on the route that
  reaches the yard without completing all three gap plates — the tunnel route — and the bunker entry is
  locked away for the rest of the level once the gap chain has fired. The port reproduces exactly this
  (`crates/nf-game/src/triggers.rs` `update`'s `lock` gate; `bathhouse_voices_live` pins the chain).

## What a vt-1 guard can say here (the game's rule)

| trigger | function [M] | Drone_SfxInfo idx | sfx id [D] | resolution in SB_2 [D] |
|---|---|---|---|---|
| attack shout (Attack/AimStandFire/CombatNoMove enters) | AttackTalk 0x13eac0 | 0x33..0x36 (1 in 2) | 115/112/111/113, repeat 120 | STREAMS (sub-track files −135..−139 → STREAMS.LUT 134..138, 11–34 kB) |
| heard noise (HeardNoiseSuspect/Alert enter, vt 1) | 0x149920 | 0x26 | 114, repeat 180 | stream (file −138 → LUT 137) |
| hit, survives | PainTalk 0x13eeb8 | 0x12 | 925 | bank samples (192/193/194) |
| death | DeathTalk 0x13eff8 | 0x04 | 139 | bank samples (188–191) |
| **first sight / CombatNewSighting** | SeenPlayerTalk 0x13f1d0 | 0x21 (vt ≠ 2 on levels 05..08) | **0 — silent by data** | — |

The group's first sight therefore produces NO voice on this level (entry 0x21's sfxId is 0 [D
Drone_SfxInfo; L speech.md §3]); the shout comes on a later Attack enter (e.g. after sight is regained).
PatrolTalk is vt-3-only; BondTalk/HandleTalking are 07/08/0c/0d-only — nothing else fires for vt 1 [M].

## The port's path (all pieces verified on this level)

`voice_type` [M drone.rs `voice_type`], the talk sites [M drone.rs `attack_talk`/`seen_player_talk`/
`pain_talk`/`death_talk`/`heard-noise` callers], `Drones::speech` + `voice_last` [M drone.rs], the EE pool
[M sound.rs `Sounds`], and the IOP driver with the level's bank (hash 2 → SBINFO slot 11) + STREAMS.LUT/BIN
[M nf-audio `sfx.rs`]. `bathhouse_voices_live` drives `nf_view::load` + `Sim::step_bond` +
`nf_view::sound_tick` and pins: the plate chain arms the trio; the guards go through the AttackTalk site;
the shout ids go live in the EE sound pool; the sister (ch-0x15 five) through the same chain; the 0x21/
0x33..0x36 data pins.

## Open items

- **Live checks (open)** (the headless run cannot see the real mixer):
  1. PCSX2, 07000002: walk the tunnel route to the yard without touching plates 385–387, step on plate
     #389 (−47.0, 6.0, −23.3) — EXPECT ch 0x81 rises and guards 455/456/457 wake; when they spot Bond and
     (re-)enter Attack, EXPECT the stream shouts (sfx 111–115) audible.
  2. Same session: gdbwatch on `Drone_SfxInfo+0x08` (0x2812d0 + 0x33·32) — EXPECT lastFrame stamped at
     the shout, and `Sound_Play3D` reached with sfxId 111..115.
  3. Confirm the map ambient id 92 (the waterfall at (−55, 0, 1)) — in the port's headless driver its
     start emitted no Play (`Driver::start` → no voice, both runs); check whether the real game plays it
     and, if so, why the port's SB_2 sample 207 start fails.
- [S] The port's driver emits the shout voice Plays (an instrumented trace showed `start id 113 → 2 Plays`)
  yet the item disappears from `items_probe` within the same tick; the EE-pool assertion in
  `bathhouse_voices_live` pins the audible half. If a real-output run still hears nothing, the next
  suspect is the driver's 3-D priority/budget reject (`setup()`, tracking-2 + priority-99 voices vs the
  map ambients' 8 live voices) — a live `NF_MPFX_LOG` run would settle it.
