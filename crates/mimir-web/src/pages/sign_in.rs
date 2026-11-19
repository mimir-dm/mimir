//! The sign-in page: the DM types the server's token once; the browser
//! keeps it.

use aurora_leptos::components::*;
use aurora_leptos::frame::{AuthCard, CenterScreen};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{api, auth};

#[component]
pub fn SignIn(
    on_signed_in: Callback<()>,
    /// A line above the form (a player link that no longer works).
    #[prop(optional, into)]
    note: String,
) -> impl IntoView {
    let token = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let submit = move || {
        let value = token.get_untracked().trim().to_string();
        if value.is_empty() {
            error.set("Type the token.".into());
            return;
        }
        busy.set(true);
        error.set(String::new());
        spawn_local(async move {
            match api::check_token(&value).await {
                Ok(_) => {
                    auth::store_token(&value);
                    on_signed_in.run(());
                }
                Err(e) if api::is_unauthorized(&e) => {
                    error.set("The server refused this token.".into())
                }
                Err(_) => error.set("The server did not answer. Try again.".into()),
            }
            busy.set(false);
        });
    };

    view! {
        <CenterScreen>
            <AuthCard title="Mimir" sub="Sign in with the token of this server.">
                {(!note.is_empty()).then(|| view! { <Alert color=aurora_leptos::tokens::token::GOLD>{note.clone()}</Alert> })}
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submit();
                }>
                    <Stack>
                        <PasswordInput
                            label="Token"
                            value=token
                            error=error
                            autocomplete="current-password"
                            name="token"
                            required=true
                        />
                        <Button button_type="submit" loading=busy loading_label="Checking…">
                            "Sign in"
                        </Button>
                    </Stack>
                </form>
            </AuthCard>
        </CenterScreen>
    }
}
