mod components;
pub mod icons;
mod state;
mod views;

use dioxus::prelude::*;

use state::{AppState, AppView, Theme};
use views::creator_view::CreatorView;
use views::main_view::MainView;
use views::settings_view::SettingsView;

const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");
const THEME_CSS: &str = include_str!("../assets/theme.css");

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

	let theme = *state.theme.read();
	let theme_attr = match theme {
		Theme::Light => "light",
		Theme::Dark => "dark",
		Theme::System => "",
	};

	rsx! {
			style { "{TAILWIND_CSS}" }
			style { "{THEME_CSS}" }
			div {
					class: "flex flex-col h-screen",
					style: "background: var(--color-background-secondary); color: var(--color-text-primary);",
					"data-theme": "{theme_attr}",
					// Header (always visible)
					components::header::Header {}

					// Error banner
					if let Some(err) = &error {
							div {
									class: "px-4 py-2 text-sm flex justify-between items-center",
									style: "background: var(--color-error-bg); color: var(--color-error);",
									span { "{err}" }
									button {
											class: "btn-icon btn-danger ml-2",
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
