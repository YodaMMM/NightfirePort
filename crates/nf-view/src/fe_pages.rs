//! What three front-end pages draw, as element lists built from the page data (the menu-def PAGE
//! records in 07000048.bin, entry 08000002) and the pages' handlers: the 007 Bonus page
//! 0x40000038 (`P_NFBONUS_Handler` 0x209670), the credits roll 0x40000030 (`P_CREDITS_Handler`
//! 0x2152b0) and the end-of-campaign win page 0x40000053 (`P_WINGAME_Handler` 0x215100).
//! `menu::menu_overlay` renders these lists; the tests pin them against the disc's page records.
//!
//! Every rect here is the control's RUNTIME rect — the +0x70/+0x72/+0x74/+0x76 words in the
//! game's 512×448 screen space, which is the port's HUD space one to one — after
//! `MenuManager_Create` 0x21b908:305-314 applied its three placement arms to the authored
//! 640×480 rect ([`control_rect`]), and after the handler's own runtime writes.
//!
//! Colours are the game's 0xRRGGBBAA words (msg 0x1a/0x1b/0x1c); the alpha byte is clamped to
//! 0x80 when drawn [M Label_Update 0x218cb8:116-121].

use crate::credits::credits_data::{Str, CREDITS};
use crate::credits::CreditsRoll;

/// How a label's text sits in its rect — `Font_GetAlignment` 0x1ef728: the format's `fe 03` is
/// centred (Label_Update x = ctrl.x + w/2 [M 0x218cb8:157-160]); no `fe` tag is 1, left at ctrl.x.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Centre,
}

/// One element's look.
#[derive(Clone, Debug, PartialEq)]
pub enum ElemWhat {
    /// A texture label: the texture hash (msg 0x24) and its uv rect (u, v, w, h in texels — the
    /// msg-0x2d words [M Label_SendMessage 0x218890 case 0x2d → sprite +0x30..+0x36]).
    Tex { tex: u32, uv: [f32; 4] },
    /// A text label: the bound label id (None for composed or literal text), the string, the
    /// font face (the format's `ff 0N` code: 1 = FONTNFont2, 2 = FONTSerpLight, 3 = FONTMedium
    /// [L FontTable 0x2dd8b0]), the alignment and whether the -1,-1 second pass draws (sprite
    /// flag 0x2000, from the label's +0x10e byte [M 0x218cb8:226-231]; msg 0x62 sets it).
    Text { label: Option<u32>, s: String, face: u8, align: Align, shadow: bool },
    /// A full-rect fill in the element colour (the fade plate's single-texel quad).
    Fill,
    /// A type-1 button's frame (its component sprite set) [S the look].
    Frame,
}

/// One drawn control.
#[derive(Clone, Debug, PartialEq)]
pub struct PageElem {
    /// The control id (several controls share one id: the credits' 26 list slots per column).
    pub ctrl: u32,
    /// The runtime rect (x, y, w, h) in the 512×448 space.
    pub rect: [i32; 4],
    /// 0xRRGGBBAA.
    pub colour: u32,
    pub what: ElemWhat,
}

/// `FixupResolution` 0x21b610 — the authored 640×480 rect into the 512×448 screen: w·0.8 (≥ 2),
/// x re-centred on (x + w/2 − 320)·0.8 + 256; h·0.9333334 (≥ 2, ≥ 16 for a type-0xa list),
/// y re-centred on (y + h/2 − 240)·0.9333334 + 224. Integer truncation as the game's casts.
pub fn fixup_resolution(x: i32, y: i32, w: i32, h: i32, list: bool) -> [i32; 4] {
    let w2 = ((w as f32 * 0.8) as i32).max(2);
    let x2 = ((x + (w >> 1) - 0x140) as f32 * 0.8 + 256.0 - (w2 >> 1) as f32) as i32;
    let mut h2 = (h as f32 * 0.933_333_4) as i32;
    h2 = if list { if h2 > 0xf { h2 } else { 0x10 } } else { h2.max(2) };
    let y2 = ((y + (h >> 1) - 0xf0) as f32 * 0.933_333_4 + 224.0 - (h2 >> 1) as f32) as i32;
    [x2, y2, w2, h2]
}

/// `MenuManager_Create` 0x21b908:305-314's placement arms for one CONTROL record: a type-5
/// full-screen label (x < 1, y < 1, w > 0x27f, h > 0x1df) becomes w 0x200 h 0x1c0 at its x/y;
/// control 0x10000224 is forced to (0x154, 0xc6) keeping its size; every other type but 0xd
/// goes through [`fixup_resolution`]; a type-0xd window keeps its rect.
pub fn control_rect(id: u32, ctype: u8, x: i32, y: i32, w: i32, h: i32) -> [i32; 4] {
    if ctype == 5 && x < 1 && y < 1 && w > 0x27f && h > 0x1df {
        [x, y, 0x200, 0x1c0]
    } else if id == 0x1000_0224 {
        [0x154, 0xc6, w, h]
    } else if ctype != 0xd {
        fixup_resolution(x, y, w, h, ctype == 0xa)
    } else {
        [x, y, w, h]
    }
}

/// The cream label colour every text control on these pages carries (msg 0x1b/0x1c 0x7d6d59ff,
/// rewritten 0x645a49ff at parse [M 0x21b908:238-243]).
pub const CREAM: u32 = 0x645a_49ff;
/// The texture labels' neutral grey (msg 0x1b/0x1c 0x7f7f7fff [D the page records]).
pub const GREY: u32 = 0x7f7f_7fff;

// ── P_NFBONUS 0x40000038 ────────────────────────────────────────────────────────────────────

/// The 007 Bonus page's authored controls [D menu-def page 0x40000038]: (id, type, x, y, w, h).
/// Header LBL 0x2c5 (NAME "\xff\x01"), hint LBL 0x2d0, the reward picture 0x100001cf TEX
/// 0x03000134 uv (0,0,172,256), the type-1 button 0x10000165 LBL 2 (colour 0 — its text is
/// invisible; the page script's msg 0x22 focuses it so ✕ is its 0x4b), the text list 0x100001d0,
/// the stamp 0x10000224 TEX 0x0300014c uv (1,1,255,255), the logo TEX 0x0300016d uv (0,0,255,127).
pub const NFBONUS_CONTROLS: [(u32, u8, i32, i32, i32, i32); 7] = [
    (0x1000_0001, 5, 50, 40, 376, 17),
    (0x1000_0001, 5, 50, 419, 540, 21),
    (0x1000_01cf, 5, 89, 112, 171, 256),
    (0x1000_0165, 1, 62, 372, 61, 59),
    (0x1000_01d0, 200, 268, 94, 284, 292),
    (0x1000_0224, 5, 340, 198, 255, 255),
    (0x1000_0001, 5, 434, 38, 156, 43),
];

/// The page's composed text [M P_NFBONUS 0x209670:16-43]: with DAT_002e1c2c down msg 0x18 sends
/// 0 — the blank string " " (0x30d5a8 [M Label_SendMessage 0x218890:44-47]); with it up tier 1's
/// "%s\n" (0x30d350) then tiers 4, 3, 2 each as "%s\n%s" (0x30d358) onto the buffer.
pub fn nfbonus_text(bonus: Option<&crate::menu::BonusFields>, bind: &dyn Fn(u32) -> String) -> String {
    let Some(b) = bonus.filter(|b| b.has_text) else { return " ".into() };
    let mut s = String::new();
    // Tier 1's sprintf runs only while its line stands [M :22-26]; text_labels lists tier 1
    // first exactly then (bonus_enter's 1→4→3→2 filter).
    let mut labels = b.text_labels.iter();
    if b.lines[0].has {
        if let Some(&l) = labels.next() { s = format!("{}\n", bind(l)); }
    }
    for &l in labels { s = format!("{s}\n{}", bind(l)); }
    s
}

/// The 007 Bonus page's element list: the page data's controls with the handler's runtime
/// sends — msg 0x24 of tier 1's texture into 0x100001cf when that line stands [M :22-26], msg
/// 0x23 "\xff\x03" (0x30d348, FONTMedium, no `fe` tag → left-aligned) and msg 0x18 with the
/// composed text into 0x100001d0 [M :15, :43]. The page has no animated control: its only
/// script is the page's own 0x2000000f whose single keyframe focuses 0x10000165 [D].
pub fn nfbonus_elems(bonus: Option<&crate::menu::BonusFields>, bind: &dyn Fn(u32) -> String) -> Vec<PageElem> {
    let r = |i: usize| { let c = NFBONUS_CONTROLS[i]; control_rect(c.0, c.1, c.2, c.3, c.4, c.5) };
    let pic = bonus.and_then(|b| b.texture).unwrap_or(0x0300_0134);
    vec![
        PageElem { ctrl: 0x1000_0001, rect: r(0), colour: CREAM, what: ElemWhat::Text { label: Some(0x2c5), s: bind(0x2c5), face: 1, align: Align::Left, shadow: true } },
        PageElem { ctrl: 0x1000_01cf, rect: r(2), colour: GREY, what: ElemWhat::Tex { tex: pic, uv: [0.0, 0.0, 172.0, 256.0] } },
        PageElem { ctrl: 0x1000_0165, rect: r(3), colour: 0, what: ElemWhat::Frame },
        PageElem { ctrl: 0x1000_01d0, rect: r(4), colour: CREAM, what: ElemWhat::Text { label: None, s: nfbonus_text(bonus, bind), face: 3, align: Align::Left, shadow: true } },
        PageElem { ctrl: 0x1000_0224, rect: r(5), colour: GREY, what: ElemWhat::Tex { tex: 0x0300_014c, uv: [1.0, 1.0, 255.0, 255.0] } },
        PageElem { ctrl: 0x1000_0001, rect: r(6), colour: GREY, what: ElemWhat::Tex { tex: 0x0300_016d, uv: [0.0, 0.0, 255.0, 127.0] } },
        PageElem { ctrl: 0x1000_0001, rect: r(1), colour: CREAM, what: ElemWhat::Text { label: Some(0x2d0), s: bind(0x2d0), face: 2, align: Align::Centre, shadow: true } },
    ]
}

// ── P_CREDITS 0x40000030 ────────────────────────────────────────────────────────────────────

/// The credits' left / right list controls (26 slots each, d1 = slot) [D page 0x40000030].
pub const CREDITS_LEFT: u32 = 0x1000_0213;
pub const CREDITS_RIGHT: u32 = 0x1000_0214;
/// The background label (0,0,640,480) TEX 0x03000129 uv (0,0,511,511) [D].
pub const CREDITS_BG: u32 = 0x1000_023c;
pub const CREDITS_BG_TEX: u32 = 0x0300_0129;
/// The fade label (0,0,640,480) TEX 0x03000042 uv (8,23,1,1) — one texel [D].
pub const CREDITS_FADE: u32 = 0x1000_0215;
/// The list slots' runtime height: the authored 16 through [`fixup_resolution`] (14) [D + M].
pub const CREDITS_ITEM_H: i32 = 14;

/// The font face of a credit kind: msg 0x23 with 0x30d560 "ff 01 fe 03" (kind 1), 0x30d568
/// "ff 03 fe 03" (kind 2), else 0x30d570 "ff 02 fe 03" [M 0x2152b0:122-147; D ELF bytes].
pub fn credit_face(kind: u8) -> u8 {
    match kind { 1 => 1, 2 => 3, _ => 2 }
}

fn credit_str(s: Str, bind: &dyn Fn(u32) -> String) -> String {
    match s {
        Str::Label(id) => bind(id),
        Str::Lit(t) => t.to_string(),
        Str::Space => " ".into(),
        Str::Null => " ".into(), // msg 0x18 (0, 0) → the blank string 0x30d5a8 [M 0x218890:44-47]
    }
}

/// The fade plate's alpha byte `frames` after the exit process started: Process_Create's id-0xe1
/// blend from the colour msg 0x1a set at enter (0) to 0x000000ff over 0xf frames
/// [M 0x2152b0:179-183; Process_RunFrame 0x21f380's start + (target − start)·tick/delay], then
/// the exact target.
pub fn credits_fade_alpha(frames: u32) -> u32 {
    if frames >= 0xf { 0xff } else { 0xff * frames / 0xf }
}

/// The credits' element list this frame. In draw order: the background — colour 0x000000ff, or
/// 0x404040ff while GameState[0x52] stands [M 0x2152b0:83-88] (msg 0x1a: the texture times black,
/// or at half brightness); each visible line's left item (x 0x10, w 0x100 / 0x1e0 when wide) and,
/// on a two-column line, its right item (x 0xf0, w 0x100) at the slot's y [M :117-161, :186-194],
/// cream, no second pass (msg 0x62/0 to both lists at enter [M :76-77]); the fade plate once
/// `fading` stands.
pub fn credits_elems(roll: &CreditsRoll, bind: &dyn Fn(u32) -> String) -> Vec<PageElem> {
    let mut out = vec![PageElem {
        ctrl: CREDITS_BG,
        rect: [0, 0, 0x200, 0x1c0],
        colour: if roll.bonus_flag { 0x4040_40ff } else { 0x0000_00ff },
        what: ElemWhat::Tex { tex: CREDITS_BG_TEX, uv: [0.0, 0.0, 511.0, 511.0] },
    }];
    for v in roll.visible() {
        let line = &CREDITS[v.line];
        let text = |s: Str, kind: u8| ElemWhat::Text { label: match s { Str::Label(id) => Some(id), _ => None }, s: credit_str(s, bind), face: credit_face(kind), align: Align::Centre, shadow: false };
        if v.wide {
            out.push(PageElem { ctrl: CREDITS_LEFT, rect: [0x10, v.y, 0x1e0, CREDITS_ITEM_H], colour: CREAM, what: text(line.l, line.kl) });
        } else {
            out.push(PageElem { ctrl: CREDITS_LEFT, rect: [0x10, v.y, 0x100, CREDITS_ITEM_H], colour: CREAM, what: text(line.l, line.kl) });
            out.push(PageElem { ctrl: CREDITS_RIGHT, rect: [0xf0, v.y, 0x100, CREDITS_ITEM_H], colour: CREAM, what: text(line.r, line.kr) });
        }
    }
    if roll.fading {
        let a = credits_fade_alpha(roll.frame.saturating_sub(crate::credits::FADE_FRAME));
        out.push(PageElem { ctrl: CREDITS_FADE, rect: [0, 0, 0x200, 0x1c0], colour: a, what: ElemWhat::Fill });
    }
    out
}

// ── P_WINGAME 0x40000053 ────────────────────────────────────────────────────────────────────

/// The win page draws no control: its PAGE record is the root window alone [D], so the screen is
/// the background movie 0x073f0048 the 0x4c starts [M 0x215100:20], full-frame on the movie
/// plane. The list is empty by the data; the host plays the movie.
pub fn wingame_elems() -> Vec<PageElem> {
    Vec::new()
}
