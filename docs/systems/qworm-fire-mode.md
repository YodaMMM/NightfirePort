# The Q-Worm's fire-mode press

Symptom: on 0700000a (reached by riding the 09 lift), pressing the fire-mode key with the Q-Worm out
reportedly left the weapon switch locked.

## What the game does for a gadget with no second mode

The mode press is `Player_Weapon` 0x1a24c8's block at :188-231 [M]: the newly-pressed edge of
action 0xc (`Input_Action(slot, 0xc, 4)`; the watch pair 0x54/0x55 ORs the fire edge, action 9,
:189-192 — the 0x54/0x55 toggle precedent), suppressed while (aim bit0 of `BL+0x60` ∧
`+0x68 & 0x40`) :194-197, and only with the view anim idle (`anim+0xf4 == 0`) :198. Then:

- `weapon_data[w]+0x05 != 0` (s8 twin) → `new = w + twin`, gated by `Player_WeaponHasAmmo`
  0x1a16c0 (carried rounds / clip / ammo-type-0 unlimited; NOT the owned byte), written to
  `inv+0x63` (the switch request `obj+0x63`), zoom floats reset :209-228 [M].
- otherwise the mode byte `BL + w*0xC + 0x1BB` cycles, ONLY when the record's mode count
  `+0x2E != 1` :210-215 [M] — count 1 makes the arm a NO-OP.

The Q-Worm's record (id 0x58) [L ee_start_0700000a.bin @ 0x2bf150 + 0x58·0x10C; identical in
ee_start_05.bin]: `+0x05 = 0` (no twin), `+0x2E = 1` (one mode), `+0xB8 = 0`, `+0xBC = 0` (the
mode arm's anim block 0x1a24c8:113-151 can never run), `+0x68 = 0x0000A0A0` (no 0x40), ammo
type `+0x90 = 0` (unlimited — `Player_WeaponHasAmmo` 0x1a16c0:24-26 returns true [M]).

**So the game's press writes nothing**: no request, no mode byte, no anim, no bit — and
`Player_WeaponChange` 0x1a6f80's refusals are not touched: the view-state set 1/3/5-8/9/0xb/
0xd-0x10 (:25-47), the aim bit0 of `BL+0x60` (:62-64 — a Q-Worm press never sets it; the aim
arm's toggle table `(&DAT_002a38cb)[slot·0x158]` reads 0 at boot [L ACTION.ELF @ 0x2a38cb], so
the aim is a hold, cleared when released :45-52). A Q-Worm mode press cannot soft-lock the
switch state in the game.

## The port

- `crates/nf-game/src/gun.rs:304` `fire_mode_switch`: the same twin/count gates; the Q-Worm
  (twin 0, count 1) returns `ModeSwitch { None, None }` — a no-op, as the game.
- `crates/nf-view/src/scene.rs:2205-2240`: the press block (idle gate, the watch fire OR, the
  aim+0x40 suppression); `fire_mode` (the mode byte) and `switch_request` are the only writes.
- The loaded record matches the snapshot [L probe: `+5=0 +0x2e=1 flags68=0x0000a0a0`].

Tests: `crates/nf-view/tests/qworm_fire_mode_live.rs` (3 tests, ISO-gated): the record pins, the
reported sequence (equip → mode press → switch away completes via `obj+0x63` and via the
number-key lane), and the sister (the P2K's real second mode still toggles 0x02↔0x03).

## The symptom — not reproduced

Nine adversarial sequences through `Sim::step_bond` all leave the switch machine usable: mode ×1
/ ×2, throw + mode, mode during the throw script (view state 9), aim + mode, mode + G-cycle,
mode + weapon cycle, fire + mode same tick, and the two request/number-key lanes afterwards. The
mode press is inert and every switch lands. What could NOT be checked
here: the reported session's exact state (no recorded inputs). Suspicion
list, in order, with the addresses to watch:

1. The throw's view-state-9 window: the Q-Worm's fire script 0x060005ae holds while the trigger
   is down and blocks every switch until it ends [M 0x1a6f80:33-47's state-9 refusal; the game's
   own 26-frame throw wait — a *delay*, not a lock, unless the port's script never ends].
2. The aim bit0: a stuck `BL+0x60` bit0 refuses EVERY switch [M 0x1a6f80:62-64]. The port's
   `aiming` is a hold (its toggle table byte is 0 [L]), but a wedged `aim_gate` would lock.
3. A `switch_state` stuck at 1/3 by a script whose length never resolves (the view machine's
   completion reads the resolved script length).

## Live checks (open)

1. PCSX2, 0700000a fresh boot, give the Q-Worm (`give 0x58`), equip it, press the mode button
   (SQUARE in Classic Bond [D docs/systems/weapons.md]), then cycle weapons. Expect: nothing
   moves on the press and the switch completes. Watch `BL+0x60` (bit0 must stay 0),
   `BL+0x7e8→+0xf4` (view state must return to 0 within ~2 s of a throw), `anim+0x63/0x62`.
2. If the lock reproduces in the port app: record the session (the recorder B-5 path) and attach
   it — the wedged field will be one of the three above.
