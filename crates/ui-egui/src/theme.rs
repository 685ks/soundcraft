//! Design tokens. The default dark palette ("Arthavan Dark") is the Arthavan Projects house theme,
//! built from `craft-tokens` (685ks/craft-libs): ink and muted text on near-black blue surfaces,
//! a steel-blue accent, a gold playhead and the house status colours for meters and automation.
//! The Light palette's colours were chosen by eye; they are ours.

use craft_tokens::Rgba;
use craft_tokens::color as house;
use egui::{Color32, FontFamily, FontId};
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the Light palette is active. Dark is the default (the incumbent is dark-only).
static LIGHT_THEME: AtomicBool = AtomicBool::new(false);

/// Unit tests that change or read the process-wide palette must own it until their assertions
/// finish. The lock is re-entrant per thread, so a test may hold it and still build apps
/// (`SoundApp::new` takes it too) without deadlocking itself.
#[cfg(test)]
pub(crate) fn lock_test_palette() -> TestPaletteGuard {
    TestPaletteGuard::acquire()
}

/// RAII handle for the re-entrant, per-thread test palette lock; see [`lock_test_palette`].
#[cfg(test)]
pub(crate) struct TestPaletteGuard;

#[cfg(test)]
impl TestPaletteGuard {
    fn acquire() -> Self {
        palette_lock::acquire();
        Self
    }
}

#[cfg(test)]
impl Drop for TestPaletteGuard {
    fn drop(&mut self) {
        palette_lock::release();
    }
}

#[cfg(test)]
mod palette_lock {
    use std::cell::{Cell, RefCell};
    use std::sync::{Mutex, MutexGuard, PoisonError};

    static PALETTE: Mutex<()> = Mutex::new(());

    thread_local! {
        /// How many guards this thread holds; the OS lock is taken on 0→1 and released on 1→0.
        static DEPTH: Cell<usize> = const { Cell::new(0) };
        static GUARD: RefCell<Option<MutexGuard<'static, ()>>> = const { RefCell::new(None) };
    }

    pub(super) fn acquire() {
        DEPTH.with(|depth| {
            if depth.get() == 0 {
                GUARD.with(|guard| *guard.borrow_mut() = Some(PALETTE.lock().unwrap_or_else(PoisonError::into_inner)));
            }
            depth.set(depth.get() + 1);
        });
    }

    pub(super) fn release() {
        DEPTH.with(|depth| {
            let left = depth.get().saturating_sub(1);
            depth.set(left);
            if left == 0 {
                GUARD.with(|guard| *guard.borrow_mut() = None);
            }
        });
    }
}

/// The Appearance setting (`ui.theme {mode}`). Dark by default; System and Light are opt-in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub const ALL: [Self; 3] = [Self::Dark, Self::Light, Self::System];

    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Arthavan Dark",
        }
    }

    /// The id used in `ui.theme {"mode": …}` and in the saved UI prefs.
    pub const fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.id().eq_ignore_ascii_case(s.trim()))
    }
}

/// The house accent for fills under text and for strokes: steel blue between the house
/// `accent.low` and `accent.base`, dark enough for ink text and bright enough on the surfaces.
const ACCENT: Rgba = mix(house::ACCENT_BASE, 64, house::ACCENT_LOW);

/// An opaque house colour as an egui colour.
const fn c(x: Rgba) -> Color32 {
    Color32::from_rgb(x.r, x.g, x.b)
}

/// `top` at opacity `alpha` (0..=255) composited over opaque `base`.
const fn mix(top: Rgba, alpha: u8, base: Rgba) -> Rgba {
    const fn ch(t: u8, b: u8, a: u8) -> u8 {
        let v = (t as u32 * a as u32 + b as u32 * (255 - a as u32) + 127) / 255;
        if v > 255 { 255 } else { v as u8 }
    }
    Rgba::rgb(ch(top.r, base.r, alpha), ch(top.g, base.g, alpha), ch(top.b, base.b, alpha))
}

/// A translucent house colour (its own alpha) over opaque `base`.
const fn over(top: Rgba, base: Rgba) -> Rgba {
    mix(top, top.a, base)
}

/// `base` lifted by white at opacity `alpha`: the house surface-fill elevations.
const fn lift(base: Rgba, alpha: u8) -> Rgba {
    mix(Rgba::rgb(255, 255, 255), alpha, base)
}

#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub window_bg: Color32,
    pub panel_bg: Color32,
    pub panel_bg2: Color32,
    pub toolbar_bg: Color32,
    pub toolbar_group: Color32,
    pub border: Color32,
    pub border_light: Color32,
    /// The lit top/left edge of raised panels: track headers and Mix strips.
    pub edge_light: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_dark: Color32,
    pub header_text: Color32,
    pub counter_bg: Color32,
    pub counter_text: Color32,
    pub counter_label: Color32,
    pub accent: Color32,
    pub accent_dark: Color32,
    pub mode_on: Color32,
    pub mode_on_text: Color32,
    pub mode_off_text: Color32,
    pub button: Color32,
    pub button_hi: Color32,
    pub button_border: Color32,
    pub name_field: Color32,
    pub name_field_sel: Color32,
    pub ruler_bg: Color32,
    pub ruler_label_bg: Color32,
    pub ruler_text: Color32,
    pub ruler_tick: Color32,
    pub tempo_ruler: Color32,
    pub meter_ruler: Color32,
    pub marker_ruler: Color32,
    pub playlist_bg: Color32,
    pub playlist_alt: Color32,
    pub grid_line: Color32,
    pub bar_line: Color32,
    pub selection: Color32,
    pub playhead: Color32,
    pub insertion: Color32,
    pub solo: Color32,
    pub mute: Color32,
    pub rec: Color32,
    pub input: Color32,
    pub auto_read: Color32,
    pub auto_write: Color32,
    pub auto_touch: Color32,
    pub auto_latch: Color32,
    pub meter_green: Color32,
    pub meter_yellow: Color32,
    pub meter_red: Color32,
    pub fader_track: Color32,
    pub strip_bg: Color32,
    pub strip_section: Color32,
    pub slot_bg: Color32,
    pub automation_line: Color32,
    /// Menu bar background.
    pub menu_bar: Color32,
    /// Outline of toolbar groups.
    pub group_border: Color32,
    /// Edit-mode cell, not selected.
    pub mode_off: Color32,
    /// Counter-panel toggle (Grid, Count Off), off.
    pub counter_off: Color32,
    /// Outline of the counter panels.
    pub counter_border: Color32,
    /// Divider inside the counter panels.
    pub counter_rule: Color32,
    /// Smart-tool bracket, off.
    pub tool_bracket_off: Color32,
    /// Selected row in the Tracks, Groups and Clips lists.
    pub row_selected: Color32,
    /// Track-visible dot in the Tracks list.
    pub visible_dot: Color32,
    /// Big Counter background.
    pub big_counter_bg: Color32,
    /// Edit-window track header, selected.
    pub header_selected: Color32,
    /// Mix-window strip, selected.
    pub strip_selected: Color32,
    /// Marker names on the marker ruler.
    pub marker_text: Color32,
    /// Alternate-playlist lanes.
    pub playlist_lane: Color32,
    /// Universe overview background.
    pub universe_bg: Color32,
    /// Universe visible-area outline.
    pub universe_view: Color32,
    /// Meter background.
    pub meter_bg: Color32,
    /// Meter clip indicator, not clipped.
    pub meter_clip_off: Color32,
    /// Pan knob body.
    pub knob_bg: Color32,
    /// Pan knob outline.
    pub knob_ring: Color32,
    /// Pan knob pointer.
    pub knob_pointer: Color32,
    /// Mix-window section captions (INSERTS A-E, I/O…).
    pub section_label: Color32,
    /// Plugin slots, as Pro Tools draws them: the well they sit in, an empty slot, a used slot's
    /// face (top, bottom), a bypassed one's, and the dots, letters and arrows on them.
    pub slot_well: Color32,
    pub slot_empty: Color32,
    pub slot_face: [Color32; 2],
    pub slot_bypassed: [Color32; 2],
    pub slot_mark: Color32,
}

impl Tokens {
    /// The house palette, "Arthavan Dark" (see the module docs).
    pub const DARK: Tokens = Tokens {
        window_bg: c(house::BG_BASE),
        panel_bg: c(house::BG_SHEET),
        panel_bg2: c(lift(house::BG_SHEET, 14)),
        toolbar_bg: c(house::BG_SKY_BOTTOM),
        toolbar_group: c(house::BG_BASE),
        border: c(house::BG_SKY_TOP),
        border_light: c(over(house::BORDER_LINE, house::BG_SHEET)),
        edge_light: Color32::from_rgba_premultiplied(20, 20, 20, 20),
        text: c(house::TEXT_INK),
        text_dim: c(house::TEXT_MUTED),
        text_dark: c(house::TEXT_ON_ACCENT),
        header_text: c(house::TEXT_STRONG),
        counter_bg: c(Rgba::rgb(3, 4, 6)),
        counter_text: c(house::STATUS_OK),
        counter_label: c(house::TEXT_MUTED),
        accent: c(ACCENT),
        accent_dark: c(house::ACCENT_LOW),
        mode_on: c(house::STATUS_OK),
        mode_on_text: c(house::TEXT_ON_ACCENT),
        mode_off_text: c(house::STATUS_OK),
        button: c(lift(house::BG_SHEET, 22)),
        button_hi: c(lift(house::BG_SHEET, 36)),
        button_border: c(house::BG_BASE),
        name_field: c(house::TEXT_STRONG),
        name_field_sel: c(house::TEXT_INK),
        ruler_bg: c(lift(house::BG_SHEET, 10)),
        ruler_label_bg: c(lift(house::BG_SHEET, 10)),
        ruler_text: c(house::TEXT_MUTED),
        ruler_tick: c(mix(house::TEXT_FAINT, 160, house::BG_SHEET)),
        tempo_ruler: c(mix(house::STATUS_OK, 90, house::BG_SHEET)),
        meter_ruler: c(mix(house::ACCENT_BASE, 90, house::BG_SHEET)),
        marker_ruler: c(lift(house::BG_SHEET, 16)),
        playlist_bg: c(lift(house::BG_SHEET, 6)),
        playlist_alt: c(lift(house::BG_SHEET, 12)),
        grid_line: c(lift(house::BG_SHEET, 20)),
        bar_line: c(lift(house::BG_SHEET, 32)),
        selection: Color32::from_rgba_premultiplied(50, 64, 76, 90),
        playhead: c(house::ACCENT_GOLD),
        insertion: c(house::ACCENT_BRIGHT),
        solo: c(house::STATUS_WARN),
        mute: Color32::from_rgb(232, 148, 40),
        rec: c(house::TIER_RED),
        input: c(house::TIER_GREEN),
        auto_read: c(house::STATUS_OK),
        auto_write: c(house::TIER_RED),
        auto_touch: c(house::STATUS_WARN),
        auto_latch: c(house::TIER_PURPLE_T1),
        meter_green: c(house::STATUS_OK),
        meter_yellow: c(house::STATUS_WARN),
        meter_red: c(house::TIER_RED),
        fader_track: c(house::BG_BASE),
        strip_bg: c(lift(house::BG_SHEET, 14)),
        strip_section: c(lift(house::BG_SHEET, 6)),
        slot_bg: c(house::BG_SKY_BOTTOM),
        automation_line: c(house::TEXT_INK),
        menu_bar: c(house::BG_SKY_BOTTOM),
        group_border: c(Rgba::rgb(2, 3, 5)),
        mode_off: c(mix(house::STATUS_OK, 24, house::BG_BASE)),
        counter_off: c(mix(house::STATUS_OK, 40, house::BG_BASE)),
        counter_border: c(over(house::BORDER_LINE, house::BG_SHEET)),
        counter_rule: c(lift(house::BG_BASE, 18)),
        tool_bracket_off: c(over(house::BORDER_LINE, house::BG_SHEET)),
        row_selected: c(mix(house::ACCENT_BASE, 60, house::BG_SHEET)),
        visible_dot: c(house::TEXT_MUTED),
        big_counter_bg: c(house::BG_BASE),
        header_selected: c(mix(house::ACCENT_BASE, 40, lift(house::BG_SHEET, 14))),
        strip_selected: c(mix(house::ACCENT_BASE, 40, lift(house::BG_SHEET, 14))),
        marker_text: c(house::TEXT_INK),
        playlist_lane: c(house::BG_SKY_BOTTOM),
        universe_bg: c(house::BG_BASE),
        universe_view: c(house::TEXT_INK),
        meter_bg: c(house::BG_SKY_TOP),
        meter_clip_off: c(mix(house::TIER_RED, 50, house::BG_BASE)),
        knob_bg: c(house::BG_SKY_BOTTOM),
        knob_ring: c(house::TEXT_FAINT),
        knob_pointer: c(house::TEXT_INK),
        section_label: c(house::TEXT_MUTED),
        slot_well: c(lift(house::BG_SHEET, 14)),
        slot_empty: c(lift(house::BG_SHEET, 10)),
        slot_face: [c(lift(house::BG_SHEET, 60)), c(lift(house::BG_SHEET, 44))],
        slot_bypassed: [c(mix(house::ACCENT_BASE, 110, house::BG_SHEET)), c(mix(house::ACCENT_BASE, 80, house::BG_SHEET))],
        slot_mark: c(house::TEXT_MUTED),
    };

    pub const LIGHT: Tokens = Tokens {
        window_bg: Color32::from_rgb(184, 186, 190),
        panel_bg: Color32::from_rgb(204, 206, 210),
        panel_bg2: Color32::from_rgb(216, 218, 222),
        toolbar_bg: Color32::from_rgb(194, 196, 200),
        toolbar_group: Color32::from_rgb(208, 210, 214),
        border: Color32::from_rgb(100, 103, 109),
        border_light: Color32::from_rgb(150, 153, 159),
        edge_light: Color32::from_rgba_premultiplied(150, 150, 150, 150),
        text: Color32::from_rgb(24, 25, 28),
        text_dim: Color32::from_rgb(68, 70, 76),
        text_dark: Color32::from_rgb(16, 17, 20),
        header_text: Color32::from_rgb(30, 32, 36),
        counter_bg: Color32::from_rgb(224, 226, 228),
        counter_text: Color32::from_rgb(18, 108, 40),
        counter_label: Color32::from_rgb(54, 57, 63),
        accent: Color32::from_rgb(62, 116, 180),
        accent_dark: Color32::from_rgb(43, 88, 145),
        mode_on: Color32::from_rgb(75, 158, 88),
        mode_on_text: Color32::from_rgb(255, 255, 255),
        mode_off_text: Color32::from_rgb(38, 106, 52),
        button: Color32::from_rgb(214, 216, 220),
        button_hi: Color32::from_rgb(200, 203, 208),
        button_border: Color32::from_rgb(108, 111, 117),
        name_field: Color32::from_rgb(236, 237, 239),
        name_field_sel: Color32::from_rgb(224, 232, 244),
        ruler_bg: Color32::from_rgb(198, 200, 204),
        ruler_label_bg: Color32::from_rgb(198, 200, 204),
        ruler_text: Color32::from_rgb(48, 50, 56),
        ruler_tick: Color32::from_rgb(108, 111, 118),
        tempo_ruler: Color32::from_rgb(92, 156, 112),
        meter_ruler: Color32::from_rgb(80, 144, 172),
        marker_ruler: Color32::from_rgb(186, 189, 194),
        playlist_bg: Color32::from_rgb(210, 212, 216),
        playlist_alt: Color32::from_rgb(202, 204, 208),
        grid_line: Color32::from_rgb(188, 191, 196),
        bar_line: Color32::from_rgb(162, 166, 172),
        selection: Color32::from_rgba_premultiplied(100, 145, 205, 90),
        playhead: Color32::from_rgb(210, 48, 42),
        insertion: Color32::from_rgb(35, 36, 40),
        solo: Color32::from_rgb(178, 145, 20),
        mute: Color32::from_rgb(194, 106, 22),
        rec: Color32::from_rgb(190, 42, 38),
        input: Color32::from_rgb(45, 126, 64),
        auto_read: Color32::from_rgb(52, 142, 68),
        auto_write: Color32::from_rgb(190, 50, 45),
        auto_touch: Color32::from_rgb(180, 122, 24),
        auto_latch: Color32::from_rgb(118, 80, 170),
        meter_green: Color32::from_rgb(36, 160, 58),
        meter_yellow: Color32::from_rgb(190, 162, 20),
        meter_red: Color32::from_rgb(195, 42, 35),
        fader_track: Color32::from_rgb(120, 123, 129),
        strip_bg: Color32::from_rgb(198, 200, 204),
        strip_section: Color32::from_rgb(210, 212, 216),
        slot_bg: Color32::from_rgb(194, 196, 201),
        automation_line: Color32::from_rgb(30, 32, 38),
        menu_bar: Color32::from_rgb(204, 206, 210),
        group_border: Color32::from_rgb(118, 121, 127),
        mode_off: Color32::from_rgb(202, 212, 204),
        counter_off: Color32::from_rgb(192, 212, 198),
        counter_border: Color32::from_rgb(118, 121, 127),
        counter_rule: Color32::from_rgb(188, 191, 196),
        tool_bracket_off: Color32::from_rgb(172, 175, 181),
        row_selected: Color32::from_rgb(174, 196, 226),
        visible_dot: Color32::from_rgb(86, 89, 96),
        big_counter_bg: Color32::from_rgb(224, 226, 228),
        header_selected: Color32::from_rgb(184, 200, 224),
        strip_selected: Color32::from_rgb(184, 200, 224),
        marker_text: Color32::from_rgb(24, 25, 28),
        playlist_lane: Color32::from_rgb(200, 202, 206),
        universe_bg: Color32::from_rgb(198, 200, 204),
        universe_view: Color32::from_rgb(40, 42, 48),
        meter_bg: Color32::from_rgb(52, 54, 58),
        meter_clip_off: Color32::from_rgb(92, 54, 54),
        knob_bg: Color32::from_rgb(232, 233, 236),
        knob_ring: Color32::from_rgb(116, 120, 128),
        knob_pointer: Color32::from_rgb(30, 32, 38),
        section_label: Color32::from_rgb(56, 58, 64),
        slot_well: Color32::from_rgb(212, 214, 218),
        slot_empty: Color32::from_rgb(198, 200, 205),
        slot_face: [Color32::from_rgb(244, 245, 247), Color32::from_rgb(214, 216, 220)],
        slot_bypassed: [Color32::from_rgb(190, 210, 240), Color32::from_rgb(160, 186, 228)],
        slot_mark: Color32::from_rgb(92, 95, 102),
    };

    pub fn current() -> Tokens {
        if is_light() { Self::LIGHT } else { Self::DARK }
    }
}

/// Whether the Light palette is the active one.
pub fn is_light() -> bool {
    LIGHT_THEME.load(Ordering::Relaxed)
}

/// Resolve each frame from the integration's current OS appearance. Unknown uses Dark.
pub fn wants_light(ctx: &egui::Context, mode: ThemeMode) -> bool {
    match mode {
        ThemeMode::Light => true,
        ThemeMode::Dark => false,
        ThemeMode::System => ctx.system_theme() == Some(egui::Theme::Light),
    }
}

/// The house small corner radius, in whole points.
const HOUSE_RADIUS: u8 = craft_tokens::radius::XS as u8;

/// A track's header and lane backgrounds in the Edit window, tinted with its colour by `tint` (0..1).
pub fn track_backgrounds(color: [u8; 3], selected: bool, tint: f32) -> (Color32, Color32) {
    let t = Tokens::current();
    let k = tint.clamp(0.0, 1.0);
    if k <= 0.0 {
        return if selected { (t.header_selected, t.playlist_alt) } else { (t.panel_bg2, t.playlist_bg) };
    }
    let c = rgb(color);
    let header = t.panel_bg2.lerp_to_gamma(c, k * 0.55);
    let header = if selected { crate::widgets::shade(header, 0.1) } else { header };
    let lane = if selected { t.playlist_alt } else { t.playlist_bg }.lerp_to_gamma(c, k * 0.2);
    (header, lane)
}

/// Track colours tinted for clip bodies.
pub fn clip_colors(rgb: [u8; 3], selected: bool) -> (Color32, Color32, Color32) {
    let [r, g, b] = rgb;
    let shade = |k: f32| Color32::from_rgb((f32::from(r) * k) as u8, (f32::from(g) * k) as u8, (f32::from(b) * k) as u8);
    let tint = |k: f32| {
        let t = |c: u8| (f32::from(c) + (255.0 - f32::from(c)) * k) as u8;
        Color32::from_rgb(t(r), t(g), t(b))
    };
    let bar = shade(0.9);
    if is_light() {
        // A paler body and a waveform in a deep shade of the clip's own colour.
        return if selected { (tint(0.2), bar, shade(0.22)) } else { (tint(0.45), bar, shade(0.32)) };
    }
    // A dark body and a light waveform in the clip's own colour; selection inverts them.
    if selected { (shade(0.7), bar, Color32::from_rgb(16, 16, 18)) } else { (shade(0.34), bar, tint(0.35)) }
}

pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Make `light` (or dark) the active palette and apply its visuals. Call only when it changes.
pub fn apply(ctx: &egui::Context, light: bool) {
    LIGHT_THEME.store(light, Ordering::Relaxed);
    let t = Tokens::current();
    let mut v = if light { egui::Visuals::light() } else { egui::Visuals::dark() };
    v.panel_fill = t.panel_bg;
    v.window_fill = t.panel_bg2;
    v.extreme_bg_color = t.slot_bg;
    v.faint_bg_color = t.panel_bg2;
    v.selection.bg_fill = t.accent;
    v.widgets.inactive.weak_bg_fill = t.button;
    v.widgets.inactive.bg_fill = t.button;
    v.widgets.hovered.weak_bg_fill = t.button_hi;
    v.widgets.noninteractive.fg_stroke.color = t.text;
    v.widgets.inactive.fg_stroke.color = t.text;
    // House radii (craft-tokens `radius`): small for menus, twice that for windows.
    v.window_corner_radius = egui::CornerRadius::same(HOUSE_RADIUS * 2);
    v.menu_corner_radius = egui::CornerRadius::same(HOUSE_RADIUS);
    // Keep receiving native OS appearance changes even when our palette is manual. On macOS,
    // pinning egui's theme also pins the window appearance and hides later OS changes.
    ctx.set_theme(egui::ThemePreference::System);
    // Both egui style branches must use our resolved palette: egui can select either branch
    // as the OS changes, including while SoundCraft's manual Light/Dark choice stays fixed.
    ctx.all_styles_mut(|s| {
        s.visuals = v.clone();
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(6.0, 2.0);
        s.interaction.tooltip_delay = 0.4;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Services, SoundApp, UiState};
    use serde_json::json;

    #[test]
    fn test_palette_lock_is_reentrant_and_excludes_other_threads() {
        // Regression for the parallel-run flakes: the process-wide palette must be owned for a
        // whole test, re-entrant on the owning thread (so `SoundApp::new` can take it too), and
        // exclusive against other threads (so a frame cannot flip it mid-test).
        let outer = lock_test_palette();
        let inner = lock_test_palette();
        let app = SoundApp::new(soundcraft_engine::Engine::default(), None, Services::default());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _held = lock_test_palette();
            let _ = tx.send(());
        });
        assert!(rx.recv_timeout(std::time::Duration::from_millis(100)).is_err(), "another thread must wait for the palette lock");
        drop(inner);
        drop(outer);
        drop(app);
        // Other tests compete for the lock too, and some hold it for seconds: allow for a queue.
        assert!(rx.recv_timeout(std::time::Duration::from_secs(120)).is_ok(), "the palette lock must be released when every holder drops it");
    }

    /// WCAG contrast ratio of two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lum = |c: Color32| {
            let lin = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
        };
        let (x, y) = (lum(a) + 0.05, lum(b) + 0.05);
        x.max(y) / x.min(y)
    }

    #[test]
    fn the_default_dark_palette_is_the_house_theme_from_craft_tokens() {
        let t = Tokens::DARK;
        assert_eq!(ThemeMode::Dark.label(), "Arthavan Dark");
        assert_eq!(t.window_bg, c(house::BG_BASE));
        assert_eq!(t.panel_bg, c(house::BG_SHEET));
        assert_eq!(t.text, c(house::TEXT_INK));
        assert_eq!(t.text_dim, c(house::TEXT_MUTED));
        assert_eq!(t.playhead, c(house::ACCENT_GOLD));
        assert_eq!((t.meter_green, t.meter_yellow, t.meter_red), (c(house::STATUS_OK), c(house::STATUS_WARN), c(house::TIER_RED)));
        // Readable text (WCAG AA) on every surface it sits on.
        for bg in [t.window_bg, t.panel_bg, t.panel_bg2, t.toolbar_bg, t.button, t.button_hi, t.strip_bg, t.playlist_bg, t.ruler_bg] {
            assert!(contrast(t.text, bg) >= 7.0, "ink on {bg:?}: {}", contrast(t.text, bg));
            assert!(contrast(t.text_dim, bg) >= 4.5, "muted on {bg:?}: {}", contrast(t.text_dim, bg));
        }
        assert!(contrast(t.text, t.accent) >= 4.5, "ink on the accent: {}", contrast(t.text, t.accent));
        assert!(contrast(t.text_dark, t.mode_on) >= 4.5);
        assert!(contrast(t.text_dark, t.name_field) >= 4.5);
        assert!(contrast(t.playhead, t.playlist_bg) >= 4.5);
        assert!(contrast(t.counter_text, t.counter_bg) >= 4.5);
    }

    #[test]
    fn dark_is_the_default() {
        assert_eq!(ThemeMode::default(), ThemeMode::Dark);
        assert_eq!(UiState::default().theme, ThemeMode::Dark);
        // Prefs saved before the setting existed load as Dark.
        let old: UiState = serde_json::from_value(json!({"show_tracks_list": true})).unwrap_or_default();
        assert_eq!(old.theme, ThemeMode::Dark);
    }

    #[test]
    fn prefs_saved_before_track_tint_get_the_default() {
        let ui: UiState = serde_json::from_value(json!({"theme": "light"})).unwrap();
        assert!((ui.track_tint - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn modes_round_trip_through_prefs() {
        for mode in ThemeMode::ALL {
            assert_eq!(ThemeMode::parse(mode.id()), Some(mode));
            let ui = UiState { theme: mode, ..UiState::default() };
            let back: UiState = serde_json::from_value(serde_json::to_value(&ui).unwrap_or_default()).unwrap_or_default();
            assert_eq!(back.theme, mode);
        }
        assert_eq!(ThemeMode::parse(" Light "), Some(ThemeMode::Light));
        assert_eq!(ThemeMode::parse("sepia"), None);
    }

    #[test]
    fn theme_command_sets_and_reports_the_mode() {
        let mut app = SoundApp::new(soundcraft_engine::Engine::default(), None, Services::default());
        assert_eq!(app.run("ui.theme", json!({})), Ok(json!({"mode": "dark"})));
        assert_eq!(app.run("ui.theme", json!({"mode": "light"})), Ok(json!({"mode": "light"})));
        assert_eq!(app.ui.theme, ThemeMode::Light);
        assert!(app.run("ui.theme", json!({"mode": "sepia"})).is_err());
        assert_eq!(app.ui.theme, ThemeMode::Light);
        assert_eq!(app.run("ui.theme", json!({"mode": "system"})), Ok(json!({"mode": "system"})));
        assert!(crate::menus::UI_COMMANDS.iter().any(|c| c.0 == "ui.theme"));
    }

    #[test]
    fn application_frames_follow_system_appearance_and_preserve_manual_palettes() {
        // Mixer rendering tests also change the process-wide custom-painted palette.
        let _palette = lock_test_palette();
        fn frame(app: &mut SoundApp, ctx: &egui::Context, system_theme: Option<egui::Theme>, light: bool) {
            let mode = app.ui.theme;
            let colors = |app: &SoundApp| {
                app.engine
                    .session()
                    .tracks
                    .iter()
                    .map(|track| {
                        (track.color, track.playlists.iter().flat_map(|playlist| &playlist.clips).map(|clip| clip.color).collect::<Vec<_>>())
                    })
                    .collect::<Vec<_>>()
            };
            let before = colors(app);
            let mut output = ctx.run_ui(egui::RawInput { system_theme, ..Default::default() }, |ui| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                let tokens = if light { Tokens::LIGHT } else { Tokens::DARK };
                assert_eq!(is_light(), light, "custom palette for {mode:?} with {system_theme:?}");
                assert_eq!(Tokens::current().panel_bg, tokens.panel_bg);
                assert_eq!(ctx.options(|options| options.theme_preference), egui::ThemePreference::System);
                for theme in [egui::Theme::Light, egui::Theme::Dark] {
                    let style = ctx.style_of(theme);
                    assert_eq!(style.visuals.dark_mode, !light);
                    assert_eq!(style.visuals.panel_fill, tokens.panel_bg);
                    assert_eq!(style.visuals.widgets.inactive.bg_fill, tokens.button);
                    assert_eq!(style.visuals.widgets.inactive.fg_stroke.color, tokens.text);
                    assert_eq!(style.spacing.button_padding, egui::vec2(6.0, 2.0));
                }
                app.ui(ui);
                assert_eq!(ui.visuals().panel_fill, tokens.panel_bg);
                assert_eq!(ui.visuals().widgets.inactive.bg_fill, tokens.button);
            });
            output.textures_delta.clear();
            assert_eq!(app.ui.theme, mode);
            assert_eq!(serde_json::to_value(&app.ui).unwrap()["theme"], mode.id());
            assert_eq!(colors(app), before, "appearance must preserve track and clip colors");
        }

        let appearances = [Some(egui::Theme::Light), Some(egui::Theme::Dark), Some(egui::Theme::Light), None];
        for initial in [Some(egui::Theme::Light), Some(egui::Theme::Dark), None] {
            let ctx = egui::Context::default();
            let mut app = SoundApp::new(soundcraft_engine::Engine::default(), None, Services::default());
            app.ui = serde_json::from_value(json!({"theme": "system"})).unwrap();
            frame(&mut app, &ctx, initial, initial == Some(egui::Theme::Light));
        }

        let ctx = egui::Context::default();
        let mut engine = soundcraft_engine::demo::demo_engine();
        let track = engine.session_mut().tracks.first_mut().unwrap();
        track.color = [19, 83, 147];
        track.playlists.iter_mut().flat_map(|playlist| &mut playlist.clips).next().unwrap().color = Some([211, 57, 99]);
        let mut app = SoundApp::new(engine, None, Services::default());
        for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
            app.run("ui.theme", json!({"mode": mode.id()})).unwrap();
            for appearance in appearances {
                let light = mode == ThemeMode::Light || (mode == ThemeMode::System && appearance == Some(egui::Theme::Light));
                frame(&mut app, &ctx, appearance, light);
            }
            if mode != ThemeMode::System {
                let latest = if mode == ThemeMode::Light { egui::Theme::Dark } else { egui::Theme::Light };
                frame(&mut app, &ctx, Some(latest), mode == ThemeMode::Light);
                app.run("ui.theme", json!({"mode": "system"})).unwrap();
                // Resolve the current context immediately, without another OS appearance event.
                app.logic(&ctx);
                assert_eq!(is_light(), latest == egui::Theme::Light);
                assert_eq!(ctx.global_style().visuals.dark_mode, latest != egui::Theme::Light);
                frame(&mut app, &ctx, Some(latest), latest == egui::Theme::Light);
            }
        }
    }
}
