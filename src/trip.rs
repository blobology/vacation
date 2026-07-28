//! Trip data: the stops shown in the itinerary panel and as pins on the map,
//! plus the driving route used for the connecting line.

use egui::Color32;
use walkers::{lon_lat, Position};

/// What kind of stop this is — drives the pin color and emoji.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StopKind {
    Drive,
    Camp,
    Canoe,
    Stay,
    Home,
}

impl StopKind {
    pub fn emoji(self) -> &'static str {
        match self {
            StopKind::Drive => "🚗",
            StopKind::Camp => "⛺",
            StopKind::Canoe => "🛶",
            StopKind::Stay => "🏡",
            StopKind::Home => "🏠",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            StopKind::Drive => Color32::from_rgb(120, 120, 120),
            StopKind::Camp => Color32::from_rgb(63, 107, 79),
            StopKind::Canoe => Color32::from_rgb(70, 120, 165),
            StopKind::Stay => Color32::from_rgb(184, 134, 15),
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
            "Fri Aug 7",
            "Roll out around 2pm, ~5¾ hr down I-81 (~360 mi). Dinner in town and an \
             early night at a pet-friendly Airbnb.",
            StopKind::Drive,
            38.8797,
            -77.1075,
        ),
        stop(
            "Damascus — trail town night",
            "Damascus, VA",
            "Fri Aug 7",
            "The little Appalachian Trail town where everything smells like woodsmoke \
             and boot leather. Sleep, big breakfast, then a short drive to the trailhead.",
            StopKind::Stay,
            36.6337,
            -81.7876,
        ),
        stop(
            "Backpack the Grayson Highlands",
            "Mount Rogers high country",
            "Sat Aug 8",
            "Park at Massie Gap (check-in 1pm), hike ~4 mi up the Rhododendron Trail and \
             the AT over Wilburn Ridge — wild pony country, Poppy on leash — to camp near \
             Rhododendron Gap / Thomas Knob. Stove-only this year (fire ban); bear boxes \
             at camp. Sunset from the rocks ~8:20pm.",
            StopKind::Camp,
            36.6570,
            -81.5360,
        ),
        stop(
            "West Jefferson — river base (2 nights)",
            "West Jefferson, NC",
            "Aug 9–10",
            "Morning ponies, back at the car by noon, then ~1 hr to our cottage a short \
             walk from downtown — hot tub and fire pit included. Hot showers, laundry, a \
             real dinner, and a dog asleep before dark.",
            StopKind::Stay,
            36.4043,
            -81.4929,
        ),
        stop(
            "Canoe the New River",
            "Todd, NC",
            "Mon Aug 10",
            "RiverGirl Fishing Co. in Todd (~15 min away) puts all three of us in a \
             canoe — dogs officially welcome. Slow, glassy Class I water; more drifting \
             than paddling. Afternoon hot tub back at the house.",
            StopKind::Canoe,
            36.2940,
            -81.6030,
        ),
        stop(
            "Camp on Grassy Ridge Bald",
            "Roan Highlands",
            "Tue Aug 11",
            "~2 hr to Carvers Gap, then just 2.5 mi over Round and Jane Balds to camp at \
             6,100 ft on the longest grassy bald in the Appalachians — 360° of mountains. \
             Water carried up, food hung. Sunset ~8:15pm, then more stars than we've seen \
             all year.",
            StopKind::Camp,
            36.0995,
            -82.0780,
        ),
        stop(
            "Hike out, head north",
            "Roanoke, VA",
            "Wed Aug 12",
            "Sunrise on the ridge ~6:35am, break camp, and point the car home — ~6¾ hr \
             in one go, or split it with a night near Roanoke.",
            StopKind::Drive,
            37.2710,
            -79.9414,
        ),
        stop(
            "Home to Arlington",
            "Arlington, VA",
            "Thu Aug 13",
            "Built-in slack day — either already home doing laundry, or an easy final \
             3½ hr from Roanoke.",
            StopKind::Home,
            38.8797,
            -77.1075,
        ),
    ]
}

/// The actual driving loop, in order, for the connecting line on the map.
/// (The canoe put-in is a short day trip, so it's a pin but not on this line.)
pub fn driving_route() -> Vec<Position> {
    vec![
        lon_lat(-77.1075, 38.8797), // Arlington
        lon_lat(-81.7876, 36.6337), // Damascus
        lon_lat(-81.5360, 36.6570), // Mount Rogers high country
        lon_lat(-81.4929, 36.4043), // West Jefferson
        lon_lat(-82.0780, 36.0995), // Grassy Ridge Bald
        lon_lat(-79.9414, 37.2710), // Roanoke
        lon_lat(-77.1075, 38.8797), // back to Arlington
    ]
}
