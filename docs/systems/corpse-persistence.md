# Corpse persistence — what the game does with a body nobody is looking at

Evidence tags: [M] decompile (ACTION.ELF, file + line), [M-dis] instruction-level disassembly,
[D] disc, [L] live PCSX2, [S] suggested. The decompiler drops stores; every store that matters
here was checked in the disassembly.

## The lifecycle

1. Death → `NDrone2_SetAsDead` 0x14a2f0 [M]: `DroneWeap_DropWeapon`, `+0x4F8 |= 0x200`
   (no-gravity) and `&= ~0x8000`, `+0x4F4 = 0`, (mission 0x700004a excepted) `+0x4F8 &= ~0x20 & ~0x80`,
   **`obj+0xF0 |= 0x20`**, and unless the KO flag (`+0x4F8 & 0x1000000`) `+0xAC = 0`. The corpse
   keeps its object: nothing here removes it.
2. The dead states: Dead 0x47 (`NDrone2_DSTATE_Dead` 0x163fe8), HostageDead 0x0f (0x172628),
   AstronautDeath 0xbf (0x1747b8). Each arms timer C to `DAT_00298F58` = `FRAME_RATE_INT * 10`
   = 600 ticks [M 0x163fe8:22 (drone+0xCC4 = now + DAT_00298f58), 0x172628:22, 0x1747b8:22;
   writer `Drone_LevelReset` 0x13a5a8:10].
3. Timer C → Fade 0x48 (`NDrone2_DSTATE_Fade` 0x172cd8 case 0xc → `Drone_SM_SetState(SM, 0x48, 0)`).
4. Fade: `NDrone2_FadeOut` 0x14a218 drains the float `drone+0xCFC` by 0.6 per tick (8.0 while the
   SM state is 0x49 FadeFast) and writes the truncated byte to `obj+0x106` (the render alpha);
   the ENTER seeded `+0xCFC` from `obj+0x106` [M 0x172cd8 case 1]. At 0 → delete (below).
5. The beyond-3 accelerator: `Drone_InitComms` 0x139520:214-244 counts the corpses (SM states
   0x0f, 0x47, 0x48, 0xbf — 0x48's key gets `+ DAT_00298F58`), picks the smallest key into
   `DAT_00298F6C`, and `DroneFunc_DeadDrone` 0x14a298 (every dead state's UPDATE) moves that one
   to FadeFast 0x49 — but only while `DAT_00298F5C (3) < DAT_00298F60` (the candidate count):
   **with ≤ 3 corpses nothing is ever collected early**. The state-entry timestamp `drone+0x11C`
   is the key; its readers read `now − +0x11C` windows (PlayScript 0x15ba68:107, AimCrouchFire
   0x167060:82) [M — no writer survives decompilation; the state-entry reading is the only one
   the readers support].

## The three object fields that decide when a corpse goes away

### `obj+0xFC` — the drawn-grace counter (NOT a boolean)

- `control_init_object` 0x134630:16 sets it to 2 at spawn [M].
- The draw passes set it to 2 for every object they draw: `View_AddObjects` 0x1e62d8:71,
  `View_AddForcedObjects` 0x1e5fb8:174 [M]. The gate before the frustum test is
  `obj+0xF0 & 0x10 == 0` (0x1e5fb8:89) — a disabled object is never drawn and never refreshed.
- The control pass decrements it once per tick when nonzero: `control_movement_object_handler`
  0x1336f8:242-246 (`if (c != 0) obj+0xFC = c − 1`, the loop after `Collide_Update`) [M].
- So `X_{K+1} = drawn_K ? 2 : max(X_K − 1, 0)`, and `X_K == 0` means "not drawn for 3 control
  passes". **A corpse that leaves the frustum survives 2 more ticks and is removed on the 3rd.**

### `obj+0xF0` bits 0x10 / 0x20 — disabled / bullet-impassable

- **0x10 = disabled.** Readers [M]: the draw walk's pre-frustum gate (`View_AddForcedObjects`
  0x1e5fb8:89 — not drawn, no +0xFC refresh); `Trigger_Activate` 0x1b2318:45 (disabled objects
  don't fire triggers); `GT_Track` 0x17f9c8:150 (turrets don't track them); `MP_GetRadarObjects`
  0x189840:178 (radar skips them). `NDrone2_Enable(0, drone)` 0x149dd0:8-15 sets `obj+0xF0 |=
  0x10 | 0x20`, clears `drone+0x4F8 & 0xffe7fe5f`, and zeroes the shadow float at `drone+0x530+0x98`.
- **0x20 = bullets and corpses.** `NDrone2_SetAsDead` sets it on every corpse [M 0x14a2f0]; the
  bullet collision walk skips objects with `obj+0xF0 & 0x60` [M 0x12ce80:124] — corpses never
  block bullets.
- Enable(0) callers: BOT_respawn 0x124e38, DroneSpawner_Init 0x137aa0, Drone_EnableAll 0x1383e8,
  Drone_FellOutMap 0x13a878, WaitSwitch 0x15b960, Death_Anim 0x1637f8, Dead 0x163fe8,
  AbseilDeath 0x169b28, Fade 0x172cd8, FadeFast 0x172dd8, DeleteMe 0x173f00 [M grep].

### `obj+0xFE` bit 1 — the control walk's delete-me flag

- **Consumer** [M-dis 0x133864-0x133888, `control_movement_object_handler`]: `lbu 0xfe($obj)`,
  `andi 1`; set → `Effect_RemoveSObjs(obj)` (0x1cf360) then `control_delete_object(obj)`
  (0x133178) **in the same control pass** — `control_delete_object` calls the class delete func
  (`Drone_Delete` 0x136ce8 for the drone classes: the +0x950/+0xA60 arrays freed, every other
  object's references nulled, `LList_Remove(0x298e14, drone)`), unlinks the object from
  `DynamicObjList` and drops `DynamicObjCount`.
- **Writers** [M grep]: the dead states' case 3 (below), `BOT_respawn` 0x124e38:69 (the old body),
  `Break_Kill` 0x12b970:60, the bullet destruction paths (0x12cd98/0x12ce80/0x12d890/0x12e660),
  `control_handle_cel_change` 0x133288:266, `Control_DeleteAllObjectsOfType` 0x1349e8:16,
  `Drone_Delete` 0x136ce8:91 (the class-0x36 carrier), `DroneSpawner_Init` 0x137aa0:18,
  `NDrone2_PreDroneControl` 0x148db0:53 (the script's channel-0xc kill).

## The dead states' case 3 (UPDATE) — the actual rule

- **Dead 0x47** [M-dis 0x1640d0-0x164104]: `if (obj+0xFC == 0) { NDrone2_Enable(0, drone);
  obj+0xFE |= 1; }` then `DroneFunc_DeadDrone`. Because the delete-me flag is consumed by the
  control walk in the same pass, **a guard corpse is disabled and removed on the 3rd control
  pass after its last drawn frame** — not "never deleted", and not "at the first undrawn tick"
  either (the 2-tick grace). A watched corpse waits for its 600-tick timer instead.
- **HostageDead 0x0f** [M 0x172628:28-30]: case 3 is `DroneFunc_DeadDrone` ONLY — there is NO
  `obj+0xFC` arm, no Enable(0), no delete-me. **A hostage corpse stays whatever the camera
  does** until timer C (600) hands it to Fade 0x48, whose own `obj+0xFC == 0` arm then deletes
  it on its first undrawn tick [M 0x172cd8 case 3], or fades it (0.6/tick) if watched.
- **Fade 0x48 / FadeFast 0x49** [M 0x172cd8 / 0x172dd8 case 3]: drawn → FadeOut; when the alpha
  lands on 0, or when `obj+0xFC == 0` (the same 2-tick grace), → Enable(0) + delete-me → gone.
- **AstronautDeath 0xbf** [M 0x1747b8 — no case 3 at all]: like HostageDead, the corpse stays
  until timer C → Fade.

## The audit premise, corrected

An earlier reading said the game does not delete the object and keeps a disabled corpse that
stays until its timer fades it. The decompile says otherwise, at instruction level on both
ends (the setter 0x1640f0-0x1640f8, the consumer 0x133864-0x133888): **guard corpses DO get
removed when off-screen** — after the 2-tick drawn grace, disabled and deleted in the same
pass. What the 2026-10-21 audit actually caught (the HostageDead fade test passing on tick 0
because the body was already gone) is the **HostageDead** arm: the port had Dead's
`!drawn → removed` branch in a state whose game handler has no such branch at all. Hostage and
astronaut corpses persist off-screen; guard corpses do not (3 ticks); both keep the 600-tick
timer → fade while watched.

## Port model (crates/nf-game/src/drone.rs)

- `drawn_grace: u8` is `obj+0xFC`: 2 at spawn [M 0x134630:16], set to 2 whenever the tick's
  `drawn` (`obj+0xFC`'s draw-pass stand-in, the frustum + ray walk `visible_to`) is true, else
  decremented — updated at the END of the per-drone walk so the state handlers read last tick's
  value, exactly the game's recurrence.
- `(DEAD, msg::UPDATE)`: `drawn_grace == 0` → `present = false; enabled = false` (the folded
  Enable(0) + delete-me; the game's delete lands in the same control pass, so one port tick).
- `(HOSTAGE_DEAD, msg::UPDATE)` / `(ASTRONAUT_DEATH, msg::UPDATE)`: no drawn arm — cull only.
- `(FADE | FADE_FAST, msg::UPDATE)`: `gone = drawn_grace == 0 || alpha drained` (0.6 / 8.0 per
  tick).
- The beyond-3 fold (`cull`): candidates 0x0f/0x47/0xbf keyed at `entered` (the state-entry
  stand-in for `drone+0x11C`), 0x48 at `entered + 600` [M 0x139520:223, DAT_00298F58 =
  600 0x13a5a8:10 — the old +300 was the pre-audit timer value]; the oldest beyond 3 goes
  FadeFast [M 0x14a298].
- `alpha` is `obj+0x106` (the render alpha the fade drains); the renderer multiplies the
  character vertices' alpha by it (fix-ai-remainder). A corpse is never "disabled but drawn":
  the game disables and deletes it in one pass, so `present`/`enabled` drop together.

## Tests (crates/nf-game/tests/ai_missing_states_live.rs)

- `a_guard_corpse_vanishes_after_the_drawn_grace_not_instantly` — the real-path kill: the body
  is still there after 2 undrawn ticks, gone on the 3rd [M 0x163fe8 case 3 + 0x1336f8]; sister:
  turn back inside the grace and it stays (the refresh 0x1e62d8:71).
- `a_hostage_corpse_survives_off_screen_until_its_fade` — the body is present at tick 595
  off-screen (the old port dropped it on tick 0 — the audit's symptom) and gone by ~615 once
  Fade's undrawn arm fires [M 0x172628:28-30 + 0x172cd8 case 3]; sister: the watched 600-tick
  fade still lands (`hostage_dead_is_a_corpse_that_fades_after_ten_seconds`).
