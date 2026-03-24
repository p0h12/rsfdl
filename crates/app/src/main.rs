mod components;
pub mod icons;
mod state;
mod views;

use dioxus::prelude::*;

use state::{AppState, AppView};
use views::creator_view::CreatorView;
use views::main_view::MainView;
use views::settings_view::SettingsView;

const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");

fn main() {
	tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();

	let window = dioxus::desktop::WindowBuilder::new()
		.with_title("rsfdl")
		.with_inner_size(dioxus::desktop::LogicalSize::new(900.0, 650.0));

	LaunchBuilder::new().with_cfg(dioxus::desktop::Config::new().with_window(window)).launch(app);
}

#[cfg(target_os = "macos")]
fn set_dock_icon() {
	use objc2::{AnyThread, MainThreadMarker};
	use objc2_app_kit::{NSApplication, NSImage};
	use objc2_foundation::NSData;

	let png_bytes = include_bytes!("../assets/icon.png");
	unsafe {
		let mtm = MainThreadMarker::new_unchecked();
		let data = NSData::with_bytes(png_bytes);
		if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
			let app = NSApplication::sharedApplication(mtm);
			app.setApplicationIconImage(Some(&image));
		}
	}
}

fn app() -> Element {
	#[cfg(target_os = "macos")]
	use_hook(set_dock_icon);

	use_context_provider(AppState::new);
	let mut state = use_context::<AppState>();
	let view = *state.current_view.read();
	let error = state.error_message.read().clone();

	rsx! {
			style { "{TAILWIND_CSS}" }
			div { class: "flex flex-col h-screen bg-white text-gray-900",
					// Header (always visible)
					components::header::Header {}

					// Error banner
					if let Some(err) = &error {
							div { class: "px-4 py-2 bg-red-100 text-red-800 text-sm flex justify-between items-center",
									span { "{err}" }
									button {
											class: "ml-2 text-red-600 hover:text-red-800 font-bold",
											onclick: move |_| state.error_message.set(None),
											"x"
									}
							}
					}

					// View router (fills remaining space, header stays fixed)
					div { class: "flex-1 min-h-0 flex flex-col overflow-hidden",
							match view {
									AppView::Main => rsx! { MainView {} },
									AppView::Settings => rsx! { SettingsView {} },
									AppView::Creator => rsx! { CreatorView {} },
							}
					}
			}
	}
}
