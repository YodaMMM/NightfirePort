# The driving levels' input path (DRIVING.ELF) — G-004, U-003

Decoded from `work/decomp/DRIVING.ELF/` and `work/elf/DRIVING.ELF` (MIPS32 o32; LOAD segment
vaddr = file offset + 0x107080). Every code fact below is [M] (function + address, checked in the
decompile and, where noted, in the disassembly); disc facts are [D]; EE-snapshot facts [L];
inferences [S]. The on-foot counterpart is `docs/systems/input.md`; the bottom line of the
comparison is in §10.

The DRIVING.SYM name hints (`PS2PadDevice`, `InputDevice::DeadZoneChop/Scale`) point at addresses
from a different build (0x180530/0x180560/0x180640 — no decompile files exist there in our ELF,
confirmed again [M-negative]). The functions below were found by shape search seeded with those
names; `PS2PadDevice::PollDevice` lands at 0x169288, which matches the Raikaru shipped-ELF address
`sub_169288` quoted in docs/research/driving-engine-ea-formats.md, so our DRIVING.ELF has the same
layout as the shipped ELF the external project analysed.

## 1. Pipeline overview

Per frame the IOModule update `FUN_00168060` [M 0x168060, size 92]:

1. `FUN_0025a4a8()` — the pad service tick [M];
2. `container[0x28] = FUN_00261038()` — the frame/time counter the repeat timers compare against [M];
3. `FUN_00167e80(container)` — pad connect/disconnect handling (mode window 3..5 = connected,
   `(counter & 3) == 1` first-probe gate, on disconnect every table record's repeat-state slot
   gets 3) [M 0x167e80];
4. if `container[0x27] == 0` (input not disabled): `FUN_00168280(container)` — poll every device,
   then evaluate every binding and send action messages [M 0x168280, 0x168678].

Container (the InputToAction owner, allocated per IOModule) fields used here: `+0x10` device
pointer array, `+0x27` disable flag, `+0x28` time, `+0x40` parallel binding-set array,
`+0xa0` time copy, `+0xa8` first repeat delay, `+0xac` second repeat delay [M 0x168678].

## 2. The raw pad state — `FUN_00259b78` 0x259b78, `FUN_00259b90` 0x259b90

`FUN_00259b78(slot) = &DAT_00419840 + slot*0x20` [M 0x259b78] — a 0x20-byte per-slot struct at
0x419840, up to 4 slots (multitap). Layout as read by the consumers:

| offset | type | meaning |
|--------|------|---------|
| +0x00 | u32 | pad mode [M 0x259b90 reads it; 4 = digital (no pressure bytes), 3..5 = connected window in 0x167e80] |
| +0x0c | u16 | buttons, active-high, same bit layout as the on-foot u16 (0x0001 L2, 0x0002 R2, 0x0004 L1, 0x0008 R1, 0x0010 △, 0x0020 ○, 0x0040 ✕, 0x0080 □, 0x0100 Select, 0x0200 L3, 0x0400 R3, 0x0800 Start, 0x1000 Up, 0x2000 Right, 0x4000 Down, 0x8000 Left) [M 0x169288 bit loop + name order of the DButton table] |
| +0x10..0x13 | u8×4 | sticks RX, RY, LX, LY — 0 = left/up, 255 = right/down, same order as the on-foot tSlot bytes [M 0x169288 reads +0x10..+0x13; order matches input.md §2's measured slot+0x128..0x12b] |
| +0x14..0x1f | u8×12 | analog button pressures, libpad order: Right, Left, Up, Down, △, ○, ✕, □, L1, R1, L2, R2 [M 0x169288 sequential loop + AButton name order] |

The struct is filled by the pad service (`FUN_0025a4a8` tick; the PADMAN/IOP path — the low-level
`scePadRead` fill is not pinned to a function yet, see U-003 residue in §11) [S]. A separate
USB keyboard/mouse device path exists in the same manager: `FUN_00259668` (case 2 = "PS2_USB_mouse"
string, case table continues) reads the same 0x419840 struct for its stick-shaped synthesised
state [M 0x259668]. The driving engine therefore uses the same libpad byte conventions the
on-foot path measured (input.md §2), not a new layout.

## 3. `PS2PadDevice::PollDevice` — `FUN_00169288` 0x169288

Virtual slot 3 of the PS2PadDevice vtable at 0x374880 (8-byte-stride entries: adjustment word then
function word; vptr stored at object+0x18 — the pattern is visible in `FUN_00167bc8`'s
`vt+0x20 = this-adjustment / vt+0x24 = function` call [M 0x167bc8]). Slots: 0 = 0x169640 (lazy
type-id singleton), 1 = 0x1695e8 (dtor), 2 = 0x168b08 (registers the 32 named values, §4),
3 = **0x169288 PollDevice**, 4 = 0x169508 (`return 32` = value count), 5 = 0x167bc8 (HasChanged:
compares the 32 floats of the object's +4 and +8 buffers) [M].

The device object: `+0x00` InputTable, `+0x04`/`+0x08`/`+0x0c`/`+0x10` four float* message
buffers (A/B/C/D), `+0x14` pad-slot index, `+0x18` vptr [M 0x168b08 reads all four, 0x169288
writes through `*(this+8)`].

PollDevice, step by step [M 0x169288–0x169508, disassembly checked at 0x169288–0x16939c]:

1. `slot = FUN_00259b78(*(this+0x14))`; `FUN_002601f0(*(this+4), *(this+8), 0x80)` — clears the
   128-byte (32-float) message into buffer B (0x80 = 32×4).
2. **All-sticks-zero guard**: if `slot[0x12]==0 && slot[0x13]==0 && slot[0x10]==0 && slot[0x11]==0`
   the four axis floats (values 0..3) stay 0.0 and the rest of the message is still built
   (buttons/pressures). This is the disconnected-pad shape (raw sticks read 0) [M].
3. **Axes** (skipping the guard): for each stick byte,
   `FUN_00169218(b) = b*0.007843138f − 1.0f` [M 0x169218; the literal is 0x3c008081 in the
   disassembly]:
   - `values[0] = f(slot[0x12])` (LX),
   - `values[1] = −f(slot[0x13])` (LY, negated: stick up (0) → +1),
   - `values[2] = f(slot[0x10])` (RX),
   - `values[3] = −f(slot[0x11])` (RY, negated) [M 0x16931c–0x16937c, `neg.s` on the 2nd/4th].
   The multiplier 0x3c008081 is exactly `2.0f32/255.0f32` (2/255 = 2^-7 × 1.00392156…, and the
   f32 mantissa fraction 32896.502/2^23 rounds up to 32897 = 0x008081) — the port keeps the bit
   pattern so the constant stays pinned to the binary.
4. **Dead zone**: `FUN_00167c48(−0.3f, 0.3f, &values[i])` for i = 0..3 [M 0x169384–0x16939c +
   the three sibling calls; constants 0xbe99999a/0x3e99999a in f12/f13]:
   `if (lo < v && v < hi) v = 0.0` [M 0x167c48 — strict inequalities]. **No rescale afterwards** —
   a value just past ±0.3 stays just past ±0.3; full deflection is ±1.0 (§10 for the on-foot
   contrast).
5. **Buttons** (always): for bit i = 0..15 of the u16 at slot+0x0c:
   `values[16+i] = bit ? 1.0 : 0.0` [M 0x1693c8 area loop].
6. **Digital-mode remap**: if `FUN_00259b90(0) == 4` (the slot's mode word == 4):
   `values[4..15] = values[{29,31,28,30,20,21,22,23,18,19,16,17}]` in that order (i.e. the
   DButton Right/Left/Up/Down/△/○/✕/□/L1/R1/L2/R2 floats are copied over the AButton slots) [M
   0x169400 area]. Else (analog mode): `values[4+i] = FUN_00169248(slot[0x14+i])` for i = 0..11,
   where `FUN_00169248(b) = min(1.0, b*0.0052083335f)` [M 0x169248; literal 0x3baaaaab = 1/192;
   the cap is `1.0 <= v → 1.0`].

The 32-value message therefore is:

| index | name (InputTable) | source |
|-------|-------------------|--------|
| 0..3 | ALX, ALY, ARX, ARY | sticks, dead-zoned ±0.3 |
| 4..15 | AButtonRight/Left/Up/Down/Triangle/Circle/Cross/Square/L1/R1/L2/R2 | pressure bytes /192 capped 1, or digital 0/1 remap |
| 16..31 | DButtonL2/R2/L1/R1/Triangle/Circle/Cross/Square/Select/LStick/RStick/Start/Up/Right/Down/Left | button bits, 0/1 |

## 4. The value table (InputTable) — `FUN_00168b08` 0x168b08, `FUN_00167788` 0x167788

`FUN_00168b08` registers 32 records, stride 0x34 [M]: `{u32 kind; char name[0x20]; float *prev
(=bufA+4i); float *cur (=bufB+4i); int *repeat_last (=bufC+4i); int *repeat_state (=bufD+4i)}`
[M 0x167788 stores kind, name copy, and the four pointers; the record's +0x24/+0x28 are read as
prev/cur by every condition helper, +0x2c/+0x30 as the auto-repeat state slots by 0x168678].
Kinds: 0 = axis (ALX/ALY/ARX/ARY), 1 = analog button, 2 = digital button [M the 0/1/2 third
argument of each 0x167788 call]. The name strings sit at 0x3746a0..0x374870 [M, listed in §3's
table]. On disconnect the repeat-state slot is set to 3 [M 0x167e80 tail].

The four buffers explain the multitap shape: each named value is readable across four pad slots'
messages, and the C/D buffers double as per-value auto-repeat scratch (as ints) [M 0x168678
writes through the record's +0x2c/+0x30 pointers].

## 5. Binding evaluation — `FUN_00168678` 0x168678 (called from `FUN_00168280` 0x168280)

`FUN_00168280` polls every device in the container's device array (vtable slot 3), then for each
(binding-set, device) pair calls `FUN_00168678(container, device, set)` [M 0x168280].

`FUN_00168678` [M 0x168678, size 880]:

- `n = device->vt[4]()` (= 32); for each value i: record = `table + i*0x34`;
  `cur = *(float*)record[+0x28]`; `changed = cur != *(float*)record[+0x24]` (A vs B buffer
  inequality — a raw float compare, so a stuck-at-value axis is "unchanged" [M]).
- The binding set holds one 0x3c-byte header per value: `{int count; …; entry* entries}` with
  0xc-byte entries `{u32 type; u32 arg; u32 action}` [M 0x168678 reads `set+4 + i*0x3c`,
  `entry[0]` type, `entry[1]` arg, `entry[2]` action].
- **Change gate**: if `container[+4] == 0` (send-all off) and type ∉ {8, 9} and `!changed`, the
  entry is skipped [M 0x168678 first test]. Types 8/9 (auto-repeat) always run.
- Per type [M the switch at 0x168678; helpers decoded below]:
  - **0** — send unconditionally (when the gate lets it through).
  - **1** — pressed edge: `FUN_001678f8` [M 0x1678f8]: kind 2: cur==1.0 && prev==0.0;
    kind 1/0: cur ≥ 0.5 && prev < 0.5.
  - **2** — released edge: `FUN_001679c8` [M 0x1679c8]: kind 2: cur==0.0 && prev==1.0;
    kind 1: cur < 0.5 && prev ≥ 0.5; kind 0 (axis): cur < −0.5 && prev ≥ −0.5.
  - **3** — positive crossing at `arg`: `FUN_00167a98(arg, rec)` [M 0x167a98]:
    cur ≥ arg && prev < arg.
  - **4** — negative crossing at `arg`: `FUN_00167ad8(arg, rec)` [M 0x167ad8]:
    cur < arg && prev ≥ arg.
  - **5** — returned to centre: `FUN_00167b18(0.2f, rec)` [M 0x167b18; 0x3e4ccccd]:
    |cur| ≤ 0.2 && (prev < −0.2 || 0.2 < prev).
  - **6** — send while the global flag is set: `FUN_001664c0()` returns the singleton at
    0x3c8780 and `FUN_00166d78` reads `*(u32*)(0x3c8780+0x20)` [M 0x1664c0/0x166d78]; non-zero →
    send. (Which runtime state that word mirrors is not pinned — §11.)
  - **7** — the same flag; send while it is **clear** [M the case-7 fallthrough].
  - **8** — pressed edge + auto-repeat: on pressed edge send and set
    `repeat_last = now, state = 1`; else if `FUN_001677f8` (is-down [M 0x1677f8: kind 2
    cur==1.0, kind 1/0 cur ≥ 0.5]) and `state != 3`: delay = (state==1 ? container[+0xa8] :
    container[+0xac]); if `repeat_last + delay < now` send and set `repeat_last = now,
    state = 2` [M 0x168678 case 8 + label 0x1688dc].
  - **9** — released edge + auto-repeat, mirror image with `FUN_001679c8` and is-up
    `FUN_00167878` [M 0x167878: kind 2 cur==0.0, kind 1 cur < 0.5, kind 0 cur < −0.5; case 9
    uses labels 0x168884/0x1688dc].
- When an entry fires the message `{action = entry[2], device = device[5], value = cur}` goes to
  `FUN_00165850` [M 0x168678 tail + iStack triple].
- After the last value an extra message with action 0x73 and value 0.0 is sent every frame
  [M 0x168678 tail — the frame-marker message].

## 6. `SendGameMessage` — `FUN_00165850` 0x165850

0x168678 sends an entry only when its action id is non-zero [M the `iStack_d0 != 0` test]. In
SendGameMessage the `0 < id < 0x41` test guards **only** the FIRESECONDARY drop: inside that window,
with `DAT_00339f54 == 0` (a global word, purpose unpinned), id 0x1f is dropped; every other id —
including the FRONTENDACTION ids 0x41.., the frame marker 0x73 and the unknown-name id 0x75 — goes
to the listener loop [M disassembly 0x165884 `blez a0 → 0x1658b0`, 0x165890 `beqz (id < 0x41) →
0x1658b0`, 0x1658b0 = the send loop]. (An earlier reading of this section said ids outside the
window were dropped; the disassembly shows they branch to the send. The pause page's drain
0x241210 switches on 0x41/0x46/0x49, which is where FrontEnd.def's actions arrive.) The message is
pushed to every listener in the singleton list at 0x3c8760 (`FUN_00165720` builds it,
`FUN_001654d8` delivers) [M 0x165720/0x165850 tail]. Port: `send_game_message_delivers`.

## 7. The action enum — data table at 0x32c098

Pairs `(u32 id, char* name)` in .data, names at 0x372aa0.. [M read directly from the ELF].
Driving-relevant ids:

```
1 STEER            0x1a BRAKE                0x2f CAMROTATEX2
2 STEERVERTICAL    0x1b HANDBRAKE            0x32 CAMSPIN
3 STEERVERTICALINV 0x1c HANDBRAKERELEASE     0x33 CAMCENTRE
4 STEERLEFT        0x1d FIREPRIMARY          0x34 CAMZOOMINOUT
5 STEERRIGHT       0x1e FIREPRIMARYRELEASE   0x37 AIMZOOMLEFT
6 STEERUP          0x1f FIRESECONDARY        0x38 AIMZOOMRIGHT
7 STEERDOWN        0x20 FIRESECONDARYRELEASE 0x39 AIMZOOMNONE
8 STEERLEFTDIGITAL 0x21 TOGGLESECONDARY      0x3a AIMRELEASE
9 STEERRIGHTDIGITAL 0x22 TOGGLESECONDARYUP   0x3b AUTODRIVELOCK
0xa STEERUPDIGITAL  0x23 TOGGLESECONDARYDOWN 0x3c AUTODRIVELOCKRELEASE
0xb STEERDOWNDIGITAL 0x24 FIREGADGET         0x3d LEANRIGHT
0x16 GAS            0x25 FIREGADGETRELEASE   0x3e LEANLEFT
0x17 GASBRAKE       0x26 CHANGECAMERA        0x3f POVRELOAD
0x18 WHEELSPIN      0x29 CAMLOOKBACK         0x40 RESUMEAUTODRIVE
0x19 WHEELSPINRELEASE 0x2b CAMZOOMBUTTON     (65.. FRONTENDACTION_*, 75.. DEBUGACTION_*)
```

0 = NULL_ACTION. The sound steering actions (0xc..0x15, `GAMEACTION_SOUND_*`) are the same table.

## 8. Ramps

There are **no attack/decay/ramp smoothers in the input path** [M-negative: the whole module
0x167788–0x169708 contains no accumulator/lerp toward a target; the only time-based behaviour is
the type-8/9 auto-repeat clock]. The action floats reach the game as the raw dead-zoned stick
value or 0/1. Any shaping of throttle/steer happens in the vehicle physics layer, outside G-004's
input path (the tuning keys `POST_BRAKE_ACCEL_*`, `TWO_WHEEL_*` exist in the physics .tun table
area at 0x3796xx–0x3797xx [D strings near PAD_DEAD_ZONE], which is a different module).

## 9. The SYM names, settled

- `PS2PadDevice::PollDevice` = 0x169288 [M, matches Raikaru's shipped-ELF `sub_169288`].
- `InputDevice::DeadZoneChop(float&, float, float)` ≈ `FUN_00167c48` 0x167c48 — the chop is
  `(min < v && v < max) → 0`, first argument the in/out float reference [M].
- `InputDevice::DeadZoneScale` — **not found as a separate function in the pad path** [M-negative;
  searched the whole 0x167788–0x169708 module and the callers]. The only scale in the path is the
  normalize constants inside 0x169218/0x169248. The SYM is from an older build, so the name may
  cover a helper that no longer exists or lives in the un-decoded USB path (`FUN_00259668`'s
  siblings) [S].
- `InputToAction::InputToAction(InputDevice*)` — the container class of §5 (device array,
  binding sets, repeat delays) [M shape; the constructor itself not pinned].
- `DeviceScalar`/`InputTable`/`StringToNumber` names sit in the EAGL symbol registry strings
  (0x374670/0x374680 area) next to the value names [M]; `InputTable` = the record array of §4.
- `GetDeviceScalar` (SYM 0x180640, absent here) ≈ the per-value read pattern
  `*(float*)record[+0x28]` [S].

## 10. Comparison with the on-foot path (input.md §2)

| | on-foot ACTION.ELF | driving DRIVING.ELF |
|---|---|---|
| stick bytes | tSlot +0x128..0x12b (RX,RY,LX,LY) | pad struct +0x10..0x13 (RX,RY,LX,LY) — same order [M] |
| buttons | `~(data[2]<<8|data[3])` active-high u16 | u16 at +0x0c, same bit layout [M] |
| normalize | none (bytes stay bytes) | `b*0x3c008081f − 1.0f` → −1..+1 [M 0x169218] |
| dead zone | per-axis byte space: `\|v\| < 40 → 0`, else `(v ∓ 40) × 1.4597701` (127/87) written back | float space: `−0.3 < v < 0.3 → 0`, **no rescale** [M 0x167c48] |
| same threshold 40 / scale 127/87? | yes | **no — different mechanism entirely** (±0.3 of full deflection ≈ byte 166 vs byte 87; and no 127/87 re-stretch) |
| curve table | `gAnalogStickMappingFunction` computed then overwritten (dead code) | none |
| output | 40 action floats via controller-style tables (`psiInput_MapInputs`) | 32 named values → data-driven bindings → `{action, device, value}` messages |
| ramps | AccelFunc0 turn/aim ramps in the input layer | none (auto-repeat only) |

So the two engines share the raw pad byte conventions but implement the dead zone and the
action mapping independently; the driving dead zone is much wider in effect (±30% of full
deflection) and does not re-stretch the surviving range.

## 11. Residue / open (U-003 remainder)

- The fill of the 0x419840 slot structs from the IOP (PADMAN mailbox) is not pinned to a function
  [S: it sits behind `FUN_0025a4a8`; the driving ELF strings contain PADMAN and PsIIlibpad].
- The binding DATA is decoded: §13 (the full image carries the configs).
- The type-6/7 flag `*(u32*)(0x3c8780+0x20)` is the invert setting (§13) [M]. Its boot value is
  `uRam00100a24 != 0` [M 0x1668a0]; which code first writes that low-memory word is not pinned
  (ACTION.ELF's decompile has no literal reference to it).
- The repeat-delay timer rate `FUN_00261170` (a gp-relative word) is not pinned, so the delays are
  known only as 300 ms / 150 ms of that clock (§13).
- `DAT_00339f54` (the FIRESECONDARY-send gate) semantics unpinned [M read]. Since the shipped
  driving config fires only through FIRESECONDARY (§13), the word must be non-zero whenever the
  car's gun works in play [S — inference, not traced].
- The USB keyboard/mouse device (`FUN_00259668` case table) is decoded only at the shape level;
  its byte synthesis into the pad-shaped state is its own pass [S].

## 12. The port (`crates/nf-game/src/input/driving.rs`)

Bit-exact ports of §3–§6 as pure functions: `stick_to_f32`, `pressure_to_f32`, `chop`,
`poll_device` (the 32-value message), the seven condition helpers, `Binding`/`evaluate` (types
0–5 and 8/9 with the change gate; types 6/7 take the global flag as a parameter), `send`-level
id validation, and the action-id constants. Unit tests pin every helper with golden tables from
the decompiled arithmetic plus sister assertions (the un-taken branch of each threshold, the
exact 0x3c008081/0x3baaaaab constants, the digital-mode remap order, the all-sticks-zero guard).

Keyboard+mouse mapping [S] (`KeyboardMouse` → `RawPadSlot`): W/S → LY bytes 0/255 (throttle up /
brake down through the Y negation), A/D → LX bytes 0/255 (steer left/right), Space → Cross bit
(handbrake), mouse buttons → △/○ (primary/secondary fire), arrow cluster → the D-pad bits
(camera). These are the port's bindings, not the shipped ones (§11) — a driving level would call
`poll_device` on its pad slot each tick (where `FUN_00168280` runs in the engine) and feed the
result through its own binding table; the driving levels themselves stay unwired.

## 13. The control configs — `data\Control\*.def` [D + M]

The full image carries them in nine archives (MIS01, MIS11, MIS13A/B/C, MIS3, MIS4, RACE: 11 RefPack
entries each; MISC: 12 plain entries, the extra one `DbgCfg.def`). Every copy of a name is
byte-identical across archives — 12 distinct files [D, `nf-game/tests/driving_controls_disc.rs`].
They are CRLF text; `nf_formats::controldef` parses them.

**Loader** `FUN_001668a0` 0x1668a0 [M]: `FUN_00166548` reads `data/Control/Master.def`
(`POV 8` + povcfg0..7.def, `Driving 1` + drivecfg.def, `End`; keywords by stricmp
`FUN_002a9198`), each listed file is loaded as `data/Control/<name>` into a 0x5c-byte record (name
+0, text +0x20, FRONTEND fields +0x24..+0x58), `data/Control/FrontEnd.def` into the manager's
+0x28, and each record's FRONTEND block is read by `FUN_00166788`. The manager singleton
0x3c8780 (`FUN_001664c0`): +4 mode (0 Driving list, 1 POV list), +8/+0xc the per-mode index
(boot values `uRam00100a14`/`uRam0010098c`, reset to 0 with "invalid control config" when out of
range), +0x10/+0x14 counts, +0x18/+0x1c record arrays, +0x20 the invert flag. `DbgCfg.def` is never
named by Master.def or the ELF [M-negative: no `DbgCfg` string in DRIVING.ELF] — dead data.

**Active set** `FUN_00166f28` [M]: the pad's binding set (`FUN_00167d80()+0x50+4·pad`) is cleared
(`FUN_00167678`), then `FUN_001674d0` appends FrontEnd.def's bindings, then the selected config's.
`FUN_00166cb0(m, mode, index)` re-selects (the mission/POV switch 0x206398 passes mode 0 / 1; the
pause page's controller list 0x2411a0 passes its index).

**Binding line** `FUN_001674d0` [M]: header line skipped; per line `strtok(" \t")`:
`<action> <value> [<method> [<param>]]`, ending at `FRONTEND` or `END` (stricmp). Action → id via
the 117-name table at 0x32c090 (ids 0..0x74; unknown → 0x75, `FUN_001651a0`); value → the 32
InputTable names (§4); method letter via the table at 0x32e5c0: `U`0 `+`1 `-`2 `>`3 `<`4 `=`5 `I`6
`N`7 `R`8 `M`9 (missing/unknown → 0); param defaults 0.5 for types 3/4, else 0, then `%f`
overrides. Names are strcmp-exact (`FUN_002a9508`; binary search over the name-sorted copy,
`FUN_002cedf0`). An unknown value name is not checked (the game would index the set at −1) — none
occurs on the disc [D].

**The invert flag** (types 6/7): FRONTENDACTION_INVERT (0x49, FrontEnd.def: DButtonCircle `+`) on
the pause page's controller state flips manager +0x20 (`FUN_00166d60(m, flag ^ 1)`, which also
stores `uRam00100a24`) [M 0x241210 case 0x49]. The POV configs bind `CAMROTATEY ... N` (type 7, sent
while not inverted) and `CAMROTATEYINV ... I` (type 6, sent while inverted) [D].

**Auto-repeat delays** [M 0x168368]: container +0xa8 = `rate·300/1000`, +0xac = `rate·150/1000`
(C integer division), `rate = FUN_00261170()` — 300 ms before the first repeat, 150 ms after.
FrontEnd.def's menu moves use `R`/`M` (types 8/9) [D].

**The shipped driving mapping** (drivecfg.def, after FrontEnd.def) [D]:

| pad | binding | action |
|---|---|---|
| left stick X | U; `-`/`+`; `=` | STEER, CAMROTATEX; SOUND_STEERLEFT/RIGHT; SOUND_STEERHORIZONTALRELEASE |
| left stick Y | U; `+`/`-`; `=` | STEERVERTICAL, CAMROTATEY; SOUND_STEERUP/DOWN; SOUND_STEERVERTICALRELEASE |
| ✕ pressure | U | GAS |
| □ pressure | U | BRAKE |
| ○ pressure | `>` 0.1 / `<` 0.1 | HANDBRAKE / HANDBRAKERELEASE |
| R1 | `+` / `-` | FIRESECONDARY / RELEASE (nothing binds FIREPRIMARY) |
| L1 | `+` / `-` | FIREGADGET / RELEASE |
| R2 | `+` | TOGGLESECONDARYDOWN (previous weapon) |
| d-pad Left / Right | `+` | TOGGLESECONDARYUP (next) / TOGGLESECONDARYDOWN (previous) |
| △ | `+` | CHANGECAMERA |
| L2 | `+` / `-` | CAMLOOKBACK / RELEASE |
| Start | `+` (FrontEnd.def) | FRONTENDACTION_START (pause) |

FrontEnd.def also puts ACCEPT/SKIPMOVIE on ✕, CANCEL on △, INVERT on ○, SCREENSHOT on Select,
the menu moves on the d-pad and left stick, and the cheat-code entries (CHEATACTION_*) on the face
buttons, d-pad and L1/R1 — all live while driving, since the sets are merged [D + M 0x166f28].
There is **no horn** action anywhere (the 117-name table has none) and no reload while driving
(POVRELOAD is bound only in the POV configs) [M + D].

**Analog shaping**: no curve exists in the data or the path — the stick is raw `b·2/255 − 1`
chopped at ±0.3 (§3), the pedals are raw pressures `min(1, b/192)`, the handbrake is a 0.1
pressure threshold, and the vehicle layer chops steer at ±0.2 and ramps it 0.25/tick
(driving-pc-input.md §1).

**Port** (`nf_game::input::driving_cfg`): `ACTION_NAMES`/`action_id`, `VALUE_NAMES`/`value_index`,
`binding_kind`, `binding_set` (load order), `repeat_delays`, `DrivingKeys` (a keyboard front end:
each key presses the pad button the shipped file binds the wanted action to, bit + full pressure,
analog mode) and `ShippedDrivingInput::tick` (keys → `poll_device` → the shipped set → messages
with the 0x1f gate → `CarInputStage`). The key choice is the port's [S]; the button → action
routing is the disc's. Unproven: the timer rate, the boot invert value, what `DAT_00339f54`
mirrors. Nothing wires a driving level to it yet.
