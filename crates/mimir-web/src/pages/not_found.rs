//! The fallback route.

use aurora_leptos::components::*;
use aurora_leptos::frame::PageHeader;
use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <PageHeader title="Page not found" />
        <Empty message="There is no page at this address." href="/" link="Go to campaigns" />
    }
}
