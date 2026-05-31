#![warn(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// ---- Native ----
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([480.0, 360.0])
            .with_title("OBX Trip — Rob, Rachael & Poppy"),
        ..Default::default()
    };

    eframe::run_native(
        "OBX Trip",
        native_options,
        Box::new(|cc| Ok(Box::new(vacation::TripApp::new(cc)))),
    )
}

// ---- Web ----
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Show panics in the browser console.
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find #the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(vacation::TripApp::new(cc)))),
            )
            .await;

        // Remove the loading text once the app starts (or report an error).
        if let Some(loading) = document.get_element_by_id("loading") {
            match start_result {
                Ok(_) => loading.remove(),
                Err(e) => {
                    loading.set_inner_html(
                        "<p>The app crashed. See the developer console for details.</p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
