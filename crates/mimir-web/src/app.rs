//! The root component: load `/api/config`, pass the sign-in gate, then show
//! the shell (Aurora `AppShell`: header with the theme switch, side
//! navigation, and the routes).

use aurora_leptos::components::*;
use aurora_leptos::frame::{AppShell, SideNav, SideNavGroup, SideNavLink};
use aurora_leptos::theme::{provide_theme, ThemeToggle};
use aurora_leptos::tokens::ApiError;
use aurora_leptos::AuroraStyles;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;
use mimir_wire::{AuthMode, Role, Session};

use crate::api;
use crate::auth::{self, Gate};
use crate::character::sheet::CharacterPage;
use crate::map::display::DisplayPage;
use crate::map::dm::DmMapPage;
use crate::pages::campaign::{CampaignDashboard, CampaignTab, ModulesTab, NpcsTab, PcsTab};
use crate::pages::player::PlayerPage;
use crate::pages::{home::Home, module::ModulePage, not_found::NotFound, sign_in::SignIn};

/// Where the app is before the shell.
#[derive(Clone)]
enum Phase {
    Loading,
    /// The server did not answer, or answered with an error.
    Failed(ApiError),
    /// Sign in; the note says why (a player link that ended).
    SignIn(String),
    Ready {
        mode: AuthMode,
        version: String,
        session: Session,
    },
}

const LINK_ENDED: &str = "This player link no longer works. Ask the DM for a new one.";

/// A visit to `/play/<token>`: keep the token as this device's, and show
/// `/player` in the address. True when the page was a player link.
fn take_play_link() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let path = window.location().pathname().unwrap_or_default();
    let Some(token) = auth::play_token(&path) else {
        return false;
    };
    auth::store_token(token);
    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/player"));
    }
    true
}

/// Read the config and the stored token, and decide the phase.
async fn boot() -> Phase {
    let from_link = take_play_link();
    let config = match api::config().await {
        Ok(c) => c,
        Err(e) => return Phase::Failed(e),
    };
    let ready = |session: Session| Phase::Ready {
        mode: config.auth,
        version: config.version.clone(),
        session,
    };
    match auth::gate(config.auth, auth::stored_token().as_deref()) {
        Gate::Open => ready(Session {
            role: Role::Dm,
            character_id: None,
            character_name: None,
            campaign_id: None,
        }),
        Gate::SignIn => Phase::SignIn(String::new()),
        Gate::CheckStored => match api::session().await {
            Ok(s) => ready(s),
            Err(e) if api::is_unauthorized(&e) => {
                auth::clear_token();
                Phase::SignIn(if from_link {
                    LINK_ENDED.into()
                } else {
                    String::new()
                })
            }
            Err(e) => Phase::Failed(e),
        },
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_theme();
    let phase = RwSignal::new(Phase::Loading);
    let start = move || {
        phase.set(Phase::Loading);
        spawn_local(async move { phase.set(boot().await) });
    };
    start();

    let retry = Callback::new(move |_| start());
    let signed_in = Callback::new(move |_| start());
    let sign_out = Callback::new(move |_| {
        auth::clear_token();
        phase.set(Phase::SignIn(String::new()));
    });
    let link_ended = Callback::new(move |_| {
        auth::clear_token();
        phase.set(Phase::SignIn(LINK_ENDED.into()));
    });

    view! {
        <AuroraStyles hyper=true />
        {move || match phase.get() {
            Phase::Loading => view! { <Loading label="Connecting to Mimir…" /> }.into_any(),
            Phase::Failed(error) => {
                view! {
                    <div class="cl-center-screen">
                        <ErrorState error=error on_retry=retry />
                    </div>
                }
                    .into_any()
            }
            Phase::SignIn(note) => view! { <SignIn on_signed_in=signed_in note=note /> }.into_any(),
            Phase::Ready { session, .. } if session.role == Role::Player => {
                view! { <PlayerPage session=session on_sign_out=sign_out on_link_ended=link_ended /> }
                    .into_any()
            }
            Phase::Ready { mode, version, .. } => {
                view! { <Shell mode=mode version=version on_sign_out=sign_out /> }.into_any()
            }
        }}
    }
}

/// The signed-in app.
#[component]
fn Shell(mode: AuthMode, version: String, on_sign_out: Callback<()>) -> impl IntoView {
    view! {
        <Router>
            <Frame mode=mode version=version on_sign_out=on_sign_out />
        </Router>
    }
}

/// The player display has no shell (a TV shows the map only); the other
/// pages sit in the app shell. The memo changes only between the two, so a
/// navigation inside the shell keeps it.
#[component]
fn Frame(mode: AuthMode, version: String, on_sign_out: Callback<()>) -> impl IntoView {
    let location = use_location();
    let bare = Memo::new(move |_| is_display_path(&location.pathname.get()));
    let can_sign_out = mode == AuthMode::Token;
    move || {
        if bare.get() {
            return view! {
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/display/:campaign") view=DisplayPage />
                </Routes>
            }
            .into_any();
        }
        let version = version.clone();
        view! {
            <AppShell
                brand=std::sync::Arc::new(|| view! { <span class="mimir-brand">"Mimir"</span> }.into_any())
                header=Box::new(move || {
                    view! {
                        <div class="mimir-header">
                            <ThemeToggle hyper=true />
                            {can_sign_out
                                .then(|| {
                                    view! {
                                        <Button variant="subtle" on_click=on_sign_out>
                                            "Sign out"
                                        </Button>
                                    }
                                })}
                        </div>
                    }
                        .into_any()
                })
                navbar=Box::new(move || view! { <Nav version=version.clone() /> }.into_any())
            >
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/") view=Home />
                    <ParentRoute path=path!("/campaigns/:id") view=CampaignDashboard>
                        <Route path=path!("") view=CampaignTab />
                        <Route path=path!("modules") view=ModulesTab />
                        <Route path=path!("npcs") view=NpcsTab />
                        <Route path=path!("pcs") view=PcsTab />
                    </ParentRoute>
                    <Route path=path!("/modules/:id") view=ModulePage />
                    <Route path=path!("/maps/:id") view=DmMapPage />
                    <Route path=path!("/characters/:id") view=CharacterPage />
                </Routes>
            </AppShell>
        }
            .into_any()
    }
}

/// Pages shown without the app shell.
pub fn is_display_path(path: &str) -> bool {
    path.starts_with("/display/")
}

/// The side navigation. Screens join it as they arrive.
#[component]
fn Nav(version: String) -> impl IntoView {
    let location = use_location();
    let at = move |prefix: &'static str| {
        Signal::derive(move || {
            let path = location.pathname.get();
            campaigns_section(&path) && prefix == "/"
                || path == prefix
                || (prefix != "/" && path.starts_with(&format!("{prefix}/")))
        })
    };
    view! {
        <SideNav footer=Box::new(move || {
            view! { <Text size="xs" dimmed=true mono=true>{format!("v{version}")}</Text> }.into_any()
        })>
            <SideNavGroup>
                <SideNavLink href="/" active=at("/")>
                    "Campaigns"
                </SideNavLink>
            </SideNavGroup>
        </SideNav>
    }
}

/// The pages under the Campaigns link of the side navigation.
pub fn campaigns_section(path: &str) -> bool {
    path == "/"
        || path.starts_with("/campaigns/")
        || path.starts_with("/modules/")
        || path.starts_with("/maps/")
        || path.starts_with("/characters/")
}

#[cfg(test)]
mod tests {
    use super::campaigns_section;

    #[test]
    fn the_display_has_no_shell() {
        assert!(super::is_display_path("/display/c1"));
        assert!(!super::is_display_path("/maps/m1"));
        assert!(!super::is_display_path("/displayed"));
    }

    #[test]
    fn campaign_pages_light_the_campaigns_link() {
        assert!(campaigns_section("/"));
        assert!(campaigns_section("/campaigns/c1/npcs"));
        assert!(campaigns_section("/modules/m1"));
        assert!(!campaigns_section("/settings"));
    }
}
