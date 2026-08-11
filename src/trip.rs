//! Trip data: the stops shown in the itinerary panel and as pins on the map,
//! plus the driving route used for the connecting line.

use egui::Color32;
use walkers::{lon_lat, Position};

/// What kind of stop this is — drives the pin color and emoji.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StopKind {
    Drive,
    Camp,
    Hike,
    Canoe,
    Swim,
    Stay,
    Home,
}

impl StopKind {
    pub fn emoji(self) -> &'static str {
        match self {
            StopKind::Drive => "🚗",
            StopKind::Camp => "⛺",
            StopKind::Hike => "🥾",
            StopKind::Canoe => "🛶",
            StopKind::Swim => "💦",
            StopKind::Stay => "🏡",
            StopKind::Home => "🏠",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            StopKind::Drive => Color32::from_rgb(120, 120, 120),
            StopKind::Camp => Color32::from_rgb(63, 107, 79),
            StopKind::Hike => Color32::from_rgb(150, 96, 54),
            StopKind::Canoe => Color32::from_rgb(70, 120, 165),
            StopKind::Swim => Color32::from_rgb(40, 145, 180),
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
            "Roll out around 2:30pm, ~4¾ hr down I-81 and I-77. Dinner on the road, \
             then an early night at a pet-friendly creekside cabin.",
            StopKind::Drive,
            38.8797,
            -77.1075,
        ),
        stop(
            "Hillsville — creekside night",
            "Hillsville, VA",
            "Fri Aug 7",
            "A creekside cabin with a stargazing deck and a fire pit. Sleep to creek \
             sounds; Saturday's trailhead is only ~1 hr 20 min away through Galax, so \
             the morning is slow — maybe breakfast in Galax and a leg-stretcher on the \
             New River Trail.",
            StopKind::Stay,
            36.7626,
            -80.7345,
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
             walk from downtown — hot tub and fire pit included. Hot showers, laundry, \
             then an evening stroll downtown or the short Mount Jefferson summit trail \
             for a sunset overlook.",
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
             than paddling. Afternoon: Rough Ridge or the Bass Lake carriage trails on \
             the Parkway, then the hot tub back at the house.",
            StopKind::Canoe,
            36.2940,
            -81.6030,
        ),
        stop(
            "Hike Hanging Rock",
            "Hanging Rock SP, NC",
            "Tue Aug 11",
            "Checkout by 10am, then ~1¾ hr east to Hanging Rock State Park. Shady \
             waterfall trails — Upper Cascades, then Window and Hidden Falls — with \
             plunge pools for Poppy, plus an optional 2.6-mi climb to the summit outcrop. \
             Hot (~89°F) but storms hold off till late afternoon.",
            StopKind::Hike,
            36.3958,
            -80.2694,
        ),
        stop(
            "Swim the Eno River",
            "Fews Ford, NC",
            "Tue Aug 11",
            "~1½ hr from Hanging Rock: a dog-friendly dip in the Eno at Fews Ford — calm \
             pools and flat rock ledges where Poppy can actually swim off the day's heat \
             before we head into town.",
            StopKind::Swim,
            36.0736,
            -78.9903,
        ),
        stop(
            "Durham — Piedmont night",
            "Durham, NC",
            "Tue Aug 11",
            "Check in, hot showers, and dinner out in a city that does dinner well. Our \
             base for the night before the easy drive home.",
            StopKind::Stay,
            35.9940,
            -78.8986,
        ),
        stop(
            "Home to Arlington",
            "Arlington, VA",
            "Wed–Thu Aug 12–13",
            "A slow Durham morning — coffee and maybe the American Tobacco Trail with \
             Poppy — then ~4½ hr up I-85 and I-95 for home. Thursday is the built-in \
             slack day either way.",
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
        lon_lat(-80.7345, 36.7626), // Hillsville
        lon_lat(-81.5360, 36.6570), // Mount Rogers high country
        lon_lat(-81.4929, 36.4043), // West Jefferson
        lon_lat(-80.2694, 36.3958), // Hanging Rock
        lon_lat(-78.8986, 35.9940), // Durham
        lon_lat(-77.1075, 38.8797), // back to Arlington
    ]
}
