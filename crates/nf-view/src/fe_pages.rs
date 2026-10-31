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
    /// One already-laid-out text line: the pen's left x at `rect[0]`, the baseline at `rect[1]`
    /// (what a memo line's sprite carries [M Memo_Update 0x21e490:139-152]); `face` as in
    /// [`ElemWhat::Text`]; `shadow` = sprite flag 0x2000 (the -1,-1 second pass).
    Pen { s: String, face: u8, shadow: bool },
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

// ── Component sprites (the SET / COMPONENT / SPRITE records) ────────────────────────────────

/// One component instance as the disc's SPRITE record authors it [D the menu defs' -0x05
/// records; M Component_InitInstance 0x1f1200]: `w` = dx, dy, w, h, u, v, uw, vh; `colour` =
/// the instance's +0x00 word (0xRRGGBBAA — Component_SetupInstance hands it to the sprite
/// [M 0x1f0d90 `lw s0, 0(inst)` → 0x1f0f34 `sw s0, 0(sp)`, Menu_InitializeSprite's colour]);
/// `flags` = +0x34 (mask bits 0x10 idle / 0x20 focused / 0x40 dec held / 0x80 inc held, anchors
/// 1 bottom / 2 right / 4 centre-x / 8 centre-y); `wf`/`hf` the span fractions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompInst {
    pub w: [u16; 8],
    pub colour: u32,
    pub flags: u32,
    pub wf: f32,
    pub hf: f32,
}

const fn ci(w: [u16; 8], colour: u32, flags: u32, wf: f32, hf: f32) -> CompInst {
    CompInst { w, colour, flags, wf, hf }
}

/// The button/arrow atlas every SET 3 record names (COMPONENT hash 0x03000042 [D]).
pub const ATLAS_BUTTONS: u32 = 0x0300_0042;
/// The card box's frame atlas (SET 0xe COMPONENT 0's hash 0x0300019d [D 08000002]).
pub const ATLAS_BOX: u32 = 0x0300_019d;

/// SET 3 COMPONENT 3 — the list box frame (`List_Init` 0x219488:26 handle set|3 [M]): a
/// translucent 9-slice [D 08000002 and the mission defs' 08000001 — the two SET 3s are equal].
pub const SET3_LIST_FRAME: [CompInst; 9] = [
    ci([0, 0, 2, 2, 0, 16, 2, 2], 0x7f7f_7f7f, 0xf0, 0.0, 0.0),
    ci([0, 0, 2, 2, 14, 16, 2, 2], 0x7f7f_7f7f, 0xf2, 0.0, 0.0),
    ci([0, 0, 2, 2, 0, 30, 2, 2], 0x7f7f_7f7f, 0xf1, 0.0, 0.0),
    ci([0, 0, 2, 2, 14, 30, 2, 2], 0x7f7f_7f7f, 0xf3, 0.0, 0.0),
    ci([0, 0, 8, 2, 7, 16, 2, 2], 0x7f7f_7f7f, 0xf4, 1.0, 0.0),
    ci([0, 0, 2, 8, 0, 23, 2, 2], 0x7f7f_7f7f, 0xf8, 0.0, 1.0),
    ci([0, 0, 8, 2, 7, 30, 2, 2], 0x7f7f_7f7f, 0xf5, 1.0, 0.0),
    ci([0, 0, 2, 8, 14, 23, 2, 2], 0x7f7f_7f7f, 0xfa, 0.0, 1.0),
    ci([0, 0, 8, 8, 7, 23, 2, 2], 0x7f7f_7f7f, 0xfc, 1.0, 1.0),
];

/// SET 3 COMPONENT 6 — a horizontal scroll's track (`Scroll_Init` 0x2214f0: w ≥ h → handle
/// set|6 [M]): the ◀ / ▶ arrows, dim while focused (0x20), the held side bright (0x40 ◀, 0x80
/// ▶); no instance carries 0x10, so an unfocused horizontal scroll draws nothing [D].
pub const SET3_SCROLL_H: [CompInst; 4] = [
    ci([0, 0, 16, 16, 48, 0, 16, 16], 0x6f6f_6fdf, 0xa8, 0.0, 0.0),
    ci([0, 0, 16, 16, 48, 16, 16, 16], 0x6f6f_6fdf, 0x6a, 0.0, 0.0),
    ci([0, 0, 16, 16, 48, 0, 16, 16], 0x7f7f_7fff, 0x48, 0.0, 0.0),
    ci([0, 0, 16, 16, 48, 16, 16, 16], 0x7f7f_7fff, 0x8a, 0.0, 0.0),
];

/// SET 3 COMPONENT 7 — a vertical scroll's track (w < h → handle set|7 [M Scroll_Init]):
/// ▲ / ▼, dimmest unfocused (0x10), dim focused (0x20), the held end bright [D].
pub const SET3_SCROLL_V: [CompInst; 6] = [
    ci([0, 0, 16, 16, 16, 0, 16, 16], 0x6060_60c0, 0x14, 0.0, 0.0),
    ci([0, 0, 16, 16, 32, 0, 16, 16], 0x6060_60c0, 0x15, 0.0, 0.0),
    ci([0, 0, 16, 16, 16, 0, 16, 16], 0x6f6f_6fdf, 0xa4, 0.0, 0.0),
    ci([0, 0, 16, 16, 32, 0, 16, 16], 0x6f6f_6fdf, 0x65, 0.0, 0.0),
    ci([0, 0, 16, 16, 16, 0, 16, 16], 0x7f7f_7fff, 0x44, 0.0, 0.0),
    ci([0, 0, 16, 16, 32, 0, 16, 16], 0x7f7f_7fff, 0x85, 0.0, 0.0),
];

/// SET 0xe COMPONENT 0 — the card box's button frame (`Button_Init` 0x1efc90:38 handle set|0
/// [M]): an opaque 9-slice on [`ATLAS_BOX`] [D 08000002].
pub const SETE_BUTTON: [CompInst; 9] = [
    ci([0, 0, 16, 16, 0, 0, 16, 16], 0x7f7f_7fff, 0xf0, 0.0, 0.0),
    ci([0, 0, 16, 16, 16, 0, 16, 16], 0x7f7f_7fff, 0xf2, 0.0, 0.0),
    ci([0, 0, 16, 16, 0, 16, 16, 16], 0x7f7f_7fff, 0xf1, 0.0, 0.0),
    ci([0, 0, 16, 16, 16, 16, 16, 16], 0x7f7f_7fff, 0xf3, 0.0, 0.0),
    ci([0, 0, 8, 16, 15, 16, 2, 16], 0x7f7f_7fff, 0xf5, 1.0, 0.0),
    ci([0, 0, 16, 8, 0, 15, 16, 2], 0x7f7f_7fff, 0xf8, 0.0, 1.0),
    ci([0, 0, 16, 8, 16, 14, 16, 3], 0x7f7f_7fff, 0xfa, 0.0, 1.0),
    ci([0, 0, 8, 16, 16, 0, 1, 16], 0x7f7f_7fff, 0xf4, 1.0, 0.0),
    ci([0, 0, 8, 8, 16, 14, 2, 2], 0x7f7f_7fff, 0xfc, 1.0, 1.0),
];

/// `Component_SetupInstance` 0x1f0950's colour under `Menu_AlphaGet`: the alpha word is added to
/// the R, G and B bytes one by one, the alpha byte kept [M 0x1f0dc4-0x1f0e08: `srl` 24/16/8,
/// `andi 0xff`, `addu`, `sll`, `or` — no clamp].
pub fn alpha_add(c: u32, a: i16) -> u32 {
    let a = a as i32 as u32;
    let r = (c >> 24).wrapping_add(a) << 24;
    let g = ((c >> 16) & 0xff).wrapping_add(a) << 16;
    let b = ((c >> 8) & 0xff).wrapping_add(a) << 8;
    r | g | b | (c & 0xff)
}

/// A control's component instances placed for `mask` over `rect` (512×448, the page at 0,0 —
/// every page's root rect [M Page_Init 0x21f1c0]) through the port's `Component_SetupInstance`
/// (crate::menu_engine): one texture element per shown instance, uv = the instance's (u, v, uw,
/// vh) [M 0x1f0f3c-0x1f0f78, the four `lhu +0x14..+0x1a` stack args], the colour pulsed by
/// `alpha` where the setup runs `Menu_AlphaGet` (mask 0x20 on a fraction-free instance).
pub fn component_elems(ctrl: u32, insts: &[CompInst], tex: u32, mask: u32, rect: [i32; 4], alpha: i16) -> Vec<PageElem> {
    use crate::menu_engine::{component_init_instance, component_setup_instance, AlphaOp, Component, MenuControl, MenuSpriteInfo};
    let c = MenuControl { x: rect[0] as i16, y: rect[1] as i16, w: rect[2] as i16, h: rect[3] as i16, ..Default::default() };
    let page = MenuControl::default();
    let mut rec = Component { hash: tex, count: insts.len() as i16, ..Default::default() };
    for i in insts {
        let mut info = MenuSpriteInfo::default();
        component_init_instance(&mut info, i.wf, i.hf, i.w, i.flags, i.colour);
        rec.instances.push(info);
    }
    let placed = component_setup_instance(&c, &page, &mut rec, &[], mask, c.x, c.y, rect[2], rect[3]);
    placed.iter().filter(|p| !p.hidden && p.w > 0 && p.h > 0).map(|p| {
        let i = insts[p.sprite_ix];
        let colour = if p.alpha == AlphaOp::Get { alpha_add(i.colour, alpha) } else { i.colour };
        PageElem { ctrl, rect: [p.x as i32, p.y as i32, p.w as i32, p.h as i32], colour, what: ElemWhat::Tex { tex, uv: [i.w[4] as f32, i.w[5] as f32, i.w[6] as f32, i.w[7] as f32] } }
    }).collect()
}

// ── P_SCREENADJUST 0x40000047 ───────────────────────────────────────────────────────────────

/// The Screen Adjust page's 24 corner brackets [D menu-def page 0x40000047]: control 0x10000001
/// type 5, TEX 0x03000042, cream, layer 0x1a; authored rects in the page's own order.
pub const SCREENADJUST_BRACKETS: [(i32, i32, i32, i32); 24] = [
    (83, 73, 4, 64), (51, 41, 96, 4), (50, 344, 4, 96), (51, 436, 96, 4),
    (115, 105, 4, 32), (116, 105, 31, 4), (115, 344, 4, 32),
    (51, 41, 4, 96), (85, 73, 62, 4), (83, 344, 4, 64), (85, 404, 62, 4), (116, 372, 31, 4),
    (494, 41, 96, 4), (494, 73, 62, 4), (494, 105, 31, 4), (586, 41, 4, 96), (554, 73, 4, 64),
    (522, 105, 4, 32), (494, 436, 96, 4), (586, 344, 4, 96), (554, 344, 4, 64), (522, 344, 4, 32),
    (494, 404, 62, 4), (494, 372, 31, 4),
];
/// The brackets' one texel (msg 0x2d 0x00260013, 0x00010001 [D]).
pub const SCREENADJUST_BRACKET_UV: [f32; 4] = [38.0, 19.0, 1.0, 1.0];
/// The full-screen reference picture: TEX 0x03000074 uv (0,0,511,511), grey, layer 0x1e [D].
pub const SCREENADJUST_PICTURE_TEX: u32 = 0x0300_0074;
/// The two scroll controls, both id 0x100001eb, type 10, SET 3, layer 0x19 [D]: the horizontal
/// (d1 1 → psiAdjustScreenPos(v, 0)) and the vertical (d1 0 → psiAdjustScreenPos(0, v))
/// [M C_SBSCREEN_Handler 0x217b08:12-19].
pub const SCREENADJUST_BAR_ID: u32 = 0x1000_01eb;
pub const SCREENADJUST_BAR_H: (i32, i32, i32, i32) = (256, 224, 128, 32);
pub const SCREENADJUST_BAR_V: (i32, i32, i32, i32) = (304, 176, 32, 128);
/// The hint label (50,419,540×21) LBL 0x67b, cream, layer 0x1a [D].
pub const SCREENADJUST_HINT: (i32, i32, i32, i32) = (50, 419, 540, 21);
pub const SCREENADJUST_HINT_LABEL: u32 = 0x67b;

/// A focused scroll's component mask this frame [M Scroll_Update 0x221b20:243-252]: nothing
/// held → 0x20; a direction held → the +0x142 word it set (0x40 dec, 0x80 inc).
pub fn scroll_mask(held: u16) -> u32 {
    if held == 0 { 0x20 } else { held as u32 }
}

/// The Screen Adjust page's element list, back to front by layer (the picture 0x1e, the
/// brackets and the hint 0x1a, the bars 0x19 [D]). Both bars carry the focused id — the page
/// script focuses 0x100001eb [D its one keyframe msg 0x22] and Scroll_Update's focus test
/// compares IDS (`page+0xac → +0x18 == self+0x18` [M 0x221b20:42-43]) — so both draw focused:
/// `held_h` / `held_v` are the two bars' +0x142 words (0 none, 0x40 ◀/▲, 0x80 ▶/▼).
pub fn screenadjust_elems(held_h: u16, held_v: u16, alpha: i16, bind: &dyn Fn(u32) -> String) -> Vec<PageElem> {
    let mut out = vec![PageElem { ctrl: 0x1000_0001, rect: [0, 0, 0x200, 0x1c0], colour: GREY, what: ElemWhat::Tex { tex: SCREENADJUST_PICTURE_TEX, uv: [0.0, 0.0, 511.0, 511.0] } }];
    for &(x, y, w, h) in SCREENADJUST_BRACKETS.iter() {
        out.push(PageElem { ctrl: 0x1000_0001, rect: control_rect(0x1000_0001, 5, x, y, w, h), colour: CREAM, what: ElemWhat::Tex { tex: ATLAS_BUTTONS, uv: SCREENADJUST_BRACKET_UV } });
    }
    let (x, y, w, h) = SCREENADJUST_HINT;
    out.push(PageElem { ctrl: 0x1000_0001, rect: control_rect(0x1000_0001, 5, x, y, w, h), colour: CREAM, what: ElemWhat::Text { label: Some(SCREENADJUST_HINT_LABEL), s: bind(SCREENADJUST_HINT_LABEL), face: 2, align: Align::Left, shadow: true } });
    let (x, y, w, h) = SCREENADJUST_BAR_H;
    out.extend(component_elems(SCREENADJUST_BAR_ID, &SET3_SCROLL_H, ATLAS_BUTTONS, scroll_mask(held_h), control_rect(SCREENADJUST_BAR_ID, 10, x, y, w, h), alpha));
    let (x, y, w, h) = SCREENADJUST_BAR_V;
    out.extend(component_elems(SCREENADJUST_BAR_ID, &SET3_SCROLL_V, ATLAS_BUTTONS, scroll_mask(held_v), control_rect(SCREENADJUST_BAR_ID, 10, x, y, w, h), alpha));
    out
}

// ── P_PARISENUM 0x4000004a + the card box page 0x4000002a ─────────────────────────────────

/// P_PARISENUM's full-screen plate: control 0x10000001 (0,0,640,480) TEX 0x03000042 uv
/// (8,23,2,2) colour 0x000000ff, layer 0x1e [D page 0x4000004a]. Its two 0x1000022c labels (the
/// LBL 0x67e line and the logo) are authored hidden (msg 0x2b 1); only the mini-mission arm
/// shows them [M P_PARISENUM 0x20a648:31-32].
pub const PARIS_PLATE_UV: [f32; 4] = [8.0, 23.0, 2.0, 2.0];
/// The card box page 0x4000002a [D]: the bottom bar 0x100001f0 (0,419,640,61) TEX 0x03000042
/// uv (127,55,4,4) colour 0x000000ff layer 0x14; the box button 0x100000fd (50,171,540,121) SET
/// 0xe layer 0x13 (its LBL 2 in colour 0 — invisible); the memo 0x100000fb (59,181,523,103) SET
/// 2 (no frame instances) colour 0x7f7f7fff layer 0x12. The choice list 0x100000fc and the hint
/// 0x10000244 stay hidden while the flow has no buttons (PS2_DoCardFlow 0x217bd8:155-156 hides
/// the list; :178-181 hides the hint on a zero row count [M]).
pub const CARDBOX_BAR: (u32, i32, i32, i32, i32) = (0x1000_01f0, 0, 419, 640, 61);
pub const CARDBOX_BAR_UV: [f32; 4] = [127.0, 55.0, 4.0, 4.0];
pub const CARDBOX_BUTTON: (u32, i32, i32, i32, i32) = (0x1000_00fd, 50, 171, 540, 121);
pub const CARDBOX_MEMO: (u32, i32, i32, i32, i32) = (0x1000_00fb, 59, 181, 523, 103);
/// The memo's text colour (msg 0x1b/0x1c 0x7f7f7fff [D]; the line sprites take +0x274 / +0x278
/// [M Memo_Update 0x21e950/0x21e954]).
pub const CARDBOX_TEXT: u32 = 0x7f7f_7fff;

/// The box's runtime geometry for `lines` memo lines and no buttons [M PS2_DoCardFlow
/// 0x217bd8:186-200]: h = lines·0x1f + 0x1e (+0x15 per button and 0x18 when any — none
/// here), y = (448 − h)·0.5; the button gets (x, y, w, h), the memo's y = y + 10. Returns
/// (button rect, memo rect) in the 512×448 space.
pub fn cardbox_rects(lines: u32) -> ([i32; 4], [i32; 4]) {
    let h = (lines as i32 & 0xff) * 0x1f + 0x1e;
    let y = ((448.0 - h as f32) * 0.5) as i32;
    let b = control_rect(CARDBOX_BUTTON.0, 1, CARDBOX_BUTTON.1, CARDBOX_BUTTON.2, CARDBOX_BUTTON.3, CARDBOX_BUTTON.4);
    let m = control_rect(CARDBOX_MEMO.0, 200, CARDBOX_MEMO.1, CARDBOX_MEMO.2, CARDBOX_MEMO.3, CARDBOX_MEMO.4);
    ([b[0], y, b[2], h], [m[0], y + 10, m[2], m[3]])
}

/// What the card box shows this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct CardBoxView {
    /// The wrapped message lines with their widths (`Font_WordWrapString` at memo w − 16 [M
    /// Memo_Update 0x21e490:110-114 — the SET 2 record's insets are 0]).
    pub lines: Vec<(String, f32)>,
    /// The memo's line count as its LAST update left it — the box height reads msg 0x65 BEFORE
    /// this frame's Memo_Update [M 0x217bd8:186-189 then :202], so a message's first frame sizes
    /// the box for the previous text (0 lines on the open).
    pub prev_lines: u32,
    /// The memo's line pitch: FONTSerpLight's extent of the A..z charset ×1.5 [M Memo_Init
    /// 0x21df38:57,70 — Font_GetTextExtent(0x309c60, 0x30d628 "ff 02")].
    pub stride: f32,
}

/// P_PARISENUM's element list: the plate, then — while the box page stands (`dialog`) — the
/// bottom bar and, while a message stands (`view`), the button frame and the memo's lines
/// (face 2, centred in the memo's w: x = memo.x + (w − extent)·0.5 + 2, baseline =
/// memo.y + (line + 1)·stride [M Memo_Update 0x21e490:121-152]; shadow = the sprite flag
/// 0x2000 [M 0x21e95c `addiu v1, zero, 0x2000` → +0x22]).
pub fn parisenum_elems(dialog: bool, view: Option<&CardBoxView>) -> Vec<PageElem> {
    let mut out = vec![PageElem { ctrl: 0x1000_0001, rect: [0, 0, 0x200, 0x1c0], colour: 0x0000_00ff, what: ElemWhat::Tex { tex: ATLAS_BUTTONS, uv: PARIS_PLATE_UV } }];
    if !dialog { return out; }
    let (id, x, y, w, h) = CARDBOX_BAR;
    out.push(PageElem { ctrl: id, rect: control_rect(id, 5, x, y, w, h), colour: 0x0000_00ff, what: ElemWhat::Tex { tex: ATLAS_BUTTONS, uv: CARDBOX_BAR_UV } });
    let Some(v) = view else { return out };
    let (b, m) = cardbox_rects(v.prev_lines);
    // The button is not the focused control (the box page's script focuses the list 0x100000fc
    // [D its keyframe msg 0x22]) → mask 0x10 [M Button_Update 0x1ef948:56-87].
    out.extend(component_elems(CARDBOX_BUTTON.0, &SETE_BUTTON, ATLAS_BOX, 0x10, b, 0));
    for (k, (s, wd)) in v.lines.iter().enumerate() {
        let x = m[0] as f32 + ((m[2] as f32 - wd) * 0.5).trunc() + 2.0;
        let y = (k as f32 * v.stride + m[1] as f32 + v.stride).trunc();
        out.push(PageElem { ctrl: CARDBOX_MEMO.0, rect: [x as i32, y as i32, *wd as i32, v.stride as i32], colour: CARDBOX_TEXT, what: ElemWhat::Pen { s: s.clone(), face: 2, shadow: true } });
    }
    out
}

// ── P_CHEATMEDAL 0x4000004c (the mission menu defs) ───────────────────────────────────────

/// The cheat-medal page's one control [D the mission defs' 08000001, page 0x4000004c]: the list
/// 0x1000021c type 6 (207,109,226×262), SET 3, colours 0x645a49ff / 0x73330fc0, column 0 at
/// 100%, layer 0x19; the page script focuses it.
pub const CHEATMEDAL_LIST: (u32, i32, i32, i32, i32) = (0x1000_021c, 207, 109, 226, 262);

/// The cheat-medal page's element list. The 0x4c empties the list (msg 0x39 returns the list,
/// msg 0x17 resets its rows [M P_CHEATMEDAL 0x2086b0:14-16; List_SendMessage 0x2196e8:169-240])
/// and nothing adds a row, so what draws is the empty, focused list: its frame (SET 3 COMPONENT
/// 3 over w − 16 — more than one visible row, no 0x7a bit 2 [M List_Update 0x21a598:530-536])
/// and its embedded vertical scroll at (x + w − 16, y, 16, h) [M :169-173], both under mask 0x20.
pub fn cheatmedal_elems(alpha: i16) -> Vec<PageElem> {
    let (id, x, y, w, h) = CHEATMEDAL_LIST;
    let r = control_rect(id, 6, x, y, w, h);
    let mut out = component_elems(id, &SET3_LIST_FRAME, ATLAS_BUTTONS, 0x20, [r[0], r[1], r[2] - 16, r[3]], alpha);
    out.extend(component_elems(id, &SET3_SCROLL_V, ATLAS_BUTTONS, 0x20, [r[0] + r[2] - 16, r[1], 16, r[3]], alpha));
    out
}
