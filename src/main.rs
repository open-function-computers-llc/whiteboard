//! Whiteboard — a distraction-free drawing surface for live screenshare calls.
//!
//! Design goals: lean native binary, near-instant launch, no persistence, minimal chrome.
//! See INSTRUCTIONS.md for the spec this implements.

use eframe::egui::{
    self, Align2, Color32, Id, Key, LayerId, Order, PointerButton, Pos2, Rect, Sense, Stroke,
    Vec2, ViewportCommand,
};
use eframe::{CreationContext, NativeOptions};

/// Ink thickness in logical pixels at 100% zoom. Stored per-stroke in world units so
/// strokes scale with the camera (the "ink lives on the canvas" behaviour).
const BASE_WIDTH: f32 = 4.0;

/// Points closer than this (in screen px) are collapsed so freehand strokes stay light.
const MIN_POINT_DIST_PX: f32 = 1.6;

const MIN_SCALE: f32 = 0.05;
const MAX_SCALE: f32 = 40.0;

/// The palette, in display order: Red, Blue, Black, Green, Purple, Orange.
/// Keyboard: 1-6 or R/B/K/G/P/O.
const PALETTE: [Color32; 6] = [
    Color32::from_rgb(214, 40, 40),   // Red
    Color32::from_rgb(38, 92, 214),    // Blue
    Color32::from_rgb(24, 24, 24),     // Black
    Color32::from_rgb(38, 168, 74),    // Green
    Color32::from_rgb(150, 62, 200),   // Purple
    Color32::from_rgb(240, 140, 28),   // Orange
];

const SELECTED_RING: Color32 = Color32::from_rgb(30, 30, 30);
const IDLE_RING: Color32 = Color32::from_rgb(196, 196, 196);
const HINT_FG: Color32 = Color32::from_rgb(150, 150, 150);

/// A single freehand stroke. Points are stored in world space; the camera transforms
/// them to screen space at paint time.
struct StrokeData {
    color: Color32,
    width_world: f32,
    points: Vec<Pos2>,
}

struct Whiteboard {
    strokes: Vec<StrokeData>,
    current: Option<StrokeData>,
    drawing: bool,
    color_idx: usize,
    scale: f32,
    offset: Vec2,
    show_palette: bool,
    /// Set when the app is created so visuals are configured exactly once.
    init: bool,
}

impl Default for Whiteboard {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            current: None,
            drawing: false,
            color_idx: 2, // Black by default
            scale: 1.0,
            offset: Vec2::ZERO,
            show_palette: true,
            init: false,
        }
    }
}

impl Whiteboard {
    #[inline]
    fn to_screen(&self, p: Pos2) -> Pos2 {
        Pos2::new(p.x * self.scale + self.offset.x, p.y * self.scale + self.offset.y)
    }

    #[inline]
    fn to_world(&self, p: Pos2) -> Pos2 {
        Pos2::new((p.x - self.offset.x) / self.scale, (p.y - self.offset.y) / self.scale)
    }

    /// Zoom by `factor` keeping the world point under `anchor` (screen px) fixed.
    fn zoom_about(&mut self, factor: f32, anchor: Pos2) {
        let world = self.to_world(anchor);
        let new_scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        self.offset.x = anchor.x - world.x * new_scale;
        self.offset.y = anchor.y - world.y * new_scale;
        self.scale = new_scale;
    }

    fn begin_stroke(&mut self, screen: Pos2) {
        self.current = Some(StrokeData {
            color: PALETTE[self.color_idx],
            width_world: BASE_WIDTH / self.scale,
            points: vec![self.to_world(screen)],
        });
        self.drawing = true;
    }

    fn extend_stroke(&mut self, screen: Pos2) {
        let w = self.to_world(screen);
        if let Some(cur) = self.current.as_mut() {
            let min = MIN_POINT_DIST_PX / self.scale;
            let far = cur
                .points
                .last()
                .is_none_or(|l| (w - *l).length() >= min);
            if far {
                cur.points.push(w);
            }
        }
    }

    fn finish_stroke(&mut self) {
        if let Some(cur) = self.current.take()
            && !cur.points.is_empty() {
                self.strokes.push(cur);
            }
        self.drawing = false;
    }

    /// Undo removes the newest committed stroke (newest -> oldest). We keep no separate
    /// history buffer — undo just peels the existing strokes list — so it costs no extra
    /// memory and needs no depth cap. Blocked mid-drag so you never erase an in-flight line.
    fn undo(&mut self) {
        if self.drawing {
            return;
        }
        self.strokes.pop();
    }

    fn draw_stroke(&self, painter: &egui::Painter, s: &StrokeData) {
        let w = (s.width_world * self.scale).max(0.6);
        let pts: Vec<Pos2> = s.points.iter().map(|p| self.to_screen(*p)).collect();
        if pts.is_empty() {
            return;
        }
        let stroke = Stroke::new(w, s.color);
        for win in pts.windows(2) {
            painter.line_segment([win[0], win[1]], stroke);
        }
        // Round caps/joints; also renders a single-point click as a dot.
        let r = w * 0.5;
        for &p in &pts {
            painter.circle_filled(p, r, s.color);
        }
    }
}

impl eframe::App for Whiteboard {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Pure white canvas, edge to edge.
        [1.0, 1.0, 1.0, 1.0]
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = _ui.ctx();

        if !self.init {
            ctx.set_visuals(egui::Visuals::light());
            self.init = true;
        }

        // ---- input snapshot (one lock, consistent within the frame) ----
        let (space, left_pressed, left_down, left_released, pointer, delta, zoom) = ctx.input(|i| {
            (
                i.key_down(Key::Space),
                i.pointer.button_pressed(PointerButton::Primary),
                i.pointer.button_down(PointerButton::Primary),
                i.pointer.button_released(PointerButton::Primary),
                i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()),
                i.pointer.delta(),
                i.zoom_delta(),
            )
        });

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            // ESC closes immediately, no confirmation.
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Tab)) {
            self.show_palette = !self.show_palette;
        }
        if let Some(c) = ctx.input(|i| {
            let hit = |n: Key, l: Key| i.key_pressed(n) || i.key_pressed(l);
            if hit(Key::Num1, Key::R) {
                Some(0)
            } else if hit(Key::Num2, Key::B) {
                Some(1)
            } else if hit(Key::Num3, Key::K) {
                Some(2)
            } else if hit(Key::Num4, Key::G) {
                Some(3)
            } else if hit(Key::Num5, Key::P) {
                Some(4)
            } else if hit(Key::Num6, Key::O) {
                Some(5)
            } else {
                None
            }
        }) {
            self.color_idx = c;
        }

        let mut repaint = false;

        // ---- Ctrl/Cmd + Z: undo the most recent stroke (newest -> oldest) ----
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::Z)) {
            self.undo();
            repaint = true;
        }

        // ---- zoom (Ctrl + wheel; egui folds the Ctrl modifier into zoom_delta) ----
        if zoom != 1.0 {
            match pointer {
                Some(c) => self.zoom_about(zoom, c),
                None => {
                    self.scale = (self.scale * zoom).clamp(MIN_SCALE, MAX_SCALE);
                }
            }
            repaint = true;
        }

        // ---- pan vs draw ----
        if space {
            // Any stray in-progress stroke is committed before we start panning.
            if self.drawing {
                self.finish_stroke();
            }
            if delta != Vec2::ZERO {
                self.offset += delta;
                repaint = true;
            }
        } else {
            let over_palette = self.show_palette
                && pointer
                .is_some_and(|p| self.palette_rect(ctx).expand(6.0).contains(p));

            if !over_palette && left_pressed
                && let Some(p) = pointer {
                    self.begin_stroke(p);
                    repaint = true;
                }
            if left_down && self.drawing
                && let Some(p) = pointer {
                    self.extend_stroke(p);
                    repaint = true;
                }
            if left_released && self.drawing {
                self.finish_stroke();
                repaint = true;
            }
        }

        // ---- paint strokes on the background layer ----
        let painter = ctx.layer_painter(LayerId::new(Order::Background, Id::unique("strokes")));
        for s in self.strokes.iter().chain(self.current.iter()) {
            self.draw_stroke(&painter, s);
        }

        // ---- palette + hint overlay ----
        if self.show_palette {
            self.show_palette_ui(ctx);
            self.show_hint_ui(ctx);
        }

        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
        if repaint {
            ctx.request_repaint();
        }
    }
}

impl Whiteboard {
    fn palette_rect(&self, ctx: &egui::Context) -> Rect {
        // Read the palette's bounding rect that egui recorded for its Area last pass.
        ctx.memory(|m| m.area_rect(Id::unique("palette")))
            .unwrap_or(Rect::NOTHING)
    }

    fn show_palette_ui(&mut self, ctx: &egui::Context) {
        let selected = self.color_idx;
        let mut picked: Option<usize> = None;
        egui::Area::new(Id::unique("palette"))
            .order(Order::Foreground)
            .anchor(Align2::LEFT_TOP, Vec2::new(12.0, 12.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgb(250, 250, 250))
                    .stroke(Stroke::new(1.0, IDLE_RING))
                    .corner_radius(12.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::splat(8.0);
                            for (i, color) in PALETTE.iter().enumerate() {
                                let (rect, resp) =
                                    ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
                                let painter = ui.painter();
                                let (border_w, border_c) = if i == selected {
                                    (3.0, SELECTED_RING)
                                } else {
                                    (1.0, IDLE_RING)
                                };
                                painter.rect_filled(rect.expand(border_w), 9.0, border_c);
                                painter.rect_filled(rect, 6.0, *color);
                                if resp.clicked() {
                                    picked = Some(i);
                                }
                            }
                        });
                    });
            });
        if let Some(i) = picked {
            self.color_idx = i;
        }
    }

    fn show_hint_ui(&self, ctx: &egui::Context) {
        egui::Area::new(Id::unique("hint"))
            .order(Order::Foreground)
            .anchor(Align2::LEFT_BOTTOM, Vec2::new(14.0, -14.0))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "1-6 / R B K G P O color   ·   drag draw   ·   Ctrl+Z undo   ·   Space pan   ·   Ctrl+Wheel zoom   ·   Tab hide   ·   Esc quit",
                    )
                    .size(12.0)
                    .color(HINT_FG),
                );
            });
    }
}

fn main() -> eframe::Result {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Whiteboard")
            .with_decorations(false) // hide the titlebar
            .with_resizable(true)
            .with_maximized(true),
        // Lean: no multisampling; egui already antialiases via feathering.
        multisampling: 0,
        ..Default::default()
    };

    eframe::run_native(
        "Whiteboard",
        options,
        Box::new(|_cc: &CreationContext<'_>| Ok(Box::new(Whiteboard::default()))),
    )
}
