//! Trip data: the stops shown in the itinerary panel and as pins on the map,
//! plus the driving route used for the connecting line.

use egui::Color32;
use walkers::{lon_lat, Position};

/// What kind of stop this is — drives the pin color and emoji.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StopKind {
    Drive,
    Beach,
    Activity,
    Friends,
    Home,
}

impl StopKind {
    pub fn emoji(self) -> &'static str {
        match self {
            StopKind::Drive => "🚗",
            StopKind::Beach => "🏖️",
            StopKind::Activity => "🐎",
            StopKind::Friends => "🎓",
            StopKind::Home => "🏠",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            StopKind::Drive => Color32::from_rgb(120, 120, 120),
            StopKind::Beach => Color32::from_rgb(0, 150, 200),
            StopKind::Activity => Color32::from_rgb(230, 150, 0),
            StopKind::Friends => Color32::from_rgb(0, 90, 190),
            StopKind::Home => Color32::from_rgb(90, 90, 90),
        }
    }
}

/// A single itinerary entry / map pin.
pub struct Stop {
    pub title: String,
    /// Short label drawn next to the pin on the map.
    pub short: String,
    pub date: String,
    pub blurb: String,
    pub kind: StopKind,
    pub lat: f64,
    pub lon: f64,
}

impl Stop {
    pub fn position(&self) -> Position {
        lon_lat(self.lon, self.lat)
    }
}

fn stop(
    title: &str,
    short: &str,
    date: &str,
    blurb: &str,
    kind: StopKind,
    lat: f64,
    lon: f64,
) -> Stop {
    Stop {
        title: title.to_owned(),
        short: short.to_owned(),
        date: date.to_owned(),
        blurb: blurb.to_owned(),
        kind,
        lat,
        lon,
    }
}

/// The full day-by-day itinerary.
pub fn itinerary() -> Vec<Stop> {
    vec![
        stop(
            "Depart Arlington",
            "Arlington, VA",
            "Mon Aug 10",
            "Hit the road south (~5.5 hr, ~330 mi). Lunch around Williamsburg/Norfolk, \
             then check in at Duck and an evening beach walk with Poppy.",
            StopKind::Drive,
            38.8797,
            -77.1075,
        ),
        stop(
            "Duck — beach base (4 nights)",
            "Duck, NC",
            "Aug 10–13",
            "Home base. Most dog-friendly OBX town — renters get off-leash beach access \
             under voice control. Dinners on the boardwalk: NC Coast Grill, AQUA, Fishbones.",
            StopKind::Beach,
            36.1668,
            -75.7507,
        ),
        stop(
            "Corolla wild horses",
            "Corolla, NC",
            "Wed Aug 12",
            "~20 min north. Guided 4×4 wild-mustang beach tour (dogs welcome, leashed) and \
             quiet 4×4 beach time.",
            StopKind::Activity,
            36.3779,
            -75.8302,
        ),
        stop(
            "Nags Head & Jockey's Ridge",
            "Nags Head, NC",
            "Thu Aug 13",
            "~30 min south. Jockey's Ridge dunes (leashed dogs, go early for cool sand) and \
             the Wright Brothers Memorial grounds. Oceanfront dinner at Tortugas' Lie.",
            StopKind::Activity,
            35.9646,
            -75.6308,
        ),
        stop(
            "Drive to Duke (Durham)",
            "Durham, NC",
            "Fri Aug 14",
            "Pack up, drive Duck → Durham (~4 hr, ~215 mi). Arrive afternoon at friends' for \
             the weekend. Durham is a serious food town.",
            StopKind::Friends,
            36.0014,
            -78.9382,
        ),
        stop(
            "Duke / Durham with friends",
            "Duke, NC",
            "Sat Aug 15",
            "Duke campus & gardens, Durham food and breweries with friends.",
            StopKind::Friends,
            36.0014,
            -78.9382,
        ),
        stop(
            "Drive home to Arlington",
            "Arlington, VA",
            "Sun Aug 16",
            "Durham → Arlington (~4.5 hr, ~260 mi). Home.",
            StopKind::Home,
            38.8797,
            -77.1075,
        ),
    ]
}

/// The actual driving loop, in order, for the connecting line on the map.
/// (Corolla / Nags Head are day trips, so they're pins but not on this line.)
pub fn driving_route() -> Vec<Position> {
    vec![
        lon_lat(-77.1075, 38.8797), // Arlington
        lon_lat(-75.7507, 36.1668), // Duck
        lon_lat(-78.9382, 36.0014), // Durham
        lon_lat(-77.1075, 38.8797), // back to Arlington
    ]
}
