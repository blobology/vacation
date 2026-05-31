//! Web-optimized trip photos, embedded into the binary and shown in the gallery.
//! (Resized to max 1100px / JPEG q68 from the originals; see assets/photos/.)

pub struct Photo {
    /// A unique `bytes://` URI so egui's image loader can cache the decoded texture.
    pub uri: &'static str,
    pub bytes: &'static [u8],
    pub caption: &'static str,
}

macro_rules! photo {
    ($file:literal, $caption:literal) => {
        Photo {
            uri: concat!("bytes://photos/", $file),
            bytes: include_bytes!(concat!("../assets/photos/", $file)),
            caption: $caption,
        }
    };
}

pub static PHOTOS: &[Photo] = &[
    photo!("poppy_rachael1.jpg", "Poppy & Rachael"),
    photo!("poppy_rob1.jpg", "Rob & Poppy"),
    photo!("poppy_rachael2.jpg", "Poppy & Rachael"),
    photo!("poppy1.jpg", "Poppy 🐾"),
    photo!("poppy_rachael3.jpg", "Poppy & Rachael"),
    photo!("poppy2.jpg", "Poppy 🐾"),
    photo!("poppy3.jpg", "Poppy 🐾"),
    photo!("poppy4.jpg", "Poppy 🐾"),
    photo!("poppy5.jpg", "Poppy 🐾"),
    photo!("poppy6.jpg", "Poppy 🐾"),
    photo!("poppy7.jpg", "Poppy 🐾"),
];
