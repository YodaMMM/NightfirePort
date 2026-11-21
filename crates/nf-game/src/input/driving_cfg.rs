//! The shipped driving control configs as the game's binding sets — G-004, the data half
//! (docs/systems/driving-input.md §13).
//!
//! `nf_formats::controldef` reads the `.def` text; this module resolves it the way DRIVING.ELF
//! does and runs it through the [`super::driving`] evaluator:
//!
//! - action names → ids through the `StringToNumber` table at 0x32c090 (117 names, ids 0..0x74);
//!   an unknown name is id 0x75 [M `FUN_001651a0` 0x1651a0];
//! - value names → the 32 InputTable indexes [M `FUN_00168b08` registration order];
//! - the active set = FrontEnd.def's bindings, then the selected config's, appended per value in
//!   file order into the cleared set [M `FUN_00166f28` 0x166f28: `FUN_00167678` clear,
//!   `FUN_001674d0(set, +0x28 FrontEnd.def)`, `FUN_001674d0(set, config)`];
//! - which config: the manager singleton 0x3c8780 keeps mode `+4` (0 = the Driving list, 1 = the
//!   POV list) and per-mode indexes `+8`/`+0xc`; mission/POV switches call `FUN_00166cb0`
//!   (0x206398 picks mode 0 or 1), the pause page's controller list picks the index (0x2411a0)
//!   [M]. An out-of-range index is reset to 0 with the "invalid control config" warning
//!   [M 0x1668a0].
//! - the type-6/7 flag is the manager's `+0x20` = the invert setting: FRONTENDACTION_INVERT (0x49)
//!   on the pause page's controller state flips it (`FUN_00166d60(m, flag ^ 1)` in 0x241210),
//!   and the shipped POV configs bind CAMROTATEY with `N` (type 7, sent while clear) and
//!   CAMROTATEYINV with `I` (type 6, sent while set) [M + D].
//! - the auto-repeat delays: container `+0xa8 = rate·300/1000`, `+0xac = rate·150/1000` with the
//!   timer's tick rate `FUN_00261170` [M 0x168368] — 300 ms first, 150 ms after. The rate word is
//!   gp-relative and its value is not pinned here.

use nf_formats::controldef::ControlDef;

use super::driving::{
    action, poll_device, send_game_message_delivers, value, ActionMsg, Binding, BindingKind, CarInputStage, Kind,
    RawPadSlot, RepeatDelays, RepeatState, ValueBindings, evaluate_value,
};

/// The action-name table, index = id [M .data 0x32c090, read pair by pair from the ELF].
pub const ACTION_NAMES: [&str; 0x75] = [
    "NULL_ACTION",
    "GAMEACTION_STEER", "GAMEACTION_STEERVERTICAL", "GAMEACTION_STEERVERTICALINV",
    "GAMEACTION_STEERLEFT", "GAMEACTION_STEERRIGHT", "GAMEACTION_STEERUP", "GAMEACTION_STEERDOWN",
    "GAMEACTION_STEERLEFTDIGITAL", "GAMEACTION_STEERRIGHTDIGITAL", "GAMEACTION_STEERUPDIGITAL",
    "GAMEACTION_STEERDOWNDIGITAL",
    "GAMEACTION_SOUND_STEERLEFT", "GAMEACTION_SOUND_STEERRIGHT", "GAMEACTION_SOUND_STEERUP",
    "GAMEACTION_SOUND_STEERDOWN", "GAMEACTION_SOUND_STEERHORIZONTALRELEASE",
    "GAMEACTION_SOUND_STEERVERTICALRELEASE", "GAMEACTION_SOUND_STEERPADLEFT",
    "GAMEACTION_SOUND_STEERPADRIGHT", "GAMEACTION_SOUND_STEERPADUP", "GAMEACTION_SOUND_STEERPADDOWN",
    "GAMEACTION_GAS", "GAMEACTION_GASBRAKE", "GAMEACTION_WHEELSPIN", "GAMEACTION_WHEELSPINRELEASE",
    "GAMEACTION_BRAKE", "GAMEACTION_HANDBRAKE", "GAMEACTION_HANDBRAKERELEASE",
    "GAMEACTION_FIREPRIMARY", "GAMEACTION_FIREPRIMARYRELEASE", "GAMEACTION_FIRESECONDARY",
    "GAMEACTION_FIRESECONDARYRELEASE", "GAMEACTION_TOGGLESECONDARY", "GAMEACTION_TOGGLESECONDARYUP",
    "GAMEACTION_TOGGLESECONDARYDOWN", "GAMEACTION_FIREGADGET", "GAMEACTION_FIREGADGETRELEASE",
    "GAMEACTION_CHANGECAMERA", "GAMEACTION_CHANGECAMERAUP", "GAMEACTION_CHANGECAMERADOWN",
    "GAMEACTION_CAMLOOKBACK", "GAMEACTION_CAMLOOKBACKRELEASE", "GAMEACTION_CAMZOOMBUTTON",
    "GAMEACTION_CAMROTATEX", "GAMEACTION_CAMROTATEY", "GAMEACTION_CAMROTATEYINV",
    "GAMEACTION_CAMROTATEX2", "GAMEACTION_CAMROTATEY2", "GAMEACTION_CAMROTATEY2INV",
    "GAMEACTION_CAMSPIN", "GAMEACTION_CAMCENTRE", "GAMEACTION_CAMZOOMINOUT", "GAMEACTION_CAMZOOMIN",
    "GAMEACTION_CAMZOOMOUT", "GAMEACTION_AIMZOOMLEFT", "GAMEACTION_AIMZOOMRIGHT",
    "GAMEACTION_AIMZOOMNONE", "GAMEACTION_AIMRELEASE", "GAMEACTION_AUTODRIVELOCK",
    "GAMEACTION_AUTODRIVELOCKRELEASE", "GAMEACTION_LEANRIGHT", "GAMEACTION_LEANLEFT",
    "GAMEACTION_POVRELOAD", "GAMEACTION_RESUMEAUTODRIVE",
    "FRONTENDACTION_UP", "FRONTENDACTION_DOWN", "FRONTENDACTION_LEFT", "FRONTENDACTION_RIGHT",
    "FRONTENDACTION_ACCEPT", "FRONTENDACTION_CANCEL", "FRONTENDACTION_START",
    "FRONTENDACTION_SCREENSHOT", "FRONTENDACTION_INVERT", "FRONTENDACTION_SKIPMOVIE",
    "DEBUGACTION_AXISX", "DEBUGACTION_AXISY", "DEBUGACTION_AXISRX", "DEBUGACTION_AXISRY",
    "DEBUGACTION_SCREENSHOT", "DEBUGACTION_STOPRECORDPATH",
    "CHEATACTION_DPAD_LEFT", "CHEATACTION_DPAD_LEFT_RELEASE", "CHEATACTION_DPAD_UP",
    "CHEATACTION_DPAD_UP_RELEASE", "CHEATACTION_DPAD_RIGHT", "CHEATACTION_DPAD_RIGHT_RELEASE",
    "CHEATACTION_DPAD_DOWN", "CHEATACTION_DPAD_DOWN_RELEASE", "CHEATACTION_BUT_LEFT",
    "CHEATACTION_BUT_LEFT_RELEASE", "CHEATACTION_BUT_UP", "CHEATACTION_BUT_UP_RELEASE",
    "CHEATACTION_BUT_RIGHT", "CHEATACTION_BUT_RIGHT_RELEASE", "CHEATACTION_BUT_DOWN",
    "CHEATACTION_BUT_DOWN_RELEASE", "CHEATACTION_TRIGGER_LEFT", "CHEATACTION_TRIGGER_LEFT_RELEASE",
    "CHEATACTION_TRIGGER_RIGHT", "CHEATACTION_TRIGGER_RIGHT_RELEASE",
    "ACTION_EXITAPPLICATION", "ACTION_MINIMIZEAPPLICATION", "ACTION_RESTOREAPPLICATION",
    "ACTION_MOUSEMOVE_X", "ACTION_MOUSEMOVE_Y", "ACTION_MOUSEMOVE", "ACTION_MOUSELBUTTONDOWN",
    "ACTION_MOUSELBUTTONUP", "ACTION_MOUSERBUTTONDOWN", "ACTION_MOUSERBUTTONUP", "ACTION_KEYDOWN",
    "ACTION_KEYUP", "ACTION_CHAR", "ACTION_CENTER", "ACTION_INPUTSYNC", "ACTION_PLUGUNPLUG",
];

/// The id `FUN_001651a0` returns for a name not in the table [M 0x1651a0].
pub const UNKNOWN_ACTION: u32 = 0x75;

/// The front-end action ids the shipped FrontEnd.def binds [M table above].
pub mod frontend {
    pub const UP: u8 = 0x41;
    pub const DOWN: u8 = 0x42;
    pub const LEFT: u8 = 0x43;
    pub const RIGHT: u8 = 0x44;
    pub const ACCEPT: u8 = 0x45;
    pub const CANCEL: u8 = 0x46;
    /// Start = pause [M the pause page drain 0x241210 receives the 0x41.. ids].
    pub const START: u8 = 0x47;
    pub const SCREENSHOT: u8 = 0x48;
    /// Flips the invert flag (manager +0x20) on the pause page's controller state [M 0x241210 case 0x49].
    pub const INVERT: u8 = 0x49;
    pub const SKIPMOVIE: u8 = 0x4a;
}

/// The 32 InputTable value names, index = message value [M 0x168b08, strings 0x3746a0..0x374870].
pub const VALUE_NAMES: [&str; value::COUNT] = [
    "ALX", "ALY", "ARX", "ARY",
    "AButtonRight", "AButtonLeft", "AButtonUp", "AButtonDown", "AButtonTriangle", "AButtonCircle",
    "AButtonCross", "AButtonSquare", "AButtonL1", "AButtonR1", "AButtonL2", "AButtonR2",
    "DButtonL2", "DButtonR2", "DButtonL1", "DButtonR1", "DButtonTriangle", "DButtonCircle",
    "DButtonCross", "DButtonSquare", "DButtonSelect", "DButtonLStick", "DButtonRStick",
    "DButtonStart", "DButtonUp", "DButtonRight", "DButtonDown", "DButtonLeft",
];

/// Exact (strcmp) name → id; unknown → 0x75 [M 0x1651a0 / 0x2cec98 / 0x2a9508].
pub fn action_id(name: &str) -> u32 {
    ACTION_NAMES.iter().position(|&n| n == name).map_or(UNKNOWN_ACTION, |i| i as u32)
}

/// Exact (strcmp) value name → message index [M 0x167278 builds the lookup from the records].
pub fn value_index(name: &str) -> Option<usize> {
    VALUE_NAMES.iter().position(|&n| n == name)
}

/// The record kind of each value [M 0x168b08's third argument per call: 0..3 axis, 4..15
/// analog button, 16..31 digital button].
pub fn value_kind(i: usize) -> Kind {
    match i {
        0..=3 => Kind::Axis,
        4..=15 => Kind::AButton,
        _ => Kind::DButton,
    }
}

/// Binding type number → evaluator kind [M the 0x168678 switch; letters in controldef].
pub fn binding_kind(t: u8) -> BindingKind {
    match t {
        1 => BindingKind::Pressed,
        2 => BindingKind::Released,
        3 => BindingKind::CrossPos,
        4 => BindingKind::CrossNeg,
        5 => BindingKind::Centred,
        6 => BindingKind::GlobalSet,
        7 => BindingKind::GlobalClear,
        8 => BindingKind::PressRepeat,
        9 => BindingKind::ReleaseRepeat,
        // 0, and anything the parser could not produce (types > 9 hit the switch default: never
        // sent [M]) — the parser only yields 0..9.
        _ => BindingKind::Always,
    }
}

/// The game's auto-repeat delays from the timer rate [M 0x168368: C integer division].
pub fn repeat_delays(rate: i32) -> RepeatDelays {
    RepeatDelays { first: rate.wrapping_mul(300) / 1000, rest: rate.wrapping_mul(150) / 1000 }
}

/// Builds the active binding set from binding files in load order (FrontEnd.def first, then the
/// selected config) [M 0x166f28 + 0x1674d0 + 0x167120 append]. A value name outside the table
/// is an error: the game would index the set at −1 [M 0x1674d0 has no check].
pub fn binding_set(files: &[&ControlDef]) -> Result<Vec<ValueBindings>, String> {
    let mut set: Vec<ValueBindings> = (0..value::COUNT)
        .map(|i| ValueBindings { kind: value_kind(i), bindings: Vec::new(), repeat: RepeatState::default() })
        .collect();
    for f in files {
        for b in &f.bindings {
            let vi = value_index(&b.value).ok_or_else(|| format!("unknown value name {}", b.value))?;
            let id = action_id(&b.action);
            set[vi].bindings.push(Binding { kind: binding_kind(b.method), arg: b.arg, action: id as u8 });
        }
    }
    Ok(set)
}

/// The pad buttons, as libpad button bits and pressure slots [M 0x169288 bit loop + pressure
/// order]. `None` pressure = a button with no analog byte (Select, L3, R3, Start).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    L2, R2, L1, R1, Triangle, Circle, Cross, Square, Select, L3, R3, Start, Up, Right, Down, Left,
}

impl PadButton {
    pub fn bit(self) -> u16 {
        1 << (self as u16)
    }
    /// The pressure slot (0x14 + slot) libpad fills for this button.
    pub fn pressure_slot(self) -> Option<usize> {
        use PadButton::*;
        Some(match self {
            Right => 0, Left => 1, Up => 2, Down => 3, Triangle => 4, Circle => 5, Cross => 6,
            Square => 7, L1 => 8, R1 => 9, L2 => 10, R2 => 11,
            Select | L3 | R3 | Start => return None,
        })
    }
}

/// A keyboard/mouse front end for the shipped driving config (drivecfg.def + FrontEnd.def).
/// Each field is one key's state; `to_raw_pad` presses the pad button the shipped file binds the
/// wanted action to, so the game's own bindings produce the action ids. The key choice is the
/// port's [S]; the button → action routing is the disc data [D drivecfg.def / FrontEnd.def]:
///
/// | field | pad button | shipped bindings on that button |
/// |---|---|---|
/// | gas | ✕ | AButtonCross U → GAS; DButtonCross + → FRONTENDACTION_ACCEPT, _SKIPMOVIE |
/// | brake | □ | AButtonSquare U → BRAKE |
/// | handbrake | ○ | AButtonCircle > 0.1 → HANDBRAKE, < 0.1 → HANDBRAKERELEASE; DButtonCircle + → FRONTENDACTION_INVERT |
/// | steer_left / steer_right | left stick X 0 / 255 | ALX U → STEER, CAMROTATEX; ± → SOUND_STEER*; = → SOUND_STEERHORIZONTALRELEASE |
/// | fire | R1 | + → FIRESECONDARY, − → FIRESECONDARYRELEASE |
/// | gadget | L1 | + → FIREGADGET, − → FIREGADGETRELEASE |
/// | weapon_next | d-pad Left | + → TOGGLESECONDARYUP (next, `FUN_0020d4c0(w,1)`) |
/// | weapon_prev | d-pad Right | + → TOGGLESECONDARYDOWN (previous) |
/// | weapon_prev_alt | R2 | + → TOGGLESECONDARYDOWN |
/// | camera | △ | + → CHANGECAMERA; FRONTENDACTION_CANCEL |
/// | look_back | L2 | + → CAMLOOKBACK, − → CAMLOOKBACKRELEASE |
/// | pause | Start | + → FRONTENDACTION_START |
/// | menu_up / menu_down | d-pad Up / Down | R → FRONTENDACTION_UP / DOWN (auto-repeat) |
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrivingKeys {
    pub gas: bool,
    pub brake: bool,
    pub handbrake: bool,
    pub steer_left: bool,
    pub steer_right: bool,
    pub fire: bool,
    pub gadget: bool,
    pub weapon_next: bool,
    pub weapon_prev: bool,
    pub weapon_prev_alt: bool,
    pub camera: bool,
    pub look_back: bool,
    pub pause: bool,
    pub menu_up: bool,
    pub menu_down: bool,
}

impl DrivingKeys {
    /// The pad buttons held this tick.
    pub fn buttons(&self) -> Vec<PadButton> {
        use PadButton::*;
        let mut v = Vec::new();
        let mut add = |on: bool, b: PadButton| if on { v.push(b) };
        add(self.gas, Cross);
        add(self.brake, Square);
        add(self.handbrake, Circle);
        add(self.fire, R1);
        add(self.gadget, L1);
        add(self.weapon_next, Left);
        add(self.weapon_prev, Right);
        add(self.weapon_prev_alt, R2);
        add(self.camera, Triangle);
        add(self.look_back, L2);
        add(self.pause, Start);
        add(self.menu_up, Up);
        add(self.menu_down, Down);
        v
    }

    /// The raw slot a DualShock 2 in analog mode reports for these keys: a held button sets its
    /// bit and a full pressure byte (255 → 1.0 after the /192 cap [M 0x169248]); the left stick
    /// X is 0 / 255 / 127 (127 reads −0.0039, chopped to 0 by the ±0.3 dead zone [M 0x167c48]).
    /// Mode 0 keeps the pressure path (mode 4 would copy the bits over the pressures [M 0x169400]).
    pub fn to_raw_pad(&self) -> RawPadSlot {
        let lx = match (self.steer_left, self.steer_right) {
            (true, false) => 0u8,
            (false, true) => 255u8,
            _ => 127u8,
        };
        let mut buttons = 0u16;
        let mut pressures = [0u8; 12];
        for b in self.buttons() {
            buttons |= b.bit();
            if let Some(s) = b.pressure_slot() {
                pressures[s] = 255;
            }
        }
        RawPadSlot { mode: 0, buttons, rx: 127, ry: 127, lx, ly: 127, pressures }
    }
}

/// One 60 Hz step of the shipped driving chain: keys → raw pad → `poll_device` → the shipped
/// binding set (FrontEnd.def + a config) → `{action, device, value}` messages (SendGameMessage's
/// 0x1f gate applied) → the car's input stage. Returns every delivered message; the car stage
/// takes the ones it handles, the rest (camera, weapon, pause page) are for their own drains.
pub struct ShippedDrivingInput {
    pub keys: DrivingKeys,
    pub stage: CarInputStage,
    /// The type-6/7 flag (manager +0x20, the invert setting). Boot value = `uRam00100a24 != 0`
    /// [M 0x1668a0]; where that word is first written is not pinned — default false [S].
    pub invert: bool,
    /// `DAT_00339f54 != 0` — while false SendGameMessage drops FIRESECONDARY [M 0x165850]. What
    /// the word mirrors is not pinned; default true so the shipped fire button fires [S].
    pub fire_gate: bool,
    pub delays: RepeatDelays,
    bindings: Vec<ValueBindings>,
    prev: [f32; value::COUNT],
}

impl ShippedDrivingInput {
    /// `files` in load order: FrontEnd.def, then the selected config [M 0x166f28]. `delays` are
    /// in the `now` clock's units (see [`repeat_delays`]).
    pub fn new(files: &[&ControlDef], delays: RepeatDelays) -> Result<Self, String> {
        Ok(ShippedDrivingInput {
            keys: DrivingKeys::default(),
            stage: CarInputStage::default(),
            invert: false,
            fire_gate: true,
            delays,
            bindings: binding_set(files)?,
            prev: [0.0; value::COUNT],
        })
    }

    pub fn bindings(&self) -> &[ValueBindings] {
        &self.bindings
    }

    pub fn tick(&mut self, now: i32) -> Vec<ActionMsg> {
        let cur = poll_device(&self.keys.to_raw_pad());
        let mut out = Vec::new();
        for (i, vb) in self.bindings.iter_mut().enumerate() {
            let changed = cur[i] != self.prev[i];
            evaluate_value(vb, cur[i], self.prev[i], changed, false, now, &self.delays, self.invert, 0, &mut out);
        }
        self.prev = cur;
        out.retain(|m| send_game_message_delivers(m.action as u32, self.fire_gate));
        for m in &out {
            self.stage.action(m);
        }
        self.stage.tick();
        out
    }
}

/// Is `id` one the car's own drain handles [M 0x18add8 switch: 1, 2, 3, 0xc–0x11, 0x16, 0x17,
/// 0x1a–0x21]? (For callers sorting the returned messages.)
pub fn is_car_action(id: u8) -> bool {
    matches!(id, 1..=3 | 0xc..=0x11 | action::GAS | action::GASBRAKE | 0x1a..=0x21)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::controldef::ControlDef;

    /// The shipped drivecfg.def binding lines, transcribed in the file's own format [D the disc
    /// file; the disc test `driving_controls_disc` checks the real bytes decode to the same set].
    const DRIVECFG: &str = "action  \t\t\t\t\t\tbutton\t\t\tupdate method\toptional parameter\r\n\
GAMEACTION_STEER\t\t\tALX\r\n\
GAMEACTION_STEERVERTICAL\t\tALY\t\t\t\r\n\
GAMEACTION_SOUND_STEERLEFT\t\tALX\t\t\t-\r\n\
GAMEACTION_SOUND_STEERRIGHT\t\tALX\t\t\t+\r\n\
GAMEACTION_GAS\t\t\t\tAButtonCross\r\n\
GAMEACTION_BRAKE\t\t\tAButtonSquare\r\n\
GAMEACTION_HANDBRAKE\t\t\tAButtonCircle\t\t\t>\t0.1\r\n\
GAMEACTION_HANDBRAKERELEASE\t\tAButtonCircle\t\t\t<\t0.1\r\n\
GAMEACTION_FIRESECONDARY\t\tDButtonR1\t\t\t+\r\n\
GAMEACTION_FIRESECONDARYRELEASE\t\tDButtonR1\t\t\t-\r\n\
GAMEACTION_FIREGADGET\t\t\tDButtonL1\t\t\t+\r\n\
GAMEACTION_FIREGADGETRELEASE\t\tDButtonL1\t\t\t-\r\n\
GAMEACTION_TOGGLESECONDARYDOWN\t\tDButtonR2\t\t\t+\r\n\
GAMEACTION_TOGGLESECONDARYUP\t\tDButtonLeft\t\t\t+\r\n\
GAMEACTION_TOGGLESECONDARYDOWN\t\tDButtonRight\t\t\t+\r\n\
GAMEACTION_CHANGECAMERA\t\t\tDButtonTriangle\t\t\t+\r\n\
GAMEACTION_CAMLOOKBACK\t\t\t\tDButtonL2\t\t+\r\n\
GAMEACTION_CAMLOOKBACKRELEASE\t\t\tDButtonL2\t\t-\r\n\
FRONTEND\t; for frontend\r\nTITLE\t\t\t2729\r\nEND\r\n";

    const FRONTEND: &str = "action\r\n\
FRONTENDACTION_UP\t\t\tDButtonUp\t\t\tR\r\n\
FRONTENDACTION_DOWN\t\t\tDButtonDown\t\t\tR\r\n\
FRONTENDACTION_ACCEPT\t\t\tDButtonCross\t\t\t+\r\n\
FRONTENDACTION_CANCEL\t\t\tDButtonTriangle\t\t\t+\r\n\
FRONTENDACTION_START\t\t\tDButtonStart\t\t\t+\r\n\
FRONTENDACTION_INVERT\t\t\tDButtonCircle\t\t\t+\r\n\
END\r\n";

    fn input() -> ShippedDrivingInput {
        let fe = ControlDef::parse(FRONTEND.as_bytes()).unwrap();
        let dc = ControlDef::parse(DRIVECFG.as_bytes()).unwrap();
        ShippedDrivingInput::new(&[&fe, &dc], repeat_delays(60)).unwrap()
    }

    fn ids(m: &[ActionMsg]) -> Vec<u8> {
        m.iter().map(|m| m.action).collect()
    }

    #[test]
    fn action_table_shape() {
        assert_eq!(action_id("NULL_ACTION"), 0);
        assert_eq!(action_id("GAMEACTION_STEER"), action::STEER as u32);
        assert_eq!(action_id("GAMEACTION_GAS"), action::GAS as u32);
        assert_eq!(action_id("GAMEACTION_HANDBRAKE"), action::HANDBRAKE as u32);
        assert_eq!(action_id("GAMEACTION_CAMLOOKBACK"), action::CAMLOOKBACK as u32);
        assert_eq!(action_id("GAMEACTION_POVRELOAD"), action::POVRELOAD as u32);
        assert_eq!(action_id("GAMEACTION_RESUMEAUTODRIVE"), action::RESUMEAUTODRIVE as u32);
        assert_eq!(action_id("FRONTENDACTION_START"), frontend::START as u32);
        assert_eq!(action_id("FRONTENDACTION_INVERT"), frontend::INVERT as u32);
        assert_eq!(action_id("ACTION_INPUTSYNC"), 0x73, "the per-frame marker 0x168678 sends");
        assert_eq!(action_id("ACTION_PLUGUNPLUG"), 0x74);
        assert_eq!(action_id("gameaction_steer"), UNKNOWN_ACTION, "strcmp, case-sensitive");
        assert_eq!(action_id("GAMEACTION_HORN"), UNKNOWN_ACTION, "no horn action exists");
        assert_eq!(value_index("ALX"), Some(value::ALX));
        assert_eq!(value_index("AButtonCross"), Some(10));
        assert_eq!(value_index("DButtonLeft"), Some(31));
        assert_eq!(value_index("DButtonCross"), Some(22));
        assert_eq!(value_index("alx"), None);
    }

    #[test]
    fn repeat_delays_are_300_and_150_ms() {
        assert_eq!(repeat_delays(1000), RepeatDelays { first: 300, rest: 150 });
        assert_eq!(repeat_delays(60), RepeatDelays { first: 18, rest: 9 });
        assert_eq!(repeat_delays(7), RepeatDelays { first: 2, rest: 1 }, "C truncation: 2100/1000, 1050/1000");
    }

    #[test]
    fn set_order_is_frontend_then_config() {
        let s = input();
        let b = s.bindings();
        // DButtonTriangle: FrontEnd's CANCEL first, then drivecfg's CHANGECAMERA [M 0x166f28].
        assert_eq!(b[20].bindings.iter().map(|x| x.action).collect::<Vec<_>>(), vec![frontend::CANCEL, action::CHANGECAMERA]);
        assert_eq!(b[9].bindings[0], Binding { kind: BindingKind::CrossPos, arg: 0.1, action: action::HANDBRAKE });
        assert_eq!(b[9].kind, Kind::AButton);
        assert_eq!(b[value::ALX].kind, Kind::Axis);
    }

    #[test]
    fn gas_brake_steer_reach_the_car() {
        let mut s = input();
        s.keys.gas = true;
        s.keys.steer_right = true;
        let m = s.tick(1);
        assert!(ids(&m).contains(&action::GAS));
        assert!(ids(&m).contains(&action::STEER));
        assert!(ids(&m).contains(&frontend::ACCEPT), "✕ also carries the front end's ACCEPT");
        assert_eq!(s.stage.gas, 1.0);
        assert_eq!(s.stage.raw_steer, 1.0);
        assert_eq!(s.stage.steer, 0.25, "the 0.25/tick vehicle ramp");
        // Held: no change → no resend (type 0 is change-gated).
        let m = s.tick(2);
        assert!(!ids(&m).contains(&action::GAS));
        assert_eq!(s.stage.steer, 0.5);
        s.keys = DrivingKeys { brake: true, ..Default::default() };
        let m = s.tick(3);
        assert!(ids(&m).contains(&action::BRAKE));
        assert!(ids(&m).contains(&action::GAS), "gas release sends GAS 0.0");
        assert_eq!((s.stage.gas, s.stage.brake), (0.0, 1.0));
        assert_eq!(s.stage.raw_steer, 0.0);
    }

    #[test]
    fn handbrake_is_the_circle_pressure_threshold() {
        let mut s = input();
        s.keys.handbrake = true;
        let m = s.tick(1);
        assert!(ids(&m).contains(&action::HANDBRAKE));
        assert!(ids(&m).contains(&frontend::INVERT), "○'s bit also carries FRONTENDACTION_INVERT");
        assert!(s.stage.handbrake);
        s.keys.handbrake = false;
        let m = s.tick(2);
        assert!(ids(&m).contains(&action::HANDBRAKERELEASE));
        assert!(!s.stage.handbrake);
    }

    #[test]
    fn fire_is_r1_secondary_and_the_gate_drops_it() {
        let mut s = input();
        s.keys.fire = true;
        assert_eq!(ids(&s.tick(1)), vec![action::FIRESECONDARY]);
        assert!(s.stage.fire_secondary);
        s.keys.fire = false;
        assert_eq!(ids(&s.tick(2)), vec![action::FIRESECONDARYRELEASE]);
        // With the gate word 0 the press is dropped, the release still goes [M 0x165850].
        s.fire_gate = false;
        s.keys.fire = true;
        assert!(s.tick(3).is_empty());
        s.keys.fire = false;
        assert_eq!(ids(&s.tick(4)), vec![action::FIRESECONDARYRELEASE]);
    }

    #[test]
    fn weapon_camera_lookback_pause_ids() {
        let cases: [(fn(&mut DrivingKeys), Vec<u8>); 6] = [
            (|k| k.weapon_next = true, vec![action::TOGGLESECONDARYUP]),
            (|k| k.weapon_prev = true, vec![action::TOGGLESECONDARYDOWN]),
            (|k| k.weapon_prev_alt = true, vec![action::TOGGLESECONDARYDOWN]),
            (|k| k.camera = true, vec![frontend::CANCEL, action::CHANGECAMERA]),
            (|k| k.look_back = true, vec![action::CAMLOOKBACK]),
            (|k| k.pause = true, vec![frontend::START]),
        ];
        for (set, want) in cases {
            let mut s = input();
            set(&mut s.keys);
            assert_eq!(ids(&s.tick(1)), want);
            assert!(s.tick(2).is_empty(), "edges fire once while held");
        }
        let mut s = input();
        s.keys.look_back = true;
        s.tick(1);
        s.keys.look_back = false;
        assert_eq!(ids(&s.tick(2)), vec![action::CAMLOOKBACKRELEASE]);
    }

    #[test]
    fn menu_keys_auto_repeat_at_the_game_delays() {
        let mut s = input(); // delays 18 / 9 ticks
        s.keys.menu_down = true;
        let mut fired = Vec::new();
        for t in 1..=40 {
            if s.tick(t).iter().any(|m| m.action == frontend::DOWN) {
                fired.push(t);
            }
        }
        // Press at 1; then strictly after last+18 → 20; then every 9 → 30, 40.
        assert_eq!(fired, vec![1, 20, 30, 40]);
    }

    #[test]
    fn invert_flag_picks_type_6_or_7() {
        let t = "h\r\nGAMEACTION_CAMROTATEY\tALY\tN\r\nGAMEACTION_CAMROTATEYINV\tALY\tI\r\nEND\r\n";
        let d = ControlDef::parse(t.as_bytes()).unwrap();
        let mut s = ShippedDrivingInput::new(&[&d], repeat_delays(60)).unwrap();
        // Stick up: LY byte 0 → ALY +1 (the Y negation [M 0x169288]).
        let ly = RawPadSlot { mode: 0, buttons: 0, rx: 127, ry: 127, lx: 127, ly: 0, pressures: [0; 12] };
        let v = poll_device(&ly);
        assert_eq!(v[value::ALY], 1.0);
        let run = |s: &mut ShippedDrivingInput, invert: bool| {
            let mut out = Vec::new();
            let vb = &mut s.bindings[value::ALY];
            evaluate_value(vb, v[value::ALY], 0.0, true, false, 0, &s.delays.clone(), invert, 0, &mut out);
            ids(&out)
        };
        assert_eq!(run(&mut s, false), vec![0x2d], "N (type 7) = CAMROTATEY while not inverted");
        assert_eq!(run(&mut s, true), vec![0x2e], "I (type 6) = CAMROTATEYINV while inverted");
    }

    #[test]
    fn unknown_value_name_is_an_error() {
        let d = ControlDef::parse(b"h\r\nGAMEACTION_GAS\tAButtonX\r\nEND\r\n").unwrap();
        assert!(binding_set(&[&d]).is_err());
    }

    #[test]
    fn car_action_filter() {
        assert!(is_car_action(action::STEER));
        assert!(is_car_action(action::HANDBRAKERELEASE));
        assert!(is_car_action(action::TOGGLESECONDARY));
        assert!(!is_car_action(action::TOGGLESECONDARYUP));
        assert!(!is_car_action(action::CAMLOOKBACK));
        assert!(!is_car_action(frontend::START));
    }
}
