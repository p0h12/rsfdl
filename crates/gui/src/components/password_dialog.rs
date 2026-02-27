use dioxus::prelude::*;

use rsfdl_core::sfdl::crypto::{decrypt_container, validate_password};

use crate::components::header::finish_container_load;
use crate::state::AppState;

#[component]
pub fn PasswordDialog() -> Element {
    let mut state = use_context::<AppState>();
    let mut password = use_signal(String::new);

    let needs_password = *state.needs_password.read();
    if !needs_password {
        return rsx! {};
    }

    let error = state.password_error.read().clone();

    rsx! {
        // Backdrop
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            // Dialog
            div { class: "bg-white rounded-lg shadow-xl p-6 w-96 space-y-4",
                h2 { class: "text-lg font-bold text-gray-900", "Password Required" }
                p { class: "text-sm text-gray-600", "This container is encrypted." }
                input {
                    class: "w-full px-3 py-2 border rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500",
                    r#type: "password",
                    placeholder: "Enter password...",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            try_decrypt(state, password.read().clone());
                        }
                    },
                }
                if let Some(err) = &error {
                    p { class: "text-sm text-red-600", "{err}" }
                }
                div { class: "flex justify-end gap-2",
                    button {
                        class: "px-3 py-1.5 bg-gray-200 hover:bg-gray-300 rounded text-sm",
                        onclick: move |_| {
                            state.needs_password.set(false);
                            state.container.set(None);
                            state.container_path.set(None);
                            state.password_error.set(None);
                        },
                        "Cancel"
                    }
                    button {
                        class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-medium",
                        onclick: move |_| {
                            try_decrypt(state, password.read().clone());
                        },
                        "Decrypt"
                    }
                }
            }
        }
    }
}

fn try_decrypt(mut state: AppState, password: String) {
    let Some(mut container) = state.container.read().clone() else {
        return;
    };

    if !validate_password(&container, &password) {
        state
            .password_error
            .set(Some("Invalid password".to_string()));
        return;
    }

    if let Err(e) = decrypt_container(&mut container, &password) {
        state
            .password_error
            .set(Some(format!("Decryption failed: {e}")));
        return;
    }

    let path = state.container_path.read().clone().unwrap_or_default();
    finish_container_load(&mut state, container, path);
}
