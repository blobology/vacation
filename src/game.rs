//! A tiny "catch Poppy's tennis balls" mini-game, shown in an overlay window.
//! Move Poppy left/right (mouse or touch-drag) to catch falling tennis balls
//! before the timer runs out.

use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

const GAME_SECONDS: f32 = 30.0;
const FALL_SPEED: f32 = 0.42; // fraction of height per second
const SPAWN_EVERY: f32 = 0.72; // seconds between balls
const CATCH_X: f32 = 0.11; // horizontal catch tolerance (fraction of width)
const POPPY_Y: f32 = 0.86; // Poppy's vertical band (fraction of height)

#[derive(PartialEq)]
enum Phase {
    Idle,
    Playing,
    Over,
}

struct Ball {
    x: f32, // 0..1 across the play area
    y: f32, // 0..1 top→bottom
}

pub struct BeachGame {
    pub active: bool,
    phase: Phase,
    score: u32,
    best: u32,
    time_left: f32,
    poppy_x: f32,
    balls: Vec<Ball>,
    spawn_timer: f32,
    rng: u32,
}

impl Default for BeachGame {
    fn default() -> Self {
        Self {
            active: false,
            phase: Phase::Idle,
            score: 0,
            best: 0,
            time_left: GAME_SECONDS,
            poppy_x: 0.5,
            balls: Vec::new(),
            spawn_timer: 0.0,
            rng: 0x2545_f491,
        }
    }
}

impl BeachGame {
    pub fn toggle(&mut self, seed_time: f64) {
        self.active = !self.active;
        if self.active {
            self.phase = Phase::Idle;
            self.rng ^= (seed_time * 1000.0) as u32 | 1;
        }
    }

    fn rand(&mut self) -> f32 {
        // Small LCG — deterministic but fine for ball placement.
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    fn start(&mut self) {
        self.phase = Phase::Playing;
        self.score = 0;
        self.time_left = GAME_SECONDS;
        self.balls.clear();
        self.spawn_timer = 0.0;
        self.poppy_x = 0.5;
    }

    /// Draws the game window when active. `poppy` is the sticker to draw.
    pub fn show(&mut self, ctx: &egui::Context, poppy_uri: &'static str, poppy_bytes: &'static [u8]) {
        if !self.active {
            return;
        }
        let screen = ctx.content_rect();
        let w = (screen.width() - 24.0).min(440.0).max(260.0);
        let h = (screen.height() - 80.0).min(560.0).max(360.0);

        let mut open = self.active;
        egui::Window::new("🎾 Beach Fetch")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .default_size([w, h])
            .show(ctx, |ui| {
                ui.set_width(w);
                self.play_area(ui, Vec2::new(w, h - 64.0), poppy_uri, poppy_bytes);
            });
        self.active = open;
    }

    fn play_area(
        &mut self,
        ui: &mut egui::Ui,
        size: Vec2,
        poppy_uri: &'static str,
        poppy_bytes: &'static [u8],
    ) {
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // Sand + sea backdrop.
        painter.rect_filled(rect, 10.0, Color32::from_rgb(238, 222, 180));
        let sea = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + rect.height() * 0.28));
        painter.rect_filled(sea, 10.0, Color32::from_rgb(94, 178, 205));

        let to_screen = |x: f32, y: f32| -> Pos2 {
            egui::pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height())
        };

        // Pointer controls Poppy's x (mouse hover or touch drag).
        if let Some(p) = ui.input(|i| i.pointer.latest_pos()) {
            if rect.contains(p) {
                self.poppy_x = ((p.x - rect.left()) / rect.width()).clamp(0.04, 0.96);
            }
        }

        match self.phase {
            Phase::Idle | Phase::Over => {
                let dim = Color32::from_black_alpha(60);
                painter.rect_filled(rect, 10.0, dim);
                let msg = if self.phase == Phase::Over {
                    format!("Time! You caught {} 🎾\nBest: {}\n\nTap to play again", self.score, self.best)
                } else {
                    "🎾 Catch Poppy's tennis balls!\nMove Poppy to catch them.\n\nTap to start".to_owned()
                };
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    msg,
                    FontId::proportional(20.0),
                    Color32::WHITE,
                );
                self.draw_poppy(ui, to_screen(self.poppy_x, POPPY_Y), poppy_uri, poppy_bytes);
                if resp.clicked() {
                    self.start();
                }
            }
            Phase::Playing => {
                let dt = ui.input(|i| i.stable_dt).min(0.05);
                self.time_left -= dt;
                if self.time_left <= 0.0 {
                    self.time_left = 0.0;
                    self.best = self.best.max(self.score);
                    self.phase = Phase::Over;
                }

                // Spawn.
                self.spawn_timer -= dt;
                if self.spawn_timer <= 0.0 {
                    self.spawn_timer = SPAWN_EVERY;
                    let x = 0.08 + self.rand() * 0.84;
                    self.balls.push(Ball { x, y: -0.05 });
                }

                // Move + collide.
                let poppy_x = self.poppy_x;
                let mut caught = 0u32;
                self.balls.retain_mut(|b| {
                    b.y += FALL_SPEED * dt;
                    if b.y >= POPPY_Y && (b.x - poppy_x).abs() < CATCH_X {
                        caught += 1;
                        return false; // caught
                    }
                    b.y < 1.05 // drop missed balls
                });
                self.score += caught;

                // Draw balls.
                for b in &self.balls {
                    let c = to_screen(b.x, b.y);
                    painter.circle_filled(c, 12.0, Color32::from_rgb(200, 222, 0));
                    painter.circle_stroke(c, 12.0, Stroke::new(1.5, Color32::from_rgb(150, 170, 0)));
                }

                self.draw_poppy(ui, to_screen(self.poppy_x, POPPY_Y), poppy_uri, poppy_bytes);

                // HUD.
                painter.text(
                    rect.left_top() + Vec2::new(10.0, 8.0),
                    egui::Align2::LEFT_TOP,
                    format!("🎾 {}", self.score),
                    FontId::proportional(22.0),
                    Color32::from_rgb(40, 40, 40),
                );
                painter.text(
                    rect.right_top() + Vec2::new(-10.0, 8.0),
                    egui::Align2::RIGHT_TOP,
                    format!("⏱ {:.0}", self.time_left.ceil()),
                    FontId::proportional(22.0),
                    Color32::from_rgb(40, 40, 40),
                );

                ui.ctx().request_repaint();
            }
        }
    }

    fn draw_poppy(
        &self,
        ui: &mut egui::Ui,
        center: Pos2,
        poppy_uri: &'static str,
        poppy_bytes: &'static [u8],
    ) {
        let size = Vec2::splat(70.0);
        let rect = Rect::from_center_size(center, size);
        egui::Image::from_bytes(poppy_uri, poppy_bytes).paint_at(ui, rect);
    }
}
