//! "Poppy Bird" — a Flappy-Bird-style mini-game shown in an overlay window.
//! Tap / click / press Space to flap; fly Poppy through the gaps. One point
//! per gap cleared. Hitting a pipe, the ground, or the ceiling ends the run.

use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

// All gameplay is in a normalized 0..1 play space, tuned by feel.
const GRAVITY: f32 = 1.5; // downward accel per second^2
const FLAP_V: f32 = -0.55; // upward velocity set on each flap
const PIPE_SPEED: f32 = 0.32; // pipes scroll left this fast (per second)
const GAP: f32 = 0.34; // vertical gap height
const PIPE_W: f32 = 0.13; // pipe width
const SPAWN_DX: f32 = 0.62; // horizontal spacing between pipes
const POPPY_X: f32 = 0.28; // Poppy's fixed horizontal position
const RADIUS: f32 = 0.06; // Poppy's collision radius
const GROUND: f32 = 0.10; // sand strip at the bottom

#[derive(PartialEq)]
enum Phase {
    Idle,
    Playing,
    Over,
}

struct Pipe {
    x: f32,
    gap_center: f32,
    passed: bool,
}

pub struct PoppyBird {
    pub active: bool,
    phase: Phase,
    score: u32,
    best: u32,
    poppy_y: f32,
    vy: f32,
    pipes: Vec<Pipe>,
    rng: u32,
}

impl Default for PoppyBird {
    fn default() -> Self {
        Self {
            active: false,
            phase: Phase::Idle,
            score: 0,
            best: 0,
            poppy_y: 0.5,
            vy: 0.0,
            pipes: Vec::new(),
            rng: 0x2545_f491,
        }
    }
}

impl PoppyBird {
    pub fn toggle(&mut self, seed_time: f64) {
        self.active = !self.active;
        if self.active {
            self.phase = Phase::Idle;
            self.rng ^= (seed_time * 1000.0) as u32 | 1;
        }
    }

    fn rand(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    fn spawn_pipe(&mut self, x: f32) {
        // Keep the gap clear of the ceiling and the sand.
        let lo = GAP / 2.0 + 0.04;
        let hi = 1.0 - GROUND - GAP / 2.0 - 0.04;
        let gap_center = lo + self.rand() * (hi - lo);
        self.pipes.push(Pipe {
            x,
            gap_center,
            passed: false,
        });
    }

    fn start(&mut self) {
        self.phase = Phase::Playing;
        self.score = 0;
        self.poppy_y = 0.42;
        self.vy = FLAP_V;
        self.pipes.clear();
        self.spawn_pipe(1.0);
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        poppy_uri: &'static str,
        poppy_bytes: &'static [u8],
    ) {
        if !self.active {
            return;
        }
        let screen = ctx.content_rect();
        let w = (screen.width() - 24.0).clamp(260.0, 440.0);
        let h = (screen.height() - 80.0).clamp(360.0, 560.0);

        let mut open = self.active;
        egui::Window::new("🐦 Poppy Bird")
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
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
        let painter = ui.painter_at(rect);

        let to_screen =
            |x: f32, y: f32| -> Pos2 { egui::pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height()) };

        // Sky + sea + sand.
        painter.rect_filled(rect, 10.0, Color32::from_rgb(155, 209, 229));
        let sand_top = 1.0 - GROUND;
        let sand = Rect::from_min_max(to_screen(0.0, sand_top), rect.max);
        painter.rect_filled(sand, 0.0, Color32::from_rgb(238, 222, 180));

        // Flap on tap/click or Space / Up arrow.
        let flap = resp.clicked()
            || ui.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::ArrowUp));

        match self.phase {
            Phase::Idle | Phase::Over => {
                self.draw_pipes(&painter, &to_screen);
                self.draw_poppy(ui, to_screen(POPPY_X, self.poppy_y), 0.0, rect.height(), poppy_uri, poppy_bytes);
                painter.rect_filled(rect, 10.0, Color32::from_black_alpha(70));
                let msg = if self.phase == Phase::Over {
                    format!("Score: {}   Best: {}\n\nTap to play again", self.score, self.best)
                } else {
                    "🐦 Poppy Bird\nTap / click / Space to flap.\nFly through the gaps!\n\nTap to start".to_owned()
                };
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    msg,
                    FontId::proportional(20.0),
                    Color32::WHITE,
                );
                if flap {
                    self.start();
                }
            }
            Phase::Playing => {
                let dt = ui.input(|i| i.stable_dt).min(0.05);

                if flap {
                    self.vy = FLAP_V;
                }
                self.vy += GRAVITY * dt;
                self.poppy_y += self.vy * dt;

                // Ceiling: clamp. Ground: game over.
                if self.poppy_y < 0.0 {
                    self.poppy_y = 0.0;
                    self.vy = 0.0;
                }
                let mut dead = self.poppy_y + RADIUS >= 1.0 - GROUND;

                // Move pipes, score, and check collisions.
                for p in &mut self.pipes {
                    p.x -= PIPE_SPEED * dt;
                    if !p.passed && p.x + PIPE_W < POPPY_X {
                        p.passed = true;
                        self.score += 1;
                    }
                    let overlap_x = POPPY_X + RADIUS > p.x && POPPY_X - RADIUS < p.x + PIPE_W;
                    if overlap_x {
                        let gap_top = p.gap_center - GAP / 2.0;
                        let gap_bottom = p.gap_center + GAP / 2.0;
                        if self.poppy_y - RADIUS < gap_top || self.poppy_y + RADIUS > gap_bottom {
                            dead = true;
                        }
                    }
                }
                self.pipes.retain(|p| p.x + PIPE_W > -0.02);

                // Spawn the next pipe once the last one has moved in far enough.
                if self.pipes.last().map(|p| p.x < 1.0 - SPAWN_DX).unwrap_or(true) {
                    self.spawn_pipe(1.0);
                }

                self.draw_pipes(&painter, &to_screen);
                let tilt = (self.vy * 0.9).clamp(-0.5, 0.9);
                self.draw_poppy(ui, to_screen(POPPY_X, self.poppy_y), tilt, rect.height(), poppy_uri, poppy_bytes);

                painter.text(
                    rect.center_top() + Vec2::new(0.0, 12.0),
                    egui::Align2::CENTER_TOP,
                    format!("{}", self.score),
                    FontId::proportional(30.0),
                    Color32::WHITE,
                );

                if dead {
                    self.best = self.best.max(self.score);
                    self.phase = Phase::Over;
                }
                ui.ctx().request_repaint();
            }
        }
    }

    fn draw_pipes(&self, painter: &egui::Painter, to_screen: &impl Fn(f32, f32) -> Pos2) {
        let green = Color32::from_rgb(76, 175, 122);
        let edge = Stroke::new(2.0, Color32::from_rgb(40, 120, 80));
        for p in &self.pipes {
            let gap_top = p.gap_center - GAP / 2.0;
            let gap_bottom = p.gap_center + GAP / 2.0;
            let top = Rect::from_min_max(to_screen(p.x, 0.0), to_screen(p.x + PIPE_W, gap_top));
            let bottom =
                Rect::from_min_max(to_screen(p.x, gap_bottom), to_screen(p.x + PIPE_W, 1.0 - GROUND));
            painter.rect(top, 3.0, green, edge, egui::StrokeKind::Inside);
            painter.rect(bottom, 3.0, green, edge, egui::StrokeKind::Inside);
        }
    }

    fn draw_poppy(
        &self,
        ui: &mut egui::Ui,
        center: Pos2,
        tilt: f32,
        rect_h: f32,
        poppy_uri: &'static str,
        poppy_bytes: &'static [u8],
    ) {
        let s = (rect_h * 0.14).clamp(44.0, 80.0);
        let r = Rect::from_center_size(center, Vec2::splat(s));
        egui::Image::from_bytes(poppy_uri, poppy_bytes)
            .rotate(tilt, Vec2::splat(0.5))
            .paint_at(ui, r);
    }
}
