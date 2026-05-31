//! The egui app: a responsive itinerary + an OpenStreetMap map (via `walkers`)
//! with a driving route, labeled pins, and an animated "Poppy journey" that
//! slides a sticker between stops day by day. Plus a photo gallery and a little
//! beach mini-game.

use egui::{Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, Pos2, Rect, Stroke};
use walkers::{sources::OpenStreetMap, HttpTiles, Map, MapMemory, Plugin, Position, Projector};

use crate::game::BeachGame;
use crate::trip::{driving_route, itinerary, Stop};

/// Below this window width (logical px) we switch to the phone layout.
const NARROW: f32 = 640.0;
/// Seconds each day is held during ▶ auto-play.
const DAY_HOLD: f32 = 1.5;

pub struct TripApp {
    tiles: HttpTiles,
    map_memory: MapMemory,
    stops: Vec<Stop>,
    current_day: usize,
    playing: bool,
    play_accum: f32,
    show_photos: bool,
    enlarged: Option<usize>,
    game: BeachGame,
}

impl TripApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut map_memory = MapMemory::default();
        let _ = map_memory.set_zoom(6.0);

        Self {
            tiles: HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()),
            map_memory,
            stops: itinerary(),
            current_day: 0,
            playing: false,
            play_accum: 0.0,
            show_photos: false,
            enlarged: None,
            game: BeachGame::default(),
        }
    }

    fn home_position() -> Position {
        walkers::lon_lat(-77.0, 37.0)
    }

    fn goto_day(&mut self, day: usize) {
        self.current_day = day.min(self.stops.len() - 1);
        self.playing = false;
        self.play_accum = 0.0;
    }

    fn toggle_play(&mut self) {
        if self.playing {
            self.playing = false;
        } else {
            if self.current_day + 1 >= self.stops.len() {
                self.current_day = 0; // replay from the start
            }
            self.playing = true;
            self.play_accum = 0.0;
        }
    }

    /// Interpolated (lon, lat) along the stop sequence at fractional day `t`.
    fn animated_lonlat(&self, t: f32) -> (f64, f64) {
        let n = self.stops.len();
        let a = (t.floor() as usize).min(n - 1);
        let b = (a + 1).min(n - 1);
        let f = (t - a as f32) as f64;
        let (sa, sb) = (&self.stops[a], &self.stops[b]);
        (sa.lon + (sb.lon - sa.lon) * f, sa.lat + (sb.lat - sa.lat) * f)
    }

    /// Advance auto-play, animate toward the current day, and pan the map to
    /// follow Poppy. Returns (sticker position, vertical bob, is-animating).
    fn update_journey(&mut self, ctx: &egui::Context) -> (Position, f32, bool) {
        let (time, dt) = ctx.input(|i| (i.time as f32, i.stable_dt.min(0.05)));

        if self.playing {
            self.play_accum += dt;
            if self.play_accum >= DAY_HOLD {
                self.play_accum = 0.0;
                if self.current_day + 1 < self.stops.len() {
                    self.current_day += 1;
                } else {
                    self.playing = false;
                }
            }
            ctx.request_repaint();
        }

        let t = ctx.animate_value_with_time(Id::new("journey_t"), self.current_day as f32, 0.9);
        let (lon, lat) = self.animated_lonlat(t);
        let pos = walkers::lon_lat(lon, lat);
        let animating = (t - self.current_day as f32).abs() > 0.001;

        if self.playing || animating {
            self.map_memory.center_at(pos);
            ctx.request_repaint();
        }

        let bob = if self.playing || animating {
            (time * 6.0).sin() * 4.0
        } else {
            0.0
        };
        (pos, bob, animating)
    }
}

impl eframe::App for TripApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let narrow = ctx.content_rect().width() < NARROW;

        let (anim_pos, bob, _animating) = self.update_journey(&ctx);

        // Panels first (they carve out space), then the map fills the rest.
        if narrow {
            self.mobile_top_bar(ui);
            self.timeline(ui, true);
        } else {
            self.desktop_panel(ui);
            self.timeline(ui, false);
        }
        self.map_panel(ui, anim_pos, bob);
        self.zoom_controls(ui);

        self.photos_window(&ctx);
        let m = crate::photos::mascot();
        self.game.show(&ctx, m.uri, m.bytes);
    }
}

// ---- Layout pieces ----

impl TripApp {
    fn stickers_row(&self, ui: &mut egui::Ui, height: f32) {
        ui.horizontal(|ui| {
            for s in crate::photos::STICKERS {
                ui.add(egui::Image::from_bytes(s.uri, s.bytes).max_height(height));
            }
        });
    }

    fn action_buttons(&mut self, ui: &mut egui::Ui) {
        let time = ui.input(|i| i.time);
        ui.horizontal_wrapped(|ui| {
            if ui.button("📷  Photos").clicked() {
                self.show_photos = !self.show_photos;
            }
            if ui.button("🎾  Beach Fetch").clicked() {
                self.game.toggle(time);
            }
        });
    }

    fn desktop_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left(Id::new("side"))
            .resizable(true)
            .default_size(340.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| self.stickers_row(ui, 88.0));
                ui.add_space(4.0);
                ui.heading("🌊🐕 OBX Road Trip");
                ui.label("Rob, Rachael & Poppy");
                ui.label("August 10–16, 2026");
                ui.add_space(6.0);
                self.action_buttons(ui);
                ui.separator();

                let mut goto = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, s) in self.stops.iter().enumerate() {
                        let selected = self.current_day == i;
                        if ui
                            .selectable_label(selected, format!("{}  {}", s.kind.emoji(), s.title))
                            .clicked()
                        {
                            goto = Some(i);
                        }
                        if selected {
                            ui.indent("details", |ui| {
                                ui.label(egui::RichText::new(&s.date).strong());
                                ui.label(&s.blurb);
                            });
                        }
                        ui.add_space(4.0);
                    }
                    ui.separator();
                    ui.hyperlink_to(
                        "Map data © OpenStreetMap contributors",
                        "https://www.openstreetmap.org/copyright",
                    );
                });
                if let Some(i) = goto {
                    self.goto_day(i);
                }
            });
    }

    fn mobile_top_bar(&mut self, ui: &mut egui::Ui) {
        let time = ui.input(|i| i.time);
        egui::Panel::top(Id::new("topbar")).show_inside(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let m = crate::photos::mascot();
                ui.add(egui::Image::from_bytes(m.uri, m.bytes).max_height(40.0));
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("🌊🐕 OBX Road Trip").strong());
                    ui.label(egui::RichText::new("Aug 10–16, 2026").small().weak());
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("🎾").clicked() {
                        self.game.toggle(time);
                    }
                    if ui.button("📷").clicked() {
                        self.show_photos = !self.show_photos;
                    }
                });
            });
            ui.add_space(4.0);
        });
    }

    /// The day timeline: prev / play / next + a scrub slider, plus the current
    /// day's label (and, on mobile, its description).
    fn timeline(&mut self, ui: &mut egui::Ui, details: bool) {
        egui::Panel::bottom(Id::new("timeline")).show_inside(ui, |ui| {
            ui.add_space(6.0);
            let n = self.stops.len();
            let mut goto = None;
            let mut toggle = false;

            ui.horizontal(|ui| {
                if ui.add_sized([40.0, 34.0], egui::Button::new("↺")).clicked() {
                    goto = Some(0);
                }
                if ui.add_sized([40.0, 34.0], egui::Button::new("◀")).clicked() {
                    goto = Some(self.current_day.saturating_sub(1));
                }
                let play = if self.playing { "⏸ Pause" } else { "▶ Play" };
                if ui.add_sized([86.0, 34.0], egui::Button::new(play)).clicked() {
                    toggle = true;
                }
                if ui.add_sized([40.0, 34.0], egui::Button::new("▶")).clicked() {
                    goto = Some((self.current_day + 1).min(n - 1));
                }
                let mut day = self.current_day as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut day, 0.0..=(n - 1) as f32)
                            .step_by(1.0)
                            .show_value(false),
                    )
                    .changed()
                {
                    goto = Some(day as usize);
                }
            });

            let s = &self.stops[self.current_day];
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}  {}", s.kind.emoji(), s.title)).strong());
                ui.label(egui::RichText::new(format!("· {}", s.date)).weak());
            });
            if details {
                ui.add(egui::Label::new(&s.blurb).wrap());
            }
            ui.add_space(4.0);

            if toggle {
                self.toggle_play();
            }
            if let Some(i) = goto {
                self.goto_day(i);
            }
        });
    }

    fn map_panel(&mut self, ui: &mut egui::Ui, anim_pos: Position, bob: f32) {
        let route = RoutePlugin {
            points: driving_route(),
        };
        let pins = PinsPlugin {
            pins: self
                .stops
                .iter()
                .enumerate()
                .map(|(i, s)| Pin {
                    position: s.position(),
                    label: s.short.clone(),
                    color: s.kind.color(),
                    selected: self.current_day == i,
                })
                .collect(),
        };
        let m = crate::photos::mascot();
        let journey = JourneyPlugin {
            pos: anim_pos,
            uri: m.uri,
            bytes: m.bytes,
            bob,
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(Margin::ZERO))
            .show_inside(ui, |ui| {
                let map = Map::new(
                    Some(&mut self.tiles),
                    &mut self.map_memory,
                    Self::home_position(),
                )
                .with_plugin(route)
                .with_plugin(pins)
                .with_plugin(journey);
                ui.add(map);
            });
    }

    fn zoom_controls(&mut self, ui: &mut egui::Ui) {
        // Anchored mid-right so it clears the top bar and bottom timeline.
        egui::Area::new(Id::new("zoom"))
            .anchor(Align2::RIGHT_CENTER, [-10.0, 0.0])
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui.add_sized([34.0, 30.0], egui::Button::new("➕")).clicked() {
                            let _ = self.map_memory.zoom_in();
                        }
                        if ui.add_sized([34.0, 30.0], egui::Button::new("➖")).clicked() {
                            let _ = self.map_memory.zoom_out();
                        }
                    });
                });
            });
    }

    fn photos_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_photos;
        egui::Window::new("📷 Trip Photos")
            .open(&mut open)
            .default_size([540.0, 620.0])
            .show(ctx, |ui| self.photo_gallery(ui));
        self.show_photos = open;
    }

    fn photo_gallery(&mut self, ui: &mut egui::Ui) {
        let photos = crate::photos::PHOTOS;
        let n = photos.len();
        if let Some(start) = self.enlarged {
            let mut idx = start;
            let mut close = false;

            // Arrow keys flip through photos.
            let (left, right, esc) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::ArrowLeft),
                    i.key_pressed(egui::Key::ArrowRight),
                    i.key_pressed(egui::Key::Escape),
                )
            });
            if left {
                idx = (idx + n - 1) % n;
            }
            if right {
                idx = (idx + 1) % n;
            }
            if esc {
                close = true;
            }

            ui.horizontal(|ui| {
                if ui.add_sized([96.0, 32.0], egui::Button::new("←  Gallery")).clicked() {
                    close = true;
                }
                if ui.add_sized([72.0, 32.0], egui::Button::new("◀ Prev")).clicked() {
                    idx = (idx + n - 1) % n;
                }
                if ui.add_sized([72.0, 32.0], egui::Button::new("Next ▶")).clicked() {
                    idx = (idx + 1) % n;
                }
                ui.label(egui::RichText::new(format!("{} / {}", idx + 1, n)).weak());
            });
            ui.separator();

            let p = &photos[idx];
            ui.vertical_centered(|ui| {
                // Tap/click the photo to advance (handy on touch).
                let img = egui::Image::from_bytes(p.uri, p.bytes)
                    .max_height(460.0)
                    .sense(egui::Sense::click());
                if ui.add(img).on_hover_text("Tap for next").clicked() {
                    idx = (idx + 1) % n;
                }
                ui.add_space(6.0);
                ui.label(egui::RichText::new(p.caption).heading());
            });

            self.enlarged = if close { None } else { Some(idx) };
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (i, p) in photos.iter().enumerate() {
                    let thumb = egui::Image::from_bytes(p.uri, p.bytes).max_height(120.0);
                    if ui
                        .add(egui::Button::image(thumb))
                        .on_hover_text(p.caption)
                        .clicked()
                    {
                        self.enlarged = Some(i);
                    }
                }
            });
        });
    }
}

// ---- Map plugins ----

/// Draws the driving loop as a connecting line.
struct RoutePlugin {
    points: Vec<Position>,
}

impl Plugin for RoutePlugin {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        _response: &egui::Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        let painter = ui.painter();
        let screen: Vec<Pos2> = self
            .points
            .iter()
            .map(|p| projector.project(*p).to_pos2())
            .collect();
        for pair in screen.windows(2) {
            painter.line_segment(
                [pair[0], pair[1]],
                Stroke::new(3.0, Color32::from_rgb(0, 120, 200)),
            );
        }
    }
}

struct Pin {
    position: Position,
    label: String,
    color: Color32,
    selected: bool,
}

struct PinsPlugin {
    pins: Vec<Pin>,
}

impl Plugin for PinsPlugin {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        _response: &egui::Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        let painter = ui.painter();
        for pin in &self.pins {
            let at = projector.project(pin.position).to_pos2();
            let radius = if pin.selected { 9.0 } else { 6.0 };
            painter.circle_filled(at, radius, pin.color);
            painter.circle_stroke(at, radius, Stroke::new(2.0, Color32::WHITE));

            let galley = painter.layout_no_wrap(
                pin.label.clone(),
                FontId::proportional(13.0),
                Color32::BLACK,
            );
            let text_pos = at + egui::vec2(radius + 4.0, -galley.size().y / 2.0);
            let bg = Rect::from_min_size(text_pos, galley.size()).expand(3.0);
            painter.rect_filled(bg, CornerRadius::same(4), Color32::from_white_alpha(220));
            painter.galley(text_pos, galley, Color32::BLACK);
        }
    }
}

/// Draws the Poppy sticker at the animated journey position.
struct JourneyPlugin {
    pos: Position,
    uri: &'static str,
    bytes: &'static [u8],
    bob: f32,
}

impl Plugin for JourneyPlugin {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        _response: &egui::Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        let at = projector.project(self.pos).to_pos2();
        let size = egui::vec2(58.0, 58.0);
        let center = at - egui::vec2(0.0, 26.0 + self.bob);
        let rect = Rect::from_center_size(center, size);
        egui::Image::from_bytes(self.uri, self.bytes).paint_at(ui, rect);
    }
}
