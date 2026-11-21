//! The driving missions' rules: the trigger/action/event machine and the mission manager that
//! decides a drive is won or lost (docs/systems/driving-missions.md).
//!
//! What runs a driving mission in DRIVING.ELF (all [M], EE addresses; [D] = disc data):
//! - **Actions.** A `Map` `Trgr` trigger or a `Rule` entry holds an `el` action list
//!   (`nf_formats::trackstart`). Running a list calls, per action, the creator
//!   `DAT_0036fda0[id]` (`FUN_001757b8`) with the action's argument block [M 0x22c890 /
//!   0x2095f8]. A creator bump-allocates an event in the frame's event buffer (`FUN_00175730`,
//!   `DAT_0032e6fc`) and runs its constructor; some constructors already change state (the timer
//!   events, `EChangeStage`).
//! - **Events.** `FUN_00175690` drains the buffer in creation order, calling each event's
//!   virtual destructor with flag 3 — the destructor *is* the handler (vtable `+0xc`, e.g.
//!   `EStartMission` 0x15c738) [M]. Events queued while draining run in the same drain.
//! - **The frame.** The sim-step event `FUN_00159d10` runs the cars, the AI, then the mission
//!   manager `FUN_00206b88` (whose `FUN_00206340` evaluates the rules while the mission state is
//!   1), the HUD, then the trigger walk `FUN_0022f0c0` [M 0x159d10:9-27]. Everything they queue
//!   runs after it in the same drain.
//! - **Triggers.** `FUN_0022d240` tests each car against the triggers near it: the trigger must
//!   have flag bit 0, not bit 13, bit 10 only while the manager's live word is set, and the car
//!   kind's bit (kind 1, the player, = bit 2); then the volume test `FUN_0022eb28`; a hit fires the
//!   trigger (`FUN_0022c890`), which runs its list and clears bit 0 when bit 1 (once) is set [M].
//! - **Rules.** `FUN_00205220` buckets the `Rule` entries: stage < 0 → the default list (every
//!   frame), else bucket `stage` [M]. `FUN_00206340` runs the default list, then the current
//!   stage, then sets current = next [M]. A rule fires when active and `test() != invert`, runs
//!   its list, and goes inactive when its once bit is set [M 0x2096d0].
//! - **The end.** `EPlayerWin` (state 3, mailbox 0x100978 = 0) and `EPlayerLose` (state 4,
//!   mailbox 1, or 4 with no health left) stop the live triggers; `EEndMission` wins and ends at
//!   once [M 0x1551b8 / 0x154cb0 / 0x14c260; 0x205b08 sets 3, 0x205bf0 sets 4].
//!
//! Inputs the port does not compute itself (the caller fills [`Facts`]): the player car's
//! position/velocity/radius, the AI elements' health/speed/position, the weapon counts the ammo
//! rule reads, the AI statistics block, the property flags. Only the player car is walked
//! against the triggers; the walks for kind-2/other cars (`FUN_0022d240` for them), the object
//! classes (`FUN_0022dae8`, `FUN_0022d5a8`) and the projectile walk (`FUN_0022def0`) are not
//! ported (their triggers never fire here).

use std::collections::HashMap;

use crate::driving::script_effects;

pub use crate::driving::script_effects::{Effect, Ref};
use nf_formats::trackstart::{
    MapScript, Trigger, TriggerAction, TRIGGER_BY_KIND2, TRIGGER_BY_OTHER, TRIGGER_BY_PLAYER, TRIGGER_ENABLED, TRIGGER_NEEDS_LIVE, TRIGGER_OBJECT_CLASS,
    TRIGGER_ONCE, TRIGGER_ONE_WAY, TRIGGER_SUPPRESSED,
};

/// Timer ticks per second, `DAT_00339f4c`. The sim step advances the game clock by
/// 0.016666668 s [M 0x159d10:28]; that the timer word is 60 is inferred from it (the writer of
/// `DAT_00339f4c` is not in the decompile).
pub const TICKS_PER_SECOND: i32 = 60;
/// The mission manager's timers: `mgr+0x150 + i*0x18`, ten of them [M 0x2057f0: `iVar7 = 9 .. -1`].
pub const TIMER_COUNT: usize = 10;
/// The programmer-event flags `mgr+0x24c + i*4` and the counters `mgr+0x3e4 + i*4`: 0x40 each
/// [M 0x2057f0, 0x206ad0 `param_2 < 0x40`].
pub const EVENT_COUNT: usize = 0x40;
/// `FUN_00208578` refuses an 8th objective [M: `uStack_70 < 7`, string 0x3976d8].
pub const MAX_OBJECTIVES: usize = 7;

/// One action id of the creator table at 0x36fda0: the event class (GCC typeinfo name), the
/// creator and the handler (the event vtable's `+0xc`) [M, read from the vtables].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionInfo {
    pub id: u32,
    pub name: &'static str,
    pub creator: u32,
    pub handler: u32,
}

/// Every action id the eight tracks use [D: every `el` list of each track's `Map` window (the
/// trigger and rule lists plus lists other records, the AI elements, point at) and of its prop
/// articles (the keyframe lists, `nf_formats::trackstart::prop_scripts`)], with its class and
/// functions [M: `DAT_0036fda0[id]` → the constructor's vtable → `+4` typeinfo, `+0xc`
/// handler].
pub const ACTIONS: &[ActionInfo] = &[
    ActionInfo { id: 0x9, name: "EPlayEffect", creator: 0x154308, handler: 0x153f80 },
    ActionInfo { id: 0xa, name: "EFireWeapon", creator: 0x14ed50, handler: 0x14ca60 },
    ActionInfo { id: 0xb, name: "EHideInstance", creator: 0x14f1c0, handler: 0x14f168 },
    ActionInfo { id: 0xc, name: "EShowInstance", creator: 0x159ae0, handler: 0x159a88 },
    ActionInfo { id: 0x13, name: "EEndMission", creator: 0x14c328, handler: 0x14c260 },
    ActionInfo { id: 0x14, name: "EStreamEvent", creator: 0x15d248, handler: 0x15d000 },
    ActionInfo { id: 0x16, name: "EStartMission", creator: 0x15c920, handler: 0x15c738 },
    ActionInfo { id: 0x17, name: "EHideInstanceStatic", creator: 0x14f2c8, handler: 0x14f218 },
    ActionInfo { id: 0x18, name: "EShowInstanceStatic", creator: 0x159be8, handler: 0x159b38 },
    ActionInfo { id: 0x19, name: "EPowerUp", creator: 0x1555d0, handler: 0x1554c0 },
    ActionInfo { id: 0x1a, name: "ESpawnSimplePhysics", creator: 0x15bb58, handler: 0x15bae0 },
    ActionInfo { id: 0x1b, name: "ESpawnSmackable", creator: 0x15bfb0, handler: 0x15bbf8 },
    ActionInfo { id: 0x1c, name: "ESpawnTraffic", creator: 0x15c3f0, handler: 0x15c080 },
    ActionInfo { id: 0x20, name: "EKillObject", creator: 0x150a98, handler: 0x1509c8 },
    ActionInfo { id: 0x21, name: "ESpawnExplosion", creator: 0x15ae90, handler: 0x15a098 },
    ActionInfo { id: 0x23, name: "EWakeupSmackable", creator: 0x15e4b0, handler: 0x15de48 },
    ActionInfo { id: 0x24, name: "EPlaySystemAnim", creator: 0x154bd0, handler: 0x154ad8 },
    ActionInfo { id: 0x28, name: "ECallStage", creator: 0x148db8, handler: 0x148d50 },
    ActionInfo { id: 0x29, name: "EChangeStage", creator: 0x1498a0, handler: 0x149870 },
    ActionInfo { id: 0x2a, name: "EControlToCPU", creator: 0x14acd0, handler: 0x14ac38 },
    ActionInfo { id: 0x2b, name: "EControlToPlayer", creator: 0x14ae10, handler: 0x14ad18 },
    ActionInfo { id: 0x2c, name: "EDisplayText", creator: 0x14b968, handler: 0x14b8f8 },
    ActionInfo { id: 0x2d, name: "EPlayerLose", creator: 0x155078, handler: 0x154cb0 },
    ActionInfo { id: 0x2e, name: "EPlayerWin", creator: 0x155418, handler: 0x1551b8 },
    ActionInfo { id: 0x2f, name: "ESetTimer", creator: 0x159270, handler: 0x1591f8 },
    ActionInfo { id: 0x30, name: "EStartTimer", creator: 0x15cb10, handler: 0x15cae0 },
    ActionInfo { id: 0x31, name: "EStopTimer", creator: 0x15cf90, handler: 0x15cf60 },
    ActionInfo { id: 0x35, name: "EDisableTrigger", creator: 0x14b7e8, handler: 0x14b790 },
    ActionInfo { id: 0x36, name: "EEnableTrigger", creator: 0x14bd30, handler: 0x14bcd8 },
    ActionInfo { id: 0x38, name: "EPowerUpAmmo", creator: 0x1557e8, handler: 0x1556b8 },
    ActionInfo { id: 0x3c, name: "EPowerUpMultiDamage", creator: 0x155ac8, handler: 0x155a70 },
    ActionInfo { id: 0x3f, name: "EPlayCameraAnim", creator: 0x153910, handler: 0x153700 },
    ActionInfo { id: 0x41, name: "EAICommand", creator: 0x146900, handler: 0x145ff8 },
    ActionInfo { id: 0x42, name: "EActivateAIElement", creator: 0x1478f0, handler: 0x1476b8 },
    ActionInfo { id: 0x43, name: "EDeactivateAIElement", creator: 0x14b5b0, handler: 0x14b510 },
    ActionInfo { id: 0x44, name: "EEndCameraAnim", creator: 0x14bfe8, handler: 0x14bdb8 },
    ActionInfo { id: 0x45, name: "ESetObjective", creator: 0x158728, handler: 0x158648 },
    ActionInfo { id: 0x4c, name: "ESetAutoDriveCamOff", creator: 0x1574f8, handler: 0x157460 },
    ActionInfo { id: 0x4d, name: "ESetAutoDriveCamOn", creator: 0x1577d8, handler: 0x1576e8 },
    ActionInfo { id: 0x4e, name: "ESetVideo", creator: 0x1595a0, handler: 0x1592d8 },
    ActionInfo { id: 0x61, name: "EPlayCarCameraAnim", creator: 0x153ee0, handler: 0x153c48 },
    ActionInfo { id: 0x67, name: "ELetterBoxOff", creator: 0x150d70, handler: 0x150ce8 },
    ActionInfo { id: 0x68, name: "ELetterBoxOn", creator: 0x150e58, handler: 0x150dc8 },
    ActionInfo { id: 0x6c, name: "EAutoDriveSpeedChange", creator: 0x148378, handler: 0x1482f8 },
    ActionInfo { id: 0x70, name: "ESetSimRate", creator: 0x158d90, handler: 0x158d28 },
    ActionInfo { id: 0x74, name: "EPathSetDesiredThrottleStatic", creator: 0x152d28, handler: 0x152cb0 },
    ActionInfo { id: 0x78, name: "EChangeMaxTraffic", creator: 0x1497e8, handler: 0x1497b8 },
    ActionInfo { id: 0x79, name: "EKickObject", creator: 0x1506d8, handler: 0x150390 },
    ActionInfo { id: 0x7d, name: "ETwoWheelsOff", creator: 0x15dbf0, handler: 0x15db78 },
    ActionInfo { id: 0x80, name: "EStartEffect", creator: 0x15c650, handler: 0x15c520 },
    ActionInfo { id: 0x81, name: "EStopEffect", creator: 0x15ccb0, handler: 0x15cb80 },
    ActionInfo { id: 0x82, name: "ESetCollisionGeometry", creator: 0x157c70, handler: 0x157be0 },
    ActionInfo { id: 0x83, name: "EAddScoreAction", creator: 0x147c88, handler: 0x147b80 },
    ActionInfo { id: 0x86, name: "EPlaySound", creator: 0x1546d0, handler: 0x154658 },
    ActionInfo { id: 0x8a, name: "EPowerUpShield", creator: 0x155be0, handler: 0x155b20 },
    ActionInfo { id: 0x8c, name: "EResetPlayerCarPos", creator: 0x156638, handler: 0x156460 },
    ActionInfo { id: 0x90, name: "EAIElementFireOff", creator: 0x146ad0, handler: 0x1469f8 },
    ActionInfo { id: 0x91, name: "EAIElementFireOn", creator: 0x146c28, handler: 0x146b48 },
    ActionInfo { id: 0x94, name: "ESpawnSentry", creator: 0x15ba00, handler: 0x15b900 },
    ActionInfo { id: 0x95, name: "EDisablePowerUp", creator: 0x14b6e0, handler: 0x14b630 },
    ActionInfo { id: 0x96, name: "EEnablePowerUp", creator: 0x14bc28, handler: 0x14bb58 },
    ActionInfo { id: 0x99, name: "EPathSetTimeStatic", creator: 0x1533d0, handler: 0x153340 },
    ActionInfo { id: 0x9a, name: "ESpawnExplosionStatic", creator: 0x15b188, handler: 0x15b010 },
    ActionInfo { id: 0x9f, name: "EAIElementSetAccuracy", creator: 0x146d70, handler: 0x146ca8 },
    ActionInfo { id: 0xa1, name: "EKillAIElement", creator: 0x150948, handler: 0x150748 },
    ActionInfo { id: 0xa2, name: "EDamagePlayer", creator: 0x14b4b8, handler: 0x14b3e0 },
    ActionInfo { id: 0xa4, name: "EPlaySoundPos", creator: 0x1549f0, handler: 0x154798 },
    ActionInfo { id: 0xa6, name: "EStopSoundPos", creator: 0x15cdf0, handler: 0x15cd90 },
    ActionInfo { id: 0xab, name: "EFireTriggerSpeedCondition", creator: 0x14c9c0, handler: 0x14c8c8 },
    ActionInfo { id: 0xad, name: "ESuppressEffect", creator: 0x15d450, handler: 0x15d320 },
    ActionInfo { id: 0xb4, name: "EDropMine", creator: 0x14baa0, handler: 0x14b9c8 },
    ActionInfo { id: 0xb5, name: "ESentryMuzzleFlash", creator: 0x1570a0, handler: 0x156fb0 },
    ActionInfo { id: 0xb8, name: "E007Logo", creator: 0x145f98, handler: 0x145f00 },
    ActionInfo { id: 0xb9, name: "EIncMagicCounter", creator: 0x14fcb8, handler: 0x14fc50 },
    ActionInfo { id: 0xbc, name: "ESetVisualDamage", creator: 0x1597b0, handler: 0x159660 },
    ActionInfo { id: 0xbe, name: "ESetJumpGravity", creator: 0x158098, handler: 0x158050 },
    ActionInfo { id: 0xc0, name: "ESetSpecialWeapon", creator: 0x158e60, handler: 0x158df0 },
    ActionInfo { id: 0xc1, name: "EButtonMsgOff", creator: 0x148b40, handler: 0x148ad8 },
    ActionInfo { id: 0xc2, name: "EButtonMsgOn", creator: 0x148c10, handler: 0x148b98 },
    ActionInfo { id: 0xc4, name: "ENuclearBlast", creator: 0x1514b0, handler: 0x151408 },
    ActionInfo { id: 0xc7, name: "EAIElementSetVehicleParams", creator: 0x147160, handler: 0x1470e8 },
    ActionInfo { id: 0xc8, name: "ESetMagicCounter", creator: 0x158168, handler: 0x1580f8 },
    ActionInfo { id: 0xca, name: "EControlToSpline", creator: 0x14af70, handler: 0x14ae60 },
    ActionInfo { id: 0xcb, name: "EFireRandomTrigger", creator: 0x14c7b8, handler: 0x14c688 },
    ActionInfo { id: 0xcc, name: "EPlayCameraSpline", creator: 0x153bb0, handler: 0x1539a8 },
    ActionInfo { id: 0xcf, name: "EChangeCarCameraView", creator: 0x149728, handler: 0x149528 },
    ActionInfo { id: 0xd1, name: "ESetNextAISpline", creator: 0x158450, handler: 0x158370 },
    ActionInfo { id: 0xd5, name: "EClearProgrammerEvent", creator: 0x149cf0, handler: 0x149c88 },
    ActionInfo { id: 0xd7, name: "EPathChangeThrottleStatic", creator: 0x151e68, handler: 0x151dc0 },
    ActionInfo { id: 0xd8, name: "EPathMultiplyThrottleStatic", creator: 0x152130, handler: 0x152078 },
    ActionInfo { id: 0xd9, name: "EPathChangeThrottleDynamic", creator: 0x151d48, handler: 0x151c88 },
    ActionInfo { id: 0xdb, name: "EEndCarStop", creator: 0x14c0c8, handler: 0x14c048 },
    ActionInfo { id: 0xdc, name: "EForceCarStop", creator: 0x14ee68, handler: 0x14ede0 },
    ActionInfo { id: 0xe1, name: "EObjectiveAdd", creator: 0x1515b0, handler: 0x151538 },
    ActionInfo { id: 0xe4, name: "EObjectivePass", creator: 0x151960, handler: 0x1518d0 },
    ActionInfo { id: 0xe5, name: "EObjectiveSetCurrent", creator: 0x151a50, handler: 0x1519c0 },
    ActionInfo { id: 0xe7, name: "EObjectiveDisplayText", creator: 0x151690, handler: 0x151618 },
    ActionInfo { id: 0xe8, name: "ESetRecSpeedOffset", creator: 0x1588c0, handler: 0x1587f8 },
    ActionInfo { id: 0xe9, name: "ESetAutoDriveCamOffMovie", creator: 0x157698, handler: 0x157540 },
    ActionInfo { id: 0xea, name: "ESetAutoDriveCamOnMovie", creator: 0x157958, handler: 0x157838 },
    ActionInfo { id: 0xef, name: "EEndControlToSpline", creator: 0x14c1e8, handler: 0x14c110 },
    ActionInfo { id: 0xf0, name: "ECustomTransition", creator: 0x14b360, handler: 0x14b020 },
    ActionInfo { id: 0xf1, name: "ESetAudioMix", creator: 0x1573e0, handler: 0x157308 },
    ActionInfo { id: 0xf2, name: "ECameraLockOn", creator: 0x149040, handler: 0x148e30 },
    ActionInfo { id: 0xf4, name: "ERevertAudioMix", creator: 0x156ad0, handler: 0x156a58 },
    ActionInfo { id: 0xf5, name: "ESetAvoidZone", creator: 0x157a70, handler: 0x1579b8 },
    ActionInfo { id: 0xf6, name: "ECameraZoom", creator: 0x1494c0, handler: 0x149338 },
    ActionInfo { id: 0xf8, name: "EGadgetOff", creator: 0x14f050, handler: 0x14efe8 },
    ActionInfo { id: 0xf9, name: "EGadgetOn", creator: 0x14f110, handler: 0x14f0a0 },
    ActionInfo { id: 0xfa, name: "EPlayGiottoEffect", creator: 0x1544c0, handler: 0x1543e8 },
    ActionInfo { id: 0xfb, name: "ECameraShake", creator: 0x1491e0, handler: 0x149118 },
    ActionInfo { id: 0xfc, name: "ESwitchEffectOff", creator: 0x15d5d0, handler: 0x15d538 },
    ActionInfo { id: 0xfd, name: "ESwitchEffectOn", creator: 0x15d6e8, handler: 0x15d650 },
    ActionInfo { id: 0xff, name: "ESetCreakLevel", creator: 0x157e68, handler: 0x157cf8 },
    ActionInfo { id: 0x101, name: "ESetCullDistanceFactor", creator: 0x157f20, handler: 0x157ec0 },
    ActionInfo { id: 0x102, name: "ESetNextPath", creator: 0x158588, handler: 0x158508 },
    ActionInfo { id: 0x103, name: "ESetSubtitle", creator: 0x159060, handler: 0x158fc0 },
    ActionInfo { id: 0x104, name: "ETimerDrawOff", creator: 0x15da60, handler: 0x15d9f8 },
    ActionInfo { id: 0x105, name: "ETimerDrawOn", creator: 0x15db20, handler: 0x15dab8 },
    ActionInfo { id: 0x108, name: "EAIElementSetWakeRange", creator: 0x147268, handler: 0x1471e0 },
    ActionInfo { id: 0x109, name: "ETargetBeaconOnOff", creator: 0x15d980, handler: 0x15d888 },
    ActionInfo { id: 0x10a, name: "EResetAudioMix", creator: 0x156338, handler: 0x1562c8 },
    ActionInfo { id: 0x10b, name: "EProfileMissionSection", creator: 0x155e28, handler: 0x155dc0 },
    ActionInfo { id: 0x10c, name: "EPlayerSetImmunity", creator: 0x155158, handler: 0x1550d8 },
    ActionInfo { id: 0x10d, name: "EUnloadWeapon", creator: 0x15dd60, handler: 0x15dcf8 },
    ActionInfo { id: 0x10e, name: "EPathSFX", creator: 0x152360, handler: 0x152200 },
    ActionInfo { id: 0x110, name: "ESwitchStingerChannel", creator: 0x15d808, handler: 0x15d760 },
    ActionInfo { id: 0x111, name: "EReleaseStream", creator: 0x156170, handler: 0x1560d8 },
    ActionInfo { id: 0x113, name: "EAwardHitToPlayer", creator: 0x1485a8, handler: 0x148540 },
    ActionInfo { id: 0x114, name: "EAwardKillToPlayer", creator: 0x148678, handler: 0x148600 },
    ActionInfo { id: 0x116, name: "EBashCarAlongObjectAxis", creator: 0x1489f0, handler: 0x1486d8 },
    ActionInfo { id: 0x117, name: "ESetReflectiveObject", creator: 0x158b08, handler: 0x1589f0 },
    ActionInfo { id: 0x119, name: "ESetRecordedGameTime", creator: 0x1589a8, handler: 0x158938 },
];

/// The action ids whose effect on the mission rules is ported here; every other id runs as an
/// [`Output::Effect`] (camera, audio, AI, traffic, props, HUD look).
pub const PORTED: &[u32] = &[
    0x13, 0x16, 0x28, 0x29, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x35, 0x36, 0x45, 0x83, 0x8c, 0xab, 0xc8, 0xcb, 0xd5, 0xe1,
    0xe4, 0xe5, 0xe7, 0xb9, 0x104, 0x105, 0x10c,
];

/// The table entry for an id.
pub fn action_info(id: u32) -> Option<&'static ActionInfo> { ACTIONS.iter().find(|a| a.id == id) }

/// The mission manager state `mgr+0x34c` [M: 0x205a38 sets 0, 0x205ac8 sets 1 and 0x205ae8 sets 2
/// unless the state is 3 or 4, 0x205b08 sets 3, 0x205bf0 sets 4].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveState {
    Loading = 0,
    Playing = 1,
    Paused = 2,
    Won = 3,
    Lost = 4,
}

/// One AI element's facts the rules read (`FUN_0012c640(DAT_00327c30, id)`): health = the
/// element's car `+0x4c` [M 0x209820], speed = element `+0x3c` [M 0x209758], position = element
/// `+0x30` [M 0x2098a0].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementFacts {
    pub health: f32,
    pub speed: f32,
    pub pos: [f32; 3],
    /// `SRulePlayerDir`'s angle in degrees (`FUN_0020ac70`: the angle between the element's car
    /// heading and the direction to the player, signed by the cross product). The port takes it
    /// as an input; how the game builds it is in the doc, not ported.
    pub bearing_deg: Option<f32>,
    /// Names `FUN_0012b138` answers true for (`SRuleProperty` with an element selector).
    pub props: Vec<String>,
}

/// Another car in the vehicle manager's car slots (`FUN_001ff088(0x339190, i)`, walked when its
/// `+0x6e` is 2 [M 0x22f0c0]): kind = `+0x6d` (2 picks trigger bit 3, anything but 1 and 2 bit 6)
/// [M 0x22d240].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarFacts {
    pub kind: u8,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub radius: f32,
    /// The car's box (`None`: [`CarBox::from_radius`]).
    pub bbox: Option<CarBox>,
}

/// A car's oriented box as `FUN_0022eb28` builds it for a box trigger: the first three rows of
/// the car body's matrix (`*(car + 0x5c)` `+0x00..+0x2c`) as the axes and the half extents at
/// `body + 0x4c0..0x4c8` (each at least 0.001; `car + 0x7c` is their length); the centre is the
/// car position `+0x10` [M 0x22eb28 -> `FUN_001f5cf0(obb, body, car + 0x10, body + 0x4c0)`;
/// 0x1f6e98 writes the extents and the radius]. That the rows are the car's side / up /
/// forward axes in world space follows `FUN_00210540` (row 2 = the direction, row 1 from up)
/// [M 0x1fee68].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarBox {
    pub axes: [[f32; 3]; 3],
    pub half: [f32; 3],
}

impl CarBox {
    /// An upright box turned `yaw` (radians about +y, 0 = facing +z) with these half extents.
    pub fn upright(yaw: f32, half: [f32; 3]) -> CarBox {
        let (s, c) = yaw.sin_cos();
        CarBox { axes: [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]], half }
    }
    /// The port's stand-in when the caller gives no box: axis-aligned, half extents r / sqrt(3)
    /// each (their length is the radius, as in the game; the shape is not the game's).
    pub fn from_radius(r: f32) -> CarBox {
        let h = r / 3f32.sqrt();
        CarBox { axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], half: [h, h, h] }
    }
}

/// What the rules and the trigger walk read each frame.
#[derive(Clone, Debug, Default)]
pub struct Facts {
    /// The other cars the trigger walk tests (AI traffic, rivals), in slot order.
    pub cars: Vec<CarFacts>,
    /// The player car `+0x10` position.
    pub player_pos: [f32; 3],
    /// The player car `+0x20` velocity (the one-way test and the speed rule's `|v|`).
    pub player_vel: [f32; 3],
    /// The player car `+0x7c` bounding radius (|half extents|, [M 0x1f6e98]).
    pub player_radius: f32,
    /// The player car's oriented box for box triggers ([`CarBox`]); `None` = the port's stand-in
    /// [`CarBox::from_radius`].
    pub player_box: Option<CarBox>,
    /// The AI elements, keyed by the selector word the rules carry (an `sr` reference whose
    /// path names the element, e.g. `CARP::<<Map>>::{AIEl}::2304`; [`MapScript::ref_path`]).
    pub elements: HashMap<u32, ElementFacts>,
    /// Player properties `SRuleProperty` reads: "NOBOOST", "ONEEMP", "2WHEELS" [M 0x209aa0].
    pub player_props: Vec<String>,
    /// The weapon display entries' `+0x18` counts `SRuleAmmo` reads (`DAT_0033a530+0x10`,
    /// stride 0x54) [M 0x209ec0].
    pub ammo: Vec<i32>,
    /// The AI statistics floats at `DAT_00327c30+0x1c` (selector kind 3) [M 0x12c440].
    pub ai_stats: [f32; 7],
    /// `mgr+0x3b8`: 0, 1 or 2 (`SRuleDifficulty`) [M 0x20aac0].
    pub difficulty: i32,
}

impl Facts {
    fn speed(&self) -> f32 {
        let v = self.player_vel;
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }
}

/// What the script asks the rest of the game to do (or reports). Ported events report their
/// visible part here too; unported ones are [`Output::Effect`].
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    /// `EStartMission`: put the player car at the trigger (x/z, ground + 1), facing its `+0x30`,
    /// velocity = dir × factor × car `+0x4c` [M 0x15c738].
    StartMission { trigger: Option<usize>, speed_factor: f32 },
    /// `EResetPlayerCarPos`: the same placement from the trigger its argument names [M 0x156460].
    ResetPlayerCar { trigger: Option<usize>, speed_factor: f32 },
    /// `EPlayerWin` ran (state 3) [M 0x1551b8].
    Won,
    /// `EPlayerLose` ran (state 4); `dead` = the health was gone (mailbox 4) [M 0x154cb0].
    Lost { dead: bool },
    /// `EEndMission` ran [M 0x14c260].
    EndMission,
    /// `ESetTimer` with its show flag: the HUD shows timer `i` [M 0x1591f8 → `FUN_0023e568`].
    ShowTimer(usize),
    /// `ETimerDrawOff` / `ETimerDrawOn` [M 0x15d9f8 / 0x15dab8].
    HideTimer(u32),
    DrawTimer(u32),
    /// Timer 1 crossed a whole second while running: "SFX_ReverseBeep" [M 0x209c88].
    TimerBeep(i32),
    /// `EObjectiveDisplayText` / the HUD flag of `EObjectivePass` / `EObjectiveSetCurrent`:
    /// `FUN_00245910(hud, mode, id, ..)` with mode 2 = passed, 1 = show [M 0x151618 / 0x1518d0].
    ObjectiveHud { mode: u32, id: u32 },
    /// `EPlayerSetImmunity` [M 0x1550d8: player vtable `+0x26c`].
    Immunity(bool),
    /// An event whose effect is not part of the mission rules, decoded for the system that owns
    /// it (camera, audio, movie, AI, traffic, props, ...; [`Effect`]); `trigger` = the trigger
    /// whose list queued it (`DAT_0032e738`), `None` for rules and prop keyframes.
    Effect { effect: Effect, trigger: Option<usize> },
}

/// One objective (`FUN_00208f50`, 0x14 bytes): id, two words, points, state [M].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    pub id: u32,
    /// `+4` (the `EObjectiveAdd` second word: the text id on disc [D]).
    pub text: u32,
    /// `+8`: added to score 0 when passed [M 0x2088b8 → 0x207ea8].
    pub points: i32,
    pub extra: u32,
    /// `+0x10`: 0 incomplete, 1 current, 2 passed, 3 failed [M 0x2089b8 / 0x208968 / 0x2088b8 /
    /// 0x208918].
    pub state: u32,
}

/// One mission timer (0x18 bytes): running, beep, start tick, value, rate, last second [M 0x209bb0].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timer {
    pub running: bool,
    pub beep: bool,
    pub start: i32,
    pub value: i32,
    pub rate: i32,
    pub last_second: i32,
}

impl Timer {
    /// `FUN_00209c88`: the value now (ticks), plus the beep side effect [M].
    fn current(&mut self, now: i32, out: &mut Vec<Output>) -> i32 {
        if !self.running {
            return self.value;
        }
        let v = self.value.wrapping_add(self.rate.wrapping_mul(now - self.start));
        if self.beep {
            let s = v / TICKS_PER_SECOND;
            if s != self.last_second && s > 0 {
                out.push(Output::TimerBeep(s));
                self.last_second = s;
            }
        }
        v
    }
    /// The value now without side effects.
    pub fn peek(&self, now: i32) -> i32 {
        if self.running { self.value.wrapping_add(self.rate.wrapping_mul(now - self.start)) } else { self.value }
    }
}

/// The driving RNG (`DAT_00333e60` × `DAT_00333e64`), shared with traffic and AI in the game;
/// `FUN_001c08c0(seed)` sets the multiplier 123456789 and state 0x3ade68b1, then steps
/// `seed % 500` times [M]. Where the game seeds it and what else draws first is not traced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriveRng {
    pub state: u32,
    pub mul: u32,
}

impl DriveRng {
    pub fn seeded(seed: i32) -> DriveRng {
        let mut r = DriveRng { state: 0x3ade_68b1, mul: 0x075b_cd15 };
        for _ in 0..seed.rem_euclid(500) {
            r.state = r.state.wrapping_mul(r.mul) & 0xffff;
        }
        r
    }
    /// One draw as `EFireRandomTrigger` takes it: the full product, state = its low 16 bits
    /// [M 0x14c688].
    pub fn draw(&mut self) -> u32 {
        let p = self.state.wrapping_mul(self.mul);
        self.state = p & 0xffff;
        p
    }
}

/// A rule's selector (`FUN_00209738`): kind 0 = the player, 4 = AI element `id`, 3 = the AI
/// statistics block (read by the rule types themselves); 1, 2 and others read 0 [M].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selector {
    pub kind: i32,
    pub id: u32,
}

impl Selector {
    fn new(kind: u32, id: u32) -> Selector { Selector { kind: kind as i32, id: if kind == 4 { id } else { 0 } } }
}

/// The 15 rule types (`FUN_00209390`'s switch; names from each descriptor's typeinfo [M]).
#[derive(Clone, Debug, PartialEq)]
pub enum RuleTest {
    /// 0 `SRuleAlways`: true [M 0x209df0].
    Always,
    /// 1 `SRuleAmmo`: weapon slot count crossing [M 0x209ec0].
    Ammo { slot: u32, up: bool, thr: i32, prev: i32 },
    /// 2 `SRuleDamage`: health lost this frame above `thr` [M 0x209ff0].
    Damage { sel: Selector, thr: i32, prev: i32 },
    /// 3 `SRuleDeath`: health == 0 [M 0x20a108].
    Death { sel: Selector },
    /// 4 `SRuleHealth`: health crossing [M 0x20a1f8].
    Health { sel: Selector, up: bool, thr: i32, prev: i32 },
    /// 5 `SRulePIP`: true [M 0x20a378].
    Pip,
    /// 6 `SRuleProg`: programmer event `id` set [M 0x20a420 → 0x208558].
    Prog { id: u32 },
    /// 7 `SRuleRange`: distance to the player below / above `thr` [M 0x20a5f0].
    Range { sel: Selector, up: bool, thr: i32 },
    /// 8 `SRuleSpeed`: speed crossing [M 0x20a788].
    Speed { sel: Selector, up: bool, thr: i32, prev: i32 },
    /// 9 `SRuleTimer`: a running timer's seconds crossing [M 0x20a940].
    Timer { timer: u32, up: bool, thr: f32, prev: f32 },
    /// 10 `SRuleProperty`: a named property [M 0x20a500 → 0x209aa0].
    Property { sel: Selector, name: String },
    /// 11 `SRuleCollision`: false [M 0x20aec0].
    Collision,
    /// 12 `SRulePlayerDir`: an element's bearing inside [lo, hi] degrees [M 0x20ac70 / 0x20abe0].
    PlayerDir { sel: Selector, lo: i32, hi: i32 },
    /// 13 `SRuleDifficulty`: difficulty bit mask [M 0x20aac0].
    Difficulty { mask: u32 },
    /// 14 `SRuleProgCounter`: counter `id` compared by `op` [M 0x20af78].
    ProgCounter { id: u32, op: u32, value: i32 },
}

/// The crossing tests' shared shape: with `up` false it fires the frame the value drops below
/// `thr` after being at or above it; with `up` true the frame it rises above `thr` after being at
/// or below it; `prev` starts on the far side (thr ∓ 1) [M 0x209df8 / 0x209ec0, the same in 0x20a1f8
/// / 0x20a788].
fn crossing(v: i32, up: bool, thr: i32, prev: &mut i32) -> bool {
    let fire = if !up { v < thr && *prev >= thr } else { v > thr && *prev <= thr };
    *prev = v;
    fire
}

impl RuleTest {
    /// Builds the runtime test from a 0x20-byte entry (the per-type constructors) [M 0x209390].
    pub fn from_entry(raw: &[u8; 0x20]) -> Option<RuleTest> {
        let w = |o: usize| u32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        let i = |o: usize| w(o) as i32;
        let side = |up: bool, thr: i32| if up { thr.wrapping_add(1) } else { thr.wrapping_sub(1) };
        Some(match w(0) {
            0 => RuleTest::Always,
            1 => {
                let (up, thr) = (w(0x14) != 0, i(0x18));
                RuleTest::Ammo { slot: w(0x10), up, thr, prev: side(up, thr) }
            }
            2 => RuleTest::Damage { sel: Selector::new(w(0x10), w(0x14)), thr: i(0x18), prev: 0 },
            3 => RuleTest::Death { sel: Selector::new(w(0x10), w(0x14)) },
            4 => {
                let (up, thr) = (w(0x18) != 0, i(0x1c));
                RuleTest::Health { sel: Selector::new(w(0x10), w(0x14)), up, thr, prev: side(up, thr) }
            }
            5 => RuleTest::Pip,
            6 => RuleTest::Prog { id: w(0x10) },
            7 => RuleTest::Range { sel: Selector::new(w(0x10), w(0x14)), up: w(0x18) != 0, thr: i(0x1c) },
            8 => {
                let (up, thr) = (w(0x18) != 0, i(0x1c));
                RuleTest::Speed { sel: Selector::new(w(0x10), w(0x14)), up, thr, prev: side(up, thr) }
            }
            9 => {
                let (up, thr) = (w(0x14) != 0, f32::from_bits(w(0x18)));
                RuleTest::Timer { timer: w(0x10), up, thr, prev: if up { thr + 1.0 } else { thr - 1.0 } }
            }
            10 => {
                let n = &raw[0x18..0x20];
                let end = n.iter().position(|&b| b == 0).unwrap_or(8);
                RuleTest::Property { sel: Selector::new(w(0x10), w(0x14)), name: String::from_utf8_lossy(&n[..end]).into_owned() }
            }
            11 => RuleTest::Collision,
            12 => RuleTest::PlayerDir { sel: Selector::new(w(0x10), w(0x14)), lo: i(0x18), hi: i(0x1c) },
            13 => RuleTest::Difficulty { mask: w(0x10) },
            14 => RuleTest::ProgCounter { id: w(0x10), op: w(0x14), value: i(0x18) },
            _ => return None, // the switch default: no object [M 0x209390]
        })
    }

    /// The type number (the entry's `+0`).
    pub fn kind(&self) -> u32 {
        match self {
            RuleTest::Always => 0,
            RuleTest::Ammo { .. } => 1,
            RuleTest::Damage { .. } => 2,
            RuleTest::Death { .. } => 3,
            RuleTest::Health { .. } => 4,
            RuleTest::Pip => 5,
            RuleTest::Prog { .. } => 6,
            RuleTest::Range { .. } => 7,
            RuleTest::Speed { .. } => 8,
            RuleTest::Timer { .. } => 9,
            RuleTest::Property { .. } => 10,
            RuleTest::Collision => 11,
            RuleTest::PlayerDir { .. } => 12,
            RuleTest::Difficulty { .. } => 13,
            RuleTest::ProgCounter { .. } => 14,
        }
    }
}

/// One rule object [M 0x209318]: list, active (bit 0), once (bit 1), invert (bit 2), the test.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub index: usize,
    pub stage: i32,
    pub active: bool,
    pub once: bool,
    pub invert: bool,
    pub test: RuleTest,
    pub actions: Vec<TriggerAction>,
}

/// A queued event: the action, the trigger that fired it (`DAT_0032e738`, null for rules —
/// `FUN_002095f8` clears 0x32e708..0x32e76c), and the fire parameters (`DAT_0032e740/744`) [M].
#[derive(Clone, Debug)]
struct Event {
    action: TriggerAction,
    trigger: Option<usize>,
    mode: u32,
    car: i32,
}

/// The driving mission: the mission manager (`DAT_0033a024`) + the trigger manager
/// (`DAT_0033aa88`) + the event buffer.
#[derive(Clone, Debug)]
pub struct DriveScript {
    pub map: MapScript,
    /// Runtime trigger flags (`+0x12`), changed by enable/disable/once.
    pub trigger_flags: Vec<u16>,
    /// The trigger manager's `+8`: the flag-0x400 triggers fire only while set [M 0x22cd30].
    pub live: bool,
    pub rules: Vec<Rule>,
    /// `mgr+0xc` / `mgr+0x10`: the current and next stage [M 0x206340 / 0x206338].
    pub stage: i32,
    pub next_stage: i32,
    pub state: DriveState,
    /// The mailbox 0x100978 the drive hands back: 3 while running [M 0x1feda0], 0 won, 1 lost,
    /// 4 lost with no health [M 0x1551b8 / 0x154cb0 / 0x14c260]; DRIVING main turns a 3 left at
    /// exit into 1 [M 0x174bf0:291].
    pub outcome: i32,
    pub timers: [Timer; TIMER_COUNT],
    /// `mgr+0x24c`: the programmer events `SRuleProg` reads [M 0x208540 / 0x208568].
    pub events: [bool; EVENT_COUNT],
    /// `mgr+0x3e4` counters and `mgr+0x4e4` limits [M 0x2057f0 / 0x206990].
    pub counters: [i32; EVENT_COUNT],
    pub limits: [i32; EVENT_COUNT],
    pub objectives: Vec<Objective>,
    /// `mgr+0x648 + i*4`: score 0 (objective points) and 1 (`EAddScoreAction`) [M 0x207ea8].
    pub score: [i32; 12],
    /// `mgr+0x5e4`: the count of positive score actions [M 0x207ea8].
    pub score_actions: i32,
    /// `ESetObjective`'s target: mgr `+0x600` mode (1/2/3), `+0x5f8` word, `+0x608` extra [M 0x158648].
    pub marker: Option<(u32, u32)>,
    pub marker_extra: u32,
    /// `mgr+0x378` / `+0x37c`: the player health and its maximum [M 0x206ec0].
    pub health: f32,
    pub health_max: f32,
    /// `FUN_00167d80()+4`: set by `EStartMission` [M 0x15c738].
    pub started: bool,
    /// `DAT_00339f44 = 2` from `EEndMission` [M 0x14c260].
    pub end_requested: bool,
    pub rng: DriveRng,
    /// The weapon `ESetSpecialWeapon` last gave the player (its word 1) [M 0x158df0].
    pub special_weapon: Option<i32>,
    /// `mgr+0x3c0`, the EMP-hit count `FUN_00206900` raises (capped by `mgr+0x3bc`, whose writer
    /// is not traced; the port leaves it uncapped) [M 0x206900].
    pub emp_hits: i32,
    /// The timer clock `DAT_00339f54` (ticks).
    pub now: i32,
    queue: Vec<Event>,
    /// Everything reported since the last [`DriveScript::take_output`].
    pub output: Vec<Output>,
}

impl DriveScript {
    /// "Init World" for the script: the rules (`FUN_00205220`, then state 1 via `FUN_00205ac8`
    /// [M 0x22faf8:127-129]), the trigger copy (`FUN_0022cbe0`: bit 9 clears bit 10), then
    /// `FUN_0022cd30`: live = 1 and every trigger holding an `EStartMission` fires at once (mode 1,
    /// car -1) [M]. The queued events run in the first drain ([`DriveScript::drain`]).
    pub fn new(map: MapScript) -> DriveScript {
        let mut rules = Vec::new();
        for r in &map.rules {
            let Some(test) = RuleTest::from_entry(&r.raw) else { continue };
            let flags = r.word(4);
            rules.push(Rule {
                index: r.index,
                stage: r.word(8) as i32,
                active: flags & 1 != 0,
                once: flags >> 1 & 1 != 0,
                invert: flags >> 2 & 1 != 0,
                test,
                actions: r.actions.clone(),
            });
        }
        let trigger_flags = map
            .triggers
            .iter()
            .map(|t| if t.flags & TRIGGER_OBJECT_CLASS != 0 { t.flags & !TRIGGER_NEEDS_LIVE } else { t.flags })
            .collect();
        // FUN_002057f0's counter limits [M].
        let mut limits = [-1i32; EVENT_COUNT];
        for (i, l) in limits.iter_mut().enumerate() {
            *l = match i {
                6 | 7 | 10 => 6,
                8 => 3,
                9 => 2,
                0x13 => 4,
                _ => -1,
            };
        }
        // FUN_00206ec0: MSet +0x280 start health, +0x27c maximum; 10000 each when +0x280 is 0 [M].
        // The difficulty and cheat scalers that follow are not ported.
        let mset_i = |o: usize| {
            map.mset.as_ref().and_then(|m| m.get(o..o + 4)).map_or(0, |b| i32::from_le_bytes(b.try_into().unwrap()))
        };
        let (health, health_max) =
            if mset_i(0x280) == 0 { (10_000.0, 10_000.0) } else { (mset_i(0x280) as f32, mset_i(0x27c) as f32) };
        let mut s = DriveScript {
            map,
            trigger_flags,
            live: true,
            rules,
            stage: 0,
            next_stage: 0,
            state: DriveState::Playing,
            outcome: 3,
            timers: [Timer::default(); TIMER_COUNT],
            events: [false; EVENT_COUNT],
            counters: [0; EVENT_COUNT],
            limits,
            objectives: Vec::new(),
            score: [0; 12],
            score_actions: 0,
            marker: None,
            marker_extra: 0,
            health,
            health_max,
            started: false,
            end_requested: false,
            rng: DriveRng::seeded(0),
            special_weapon: None,
            emp_hits: 0,
            now: 0,
            queue: Vec::new(),
            output: Vec::new(),
        };
        for i in 0..s.map.triggers.len() {
            if s.map.triggers[i].actions.iter().any(|a| a.id == 0x16) {
                s.fire(i, 1, -1);
            }
        }
        s
    }

    /// Takes the outputs gathered so far.
    pub fn take_output(&mut self) -> Vec<Output> { std::mem::take(&mut self.output) }

    pub fn won(&self) -> bool { self.state == DriveState::Won }
    pub fn lost(&self) -> bool { self.state == DriveState::Lost }

    /// One sim step (`FUN_00159d10`): the clock, the mission manager's rules (state 1 only), the
    /// player-car trigger walk, then the drain [M]. The timer clock advancing once per step is
    /// inferred (its writer is not in the decompile).
    pub fn step(&mut self, facts: &Facts) {
        self.now += 1;
        if self.state == DriveState::Playing {
            self.eval_bucket(None, facts);
            let cur = self.stage;
            self.eval_bucket(Some(cur), facts);
            self.stage = self.next_stage;
        }
        self.walk_cars(facts);
        self.drain(facts);
    }

    /// `FUN_00175690`: runs every queued event in order, including ones queued meanwhile [M].
    pub fn drain(&mut self, facts: &Facts) {
        let mut i = 0;
        while i < self.queue.len() {
            let ev = self.queue[i].clone();
            self.execute(ev, facts);
            i += 1;
        }
        self.queue.clear();
    }

    /// `FUN_00206228` over one bucket: `None` = the default list (stage < 0) [M 0x206340].
    fn eval_bucket(&mut self, stage: Option<i32>, facts: &Facts) {
        for r in 0..self.rules.len() {
            let in_bucket = match stage {
                None => self.rules[r].stage < 0,
                Some(g) => self.rules[r].stage == g,
            };
            if in_bucket {
                self.eval_rule(r, facts);
            }
        }
    }

    /// `FUN_002096d0` [M].
    fn eval_rule(&mut self, r: usize, facts: &Facts) {
        if !self.rules[r].active {
            return;
        }
        let mut test = std::mem::replace(&mut self.rules[r].test, RuleTest::Always);
        let pass = self.run_test(&mut test, facts);
        self.rules[r].test = test;
        if self.rules[r].invert != pass {
            let actions = self.rules[r].actions.clone();
            // FUN_002095f8: no current trigger; the fire params are the player car's [M].
            for a in actions {
                self.create(a, None, 0, 0);
            }
            if self.rules[r].once {
                self.rules[r].active = false;
            }
        }
    }

    fn health_of(&self, sel: Selector, facts: &Facts) -> Option<i32> {
        match sel.kind {
            0 => Some(self.health as i32),
            4 => facts.elements.get(&sel.id).map(|e| e.health as i32),
            _ => Some(0),
        }
    }

    fn run_test(&mut self, t: &mut RuleTest, facts: &Facts) -> bool {
        let stat = |i: usize| facts.ai_stats[i] as i32;
        match t {
            RuleTest::Always | RuleTest::Pip => true,
            RuleTest::Collision => false,
            RuleTest::Ammo { slot, up, thr, prev } => {
                let v = facts.ammo.get(*slot as usize).copied().unwrap_or(0);
                crossing(v, *up, *thr, prev)
            }
            RuleTest::Damage { sel, thr, prev } => {
                if sel.kind == 3 {
                    return *thr < stat(2);
                }
                let Some(h) = self.health_of(*sel, facts) else { return false };
                let d = *prev - h;
                *prev = h;
                *thr < d
            }
            RuleTest::Death { sel } => self.health_of(*sel, facts) == Some(0),
            RuleTest::Health { sel, up, thr, prev } => {
                let v = if sel.kind == 3 {
                    if !*up { stat(1) } else { stat(0) }
                } else {
                    match self.health_of(*sel, facts) {
                        Some(h) => h,
                        None => return false,
                    }
                };
                crossing(v, *up, *thr, prev)
            }
            RuleTest::Prog { id } => self.events.get(*id as usize).copied().unwrap_or(false),
            RuleTest::Range { sel, up, thr } => {
                let d = if sel.kind == 3 {
                    if !*up { stat(6) } else { stat(5) }
                } else {
                    let p = match sel.kind {
                        0 => facts.player_pos,
                        4 => match facts.elements.get(&sel.id) {
                            Some(e) => e.pos,
                            None => return false,
                        },
                        _ => [0.0; 3],
                    };
                    let q = facts.player_pos;
                    let (dx, dy, dz) = (q[0] - p[0], q[1] - p[1], q[2] - p[2]);
                    (dx * dx + dy * dy + dz * dz).sqrt() as i32
                };
                if !*up { d < *thr } else { *thr < d }
            }
            RuleTest::Speed { sel, up, thr, prev } => {
                let v = match sel.kind {
                    3 => {
                        if !*up { stat(4) } else { stat(3) }
                    }
                    0 => facts.speed() as i32,
                    4 => match facts.elements.get(&sel.id) {
                        Some(e) => e.speed as i32,
                        None => return false,
                    },
                    _ => 0,
                };
                crossing(v, *up, *thr, prev)
            }
            RuleTest::Timer { timer, up, thr, prev } => {
                let Some(tm) = self.timers.get_mut(*timer as usize) else { return false };
                if !tm.running {
                    return false;
                }
                let secs = tm.current(self.now, &mut self.output) as f32 / TICKS_PER_SECOND as f32;
                let fire = if !*up { secs < *thr && *prev >= *thr } else { secs > *thr && *prev <= *thr };
                *prev = secs;
                fire
            }
            RuleTest::Property { sel, name } => match sel.kind {
                0 => facts.player_props.iter().any(|p| p == name.as_str()) && ["NOBOOST", "ONEEMP", "2WHEELS"].contains(&name.as_str()),
                4 => facts.elements.get(&sel.id).is_some_and(|e| e.props.iter().any(|p| p == name.as_str())),
                _ => false,
            },
            RuleTest::PlayerDir { sel, lo, hi } => {
                if sel.kind != 4 {
                    return false;
                }
                let Some(a) = facts.elements.get(&sel.id).and_then(|e| e.bearing_deg) else { return false };
                bearing_inside(a, *lo as f32, *hi as f32)
            }
            RuleTest::Difficulty { mask } => {
                let d = facts.difficulty;
                (*mask & 1 != 0 && d == 0) || (*mask & 2 != 0 && d == 1) || (*mask & 4 != 0 && d == 2)
            }
            RuleTest::ProgCounter { id, op, value } => {
                let c = self.counters.get(*id as usize).copied().unwrap_or(0);
                match op {
                    0 => c as u32 == *value as u32,
                    1 => *value < c,
                    2 => c < *value,
                    3 => c as u32 & *value as u32 != 0,
                    4 => c as u32 & *value as u32 == 0,
                    _ => false,
                }
            }
        }
    }

    /// The car walk `FUN_0022d240`: the player (kind 1, flag bit 2) first, then [`Facts::cars`]
    /// [M]. The game takes the cars in slot order and the triggers from its grid cells; the port
    /// takes the player as slot 0 and the triggers in index order (the order among several hits
    /// in one frame is not traced).
    fn walk_cars(&mut self, facts: &Facts) {
        let player =
            CarFacts { kind: 1, pos: facts.player_pos, vel: facts.player_vel, radius: facts.player_radius, bbox: facts.player_box };
        for (slot, car) in std::iter::once(player).chain(facts.cars.iter().copied()).enumerate() {
            let bit = match car.kind {
                1 => TRIGGER_BY_PLAYER,
                2 => TRIGGER_BY_KIND2,
                _ => TRIGGER_BY_OTHER,
            };
            for i in 0..self.map.triggers.len() {
                let f = self.trigger_flags[i];
                if f & TRIGGER_ENABLED == 0 || f & TRIGGER_SUPPRESSED != 0 {
                    continue;
                }
                if f & TRIGGER_NEEDS_LIVE != 0 && !self.live {
                    continue;
                }
                if f & bit == 0 {
                    continue;
                }
                let bbox = car.bbox.unwrap_or_else(|| CarBox::from_radius(car.radius));
                if car_hits(&self.map.triggers[i], f, car.pos, car.radius, car.vel, &bbox) {
                    self.fire(i, 0, slot as i32);
                }
            }
        }
    }

    /// Runs an action list outside a trigger (no current trigger), as `FUN_002095f8` does for a
    /// rule and the animation player `FUN_001a35f0` does for a prop keyframe (context = the prop
    /// part, no trigger) [M]. For the prop keyframe lists (`nf_formats::trackstart::prop_scripts`:
    /// MIS13A `JA_SAL` counts 0xb, MIS13C `subMissiles` calls stage 3) and the lists the AI
    /// elements own (when the game runs those is not traced). Queued: they run in the next drain.
    pub fn run_actions(&mut self, actions: &[TriggerAction]) {
        for a in actions {
            self.create(a.clone(), None, 0, 0);
        }
    }

    /// `FUN_0022c890`: run a trigger's list as the current trigger; clear bit 0 when once [M].
    pub fn fire(&mut self, trigger: usize, mode: u32, car: i32) {
        let actions = self.map.triggers[trigger].actions.clone();
        for a in actions {
            self.create(a, Some(trigger), mode, car);
        }
        if self.trigger_flags[trigger] & TRIGGER_ONCE != 0 {
            self.trigger_flags[trigger] &= !TRIGGER_ENABLED;
        }
    }

    /// The creator + constructor part of an action (what happens at once) [M per creator].
    fn create(&mut self, a: TriggerAction, trigger: Option<usize>, mode: u32, car: i32) {
        match a.id {
            // ESetTimer ctor 0x159178: value = (int)(secs * rate + 1.0), start = now.
            0x2f => {
                let i = a.word(1) as usize;
                if let Some(t) = self.timers.get_mut(i) {
                    t.value = (a.float(2) * TICKS_PER_SECOND as f32 + 1.0) as i32;
                    t.start = self.now;
                }
            }
            // EStartTimer ctor 0x15ca58 → FUN_00209c08; timer 1 also beeps (0x209d58).
            0x30 => {
                let i = a.word(1) as usize;
                if let Some(t) = self.timers.get_mut(i) {
                    if !t.running {
                        t.rate = a.word(2) as i32;
                        t.running = true;
                        t.start = self.now;
                    }
                    if i == 1 {
                        t.beep = true;
                    }
                }
            }
            // EStopTimer ctor 0x15cf10 → FUN_00209c38.
            0x31 => {
                let i = a.word(1) as usize;
                let now = self.now;
                if let Some(mut t) = self.timers.get(i).copied() {
                    if t.running {
                        t.value = t.current(now, &mut self.output);
                        t.running = false;
                        self.timers[i] = t;
                    }
                }
            }
            // EChangeStage ctor 0x149828 → FUN_00206338: next = n.
            0x29 => self.next_stage = a.word(1) as i32,
            _ => {}
        }
        self.queue.push(Event { action: a, trigger, mode, car });
    }

    /// The handler part (the event destructor) [M per handler].
    fn execute(&mut self, ev: Event, facts: &Facts) {
        let a = &ev.action;
        match a.id {
            0x16 => {
                self.started = true;
                self.output.push(Output::StartMission { trigger: ev.trigger, speed_factor: a.float(1) });
            }
            0x8c => {
                let t = self.map.trigger_ref(a.word(1));
                self.output.push(Output::ResetPlayerCar { trigger: t, speed_factor: a.float(2) });
            }
            // EPlayerWin 0x1551b8: unless lost; state 3, live off, mailbox 0.
            0x2e => {
                if self.state != DriveState::Lost {
                    self.state = DriveState::Won;
                    self.live = false;
                    self.outcome = 0;
                    self.output.push(Output::Won);
                }
            }
            // EPlayerLose 0x154cb0: unless won; state 4, live off, mailbox 1 or 4 (no health).
            // The restart-loop debug path (iRam00100628) and the cheat gate (DAT_0033a008) are
            // taken as off.
            0x2d => {
                if self.state != DriveState::Won {
                    self.state = DriveState::Lost;
                    self.live = false;
                    let dead = self.health <= 0.0;
                    self.outcome = if dead { 4 } else { 1 };
                    self.output.push(Output::Lost { dead });
                }
            }
            // EEndMission 0x14c260: mailbox 0, state 3 (FUN_00205b08), exit request.
            0x13 => {
                self.end_requested = true;
                self.outcome = 0;
                self.state = DriveState::Won;
                self.output.push(Output::EndMission);
            }
            0x2f => {
                if a.word(3) != 0 {
                    self.output.push(Output::ShowTimer(a.word(1) as usize));
                }
            }
            0x104 => self.output.push(Output::HideTimer(a.word(1))),
            0x105 => self.output.push(Output::DrawTimer(a.word(1))),
            // ECallStage 0x148d50 → FUN_00206310: evaluate bucket n now.
            0x28 => {
                let g = a.word(1) as i32;
                self.eval_bucket(Some(g), facts);
            }
            0x35 => {
                if let Some(t) = self.map.trigger_ref(a.word(1)) {
                    self.trigger_flags[t] &= !TRIGGER_ENABLED;
                }
            }
            0x36 => {
                if let Some(t) = self.map.trigger_ref(a.word(1)) {
                    self.trigger_flags[t] |= TRIGGER_ENABLED;
                }
            }
            // EFireTriggerSpeedCondition 0x14c8c8: fire when the car's |v| is above (dir 0) or
            // below the threshold. The car is the firing context's; only the player is modelled.
            0xab => {
                if let Some(t) = self.map.trigger_ref(a.word(1)) {
                    let (thr, below) = (a.float(2), a.word(3) != 0);
                    let v = facts.speed();
                    if (!below && v > thr) || (below && v < thr) {
                        self.fire(t, ev.mode, ev.car);
                    }
                }
            }
            // EFireRandomTrigger 0x14c688.
            0xcb => {
                let p = self.rng.draw();
                let pct = ((p >> 8) & 0xffff) as f32 * 100.0 * 1.525_878_9e-5;
                let (p0, p1, p2, p3) = (a.float(2), a.float(4), a.float(6), a.float(8));
                let c1 = p1 + p0;
                let c2 = p2 + c1;
                let pick = if pct <= p0 {
                    a.word(1)
                } else if pct <= c1 {
                    a.word(3)
                } else if pct <= c2 {
                    a.word(5)
                } else if pct <= p3 + c2 {
                    a.word(7)
                } else {
                    0
                };
                if let Some(t) = self.map.trigger_ref(pick) {
                    self.fire(t, ev.mode, ev.car);
                }
            }
            // ESetMagicCounter 0x1580f8 → FUN_00206ad0: counter = n - 1, then count once.
            0xc8 => {
                let id = a.word(1) as usize;
                if id < EVENT_COUNT {
                    self.counters[id] = (a.word(2) as i32).wrapping_sub(1);
                    self.count(id);
                }
            }
            // EIncMagicCounter 0x14fc50 → FUN_00206990(mgr, n).
            0xb9 => self.count(a.word(1) as usize),
            0xd5 => {
                if let Some(e) = self.events.get_mut(a.word(1) as usize) {
                    *e = false;
                }
            }
            // EObjectiveAdd 0x151538 → FUN_00208578.
            0xe1 => {
                if self.objectives.len() < MAX_OBJECTIVES {
                    self.objectives.push(Objective {
                        id: a.word(1),
                        text: a.word(2),
                        points: a.word(3) as i32,
                        extra: a.word(4),
                        state: 0,
                    });
                    if self.objectives.len() == 1 {
                        self.objectives[0].state = 1;
                    }
                }
            }
            // EObjectivePass 0x1518d0 → FUN_002088b8: state 2, score 0 += points.
            0xe4 => {
                let id = a.word(1);
                if let Some(o) = self.objectives.iter_mut().find(|o| o.id == id) {
                    o.state = 2;
                    let pts = o.points;
                    self.score[0] += pts;
                }
                if a.word(2) != 0 {
                    self.output.push(Output::ObjectiveHud { mode: 2, id });
                }
            }
            // EObjectiveSetCurrent 0x1519c0 → FUN_00208968: state 1.
            0xe5 => {
                let id = a.word(1);
                if let Some(o) = self.objectives.iter_mut().find(|o| o.id == id) {
                    o.state = 1;
                }
                if a.word(2) != 0 {
                    self.output.push(Output::ObjectiveHud { mode: 1, id });
                }
            }
            0xe7 => self.output.push(Output::ObjectiveHud { mode: 1, id: a.word(1) }),
            // ESetObjective 0x158648.
            0x45 => {
                self.marker_extra = a.word(4);
                self.marker = if a.word(1) != 0 {
                    Some((1, a.word(1)))
                } else if a.word(2) != 0 {
                    Some((2, a.word(2)))
                } else if a.word(3) != 0 {
                    Some((3, a.word(3)))
                } else {
                    self.marker_extra = 0;
                    None
                };
            }
            // EAddScoreAction 0x147b80 → FUN_00207ea8(mgr, 1, n).
            0x83 => {
                let n = a.word(1) as i32;
                self.score[1] += n;
                if n > 0 {
                    self.score_actions += 1;
                }
            }
            0x10c => self.output.push(Output::Immunity(a.word(1) != 0)),
            // Handled at construction or nothing more to do.
            0x29 | 0x30 | 0x31 => {}
            // ESetSpecialWeapon 0x158df0: reported, and remembered for `player_fired`.
            0xc0 => {
                self.special_weapon = Some(a.word(1) as i32);
                self.output.push(Output::Effect { effect: script_effects::decode(a, ev.trigger, &self.map), trigger: ev.trigger });
            }
            _ => self.output.push(Output::Effect { effect: script_effects::decode(a, ev.trigger, &self.map), trigger: ev.trigger }),
        }
    }

    /// `FUN_00206990`: counter `id` += 1; at its limit it sets programmer event `id`; counter 12
    /// at 1/2 sets events 0xc/0xd, counter 14 at 1/2/3 sets 0xe/0xf/0x10 [M].
    pub fn count(&mut self, id: usize) {
        if id >= EVENT_COUNT {
            return;
        }
        self.counters[id] += 1;
        if self.limits[id] >= 0 && self.counters[id] >= self.limits[id] {
            self.events[id] = true;
        }
        match self.counters[12] {
            1 => self.events[0xc] = true,
            2 => self.events[0xd] = true,
            _ => {}
        }
        match self.counters[14] {
            1 => self.events[0xe] = true,
            2 => self.events[0xf] = true,
            3 => self.events[0x10] = true,
            _ => {}
        }
    }

    /// `FUN_0018ac28`, the `PBondCar` / `PTank` virtual (vtable slot at 0x37a144 / 0x37ac4c) the
    /// player's EMP calls on a car it hits (`FUN_00182978` weapon 0xe, `FUN_0020d788`) [M]: the
    /// first time a car is hit (`car + 0x380` still 0) it plays "SFX_EMPdetonate", counts an EMP
    /// hit (`FUN_00206900`) and, when the car's name (vtable `+0x84`) starts with the 13
    /// characters "paris_bombvan", sets programmer event 0. `first` = that car had not been
    /// EMP'd yet; `vehicle` = its name. MIS01's bomb van elements use the vehicle
    /// "paris_bombvan" [D `AIEl` +0x54 of elements 0x100, 0xa00, 0xc00, 0x1400, 0x1600].
    pub fn emp_hit(&mut self, vehicle: &str, first: bool) {
        if !first {
            return;
        }
        self.emp_hits += 1;
        if vehicle.as_bytes().get(..13) == Some(b"paris_bombvan".as_slice()) {
            self.set_event(0);
        }
    }

    /// The mission counters a player weapon use changes (`FUN_0020de20(weapons, slot)`, after
    /// its ammo / reload / lock checks pass, which stay the caller's) [M 0x20de20 switch]:
    /// slot 0xc ("SFX_uploadcode") counts 0x1f, slot 0xf ("SFX_downloadcode",
    /// `FUN_0020d980`) counts 0x20, slot 0x14 sets counter 0x22 to 1 (`FUN_00206ad0(mgr, 0x22,
    /// 1)`). MIS11's upload points hand out slot 0x14 (`ESetSpecialWeapon`) and their stage
    /// rules wait for counter 0x22 > 0 [D R168 / R171 / R172].
    pub fn player_fired(&mut self, slot: i32) {
        match slot {
            0xc => self.count(0x1f),
            0xf => self.count(0x20),
            0x14 => {
                self.counters[0x22] = 0;
                self.count(0x22);
            }
            _ => {}
        }
    }

    /// `FUN_00208540`: the game's own code sets programmer events (AI deaths, pickups, …) [M].
    pub fn set_event(&mut self, id: usize) {
        if let Some(e) = self.events.get_mut(id) {
            *e = true;
        }
    }
}

/// `FUN_0020abe0`: `a - lo` and `hi - a`, each wrapped into [0, 360), both under 180 [M].
pub fn bearing_inside(a: f32, lo: f32, hi: f32) -> bool {
    let mut d1 = a - lo;
    if d1 < 0.0 {
        d1 += 360.0;
    }
    if d1 >= 180.0 {
        return false;
    }
    let mut d2 = hi - a;
    if d2 < 0.0 {
        d2 += 360.0;
    }
    d2 < 180.0
}

/// The trigger's box as `FUN_0022eb28` builds it for shape 1 [M 0x22eb28]: axes `+0x20` (side),
/// up = (0, 1, 0), or with flag 0x1000 `cross(+0x30, +0x20)` (`FUN_0025a8e8`), and `+0x30`
/// (forward), none normalised; half extents width / 2, height / 2, length / 2; centre = base
/// point + up × height / 2.
pub fn trigger_box(t: &Trigger, flags: u16) -> ([f32; 3], [[f32; 3]; 3], [f32; 3]) {
    let (d, a) = (t.dir, t.axis);
    let up = if flags & 0x1000 == 0 {
        [0.0, 1.0, 0.0]
    } else {
        [d[1] * a[2] - d[2] * a[1], d[2] * a[0] - d[0] * a[2], d[0] * a[1] - d[1] * a[0]]
    };
    let h = t.height * 0.5;
    let c = [t.pos[0] + h * up[0], t.pos[1] + h * up[1], t.pos[2] + h * up[2]];
    (c, [t.axis, up, t.dir], [t.width * 0.5, h, t.length * 0.5])
}

/// `FUN_001f5eb0`: two oriented boxes overlap unless one of the six face axes separates them
/// (`|dot(cA - cB, aA_i)| - hA_i - sum_j |dot(aA_i, aB_j * hB_j)| > 0`); the edge-cross axes are
/// not tested. Axes are used as stored (`FUN_001f5cf0` scales the rows by the half extents
/// without normalising) [M 0x1f5eb0 / 0x1f5cf0].
pub fn boxes_overlap(ca: [f32; 3], aa: &[[f32; 3]; 3], ha: [f32; 3], cb: [f32; 3], ab: &[[f32; 3]; 3], hb: [f32; 3]) -> bool {
    let dot = |x: [f32; 3], y: [f32; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
    let sep = |c1: [f32; 3], a1: &[[f32; 3]; 3], h1: [f32; 3], c2: [f32; 3], a2: &[[f32; 3]; 3], h2: [f32; 3]| {
        let d = [c1[0] - c2[0], c1[1] - c2[1], c1[2] - c2[2]];
        // The game walks the axes 0, 2, 1 (iVar4 0/1/2 -> row 0/2/1); the order changes nothing.
        [0usize, 2, 1].iter().any(|&i| {
            let mut s = dot(d, a1[i]).abs() - h1[i];
            for j in 0..3 {
                s -= dot(a1[i], [a2[j][0] * h2[j], a2[j][1] * h2[j], a2[j][2] * h2[j]]).abs();
            }
            s > 0.0
        })
    };
    !sep(ca, aa, ha, cb, ab, hb) && !sep(cb, ab, hb, ca, aa, ha)
}

/// `FUN_0022eb28` for a car at `p` (radius `r`, velocity `v`, box `b`) [M]: the reach test, the
/// one-way test, then the shape: sphere, vertical cylinder, or the car's box against the
/// trigger's box ([`trigger_box`], [`boxes_overlap`]).
pub fn car_hits(t: &Trigger, flags: u16, p: [f32; 3], r: f32, v: [f32; 3], b: &CarBox) -> bool {
    let reach = r + t.radius;
    let (dx, dy, dz) = (p[0] - t.pos[0], p[1] - t.pos[1], p[2] - t.pos[2]);
    let one_way_ok = || flags & TRIGGER_ONE_WAY == 0 || t.dir[0] * v[0] + t.dir[1] * v[1] + t.dir[2] * v[2] >= 0.0;
    if t.shape == 2 {
        return dx * dx + dy * dy + dz * dz < reach * reach && one_way_ok();
    }
    if !(dx * dx + dz * dz < reach * reach) || !one_way_ok() {
        return false;
    }
    match t.shape {
        3 => !(p[1] + r < t.pos[1]) && p[1] - r < t.pos[1] + t.height,
        1 => {
            let (c, axes, half) = trigger_box(t, flags);
            boxes_overlap(c, &axes, half, p, &b.axes, b.half)
        }
        _ => false, // FUN_002a8148(0x39af58): unknown shape, no fire [M]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::trackstart::RuleDef;

    fn act(id: u32, words: &[u32]) -> TriggerAction {
        let mut args = id.to_le_bytes().to_vec();
        for w in words {
            args.extend_from_slice(&w.to_le_bytes());
        }
        TriggerAction { id, args }
    }

    fn trig(index: usize, pos: [f32; 3], radius: f32, shape: u8, flags: u16, actions: Vec<TriggerAction>) -> Trigger {
        Trigger {
            index,
            pos,
            radius,
            kind: 0,
            shape,
            flags,
            height: 10.0,
            axis: [1.0, 0.0, 0.0],
            width: 10.0,
            dir: [0.0, 0.0, 1.0],
            length: 10.0,
            actions,
        }
    }

    fn rule(index: usize, kind: u32, flags: u32, stage: i32, payload: [u32; 4], actions: Vec<TriggerAction>) -> RuleDef {
        let mut raw = [0u8; 0x20];
        for (k, w) in [kind, flags, stage as u32, 0x656c_0000].iter().chain(payload.iter()).enumerate() {
            raw[k * 4..k * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        RuleDef { index, raw, actions }
    }

    /// A `sr` reference word for trigger `i`.
    fn tref(map: &mut MapScript, i: usize) -> u32 {
        let w = 0x7372_0100 + i as u32;
        map.refs.insert(w, format!("CARP::<<Map>>::{{Trgr}}::{}", i * 0x40));
        w
    }

    fn at(p: [f32; 3]) -> Facts { Facts { player_pos: p, player_radius: 2.0, ..Default::default() } }

    #[test]
    fn rule_type_numbers_follow_the_descriptor_typeinfo() {
        // 7 = Range, 8 = Speed, 9 = Timer, 10 = Property, 11 = Collision, 12 = PlayerDir,
        // 13 = Difficulty [M: the vtable +4 typeinfo of each FUN_00209390 arm].
        let k = |kind: u32| RuleTest::from_entry(&rule(0, kind, 0, 0, [0; 4], vec![]).raw).unwrap();
        assert!(matches!(k(7), RuleTest::Range { .. }));
        assert!(matches!(k(8), RuleTest::Speed { .. }));
        assert!(matches!(k(9), RuleTest::Timer { .. }));
        assert!(matches!(k(10), RuleTest::Property { .. }));
        assert!(matches!(k(11), RuleTest::Collision));
        assert!(matches!(k(12), RuleTest::PlayerDir { .. }));
        assert!(matches!(k(13), RuleTest::Difficulty { .. }));
        assert!(RuleTest::from_entry(&rule(0, 15, 0, 0, [0; 4], vec![]).raw).is_none());
    }

    #[test]
    fn checkpoint_trigger_fires_once_for_the_player_and_wins() {
        // A cylinder checkpoint (flags: enabled | once | player | live) whose list wins.
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [100.0, 0.0, 0.0], 5.0, 3, 0x407, vec![act(0x2e, &[0, 0])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0, 0.0, 0.0]));
        assert_eq!(s.state, DriveState::Playing);
        assert_eq!(s.outcome, 3);
        // Inside the reach (5 + 2) in plan view, but above the cylinder top (y + 10): no fire.
        s.step(&at([104.0, 13.0, 0.0]));
        assert_eq!(s.state, DriveState::Playing);
        s.step(&at([104.0, 1.0, 0.0]));
        assert_eq!(s.state, DriveState::Won);
        assert_eq!(s.outcome, 0);
        assert!(!s.live, "the win stops the live triggers");
        assert_eq!(s.trigger_flags[0] & TRIGGER_ENABLED, 0, "once clears bit 0");
        assert_eq!(s.take_output(), vec![Output::Won]);
    }

    #[test]
    fn trigger_without_the_player_bit_or_disabled_never_fires() {
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 5.0, 2, 0x40b, vec![act(0x2d, &[0, 0, 0, 0])])); // AI-only
        map.triggers.push(trig(1, [0.0; 3], 5.0, 2, 0x406, vec![act(0x2d, &[0, 0, 0, 0])])); // disabled
        map.triggers.push(trig(2, [0.0; 3], 5.0, 2, 0x2407, vec![act(0x2d, &[0, 0, 0, 0])])); // bit 13
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!(s.state, DriveState::Playing);
    }

    #[test]
    fn a_rival_car_fires_its_own_kind_bit_only() {
        // RACE-shaped: the finish trigger for kind 2 cars loses; the player does not fire it.
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 5.0, 2, 0x40b, vec![act(0x2d, &[1, 0, 0, 0])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!(s.state, DriveState::Playing, "the player is kind 1: bit 2, not bit 3");
        let mut f = at([100.0, 0.0, 0.0]);
        f.cars.push(CarFacts { kind: 3, pos: [0.0; 3], vel: [0.0; 3], radius: 2.0, bbox: None });
        s.step(&f);
        assert_eq!(s.state, DriveState::Playing, "kind 3 picks bit 6");
        f.cars.push(CarFacts { kind: 2, pos: [1.0, 0.0, 0.0], vel: [0.0; 3], radius: 2.0, bbox: None });
        s.step(&f);
        assert_eq!((s.state, s.outcome), (DriveState::Lost, 1));
    }

    #[test]
    fn one_way_trigger_needs_the_car_moving_along_its_direction() {
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 5.0, 2, 0xc07, vec![act(0x2e, &[0, 0])]));
        let mut s = DriveScript::new(map);
        let mut f = at([0.0; 3]);
        f.player_vel = [0.0, 0.0, -10.0];
        s.step(&f);
        assert_eq!(s.state, DriveState::Playing, "moving against +0x30");
        f.player_vel = [0.0, 0.0, 10.0];
        s.step(&f);
        assert_eq!(s.state, DriveState::Won);
    }

    #[test]
    fn lose_after_win_is_ignored_and_dead_lose_reports_4() {
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 5.0, 2, 0x007, vec![act(0x2e, &[0, 0]), act(0x2d, &[0, 0, 0, 0])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!((s.state, s.outcome), (DriveState::Won, 0));

        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 5.0, 2, 0x007, vec![act(0x2d, &[0, 0, 0, 0])]));
        let mut s = DriveScript::new(map);
        s.health = 0.0;
        s.step(&at([0.0; 3]));
        assert_eq!((s.state, s.outcome), (DriveState::Lost, 4));
    }

    #[test]
    fn countdown_timer_rule_fails_the_mission_when_time_runs_out() {
        // A start trigger sets timer 1 to 2 s and starts it counting down (rate -1); a stage-0
        // SRuleTimer (timer 1, below 0.0) loses.
        let mut map = MapScript::default();
        map.triggers.push(trig(
            0,
            [0.0; 3],
            1.0,
            2,
            0x003,
            vec![act(0x16, &[0]), act(0x2f, &[1, 2.0f32.to_bits(), 0]), act(0x30, &[1, u32::MAX])],
        ));
        map.rules.push(rule(0, 9, 0b011, 0, [1, 0, 0.0f32.to_bits(), 0], vec![act(0x2d, &[0, 0, 0, 0])]));
        let mut s = DriveScript::new(map);
        let far = at([500.0, 0.0, 500.0]);
        s.drain(&far); // the world-init drain: EStartMission ran
        assert!(s.started);
        assert!(s.timers[1].running && s.timers[1].value == 121);
        // value 121 - (now - 0): below 0 at now = 122 → the 122nd step.
        for n in 1..122 {
            s.step(&far);
            assert_eq!(s.state, DriveState::Playing, "step {n}");
        }
        s.step(&far);
        assert_eq!((s.state, s.outcome), (DriveState::Lost, 1));
    }

    #[test]
    fn stages_advance_one_frame_after_the_change() {
        // Stage 0: an Always rule changes to stage 1. Stage 1: an Always rule wins.
        let mut map = MapScript::default();
        map.rules.push(rule(0, 0, 0b011, 0, [0; 4], vec![act(0x29, &[1])]));
        map.rules.push(rule(1, 0, 0b011, 1, [0; 4], vec![act(0x2e, &[0, 0])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!((s.stage, s.state), (1, DriveState::Playing), "stage 1 set at the end of frame 1");
        s.step(&at([0.0; 3]));
        assert_eq!(s.state, DriveState::Won);
    }

    #[test]
    fn call_stage_runs_the_bucket_at_once_and_rules_stop_after_the_end() {
        let mut map = MapScript::default();
        map.rules.push(rule(0, 0, 0b011, -1, [0; 4], vec![act(0x28, &[5])]));
        map.rules.push(rule(1, 0, 0b001, 5, [0; 4], vec![act(0x83, &[10])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!(s.score[1], 10, "ECallStage evaluated bucket 5 in the same drain");
        s.state = DriveState::Won;
        s.step(&at([0.0; 3]));
        assert_eq!(s.score[1], 10, "no rule evaluation outside state 1");
    }

    #[test]
    fn death_rule_on_an_element_and_inverted_once_rules() {
        let mut map = MapScript::default();
        map.rules.push(rule(0, 3, 0b011, 0, [4, 7, 0, 0], vec![act(0xe4, &[0x10, 0])]));
        map.rules.push(rule(1, 0, 0b111, 0, [0; 4], vec![act(0x2e, &[0, 0])])); // invert: never
        let mut s = DriveScript::new(map);
        s.objectives.push(Objective { id: 0x10, text: 0, points: 50, extra: 0, state: 1 });
        let mut f = at([0.0; 3]);
        f.elements.insert(7, ElementFacts { health: 300.0, ..Default::default() });
        s.step(&f);
        assert_eq!(s.objectives[0].state, 1);
        f.elements.get_mut(&7).unwrap().health = 0.0;
        s.step(&f);
        assert_eq!((s.objectives[0].state, s.score[0]), (2, 50));
        assert_eq!(s.state, DriveState::Playing, "the inverted Always rule never fires");
    }

    #[test]
    fn speed_rule_fires_on_the_crossing_only() {
        // SRuleSpeed player, up, 30: fires the frame |v| goes above 30 after being at or below.
        let mut map = MapScript::default();
        map.rules.push(rule(0, 8, 0b001, 0, [0, 0, 1, 30], vec![act(0x83, &[1])]));
        let mut s = DriveScript::new(map);
        let mut f = at([0.0; 3]);
        f.player_vel = [0.0, 0.0, 40.0];
        s.step(&f);
        assert_eq!(s.score[1], 0, "prev starts at thr + 1: no crossing yet");
        f.player_vel = [0.0, 0.0, 20.0];
        s.step(&f);
        f.player_vel = [0.0, 0.0, 40.0];
        s.step(&f);
        s.step(&f);
        assert_eq!(s.score[1], 1, "one crossing, one fire");
    }

    #[test]
    fn enable_disable_and_random_and_speed_condition_triggers() {
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 1.0, 2, 0x000, vec![act(0x83, &[100])])); // fired by others only
        map.triggers.push(trig(1, [0.0; 3], 1.0, 2, 0x006, vec![act(0x83, &[1])])); // disabled player trigger
        let r0 = tref(&mut map, 0);
        let r1 = tref(&mut map, 1);
        map.triggers.push(trig(
            2,
            [50.0, 0.0, 0.0],
            1.0,
            2,
            0x007,
            vec![act(0x36, &[r1]), act(0xab, &[r0, 50.0f32.to_bits(), 1]), act(0xcb, &[r0, 100.0f32.to_bits(), 0, 0, 0, 0, 0, 0])],
        ));
        let mut s = DriveScript::new(map);
        let mut f = at([50.0, 0.0, 0.0]);
        f.player_vel = [10.0, 0.0, 0.0]; // below 50: the speed condition fires trigger 0
        s.step(&f);
        assert_eq!(s.trigger_flags[1] & TRIGGER_ENABLED, 1, "EEnableTrigger set bit 0");
        assert_eq!(s.score[1], 200, "speed condition + random pick (100 %) each fired trigger 0");
        f.player_pos = [0.0; 3];
        s.step(&f);
        assert_eq!(s.score[1], 201, "the enabled trigger now fires for the player");
    }

    #[test]
    fn magic_counter_reaches_its_limit_and_sets_the_programmer_event() {
        // Counter 8 has limit 3 [M 0x2057f0]; ESetMagicCounter(8, 3) → counter 3 → event 8;
        // a Prog rule on event 8 then wins.
        let mut map = MapScript::default();
        map.rules.push(rule(0, 0, 0b011, 0, [0; 4], vec![act(0xc8, &[8, 3])]));
        map.rules.push(rule(1, 6, 0b011, 0, [8, 0, 0, 0], vec![act(0x2e, &[0, 0])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert!(s.events[8]);
        assert_eq!(s.state, DriveState::Playing, "the Prog rule ran before the event was set");
        s.step(&at([0.0; 3]));
        assert_eq!(s.state, DriveState::Won);
        let mut s2 = DriveScript::new(MapScript::default());
        s2.counters[9] = 0;
        s2.count(9);
        assert!(!s2.events[9]);
        s2.count(9);
        assert!(s2.events[9], "counter 9's limit is 2");
    }

    #[test]
    fn objectives_first_is_current_and_cap_is_seven() {
        let mut map = MapScript::default();
        let adds: Vec<_> = (0..9).map(|i| act(0xe1, &[0x100 + i, 0, 10, 0])).collect();
        map.rules.push(rule(0, 0, 0b011, 0, [0; 4], adds));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!(s.objectives.len(), 7);
        assert_eq!(s.objectives.iter().map(|o| o.state).collect::<Vec<_>>(), vec![1, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn box_trigger_tests_the_car_box_not_its_sphere() {
        // A 2 x 2 x 2 box trigger at the origin (reach radius 10 so the plan-view pre-test
        // passes). The car (half extents 1, 0.5, 3: radius 3.2) stands 3 to the side.
        let mut t = trig(0, [0.0, -1.0, 0.0], 10.0, 1, 0x7, vec![]);
        (t.width, t.height, t.length) = (2.0, 2.0, 2.0);
        let half = [1.0, 0.5, 3.0];
        let r = (1.0f32 + 0.25 + 9.0).sqrt();
        let p = [3.0, 0.0, 0.0];
        // Facing +z its side reaches x = 2: no overlap (the sphere of radius 3.2 would reach in).
        assert!(!car_hits(&t, t.flags, p, r, [0.0; 3], &CarBox::upright(0.0, half)));
        // Turned to face +x its nose reaches x = 0: overlap.
        assert!(car_hits(&t, t.flags, p, r, [0.0; 3], &CarBox::upright(std::f32::consts::FRAC_PI_2, half)));
        // FUN_001f5eb0 tests both boxes' face axes (iVar7 = 0, 1 swaps them): a cube turned 45
        // degrees at (d, 0, d) is not separated by the upright cube's axes for d <= 2.414, but
        // its own diagonal axis separates them for d > 1.707.
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let rot = [[s, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, s]];
        assert!(!boxes_overlap([0.0; 3], &id, [1.0; 3], [2.0, 0.0, 2.0], &rot, [1.0; 3]));
        assert!(boxes_overlap([0.0; 3], &id, [1.0; 3], [1.6, 0.0, 1.6], &rot, [1.0; 3]));
    }

    #[test]
    fn box_flag_0x1000_takes_up_from_the_cross_product() {
        // FUN_0025a8e8(dir, axis): (d.y a.z - d.z a.y, d.z a.x - d.x a.z, d.x a.y - d.y a.x)
        // [M 0x25a8e8]; dir (0, 0, 1) x axis (0, 1, 0) = (-1, 0, 0). Centre = pos + up * h / 2.
        let mut t = trig(0, [5.0, 0.0, 0.0], 1.0, 1, 0x1007, vec![]);
        t.axis = [0.0, 1.0, 0.0];
        t.dir = [0.0, 0.0, 1.0];
        t.height = 4.0;
        let (c, axes, half) = trigger_box(&t, t.flags);
        assert_eq!((c, axes[1], half), ([3.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [5.0, 2.0, 5.0]));
        let (c, axes, _) = trigger_box(&t, 0x7);
        assert_eq!((c, axes[1]), ([5.0, 2.0, 0.0], [0.0, 1.0, 0.0]));
    }

    #[test]
    fn emp_on_the_bomb_van_sets_event_0_once_and_weapon_0x14_sets_counter_0x22() {
        let mut s = DriveScript::new(MapScript::default());
        s.emp_hit("paris_bombvan", false);
        assert!(!s.events[0], "a car already EMP'd: FUN_0018ac28 does nothing");
        s.emp_hit("paris_taxi", true);
        assert!(!s.events[0]);
        assert_eq!(s.emp_hits, 1);
        s.emp_hit("paris_bombvan", true);
        assert!(s.events[0]);
        s.counters[0x22] = 5;
        s.player_fired(0x14);
        assert_eq!(s.counters[0x22], 1, "FUN_00206ad0(mgr, 0x22, 1): set to 0, then counted");
        s.player_fired(0xc);
        s.player_fired(0xf);
        assert_eq!((s.counters[0x1f], s.counters[0x20]), (1, 1));
    }

    #[test]
    fn unported_actions_come_out_as_typed_effects_with_the_firing_trigger() {
        let mut map = MapScript::default();
        map.triggers.push(trig(0, [0.0; 3], 5.0, 2, 0x7, vec![act(0x78, &[3]), act(0xfb, &[0x3c, 0x12c, 1]), act(0xc0, &[0x14, 0, 0])]));
        let mut s = DriveScript::new(map);
        s.step(&at([0.0; 3]));
        assert_eq!(
            s.take_output(),
            vec![
                Output::Effect { effect: Effect::MaxTraffic { max: 3 }, trigger: Some(0) },
                Output::Effect { effect: Effect::CameraShake { time: 0x3c, power: 0x12c, affect_all: true }, trigger: Some(0) },
                Output::Effect { effect: Effect::SpecialWeapon { weapon: 0x14, trigger: None, arg3: 0.0 }, trigger: Some(0) },
            ]
        );
        assert_eq!(s.special_weapon, Some(0x14));
    }

    #[test]
    fn rng_and_bearing_follow_the_game() {
        let mut r = DriveRng::seeded(0);
        assert_eq!(r.state, 0x3ade_68b1);
        let p = r.draw();
        assert_eq!(p, 0x3ade_68b1u32.wrapping_mul(0x075b_cd15));
        assert_eq!(r.state, p & 0xffff);
        assert_eq!(DriveRng::seeded(2).state, ((0x3ade_68b1u32.wrapping_mul(0x075b_cd15) & 0xffff) * 0x075b_cd15) & 0xffff);
        assert!(bearing_inside(10.0, -30.0, 30.0));
        assert!(!bearing_inside(40.0, -30.0, 30.0));
        assert!(bearing_inside(350.0, 330.0, 20.0));
    }
}
