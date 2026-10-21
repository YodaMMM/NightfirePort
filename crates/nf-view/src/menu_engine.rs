//! The menu engine's control/component/sprite helpers — task unported-front-end-menus-4.
//! Data/state/flow only (nf-view's `menu_overlay` does the drawing, which the main agent
//! renders). Every function cites its game address; the full evidence write-up is
//! docs/systems/menu-engine.md. Facts are [M] unless marked [S].
//!
//! Game base: ACTION.ELF decompile under work/decomp/ACTION.ELF/. Where Ghidra dropped
//! arguments the disassembly was checked (Menu_Assert's a0 + unsigned gate, Menu_CreateSprite's
//! dead MENUSPRITETYPE + hardcoded 7, Menu_FindControl's QuickSort comparator 7 =
//! CompMenuCtrl 0x1d34c8 ascending-by-score, Combo_Init's 3-register prologue).

/// Control +0x7b type values [M Combo_Init 0x1f0248 stores 3; Menu_GetPage 0x1f27f8 walks
/// to 8; Menu_GetFirstControl/FindControl/CursorOverMe skip 0xd].
pub const TYPE_PAGE_HEADER: u8 = 0xd;
pub const TYPE_PAGE_CONTAINER: u8 = 8;
pub const TYPE_COMBO: u8 = 3;

/// The static backup control's fields Menu_GetFirstControl falls back to
/// [M 0x1f2708: memset 0x94 bytes; name 0x1fffffff, x=y=0xff9c, w=h=0, byte +0x6b = 0xff].
pub const BACKUP_NAME: u32 = 0x1fff_ffff;
pub const BACKUP_XY: i16 = -100; // 0xff9c
/// The menu allocator's Mem_Malloc tag [M Menu_Malloc 0x1f3008: Mem_Malloc(size, 0x3104, 0)].
pub const MENU_MALLOC_TAG: u32 = 0x3104;

/// A runtime menu control — only the fields the ported engine reads/writes [M the offsets
/// in each cited function]. The game's control also carries scripts/messages/... (those
/// are menudef.rs's `CtrlState` job).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MenuControl {
    /// +0x0 — next sibling (the page's intrusive list) [M 0x1f2708 walk `*puVar4`].
    pub next: Option<usize>,
    /// +0xc — the parent/page backlink [M Menu_GetPage 0x1f27f8 walks +0xc; Menu_Assert
    /// requires it non-null].
    pub parent: Option<usize>,
    /// +0x8 — the control's name/id [M 0x1f2708's backup writes 0x1fffffff to +0x8].
    pub name: u32,
    /// +0x70..+0x76 — the animated rect [M Menu_ResizeControl 0x1f2e30].
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    /// +0x78 — draw layer [M CursorOverMe picks the minimum; FindControl writes cursor
    /// +0x38 = ctrl layer + 1].
    pub layer: i16,
    /// +0x7a — disabled byte (0 = usable) [M 0x1f2708/0x1f16d8/0x1f13c8 skip non-zero].
    pub disabled: u8,
    /// +0x7b — the control type [M Menu_Assert membership; Combo_Init stores 3].
    pub ctype: u8,
    /// +0x14 — the component handle: (set id << 16) | record index [M Component_GetHandle
    /// 0x1f1268; Combo_Init builds (u16)<<16 | 2 — disasm 0x1f0290].
    pub handle: u32,
}

impl MenuControl {
    pub fn rect(&self) -> (i16, i16, i16, i16) {
        (self.x, self.y, self.w, self.h)
    }
}

/// `Menu_ResizeControl` 0x1f2e30 [M: +0x70..+0x76 = x, y, w; +0x76 = h].
pub fn menu_resize_control(c: &mut MenuControl, x: i16, y: i16, w: i16, h: i16) {
    c.x = x;
    c.y = y;
    c.w = w;
    c.h = h;
}

/// `Menu_ResizeControl` 0x1f2e48 (copy form) [M: four shorts +0x70..+0x76 from src].
pub fn menu_resize_control_copy(dst: &mut MenuControl, src: &MenuControl) {
    dst.x = src.x;
    dst.y = src.y;
    dst.w = src.w;
    dst.h = src.h;
}

/// `Menu_Assert` 0x1f2f18: 1 iff ctrl != 0, ctrl+0xc != 0 and type ∈ {1,2,3,5,6,9,10,11,
/// 12,200,114}. The (t−1) < 3 gate is UNSIGNED (disasm `sltiu` 0x1f2f40) so t = 0 fails —
/// the decompiled C's signed `(2 < bVar1-1)` would wrongly admit 0.
pub fn menu_assert(ctrl: Option<&MenuControl>) -> bool {
    let Some(c) = ctrl else { return false };
    if c.parent.is_none() {
        return false; // +0xc != 0 [M 0x1f2f1c lw 0xc($a0) + the != 0 test 0x1f2f28]
    }
    matches!(c.ctype, 1 | 2 | 3 | 5 | 6 | 9 | 10 | 11 | 12 | 200 | 114)
}

/// `Menu_GetPage` 0x1f27f8. Assert-gated [M: a failed assert returns 0 — a page header's
/// own type 0xd is NOT in the assert set, so GetPage(page header) = 0]. Type 0xd would
/// return the page's control list (+0x94); otherwise walk +0xc to a type-8 control.
/// Returns the index of the owning control (the port models the list as owned by the
/// page-header control; [S] modeling choice, the flow matches).
pub fn menu_get_page(ctrls: &[MenuControl], ix: usize) -> Option<usize> {
    let c = ctrls.get(ix)?;
    if !menu_assert(Some(c)) {
        return None;
    }
    if c.ctype == TYPE_PAGE_HEADER {
        return Some(ix); // the game returns *(ctrl+0x94) — the list this control owns
    }
    let mut p = c.parent;
    while let Some(pix) = p {
        let pc = ctrls.get(pix)?;
        if pc.ctype == TYPE_PAGE_CONTAINER {
            return Some(pix);
        }
        p = pc.parent;
    }
    None
}

/// `Menu_GetFirstControl` 0x1f2708. Walks the page's controls from the list head, skipping
/// type-0xd headers and disabled (+0x7a) controls; returns the first usable control, or
/// `None` for the game's static backup control (BACKUP_* fields).
pub fn menu_get_first_control(ctrls: &[MenuControl], first: Option<usize>) -> Option<usize> {
    let mut cur = first;
    while let Some(ix) = cur {
        let c = ctrls.get(ix)?;
        if c.ctype == TYPE_PAGE_HEADER {
            cur = c.next;
            continue;
        }
        if c.disabled == 0 {
            return Some(ix);
        }
        cur = c.next;
    }
    None
}

/// `Menu_CursorOverMe` 0x1f13c8: is `cand` the control under `point` (page space)?
/// Walks the page's controls (skip 0x7a-disabled and 0xd headers), collects rects that
/// contain the point, returns the LOWEST-layer (+0x78) hit — the walk keeps the min in
/// slot 0 [M 0x1f13c8's `if (hit+0x78 < best+0x78) best = hit`].
pub fn menu_cursor_over_me(
    ctrls: &[MenuControl],
    first: Option<usize>,
    point: (i32, i32),
    cand: usize,
) -> bool {
    let mut best: Option<usize> = None;
    let mut cur = first;
    while let Some(ix) = cur {
        let c = ctrls.get(ix).cloned();
        cur = c.as_ref().and_then(|c| c.next);
        let Some(c) = c else { break };
        if c.ctype == TYPE_PAGE_HEADER || c.disabled != 0 {
            continue;
        }
        let (x, y, w, h) = c.rect();
        let (px, py) = point;
        if px < x as i32 || py < y as i32 {
            continue;
        }
        if (x as i32 + w as i32) < px || (y as i32 + h as i32) < py {
            continue;
        }
        match best {
            Some(b) if ctrls[b].layer <= c.layer => {}
            _ => best = Some(ix),
        }
    }
    best == Some(cand)
}

/// `Menu_FindControl` 0x1f16d8's direction byte [M disasm: 1=UP, 2=DOWN, 4=LEFT, 8=RIGHT;
/// other values score nothing].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindDir {
    Up,
    Down,
    Left,
    Right,
}

/// One scored candidate: element = {control*, score} [M the 8-byte QuickSort elements,
/// signed short at elem+4].
#[derive(Debug, Clone, Copy)]
struct FindCandidate {
    ctrl: usize,
    score: i16,
}

/// The walk's skip rules and per-candidate scoring, exactly as the decompute computes
/// them. Returns (same-direction candidates, opposite/wrap candidates).
fn find_candidates(
    ctrls: &[MenuControl],
    first: Option<usize>,
    cur: usize,
    dir: FindDir,
) -> (Vec<FindCandidate>, Vec<FindCandidate>) {
    let c = &ctrls[cur];
    let (cx, cy, cw, ch) = (c.x as i32, c.y as i32, c.w as i32, c.h as i32);
    let mut fwd = Vec::new();
    let mut back = Vec::new();
    let mut node = first;
    while let Some(ix) = node {
        let t = ctrls.get(ix).cloned();
        node = t.as_ref().and_then(|t| t.next);
        let Some(t) = t else { break };
        if ix == cur || t.disabled != 0 || t.ctype == TYPE_PAGE_HEADER || t.w == 0 || t.h == 0 {
            continue; // [M 0x1f1768/0x1f1770/0x1f1784/0x1f1798]
        }
        let (tx, ty, tw, th) = (t.x as i32, t.y as i32, t.w as i32, t.h as i32);
        let dx = (cx - tx) as f32; // fVar13 = cur.x - t.x
        let dx2 = ((cx + cw) - tx) as f32; // fVar15 = cur.x+cur.w - t.x
        let dy = (cy - ty) as f32; // fVar14 = cur.y - t.y
        let dy2 = ((cy + ch) - ty) as f32; // fVar16 = cur.y+cur.h - t.y
        let x_overlap = dx < tw as f32 && 0.0 < dx2; // bVar4
        let y_overlap = 0.0 < dy2 && dy < th as f32; // bVar5
        const NONE: i16 = 9999; // [M the 0x270f sentinel]
        let mut f: i16 = NONE;
        let mut b: i16 = NONE;
        match dir {
            // Vertical: 2·|Δy| + 0.1·|Δx| [M 0x1f18d8-0x1f191c]
            FindDir::Up => {
                if (th as f32) < dy && x_overlap {
                    f = (dy.abs() + dy.abs() + dx.abs() * 0.1) as i16;
                }
                if dy2 < 0.0 && x_overlap {
                    b = (dy2 + dy2 + dx.abs() * 0.1) as i16;
                }
            }
            FindDir::Down => {
                if dy < (th as f32) && x_overlap {
                    f = (dy2.abs() + dy2.abs() + dx.abs() * 0.1) as i16;
                }
                if (th as f32) < dy && x_overlap {
                    b = (-(dy) - dy + dx.abs() * 0.1) as i16;
                }
            }
            // Horizontal: |Δx| + |Δy| [M 0x1f1a04-0x1f1a5c]
            FindDir::Left => {
                if (tw as f32) < dx && y_overlap {
                    f = (dx.abs() + dy.abs()) as i16;
                }
                if dx2 < 0.0 && y_overlap {
                    b = (dx2 + dy.abs()) as i16;
                }
            }
            FindDir::Right => {
                if dx < (tw as f32) && y_overlap {
                    f = (dx2.abs() + dy.abs()) as i16;
                }
                if (tw as f32) < dx && y_overlap {
                    b = (dy.abs() - dx) as i16;
                }
            }
        }
        if f != NONE {
            fwd.push(FindCandidate { ctrl: ix, score: f });
        }
        if b != NONE {
            back.push(FindCandidate { ctrl: ix, score: b });
        }
    }
    (fwd, back)
}

/// `Menu_FindControl` 0x1f16d8: pick the minimum-score same-direction candidate; when
/// there is none, the minimum opposite (wrap-around) one; when neither, stay.
/// QuickSort comparator 7 = CompMenuCtrl 0x1d34c8: ascending by the signed short score,
/// min first (ties unspecified — the sort is not stable; the port takes the first minimum
/// [S] tie-break). Cap 63 per array [M `sltiu $v0, $s3, 0x3f`].
pub fn menu_find_control(
    ctrls: &[MenuControl],
    first: Option<usize>,
    cur: usize,
    dir: FindDir,
) -> Option<usize> {
    let (mut fwd, mut back) = find_candidates(ctrls, first, cur, dir);
    fwd.sort_by_key(|c| c.score);
    back.sort_by_key(|c| c.score);
    if let Some(w) = fwd.first() {
        if w.ctrl != cur {
            return Some(w.ctrl);
        }
    }
    if let Some(w) = back.first() {
        if w.ctrl != cur {
            return Some(w.ctrl);
        }
    }
    None
}

/// The cursor reposition FindControl performs on the manager's cursor object
/// [M 0x1f1b74-0x1f1bf4: x/y = page rect + ctrl x/y + w/2 / h/2; +0x2c/+0x2e = w+10/h+10;
/// +0x38 = ctrl layer + 1]. Returns (x, y, w+10, h+10, layer+1).
pub fn find_cursor_update(ctrl: &MenuControl, page: &MenuControl) -> (i16, i16, i16, i16, i16) {
    (
        page.x + ctrl.x + ctrl.w / 2,
        page.y + ctrl.y + ctrl.h / 2,
        ctrl.w + 10,
        ctrl.h + 10,
        ctrl.layer + 1,
    )
}

// ---------------------------------------------------------------------------
// Component sprite sets
// ---------------------------------------------------------------------------

/// The 0x38-byte instance record [M Component_InitInstance 0x1f1200 writes; the parse's
/// SPRITE −0x05 {8×u16, u32, u32, f32, f32} feeds the fields in that order].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSpriteInfo {
    /// +0x00 — the texture hash [M SetupInstance passes *record; InitInstance param 13].
    pub hash: u32,
    /// +0x04 — (uninitialised in InitInstance).
    pub pad04: u16,
    /// +0x06 — InitInstance param 4.
    pub v06: u16,
    /// +0x08 — init 0xffff [M 0x1f1200 `*(undefined2 *)(param_3 + 2) = 0xffff`].
    pub u1: u16,
    /// +0x0a — init 0 [M].
    pub z0a: u16,
    /// +0x0c / +0x0e — offsets [M SetupInstance reads them for x/y anchoring].
    pub dx: i16,
    pub dy: i16,
    /// +0x10 / +0x12 — width/height (may be overwritten by the 0x100/0x200 cross-fit).
    pub w: i16,
    pub h: i16,
    /// +0x14..+0x1a — the remaining four u16s.
    pub p14: u16,
    pub p16: u16,
    pub p18: u16,
    pub p1a: u16,
    /// +0x1c — init 0xffffffff [M].
    pub neg1: u32,
    /// +0x20 — text length slot (Menu_ClipString writes it).
    pub text_len: u16,
    /// +0x22 — (the game zeroes the +0x20 dword; this half is padding).
    pub pad22: u16,
    /// +0x24 — placed x [M Menu_InitializeSprite].
    pub x: u16,
    /// +0x26 — placed y [M Menu_InitializeSprite].
    pub y: u16,
    /// +0x28 — sprite kind; 7 = a menu sprite [M Menu_CreateSprite 0x1f2fa8 hardcodes 7,
    /// disasm `addiu $v0, $zero, 7` in the delay slot; the MENUSPRITETYPE arg is dead].
    pub kind: u8,
    /// +0x29..+0x2c — padding (the game's byte +0x28 write covers +0x29's neighbours).
    pub pad29: [u8; 3],
    /// +0x2c / +0x30 — the width/height fractions [M SetupInstance multiplies them].
    pub wf: f32,
    pub hf: f32,
    /// +0x34 — visibility/anchor flags [M SetupInstance's mask test].
    pub flags: u32,
}

impl Default for MenuSpriteInfo {
    fn default() -> Self {
        MenuSpriteInfo {
            hash: 0, pad04: 0, v06: 0, u1: 0xffff, z0a: 0,
            dx: 0, dy: 0, w: 0, h: 0, p14: 0, p16: 0, p18: 0, p1a: 0,
            neg1: 0xffff_ffff, text_len: 0, pad22: 0, x: 0, y: 0, kind: 0, pad29: [0; 3],
            wf: 0.0, hf: 0.0, flags: 0,
        }
    }
}

/// Component_InitInstance 0x1f1200 [M field-for-field].
#[allow(clippy::too_many_arguments)]
pub fn component_init_instance(
    i: &mut MenuSpriteInfo,
    wf: f32,
    hf: f32,
    u16s: [u16; 8],
    flags: u32,
    hash: u32,
) {
    i.wf = wf;
    i.hf = hf;
    i.dx = u16s[0] as i16;
    i.dy = u16s[1] as i16;
    i.w = u16s[2] as i16;
    i.h = u16s[3] as i16;
    i.p14 = u16s[4];
    i.p16 = u16s[5];
    i.p18 = u16s[6];
    i.p1a = u16s[7];
    i.flags = flags;
    i.hash = hash;
    i.u1 = 0xffff;
    i.neg1 = 0xffff_ffff;
    i.z0a = 0;
    i.text_len = 0;
    i.pad22 = 0;
    i.x = 0;
    i.y = 0;
    i.kind = 0;
    i.pad29 = [0; 3];
}

/// A component record — M_COMPONENT [M Component_AddComponent 0x1f1198 writes +0 hash,
/// +4 count, +6/+8/+0xa/+0xc a-d, +0x10 = malloc(count · 0x38)].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Component {
    pub hash: u32,
    pub count: i16,
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub instances: Vec<MenuSpriteInfo>,
}

/// Component_AddComponent 0x1f1198 [M: count 0 → no array].
pub fn component_add_component(a: u16, b: u16, c: u16, d: u16, hash: u32, count: i16) -> Component {
    let instances = if count == 0 {
        Vec::new()
    } else {
        // the game mallocs count · 0x38 [M]
        vec![MenuSpriteInfo::default(); count as usize]
    };
    Component { hash, count, a, b, c, d, instances }
}

/// A component set node — the game's component_list entry: 13 record slots
/// [M Component_Destroy 0x1f1368 walks 0xd], a set id [M GetHandle node+0x48 int-words]
/// and a page byte [M DestroyAll node+0x49 int-words].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComponentSet {
    pub set_id: u32,
    pub page: u8,
    pub records: Vec<Component>,
}

pub const COMPONENT_SLOTS: usize = 13;

/// Component_InitComponents 0x1f1038: LinkedList_Init(0x2dda70, 0) — the global registry
/// starts empty.
pub fn component_init_components() -> Vec<ComponentSet> {
    Vec::new()
}

/// Component_GetHandle 0x1f1268: find the set whose id == handle >> 0x10, return the
/// record at handle & 0xffff [M: record = node + 8 + index·0x14].
pub fn component_get_handle<'a>(list: &'a [ComponentSet], handle: u32) -> Option<&'a Component> {
    let set = handle >> 16;
    let ix = (handle & 0xffff) as usize;
    list.iter().find(|s| s.set_id == set).and_then(|s| s.records.get(ix))
}

/// Component_Free 0x1f12e0: delete every sprite the control owns, then drop the array.
pub fn component_free(sprites: &mut Vec<MenuSprite>) {
    sprites.clear();
}

/// Component_Destroy 0x1f1368: free each record's instances.
pub fn component_destroy(set: &mut ComponentSet) {
    for r in set.records.iter_mut() {
        r.instances.clear();
    }
}

/// Component_DestroyAll 0x1f10e8: drop every set whose page byte matches.
pub fn component_destroy_all(list: &mut Vec<ComponentSet>, page: u8) {
    list.retain(|s| s.page != page);
}

/// The alpha side effect of Component_SetupInstance — computed per visible instance
/// [M 0x1f0950: (mask & 0x20 == 0 || inst.wf != 0 || inst.hf != 0) →
/// mask & 0xc0 ? Menu_AlphaReset : nothing; else Menu_AlphaGet].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaOp {
    None,
    Get,
    Reset,
}

/// One placed sprite request — what SetupInstance hands Menu_InitializeSprite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedSprite {
    pub sprite_ix: usize,
    /// ctrl.layer − page.layer, u16-wrapped [M 0x1f0950's (u16−u16)·0x10000>>0x10].
    pub layer_delta: i16,
    /// page.x + param_x + anchor offset [M 0x1f0950's x branches].
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    /// hidden instances got the sprite's layer byte 0xff [M *(u8*)(sprite+0x38) = 0xff].
    pub hidden: bool,
    pub alpha: AlphaOp,
}

/// Component_SetupInstance 0x1f0950 — the layout math. `ctrl`/`page` are the control and
/// its page (rects +0x70..+0x76, layer +0x78); `record` = the control's component record;
/// `mask`/`x`/`y`/`w`/`h` are the five args (x/y = −1 → the control's own +0x70/+0x72,
/// w/h = −1 → +0x74/+0x76). Flag semantics [M 0x1f0950, quoted in
/// docs/systems/menu-engine.md]: 0x01 bottom-align y, 0x02 right-align x, 0x04 centre-x
/// (dx ignored), 0x08 centre-y, 0x100 w slot := hf · vertical span (or the sprite's current
/// h when hf == 0), 0x200 h slot := wf · horizontal span (or the w slot when wf == 0),
/// 0x400/0x800: a fixed instance doesn't grow/shrink the span, 2/1: right/bottom-anchored
/// fixed instances pull the span in. The span accumulators run sequentially per instance
/// in list order [M the decompile's statement order].
#[allow(clippy::too_many_arguments)]
pub fn component_setup_instance(
    ctrl: &MenuControl,
    page: &MenuControl,
    record: &mut Component,
    sprite_h: &[i16],
    mask: u32,
    x: i16,
    y: i16,
    w: i32,
    h: i32,
) -> Vec<PlacedSprite> {
    let px = x as i32;
    let py = y as i32;
    let aw = if w == -1 { ctrl.w as i32 } else { w };
    let ah = if h == -1 { ctrl.h as i32 } else { h };
    let mut acc_top: i32 = 0; // iStack_cc
    let mut acc_bot: i32 = aw; // iStack_c8
    let mut acc_left: i32 = 0; // iStack_c4
    let mut acc_right: i32 = ah; // iVar14
    let mut out = Vec::new();
    for (i, inst) in record.instances.iter_mut().enumerate() {
        if inst.flags & mask == 0 {
            out.push(PlacedSprite {
                sprite_ix: i, layer_delta: 0, x: 0, y: 0, w: 0, h: 0, hidden: true,
                alpha: AlphaOp::None,
            });
            continue;
        }
        let flags = inst.flags;
        // the instance's w/h slots (+0x10/+0x12), possibly overwritten by the cross-fit
        let mut inst_w_slot = inst.w;
        let mut inst_h_slot = inst.h;
        // cross-axis fits first [M the 0x100 then 0x200 blocks]
        if flags & 0x100 != 0 {
            inst_w_slot = if inst.hf == 0.0 {
                // the game copies the control SPRITE's current +0x2e (its placed h), not the
                // instance's +0x12 [M 0x1f0950 `*(sprites[i]) + 0x2e`]
                sprite_h.get(i).copied().unwrap_or(0)
            } else {
                ((acc_right - acc_left) as f32 * inst.hf) as i16
            };
        }
        if flags & 0x200 != 0 {
            inst_h_slot = if inst.wf == 0.0 {
                inst_w_slot // the game copies the just-written +0x10 [M]
            } else {
                ((acc_bot - acc_top) as f32 * inst.wf) as i16
            };
        }
        // the slot writes land in the record (+0x10/+0x12) and persist to the next call [M]
        inst.w = inst_w_slot;
        inst.h = inst_h_slot;
        // span-driven sizes [M the wf/hf branches with the (v<0→0, v>span→span) clamp]
        let w_f = if inst.wf == 0.0 {
            inst_w_slot as i32
        } else {
            let span = (acc_bot - acc_top) as f32;
            let v = span * inst.wf;
            (if v < 0.0 { 0 } else if v > span { span as i32 } else { v as i32 })
        };
        // horizontal span accumulation happens inside the wf == 0 arm [M]
        if inst.wf == 0.0 && flags & 0x400 == 0 {
            if flags & 2 == 0 {
                acc_top = acc_top.max(inst.dx as i32 + inst_w_slot as i32);
            } else {
                acc_bot = acc_bot.min((aw - inst.dx as i32) - inst_w_slot as i32);
            }
        }
        let h_f = if inst.hf == 0.0 {
            inst_h_slot as i32
        } else {
            let span = (acc_right - acc_left) as f32;
            let v = span * inst.hf;
            (if v < 0.0 { 0 } else if v > span { span as i32 } else { v as i32 })
        };
        if inst.hf == 0.0 && flags & 0x800 == 0 {
            if flags & 1 == 0 {
                acc_left = acc_left.max(inst.dy as i32 + inst_h_slot as i32);
            } else {
                acc_right = acc_right.min((ah - inst.dy as i32) - inst_h_slot as i32);
            }
        }
        // placement [M 0x1f0950's x/y anchor branches — final pos adds the PAGE rect]
        let x16: i16 = if flags & 2 != 0 {
            (page.x as i32 + px + (aw - w_f) - inst.dx as i32) as i16
        } else if flags & 4 != 0 {
            (page.x as i32 + px + ((aw - w_f) >> 1)) as i16
        } else {
            (page.x as i32 + px + inst.dx as i32) as i16
        };
        let y16: i16 = if flags & 1 != 0 {
            (page.y as i32 + py + (ah - h_f) - inst.dy as i32) as i16
        } else if flags & 8 != 0 {
            (page.y as i32 + py + ((ah - h_f) >> 1)) as i16
        } else {
            (page.y as i32 + py + inst.dy as i32) as i16
        };
        // [M 0x1f0950]: (mask & 0x20 == 0 || wf != 0 || hf != 0) → AlphaReset only when
        // mask & 0xc0, else nothing; the remaining case (0x20 set, no fractions) → AlphaGet.
        let alpha = if mask & 0x20 == 0 || inst.wf != 0.0 || inst.hf != 0.0 {
            if mask & 0xc0 != 0 {
                AlphaOp::Reset
            } else {
                AlphaOp::None
            }
        } else {
            AlphaOp::Get
        };
        out.push(PlacedSprite {
            sprite_ix: i,
            layer_delta: (ctrl.layer as i32 - page.layer as i32) as i16,
            x: x16,
            y: y16,
            w: w_f as i16,
            h: h_f as i16,
            hidden: false,
            alpha,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Menu sprites
// ---------------------------------------------------------------------------

/// The runtime menu sprite — the fields Menu_InitializeSprite 0x1f2870 writes
/// [M offsets in the doc; +0x30..+0x36 holds the UV quad AND the clip extents].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSprite {
    /// +0x10 — colour; 0 → 0x7f7f7fc0 [M 0x1f2870].
    pub colour: u32,
    /// +0x14 — param 10.
    pub p14: u32,
    /// +0x18/+0x1c/+0x20 — text pointer / font (&DAT_0030d170) / strlen-or-clip-count.
    pub text: u32,
    pub font: u32,
    pub text_len: u16,
    /// +0x22 — param 15.
    pub v22: u16,
    /// +0x24/+0x26 — position.
    pub x: i16,
    pub y: i16,
    /// +0x28 — the sprite kind (7 = menu sprite, set by Menu_CreateSprite [M disasm]).
    pub kind: u8,
    /// +0x2c/+0x2e — size.
    pub w: i16,
    pub h: i16,
    /// +0x30..+0x36 — the UV quad, written only when the UV param != -1; Menu_ClipString
    /// writes the measured extent into +0x34/+0x36 (uv[2]/uv[3]).
    pub uv: [i16; 4],
    /// +0x38 — draw layer.
    pub layer: u8,
    /// +0x39 — always 7 [M 0x1f2870].
    pub type7: u8,
    /// The texture hash (the game's hashtable_set_sprite arg [M 0x1f2870 param 8]).
    pub hash: u32,
}

impl Default for MenuSprite {
    fn default() -> Self {
        MenuSprite {
            colour: 0, p14: 0, text: 0, font: 0, text_len: 0, v22: 0,
            x: 0, y: 0, kind: 0, w: 0, h: 0, uv: [0; 4], layer: 0, type7: 7, hash: 0,
        }
    }
}

/// Menu_InitializeSprite 0x1f2870 [M field-for-field].
#[allow(clippy::too_many_arguments)]
pub fn menu_initialize_sprite(
    s: &mut MenuSprite,
    layer: u8,
    x: i16,
    y: i16,
    w: i16,
    h: i16,
    text: Option<u32>, // the text pointer; the port takes the length separately
    text_len: u16,
    hash: u32,
    colour: u32,
    p14: u32,
    uv: Option<[i16; 4]>,
    v22: u16,
) {
    s.layer = layer;
    s.x = x;
    s.y = y;
    s.w = w;
    s.h = h;
    s.type7 = 7;
    s.v22 = v22;
    if let Some(uv) = uv {
        s.uv = uv;
    }
    if let Some(text) = text {
        s.text = text;
        s.font = 0x30d170; // &DAT_0030d170 [M 0x1f2870]
        s.text_len = text_len;
    }
    s.colour = if colour == 0 { 0x7f7f_7fc0 } else { colour };
    s.p14 = p14;
    s.hash = hash;
}

/// Menu_CreateSprite 0x1f2fa8: no info → a fresh sprite (Sprite_Create); else kind = 7 +
/// create from it [M: the hardcoded `addiu $v0, $zero, 7`; MENUSPRITETYPE dead].
pub fn menu_create_sprite(existing: &mut Option<MenuSprite>) {
    match existing {
        None => *existing = Some(MenuSprite::default()),
        Some(s) => s.kind = 7,
    }
}

/// Menu_DeleteSprite 0x1f2fe8: null-guarded delete.
pub fn menu_delete_sprite(s: &mut Option<MenuSprite>) {
    *s = None;
}

/// Menu_ClipString 0x1f2e70. `measure` = the (Font_GetTextCountExtent, Font_GetTextExtent)
/// pair: given the width limit, return (count, w, h). Sets text_len (+0x20), the measured
/// extent +0x34/+0x36 (uv[2]/uv[3]), returns the width; 0 when the sprite has no text.
/// The port injects the measurer (the game's Font_* are not separately ported; the port's
/// glyph metrics live in nf-formats::hud [S]).
pub fn menu_clip_string(
    s: &mut MenuSprite,
    width_limit: i32,
    measure: impl Fn(i32) -> (u16, i32, i32),
) -> i32 {
    if s.text == 0 {
        return 0;
    }
    let (count, w, h) = measure(width_limit);
    s.text_len = count;
    s.uv[2] = w as i16;
    s.uv[3] = h as i16;
    w
}

// ---------------------------------------------------------------------------
// Combo widget
// ---------------------------------------------------------------------------

/// What Combo_SendMessage 0x1f0868 did with a message [M the switch].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComboMsg {
    /// 0x14 — Script_ExitLoop.
    ExitLoop,
    /// 0x33 — List_SendMessage(list child, 0x33).
    ListRefresh,
    /// 0x39 — nothing [M the empty case].
    Nothing,
    /// 0x41 — return +0x718 (rows − 1, the max item index).
    QueryMaxItem(i32),
    /// 0x47 — destroy: label/list 0x47, Component_Free, Script_DelAllScripts, Mem_Free(self).
    Destroy,
    /// 0x49/0x4b — forward to __Menu_SendMessage(page, msg, self, 0).
    Forward(u8),
    /// anything else — 0 [M the default].
    Unknown,
}

/// A combo widget — the game's M_COMBO, only what Combo_Init 0x1f0248 sets up [M].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Combo {
    /// +0x7b — 3 [M disasm `sb $v0, 0x7b($s0)`].
    pub ctype: u8,
    /// +0x7a — 0.
    pub disabled: u8,
    /// +0xc — the page control [M `sw $a2, 0xc($s0)`].
    pub parent: u32,
    /// +0x14 — (u16 NEW_CONTROL+0x14) << 16 | 2 [M disasm 0x1f0290].
    pub handle: u32,
    /// +0x18 — NEW_CONTROL+0x00 (opaque) / +0x20 — NEW_CONTROL+0x0c.
    pub v18: u32,
    pub v20: u32,
    /// +0x70..+0x76 — the rect from NEW_CONTROL +0x18/+0x1a/+0x1c/+0x1e.
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    /// +0x78 — the draw layer = u16 NEW_CONTROL+0x16 [M 0x1f0248 `*(u16*)(combo+0x78) =
    /// nc+0x16`; +0x78 is the layer every control carries — Menu_CursorOverMe/FindControl].
    pub layer: u16,
    /// +0x714 — the item text extent + 2.0 [M `fStack_cc + 2.0`].
    pub item_w: f32,
    /// +0x718 — (h ÷ row height) − 1; the row height is the control's own +0x5c4 float
    /// (set by the list machinery before Init) [M]; the port takes it as an argument [S].
    pub max_item: i32,
    /// The label child at +0x600 / the list child at +0x94 [M Label_Init/List_Init].
    pub label: MenuControl,
    pub list: MenuControl,
}

impl Combo {
    /// Combo_Init 0x1f0248. `new_control` = (v00, v0c, handle_hi, layer, x, y, w, h) — the
    /// NEW_CONTROL fields the game reads (+0x00 u32, +0x0c u32, +0x14 u16, +0x16 u16 layer,
    /// +0x18 x, +0x1a y, +0x1c w, +0x1e h). `item_extent` = the Font_GetTextExtent height-row
    /// value for the combo's charset (0x305c60/0x30d150). `row_h` = the control's +0x5c4 row
    /// height. `comp` = the component record's (+6, +8, +10) shorts (Component_GetHandle).
    ///
    /// [M 0x1f0248:77-109]: label child = the NEW_CONTROL with x += comp+6, layer − 1, h = the
    /// combo's +0x76 read BEFORE its :109 write (whatever the allocation left — the port takes 0 [S]);
    /// list child = x += comp+6, y += (int)item_w + comp+8 + 4, w −= (comp+6 − comp+10), h = 1,
    /// layer − 1; the combo's own rect = the NEW_CONTROL x/y/w with h = (int)item_w + 5.
    pub fn init(
        new_control: (u32, u32, u16, u16, i16, i16, i16, i16),
        item_extent: f32,
        row_h: f32,
        comp: (i16, i16, i16),
    ) -> Combo {
        let (v00, v0c, handle_hi, layer, x, y, w, h) = new_control;
        let (c6, c8, c10) = comp;
        let item_w = item_extent + 2.0; // [M :82 fStack_cc + 2.0]
        let iw = item_w as i32 as i16;
        let child_layer = (layer as i16).wrapping_sub(1);
        let mut c = Combo {
            ctype: TYPE_COMBO,
            disabled: 0,
            parent: 0,
            handle: ((handle_hi as u32) << 16) | 2,
            v18: v00,
            v20: v0c,
            x, y, w,
            h: iw.wrapping_add(5), // [M :109 +0x76 = (int)item_w + 5]
            layer,
            item_w,
            // [M :80 (int)((float)nc.h / +0x5c4) − 1]
            max_item: (h as f32 / row_h) as i32 - 1,
            label: MenuControl::default(),
            list: MenuControl::default(),
        };
        // Label child [M :84-91]
        c.label.x = x.wrapping_add(c6);
        c.label.y = y;
        c.label.w = w;
        c.label.h = 0;
        c.label.layer = child_layer;
        // List child [M :93-105]
        c.list.x = x.wrapping_add(c6);
        c.list.y = y.wrapping_add(iw).wrapping_add(c8).wrapping_add(4);
        c.list.w = w.wrapping_sub(c6.wrapping_sub(c10));
        c.list.h = 1;
        c.list.layer = child_layer;
        c
    }

    pub fn send_message(&mut self, msg: u8) -> ComboMsg {
        match msg {
            0x14 => ComboMsg::ExitLoop,
            0x33 => ComboMsg::ListRefresh,
            0x39 => ComboMsg::Nothing,
            0x41 => ComboMsg::QueryMaxItem(self.max_item),
            0x47 => ComboMsg::Destroy,
            0x49 | 0x4b => ComboMsg::Forward(msg),
            _ => ComboMsg::Unknown,
        }
    }
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

/// Menu_SetAction 0x1f3048: which pad slots receive the action. Slot −1 broadcasts to
/// pads 0..3 [M: four Input_SetAction calls], else just that pad.
pub fn menu_set_action_targets(slot: i16) -> Vec<u8> {
    if slot == -1 {
        vec![0, 1, 2, 3]
    } else {
        vec![slot as u8]
    }
}

/// Menu_GetLastController 0x1f30d8 — the whole body is `return 0` [M].
pub fn menu_get_last_controller() -> u8 {
    0
}

// ---------------------------------------------------------------------------
// Tests — every assertion pins decompile-backed behaviour ([M] cited inline); each has a
// sister that can fail. Pure data: no ISO needed.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl(x: i16, y: i16, w: i16, h: i16, t: u8) -> MenuControl {
        MenuControl { x, y, w, h, ctype: t, ..MenuControl::default() }
    }

    #[test]
    fn resize_control_writes_the_rect_slots() {
        // [M 0x1f2e30: +0x70..+0x76 = x, y, w; +0x76 = h]
        let mut c = ctrl(0, 0, 0, 0, 5);
        menu_resize_control(&mut c, 10, 20, 30, 40);
        assert_eq!((c.x, c.y, c.w, c.h), (10, 20, 30, 40));
        // sister: the copy form moves the same four slots [M 0x1f2e48]
        let mut d = ctrl(1, 2, 3, 4, 5);
        menu_resize_control_copy(&mut d, &c);
        assert_eq!((d.x, d.y, d.w, d.h), (10, 20, 30, 40));
        // sister: the copy form does NOT touch type/layer
        assert_eq!((d.ctype, d.layer), (5, 0));
    }

    #[test]
    fn assert_membership_is_the_disassembly_set() {
        // [M 0x1f2f18 disasm: sltiu (t-1) < 3 → {1,2,3} + {5,6,200,9,10,0xb,0xc,0x72}]
        for t in [1u8, 2, 3, 5, 6, 9, 10, 11, 12, 200, 114] {
            let c = MenuControl { ctype: t, parent: Some(1), ..Default::default() };
            assert!(menu_assert(Some(&c)), "type {} should assert", t);
        }
        // sister: t = 0 fails the UNSIGNED gate (the decompiled C would admit it)
        let z = MenuControl { ctype: 0, parent: Some(1), ..Default::default() };
        assert!(!menu_assert(Some(&z)));
        // sisters: null control, and the +0xc != 0 gate; page/container/combo-adjacent
        // types 4, 7, 8, 13 are outside the set
        assert!(!menu_assert(None));
        let orphan = MenuControl { ctype: 3, parent: None, ..Default::default() };
        assert!(!menu_assert(Some(&orphan)));
        for t in [4u8, 7, 8, 13] {
            let c = MenuControl { ctype: t, parent: Some(1), ..Default::default() };
            assert!(!menu_assert(Some(&c)), "type {} should not assert", t);
        }
    }

    #[test]
    fn get_page_walks_parents_to_the_type8_container() {
        // [M 0x1f27f8: walk +0xc to a type-8 control; assert-gated]
        let page = ctrl(0, 0, 640, 480, TYPE_PAGE_CONTAINER);
        let combo = MenuControl { ctype: TYPE_COMBO, parent: Some(0), ..Default::default() };
        let ctrls = vec![page, combo];
        assert_eq!(menu_get_page(&ctrls, 1), Some(0));
        // sister: no type-8 ancestor → 0
        let lone = MenuControl { ctype: TYPE_COMBO, parent: Some(2), ..Default::default() };
        let other = ctrl(0, 0, 10, 10, 5);
        let ctrls2 = vec![lone, other];
        assert_eq!(menu_get_page(&ctrls2, 0), None);
    }

    #[test]
    fn get_page_on_a_page_header_returns_zero_like_the_game() {
        // [M 0x1f27f8 + 0x1f2f18: the assert runs FIRST and type 0xd is not in its set,
        // so GetPage(page header) = 0 even though the type-0xd branch exists]
        let hdr = ctrl(0, 0, 640, 480, TYPE_PAGE_HEADER);
        let ctrls = vec![hdr];
        assert_eq!(menu_get_page(&ctrls, 0), None);
        // sister: even a header WITH a parent is None — 0xd is outside the assert set,
        // so the game's `if (type == 0xd) return +0x94` branch is unreachable [M]
        let mut hdr2 = ctrl(0, 0, 640, 480, TYPE_PAGE_HEADER);
        hdr2.parent = Some(1);
        let ctrls2 = vec![hdr2, ctrl(0, 0, 1, 1, 1)];
        assert_eq!(menu_get_page(&ctrls2, 0), None);
    }

    #[test]
    fn get_first_control_skips_disabled_and_headers() {
        // [M 0x1f2708: skip 0x7b == 0xd and +0x7a != 0; first clean control wins]
        let mut a = ctrl(0, 0, 10, 10, 5);
        a.disabled = 1;
        let mut b = ctrl(0, 20, 10, 10, TYPE_PAGE_HEADER);
        let mut c = ctrl(0, 40, 10, 10, 5);
        c.disabled = 1;
        let d = ctrl(0, 60, 10, 10, 5);
        a.next = Some(1);
        b.next = Some(2);
        c.next = Some(3);
        let ctrls = vec![a, b, c, d];
        assert_eq!(menu_get_first_control(&ctrls, Some(0)), Some(3));
        // sister: everything skipped → the backup control (BACKUP_* fields) [M 0x1f2708:8-14]
        let mut e = ctrl(0, 0, 10, 10, 5);
        e.disabled = 1;
        e.next = Some(1);
        let hdr = ctrl(0, 0, 10, 10, TYPE_PAGE_HEADER);
        let ctrls2 = vec![e, hdr];
        assert_eq!(menu_get_first_control(&ctrls2, Some(0)), None);
        assert_eq!(BACKUP_NAME, 0x1fff_ffff);
        assert_eq!(BACKUP_XY, -100);
    }

    #[test]
    fn cursor_over_me_takes_the_lowest_layer_hit() {
        // [M 0x1f13c8: rect containment + the min +0x78 layer among hits]
        let mut a = ctrl(0, 0, 100, 100, 5); // layer 0
        let mut b = ctrl(0, 0, 100, 100, 5);
        b.layer = 3;
        a.next = Some(1);
        let ctrls = vec![a, b];
        assert!(menu_cursor_over_me(&ctrls, Some(0), (50, 50), 0)); // lower layer wins
        assert!(!menu_cursor_over_me(&ctrls, Some(0), (50, 50), 1));
        // sister: a point in neither rect → false
        assert!(!menu_cursor_over_me(&ctrls, Some(0), (150, 50), 0));
        // sister: disabled controls are not hit
        let mut c = ctrl(0, 0, 100, 100, 5);
        c.disabled = 1;
        let ctrls2 = vec![c];
        assert!(!menu_cursor_over_me(&ctrls2, Some(0), (50, 50), 0));
    }

    #[test]
    fn find_control_up_scores_2dy_plus_point1_dx() {
        // [M 0x1f16d8 vertical scoring: 2·|Δy| + 0.1·|Δx|, gate = x-overlap + target above]
        let mut cur = ctrl(100, 100, 50, 20, 5);
        let mut above = ctrl(102, 40, 50, 20, 5); // dy = 60, dx = -2 → 120 + 0 (trunc 0.2)
        let mut below = ctrl(100, 200, 50, 20, 5);
        cur.next = Some(1);
        above.next = Some(2);
        below.next = None;
        let ctrls = vec![cur, above, below];
        assert_eq!(menu_find_control(&ctrls, Some(0), 0, FindDir::Up), Some(1));
        // sister: Down from `above` skips cur and reaches `below`… wait, Down's forward
        // gate is dy < th (target below); from ctrl 1 (y=40) cur 0 is below (dy2 = 40+20-100 = -40 <0
        // → backward/wrap). Use exact numbers instead:
        // From ctrl 1 Down: t=0: dy = 40-100 = -60, dy2 = 60-100 = -40 → forward gate
        // dy < th (−60 < 20 ✓) and x_overlap ✓ → score = 2·|−40| + 0.1·2 = 80.0 → 80.
        assert_eq!(menu_find_control(&ctrls, Some(0), 1, FindDir::Down), Some(0));
    }

    #[test]
    fn find_control_wraps_around_when_no_same_direction_candidate() {
        // [M 0x1f16d8: empty array1 → sort array2 (the wrap-around candidates)]
        let mut cur = ctrl(100, 100, 50, 20, 5);
        let mut below = ctrl(100, 300, 50, 20, 5); // not above, so Up has only a wrap hit
        cur.next = Some(1);
        let ctrls = vec![cur, below];
        assert_eq!(menu_find_control(&ctrls, Some(0), 0, FindDir::Up), Some(1));
        // sister: no candidates at all → stay (None)
        let alone = ctrl(0, 0, 50, 20, 5);
        let ctrls2 = vec![alone];
        assert_eq!(menu_find_control(&ctrls2, Some(0), 0, FindDir::Up), None);
    }

    #[test]
    fn find_control_horizontal_scores_plain_distance() {
        // [M 0x1f16d8 horizontal scoring: |Δx| + |Δy| — NOT the 2·|Δy| vertical weight]
        let mut cur = ctrl(100, 100, 50, 20, 5);
        let mut left = ctrl(20, 104, 50, 20, 5); // dx = 80, dy = -4 → 84
        let mut right = ctrl(200, 100, 50, 20, 5); // wrap for Left
        cur.next = Some(1);
        left.next = Some(2);
        let ctrls = vec![cur.clone(), left, right];
        assert_eq!(menu_find_control(&ctrls, Some(0), 0, FindDir::Left), Some(1));
        assert_eq!(menu_find_control(&ctrls, Some(0), 0, FindDir::Right), Some(2));
        // sister: zero-size controls (w·h == 0) are skipped [M 0x1f1798]
        let mut zero = ctrl(50, 100, 50, 0, 5);
        zero.next = None;
        let ctrls2 = vec![cur.clone(), zero];
        assert_eq!(menu_find_control(&ctrls2, Some(0), 0, FindDir::Left), None);
    }

    #[test]
    fn find_control_cursor_update_uses_the_page_rect() {
        // [M 0x1f1b74-0x1f1bf4: cursor x/y = page + ctrl + w/2, h/2; +2c/+2e = w+10/h+10;
        // +0x38 = layer + 1]
        let page = ctrl(5, 7, 640, 480, TYPE_PAGE_CONTAINER);
        let c = ctrl(10, 20, 50, 30, 5);
        assert_eq!(find_cursor_update(&c, &page), (5 + 10 + 25, 7 + 20 + 15, 60, 40, 1));
        // sister: different layer → +1 shifts
        let mut c2 = c.clone();
        c2.layer = 4;
        assert_eq!(find_cursor_update(&c2, &page).4, 5);
    }

    #[test]
    fn add_component_allocates_count_times_0x38() {
        // [M 0x1f1198: count · 0x38 instances, or none when count == 0]
        let c = component_add_component(1, 2, 3, 4, 0x1234, 3);
        assert_eq!(c.instances.len(), 3);
        assert_eq!(c.count, 3);
        assert_eq!(std::mem::size_of::<MenuSpriteInfo>(), 0x38);
        // sister: count 0 → empty
        let z = component_add_component(1, 2, 3, 4, 0x1234, 0);
        assert!(z.instances.is_empty());
    }

    #[test]
    fn init_instance_defaults_match_the_decompile() {
        // [M 0x1f1200: +8 = 0xffff, +0xa = 0, +0x1c = 0xffffffff, +0x20/+0x24 = 0,
        // byte +0x28 = 0; the u16 params land +0xc..+0x1a; wf/hf +0x2c/+0x30; flags +0x34]
        let mut i = MenuSpriteInfo::default();
        component_init_instance(&mut i, 0.5, 0.25, [1, 2, 3, 4, 5, 6, 7, 8], 0x9c0, 0xfeed);
        assert_eq!(i.u1, 0xffff);
        assert_eq!(i.z0a, 0);
        assert_eq!(i.neg1, 0xffff_ffff);
        assert_eq!(i.text_len, 0);
        assert_eq!(i.x, 0);
        assert_eq!(i.y, 0);
        assert_eq!(i.kind, 0);
        assert_eq!((i.dx, i.dy, i.w, i.h), (1, 2, 3, 4));
        assert_eq!((i.p14, i.p16, i.p18, i.p1a), (5, 6, 7, 8));
        assert_eq!((i.wf, i.hf), (0.5, 0.25));
        assert_eq!(i.flags, 0x9c0);
        assert_eq!(i.hash, 0xfeed);
        // sister: a second init overwrites everything (no state leaks)
        component_init_instance(&mut i, 0.0, 0.0, [0; 8], 0, 0);
        assert_eq!(i.wf, 0.0);
        assert_eq!(i.u1, 0xffff);
    }

    #[test]
    fn get_handle_resolves_set_and_record_index() {
        // [M 0x1f1268: match set id == handle >> 0x10, record index = handle & 0xffff]
        let mut set = ComponentSet { set_id: 0xab, page: 1, records: Vec::new() };
        set.records.resize(5, Component::default());
        set.records[2].hash = 0x77;
        let list = vec![set];
        let handle = (0xabu32 << 16) | 2;
        assert_eq!(component_get_handle(&list, handle).unwrap().hash, 0x77);
        // sister: wrong set id / out-of-range index → none
        assert!(component_get_handle(&list, (0xcd << 16) | 2).is_none());
        assert!(component_get_handle(&list, (0xab << 16) | 9).is_none());
    }

    #[test]
    fn setup_instance_default_placement_adds_page_and_dx() {
        // [M 0x1f0950 default x branch: x = page.x + param_x + inst.dx]
        let page = ctrl(0, 0, 640, 480, TYPE_PAGE_CONTAINER);
        let ctrlc = ctrl(10, 20, 100, 50, 5);
        let mut rec = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec.instances[0], 0.0, 0.0, [5, 6, 30, 10, 0, 0, 0, 0], 0x80, 0);
        let out = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0x80, 0, 0, -1, -1);
        assert_eq!(out.len(), 1);
        // [M] x = page.x + param_x + dx — the page rect, not the control's
        assert_eq!((out[0].x, out[0].y, out[0].w, out[0].h), (5, 6, 30, 10));
        assert!(!out[0].hidden);
        // sister: mask misses the flags → hidden with layer 0xff [M *(u8*)(sp+0x38)=0xff]
        let out2 = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0x10, 0, 0, -1, -1);
        assert!(out2[0].hidden);
    }

    #[test]
    fn setup_instance_anchor_flags_pick_the_branches() {
        // [M 0x1f0950: flag 2 right-align x = page.x + x + w − w' − dx; flag 4 centre-x
        // = page.x + x + (w − w')/2 and does NOT add dx]
        let page = ctrl(0, 0, 640, 480, TYPE_PAGE_CONTAINER);
        let ctrlc = ctrl(10, 20, 100, 50, 5);
        let mut rec = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec.instances[0], 0.0, 0.0, [5, 6, 30, 10, 0, 0, 0, 0], 0x80 | 2, 0);
        let out = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0x80, 0, 0, -1, -1);
        assert_eq!(out[0].x, (0 + 0 + 100 - 30 - 5) as i16); // page.x + x + aw − w' − dx
        // sister: centre ignores dx
        let mut rec2 = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec2.instances[0], 0.0, 0.0, [5, 6, 30, 10, 0, 0, 0, 0], 0x80 | 4, 0);
        let out2 = component_setup_instance(&ctrlc, &page, &mut rec2, &[], 0x80, 0, 0, -1, -1);
        assert_eq!(out2[0].x, (0 + (100 - 30) / 2) as i16);
        // sister: bottom-align y
        let mut rec3 = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec3.instances[0], 0.0, 0.0, [5, 6, 30, 10, 0, 0, 0, 0], 0x80 | 1, 0);
        let out3 = component_setup_instance(&ctrlc, &page, &mut rec3, &[], 0x80, 0, 0, -1, -1);
        assert_eq!(out3[0].y, (0 + 50 - 10 - 6) as i16);
    }

    #[test]
    fn setup_instance_fraction_width_clamps_to_the_span() {
        // [M 0x1f0950 wf branch: w = clamp(wf · span, 0, span), span = aw at start]
        let page = ctrl(0, 0, 640, 480, TYPE_PAGE_CONTAINER);
        let ctrlc = ctrl(10, 20, 100, 50, 5);
        let mut rec = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec.instances[0], 0.5, 0.0, [0, 0, 0, 0, 0, 0, 0, 0], 0x80, 0);
        let out = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0x80, 0, 0, -1, -1);
        assert_eq!(out[0].w, 50); // 0.5 · 100
        // sister: wf > 1 clamps to the span
        let mut rec2 = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec2.instances[0], 2.0, 0.0, [0, 0, 0, 0, 0, 0, 0, 0], 0x80, 0);
        let out2 = component_setup_instance(&ctrlc, &page, &mut rec2, &[], 0x80, 0, 0, -1, -1);
        assert_eq!(out2[0].w, 100);
    }

    #[test]
    fn setup_instance_alpha_op_follows_the_mask() {
        // [M 0x1f0950: mask & 0x20 == 0 or a fraction set → mask & 0xc0 ? Reset : nothing; else Get]
        let page = ctrl(0, 0, 640, 480, TYPE_PAGE_CONTAINER);
        let ctrlc = ctrl(10, 20, 100, 50, 5);
        let mut rec = component_add_component(0, 0, 0, 0, 0x55, 1);
        component_init_instance(&mut rec.instances[0], 0.0, 0.0, [0, 0, 5, 5, 0, 0, 0, 0], 0x10, 0);
        let out = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0x10, 0, 0, -1, -1);
        // mask 0x10 has no 0x20 bit → the first arm; 0xc0 unset → nothing
        assert_eq!(out[0].alpha, AlphaOp::None);
        let out2 = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0x30, 0, 0, -1, -1);
        assert_eq!(out2[0].alpha, AlphaOp::Get); // 0x20 set, wf == hf == 0 → Menu_AlphaGet
        let out3 = component_setup_instance(&ctrlc, &page, &mut rec, &[], 0xd0, 0, 0, -1, -1);
        assert_eq!(out3[0].alpha, AlphaOp::Reset); // 0xc0 set, no 0x20 → Reset
    }

    #[test]
    fn init_components_starts_empty_and_destroy_all_filters_by_page() {
        // [M 0x1f1038: LinkedList_Init(0x2dda70, 0)]
        let mut list = component_init_components();
        assert!(list.is_empty());
        // [M 0x1f10e8: keep sets whose page byte != the argument]
        list.push(ComponentSet { set_id: 1, page: 7, records: Vec::new() });
        list.push(ComponentSet { set_id: 2, page: 8, records: Vec::new() });
        component_destroy_all(&mut list, 7);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].set_id, 2);
        // sister: Component_Destroy empties the records' instances [M 0x1f1368]
        let mut set = ComponentSet { set_id: 3, page: 1, records: vec![component_add_component(0, 0, 0, 0, 0, 2)] };
        component_destroy(&mut set);
        assert!(set.records[0].instances.is_empty());
        assert_eq!(COMPONENT_SLOTS, 13);
    }

    #[test]
    fn combo_init_builds_the_handle_and_children() {
        // [M 0x1f0248: handle = (u16 NEW_CONTROL+0x14) << 16 | 2; type = 3; NEW_CONTROL+0x16 → +0x78 layer;
        // item extent + 2.0 → +0x714; h / row_h − 1 → +0x718]
        let c = Combo::init((9, 11, 0x1234, 4, 100, 60, 120, 40), 50.0, 20.0, (2, 30, 50));
        assert_eq!(c.ctype, TYPE_COMBO);
        assert_eq!(c.handle, (0x1234u32 << 16) | 2);
        assert_eq!(c.layer, 4);
        assert_eq!(c.item_w, 52.0);
        assert_eq!(c.max_item, 1); // 40 / 20 − 1
        assert_eq!((c.x, c.y, c.w, c.h), (100, 60, 120, 52 + 5)); // [M :106-109]
        // sister: the children carry the offsets the game computes [M :84-105]
        assert_eq!((c.label.x, c.label.y, c.label.layer), (102, 60, 3));
        assert_eq!((c.list.x, c.list.y, c.list.w, c.list.h), (102, 60 + 52 + 30 + 4, 120 - (2 - 50), 1));
    }

    #[test]
    fn combo_send_message_switch_matches_the_decompile() {
        // [M 0x1f0868: 0x14 exit-loop; 0x33 list refresh; 0x39 nothing; 0x41 → +0x718;
        // 0x47 destroy; 0x49/0x4b forward; default 0]
        let mut c = Combo::init((0, 0, 1, 4, 0, 0, 100, 40), 50.0, 20.0, (0, 0, 0));
        assert_eq!(c.send_message(0x41), ComboMsg::QueryMaxItem(1));
        assert_eq!(c.send_message(0x14), ComboMsg::ExitLoop);
        assert_eq!(c.send_message(0x33), ComboMsg::ListRefresh);
        assert_eq!(c.send_message(0x39), ComboMsg::Nothing);
        assert_eq!(c.send_message(0x47), ComboMsg::Destroy);
        assert_eq!(c.send_message(0x49), ComboMsg::Forward(0x49));
        assert_eq!(c.send_message(0x4b), ComboMsg::Forward(0x4b));
        // sister: unknown messages → Unknown (the game returns 0)
        assert_eq!(c.send_message(0x00), ComboMsg::Unknown);
        assert_eq!(c.send_message(0x42), ComboMsg::Unknown);
    }

    #[test]
    fn initialize_sprite_defaults_the_colour_and_type7() {
        // [M 0x1f2870: colour 0 → 0x7f7f7fc0; +0x39 = 7; UV only when param != -1;
        // text → +0x18/+0x1c = &DAT_0030d170/+0x20 = len]
        let mut s = MenuSprite::default();
        menu_initialize_sprite(&mut s, 3, 11, 22, 33, 44, None, 0, 0x77, 0, 0x99, Some([1, 2, 3, 4]), 6);
        assert_eq!(s.colour, 0x7f7f_7fc0);
        assert_eq!(s.type7, 7);
        assert_eq!(s.layer, 3);
        assert_eq!(s.uv, [1, 2, 3, 4]);
        assert_eq!(s.hash, 0x77);
        assert_eq!(s.p14, 0x99);
        assert_eq!(s.v22, 6);
        // sister: an explicit colour passes through; no UV → uv untouched
        let mut s2 = MenuSprite::default();
        menu_initialize_sprite(&mut s2, 0, 0, 0, 0, 0, Some(0x1000), 4, 0, 0x123456, 0, None, 0);
        assert_eq!(s2.colour, 0x123456);
        assert_eq!(s2.uv, [0, 0, 0, 0]);
        assert_eq!(s2.font, 0x30d170);
        assert_eq!(s2.text_len, 4);
    }

    #[test]
    fn create_and_delete_sprite_roundtrip() {
        // [M 0x1f2fa8: kind 7 on create-from-info; 0x1f2fe8: delete = drop]
        let mut s: Option<MenuSprite> = Some(MenuSprite::default());
        menu_create_sprite(&mut s);
        assert_eq!(s.as_ref().unwrap().kind, 7);
        // sister: creating from nothing makes a fresh sprite
        let mut none: Option<MenuSprite> = None;
        menu_create_sprite(&mut none);
        assert!(none.is_some());
        menu_delete_sprite(&mut none);
        assert!(none.is_none());
    }

    #[test]
    fn clip_string_sets_count_and_extent_only_with_text() {
        // [M 0x1f2e70: text != 0 → count → +0x20, extent → +0x34/+0x36, returns w; else 0]
        let mut s = MenuSprite { text: 0x1000, ..MenuSprite::default() };
        let w = menu_clip_string(&mut s, 100, |_lim| (7, 55, 12));
        assert_eq!(w, 55);
        assert_eq!(s.text_len, 7);
        assert_eq!((s.uv[2], s.uv[3]), (55, 12));
        // sister: no text → 0 and no writes
        let mut bare = MenuSprite::default();
        let w2 = menu_clip_string(&mut bare, 100, |_lim| (7, 55, 12));
        assert_eq!(w2, 0);
        assert_eq!(bare.text_len, 0);
    }

    #[test]
    fn set_action_broadcasts_minus_one_and_get_last_controller_is_zero() {
        // [M 0x1f3048: slot −1 → Input_SetAction on pads 0..3, else only that pad]
        assert_eq!(menu_set_action_targets(-1), vec![0, 1, 2, 3]);
        assert_eq!(menu_set_action_targets(2), vec![2]);
        // sister: a specific slot does NOT touch the others
        assert_ne!(menu_set_action_targets(1), vec![0, 1, 2, 3]);
        // [M 0x1f30d8: the body is `return 0`]
        assert_eq!(menu_get_last_controller(), 0);
    }

    #[test]
    fn malloc_tag_is_the_menu_allocator_tag() {
        // [M 0x1f3008: Mem_Malloc(size, 0x3104, 0)]
        assert_eq!(MENU_MALLOC_TAG, 0x3104);
    }
}
