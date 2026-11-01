# level-16-chaser-real-damage — the chaser fight on real health

Branch `s/level-16-chaser-real-damage` (from origin/master 43752669). No game code changed —
every change is in the walkreal test file and one new pin file; the sim is origin/master's.

## What's done

1. **The chaser's melee chain, pinned end to end.**
   - Her skin 0x0500001e gives kind 0xb, **anim set 7**, fists [M 0x14b300's switch]; her
     placement weapon (param 9 = 0x12 [D]) does not replace the set (the 0x12/0x14/0x15 arm
     guards on `kind != 0xb`) — so `DroneWeap_CloseCombatImpact` 0x1789c0:26-40 always takes
     the kick arm: record 99, the 2.0 band, no facing gate.
   - Record 99's damage f32 = **50.0** [D +0x0C]; the non-kick records 100/0x65/0x66 = 15.0
     [D] (`chaser_melee_records.rs` pins the bytes through the real loader).
   - Player side: `Player_HandlePain` 0x1902c8 × `damage_factor(0x07000016, difficulty)` —
     the 14–16 Plr_DMod row 0.4/0.6/1.0 [M 0x1d4424] — so **the punch is 50 raw, 20.0 health
     a hit at Operative, 30.0 at Agent** (the task's "50 raw / 20 a hit [M]").
   - Her star (weapon 0x12, record 1.5 [D]) measured through the real path at Operative:
     **0.6 ranged, 1.2 point-blank** (= ×2.0 DroneFiring_TooClose [M 0x139fac-0x139fc8]).
     `the_chasers_punch_lands_the_record_damage` pins every landed hit to that chain.
2. **The real-damage variant** (`walk_the_chasers_defeat_completes_the_mission_on_real_damage`,
   `with_real_damage` — top-up off, death panics): the route survived on the walker's return
   fire alone (8 fights, 22 health taken over the whole approach), both blast-room shelters
   held (sit_out_pulse zero-contact), **the chaser fell to the range-keeping kill at +3364**
   (hold her outside 3.0 — her melee fires only inside 2.0 [M 0x1789c0]; under 3.0 u every
   drone round hits [M 0x179740:113-115] — hers included, so the band is held, not traded),
   pulse 2 held. Bond then died in the **pit-squad phase at ~48 health** (the 4-squad + pit
   shooter #8 alive at the room's door). The test is `#[ignore]`d with that reason — the
   squad-phase cover tactics are the remaining leg.
3. **The pit floor's placed soldiers** (#6/#7/#8, skins 0x5000093/0x5000087/0x5000090, weapon
   0x14) shoot the chaser-fight stand — the allowance walks top through them (the walkreal
   header's "damage is suppressed"); `clear_the_pit_shooters` is the explicit return-fire
   answer (the 07000002 room idiom: nearest visible shooter, stand_and_fire, dry-tap give-up).

## How it was verified

- `walkreal_07000016_live`: **13 passed / 0 failed / 1 ignored** (the ignored = the variant,
  reason above), 57 s, NF_ISO on the slim on-foot image.
- `interactables_07000016_live`: **8 passed / 0 failed**, 10 s.
- `nf-game`: **not run in this window** — the sandbox rebuilt from scratch (repo, ISO, toolchain,
  ALSA) and the 1013-binary build filled the disk twice (worked around by stripping the built
  executables + staged resumes; nf-game/nf-formats/nf-audio/nf-view are ~85% built). The next
  continuation must run `nf-game` (and the full fast suite) before pushing further work.
- Fast suite (`tools/test/suite.py`): not run — the nohup background start is killed by this
  sandbox between tool calls (the known issue), and the foreground window went to the build +
  the fight iterations.

## What's left

- Finish the variant's last leg: the pit-squad phase at ~48 health (kill pit shooter #8 first —
  he stands at the shelter door — then take the squad from the room's doorway, or hold the room
  through their approach; the blast room is outside HurtPoly-ShuttleLaunch [D] and the squad
  cannot follow through the shut leaves? unverified).
- Run `nf-game` and the full fast suite on the final tree.
- Her point-blank behaviour (see below) may be a real port gap worth a cited fix after a live
  measurement.

## Live checks for the main agent

1. The chaser's melee: run the 07000016 chaser fight in PCSX2 (fresh boot, any difficulty) and
   log her state sequence when she is inside 2.5 u of Bond — does she ever enter 0xac (the
   sword/kick anim) and land a punch? Expected from the decompile: 0xaa's UPDATE routes ≤2 →
   0xa7 [M 0x16c650] and the close band rolls 0xac at ~50% per cycle [M 0x16bb50], so a punch
   (record 99, 50 raw) should land within a few seconds of her reaching melee range.
2. Her tracked distance +0x1A0 while point-blank (gdbwatch write-watch): does it track the real
   dxz (~1.2) or lag? The port's runs showed her cycling 0xaa/0xb2 star bursts at d 1.1–1.5
   every ~6 ticks for thousands of ticks without routing — if the game's +0x1A0 stays ≤ 2 there,
   the port's UPDATE arm or the dist upkeep differs and is fixable with this cite.

## Usage

not visible to me
