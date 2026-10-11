# Home Screen and Navigation

The campaign list is the start page of Mimir (`/`). This page also describes the sign-in page and the frame around every DM page.

## Sign-In Page

Mimir shows the sign-in page when the server has a DM token and the browser has no valid token.

| Element | Function |
|---------|----------|
| **Token** | Type the DM token of the server |
| **Sign in** | Check the token and open Mimir |

Messages:
- "The server refused this token.": the token is wrong.
- "The server did not answer. Try again.": the server cannot be reached.
- "This player link no longer works. Ask the DM for a new one.": a player link was revoked or replaced.

The browser keeps the token. A server in open mode (no DM token) has no sign-in page.

## The Frame

Every DM page, except the player display, sits in the same frame.

### Side Navigation

| Link | Opens |
|------|-------|
| **Campaigns** | The campaign list. It also stays lit on campaign, module and map pages. |
| **Characters** | All characters, by campaign |
| **Reference** | The catalog and the book reader |
| **Settings** | Campaign archives, books, integrations, version |

The version of Mimir is at the foot of the navigation.

### Header

| Element | Function |
|---------|----------|
| **Campaign** | Open another campaign: the active ones, then the archived ones (marked). It shows the campaign you opened last. |
| Theme switch | Light, dark or hyper theme. The browser keeps the choice. |
| **Sign out** | Forget the token in this browser. Shown on a server with a DM token. |

## Campaigns Page

| Element | Function |
|---------|----------|
| **Continue …** | Open the campaign you opened last. The browser keeps it. |
| Campaign cards | One card for each campaign, with its description. Click a card to open the [dashboard](./campaign-dashboard.md). |
| **Show archived** | Also show archived campaigns. They have the **Archived** label. |
| **Import campaign** | Import a campaign archive. See [Export Campaign](../../how-to/campaigns/export-campaign.md). |
| **New campaign** | Make a campaign: **Name**, **Description**, **Create** |

With no campaign, the page shows "No campaigns yet."

## Other Pages

- An unknown address shows "Page not found".
- An unknown campaign shows "Campaign not found".

## See Also

- [Create a Campaign](../../how-to/campaigns/create-campaign.md)
- [Campaign Dashboard](./campaign-dashboard.md)
- [Settings](./settings.md)
