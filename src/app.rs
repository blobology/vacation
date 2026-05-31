//! The egui app: an itinerary side panel + an OpenStreetMap map (via `walkers`)
//! with a driving-route line and labeled pins for each stop.

use egui::{Align2, Color32, CornerRadius, FontId, Id, Margin, Pos2, Rect, Stroke};
use walkers::{
    sources::OpenStreetMap, HttpTiles, Map, MapMemory, Plugin, Position, Projector,
};

use crate::trip::{driving_route, itinerary, Stop};

pub struct TripApp {
    tiles: HttpTiles,
    map_memory: MapMemory,
    stops: Vec<Stop>,
    selected: Option<usize>,
}

impl TripApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut map_memory = MapMemory::default();
        // Start zoomed out enough to see VA → the NC coast.
        let _ = map_memory.set_zoom(6.0);

        Self {
            tiles: HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()),
            map_memory,
            stops: itinerary(),
            selected: None,
        }
    }

    /// Default map center when not dragged — between Arlington and the coast.
    fn home_position() -> Position {
        walkers::lon_lat(-77.0, 37.0)
    }
}

impl eframe::App for TripApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.itinerary_panel(ui);
        self.map_panel(ui);
        self.zoom_controls(ui);
    }
}

impl TripApp {
    fn itinerary_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left(Id::new("itinerary"))
            .resizable(true)
            .default_size(340.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.heading("🌊🐕 OBX Road Trip");
                ui.label("Rob, Rachael & Poppy");
                ui.label("August 10–16, 2026");
                ui.separator();

                let mut newly_selected = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, s) in self.stops.iter().enumerate() {
                        let selected = self.selected == Some(i);
                        let label = format!("{}  {}", s.kind.emoji(), s.title);
                        if ui.selectable_label(selected, label).clicked() {
                            newly_selected = Some(i);
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
                    ui.label(
                        egui::RichText::new(
                            "Tap a stop to center the map. Drag to pan, +/− to zoom.",
                        )
                        .small()
                        .weak(),
                    );
                    ui.hyperlink_to(
                        "Map data © OpenStreetMap contributors",
                        "https://www.openstreetmap.org/copyright",
                    );
                });

                // Apply the click after the immutable borrow of `self.stops` ends.
                if let Some(i) = newly_selected {
                    self.selected = Some(i);
                    self.map_memory.center_at(self.stops[i].position());
                }
            });
    }

    fn map_panel(&mut self, ui: &mut egui::Ui) {
        // Build plugin data up front so it owns its data (no borrow of `self`
        // while the map mutably borrows `self.tiles` / `self.map_memory`).
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
                    selected: self.selected == Some(i),
                })
                .collect(),
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
                .with_plugin(pins);
                ui.add(map);
            });
    }

    fn zoom_controls(&mut self, ui: &mut egui::Ui) {
        egui::Area::new(Id::new("zoom"))
            .anchor(Align2::RIGHT_BOTTOM, [-12.0, -12.0])
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button("➕").clicked() {
                            let _ = self.map_memory.zoom_in();
                        }
                        if ui.button("➖").clicked() {
                            let _ = self.map_memory.zoom_out();
                        }
                        if ui.button("⟲ fit").clicked() {
                            self.map_memory.follow_my_position();
                            let _ = self.map_memory.set_zoom(6.0);
                            self.selected = None;
                        }
                    });
                });
            });
    }
}

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

/// Draws a labeled dot for each stop.
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

            // Label with a translucent background so it stays readable over tiles.
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
