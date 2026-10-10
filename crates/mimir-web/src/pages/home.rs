//! `/`: the campaign list (filled in by the campaign API).

use aurora_leptos::components::*;
use aurora_leptos::frame::PageHeader;
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <PageHeader title="Campaigns" />
        <Empty message="No campaigns to show yet." hint="The campaign list comes next." />
    }
}
