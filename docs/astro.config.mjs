// The Mimir book (MIMIR-T-0735): Astro Starlight with the Colliery theme
// starlight-aurora, in the four Diátaxis sections. It was an mdBook book.
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';
import starlightAurora from 'starlight-aurora';
import mdbookRedirects, { MDBOOK_EXTRA } from 'starlight-aurora/mdbook-redirects';
import { OLD_PAGES } from './old-addresses.mjs';

// The docs workflow sets DOCS_SITE and DOCS_BASE on main. On a pull request
// and locally they are empty, and the site is at /.
const site = process.env.DOCS_SITE || undefined;
const base = process.env.DOCS_BASE || '/';

const PROJECT = 'Mimir';
const REPO = 'https://github.com/mimir-dm/mimir';

// The sidebar is written out, not generated: it keeps the order and the short
// labels of the old SUMMARY.md. Add a new page here.
const page = (slug, label) => ({ slug, label });

export default defineConfig({
	site,
	base,
	// Mermaid is one large chunk, loaded only on a page with a diagram.
	vite: { build: { chunkSizeWarningLimit: 4096 } },
	integrations: [
		// Before Starlight. Light diagrams in the light theme; dark diagrams in
		// the dark theme and in hyper.
		mermaid({ theme: 'dark', autoTheme: true }),
		// Old mdBook addresses (page.html) send the reader to the new page.
		// `extra` replaces the defaults, so they are spread in.
		mdbookRedirects({ extra: { ...MDBOOK_EXTRA, 'toc.html': './', ...OLD_PAGES } }),
		starlight({
			title: PROJECT,
			description: 'A local-first D&D campaign assistant for Dungeon Masters',
			social: [{ icon: 'github', label: 'GitHub', href: REPO }],
			editLink: { baseUrl: `${REPO}/edit/main/docs/` },
			plugins: [starlightAurora({ hyper: true })],
			sidebar: [
				{ label: 'Welcome', link: '/' },
			{
				label: "Tutorials",
				items: [
					page("tutorials", "Overview"),
					page("tutorials/01-first-campaign", "Your First Campaign"),
					page("tutorials/02-first-module", "Your First Module"),
					page("tutorials/03-first-session", "Running Your First Session"),
					page("tutorials/04-player-display", "Player Display Setup"),
					page("tutorials/05-homebrew-content", "Creating Homebrew Content"),
				],
			},
			{
				label: "How-To Guides",
				items: [
					page("how-to", "Overview"),
					{
						label: "Campaigns",
						collapsed: true,
						items: [
							page("how-to/campaigns", "Overview"),
							page("how-to/campaigns/create-campaign", "Create a Campaign"),
							page("how-to/campaigns/manage-documents", "Manage Documents"),
							page("how-to/campaigns/manage-sources", "Manage Sources"),
							page("how-to/campaigns/archive-campaign", "Archive & Restore"),
							page("how-to/campaigns/export-campaign", "Export Campaign"),
						],
					},
					{
						label: "Maps",
						collapsed: true,
						items: [
							page("how-to/maps", "Overview"),
							page("how-to/maps/upload-map", "Upload a Map"),
							page("how-to/maps/configure-grid", "Configure Grid"),
							page("how-to/maps/place-tokens", "Place Tokens"),
							page("how-to/maps/manage-light-sources", "Manage Light Sources"),
							page("how-to/maps/print-map", "Print Maps"),
							page("how-to/maps/generate-map", "Generate Maps"),
							page("how-to/maps/mapgen-standalone", "Mapgen Standalone Tool"),
						],
					},
					{
						label: "Characters",
						collapsed: true,
						items: [
							page("how-to/characters", "Overview"),
							page("how-to/characters/create-pc", "Create a Player Character"),
							page("how-to/characters/create-npc", "Create an NPC"),
							page("how-to/characters/assign-to-campaign", "Assign to Campaign"),
							page("how-to/characters/manage-inventory", "Manage Inventory"),
							page("how-to/characters/manage-spells", "Manage Spells"),
							page("how-to/characters/level-up", "Level Up"),
							page("how-to/characters/print-character-sheet", "Print Character Sheet"),
						],
					},
					{
						label: "Modules",
						collapsed: true,
						items: [
							page("how-to/modules", "Overview"),
							page("how-to/modules/create-module", "Create a Module"),
							page("how-to/modules/add-monsters", "Add Monsters"),
							page("how-to/modules/module-documents", "Module Documents"),
						],
					},
					{
						label: "Play Mode",
						collapsed: true,
						items: [
							page("how-to/play-mode", "Overview"),
							page("how-to/play-mode/start-session", "Start a Session"),
							page("how-to/play-mode/manage-encounters", "Manage Encounters"),
							page("how-to/play-mode/fog-of-war", "Fog of War"),
							page("how-to/play-mode/manage-traps-and-pois", "Manage Traps & POIs"),
							page("how-to/play-mode/use-player-display", "Use Player Display"),
						],
					},
					{
						label: "Homebrew",
						collapsed: true,
						items: [
							page("how-to/homebrew", "Overview"),
							page("how-to/homebrew/create-item", "Create Items"),
							page("how-to/homebrew/create-monster", "Create Monsters"),
							page("how-to/homebrew/create-spell", "Create Spells"),
						],
					},
					page("how-to/ai-assistant", "AI Assistant"),
				],
			},
			{
				label: "Reference",
				items: [
					page("reference", "Overview"),
					{
						label: "UI Reference",
						collapsed: true,
						items: [
							page("reference/ui", "Overview"),
							page("reference/ui/home-screen", "Home Screen"),
							page("reference/ui/campaign-dashboard", "Campaign Dashboard"),
							page("reference/ui/module-prep-view", "Module Prep View"),
							page("reference/ui/play-mode", "Play Mode"),
							page("reference/ui/dm-map-window", "DM Map Window"),
							page("reference/ui/token-setup-modal", "Token Setup Modal"),
							page("reference/ui/player-display", "Player Display"),
							page("reference/ui/sources-window", "Sources Window"),
							page("reference/ui/settings", "Settings"),
						],
					},
					page("reference/data-model", "Data Model"),
					page("reference/characters", "Characters"),
					page("reference/sources", "Sources & Catalog"),
					page("reference/keyboard-shortcuts", "Keyboard Shortcuts"),
					page("reference/file-formats", "File Formats"),
					page("reference/vision-and-lighting", "Vision & Lighting"),
					page("reference/environment-variables", "Environment Variables"),
					page("reference/campaign-archive-format", "Campaign Archive Format"),
					page("reference/pdf-export", "PDF Export"),
					page("reference/mcp-server", "MCP Server"),
					page("reference/mapgen", "Mapgen"),
					page("reference/dd-texture-catalog", "Dungeondraft Texture Catalog"),
					page("reference/troubleshooting", "Troubleshooting"),
					page("reference/glossary", "Glossary"),
					{
						label: "Developer Documentation",
						collapsed: true,
						items: [
							page("reference/developer", "Overview"),
							page("reference/developer/contributing", "Contributing"),
							page("reference/developer/development", "Development Setup"),
							page("reference/developer/architecture", "Architecture"),
							{
								label: "Internal Reference",
								collapsed: true,
								items: [
									{
										label: "Source Data Format",
										collapsed: true,
										items: [
											page("reference/developer/reference/source-data", "Overview"),
											page("reference/developer/reference/source-data/archive-structure", "Archive Structure"),
											page("reference/developer/reference/source-data/schemas/class-schema", "Class Schema Documentation"),
											page("reference/developer/reference/source-data/schemas/content/bestiary-schema", "Bestiary Schema"),
											page("reference/developer/reference/source-data/schemas/content/class-schema", "Class Schema Reference"),
											page("reference/developer/reference/source-data/schemas/content/spell-schema", "Spell Schema"),
											page("reference/developer/reference/source-data/schemas/content/equipment-schema", "Equipment Schema"),
											page("reference/developer/reference/source-data/schemas/content/magic-items-schema", "Magic Items Schema"),
											page("reference/developer/reference/source-data/schemas/content/background-schema", "Background Schema"),
											page("reference/developer/reference/source-data/schemas/content/race-schema", "Race Schema"),
											page("reference/developer/reference/source-data/schemas/content/feat-schema", "Feat Schema"),
											page("reference/developer/reference/source-data/schemas/content/condition-schema", "Condition & Disease Schema"),
											page("reference/developer/reference/source-data/schemas/content/additional-schemas", "Additional Content Schemas"),
										],
									},
									page("reference/developer/reference/frontend", "Frontend Architecture"),
									page("reference/developer/reference/llm-tools", "LLM Tools (Removed)"),
									page("reference/developer/reference/5e-tools-processing/5etools-item-processing", "5etools Item Processing"),
									page("reference/developer/reference/5e-tools-processing/magic-variant-implementation", "Magic Variant Implementation"),
								],
							},
						],
					},
				],
			},
			{
				label: "Understanding Mimir",
				items: [
					page("explanation", "Overview"),
					page("explanation/architecture-overview", "Architecture Overview"),
					page("explanation/campaign-vs-module", "Campaigns vs Modules"),
					page("explanation/two-board-system", "The Two-Board System"),
					page("explanation/multi-window-system", "The Multi-Window System"),
					page("explanation/catalog-system", "The Catalog System"),
					page("explanation/map-formats", "Map Formats"),
					page("explanation/document-workflow", "Document Workflow"),
					page("explanation/homebrew-system", "The Homebrew System"),
					page("explanation/vision-system", "Vision System"),
					{
						label: "Campaign Framework",
						collapsed: true,
						items: [
							page("explanation/campaign-framework", "Introduction"),
							page("explanation/campaign-framework/cheat-sheet", "Cheat Sheet"),
							page("explanation/campaign-framework/board-workflow-guide", "Board Workflow Guide"),
							{
								label: "Framework Philosophy",
								collapsed: true,
								items: [
									page("explanation/campaign-framework/01-foundations", "Overview"),
									page("explanation/campaign-framework/01-foundations/three-board-system", "The Two-Board Method"),
								],
							},
							{
								label: "Campaign Genesis",
								collapsed: true,
								items: [
									page("explanation/campaign-framework/02-campaign-genesis", "Overview"),
									page("explanation/campaign-framework/02-campaign-genesis/phase-1-spark", "Phase 1: The Spark"),
									page("explanation/campaign-framework/02-campaign-genesis/phase-2-session-zero-prep", "Phase 2: Session Zero Prep"),
									page("explanation/campaign-framework/02-campaign-genesis/phase-3-session-zero", "Phase 3: Session Zero"),
									page("explanation/campaign-framework/02-campaign-genesis/phase-4-integration", "Phase 4: Integration & Launch"),
									page("explanation/campaign-framework/02-campaign-genesis/phase-5-conclusion", "Phase 5: Conclusion"),
								],
							},
							{
								label: "Module Creation",
								collapsed: true,
								items: [
									page("explanation/campaign-framework/03-module-creation", "Overview"),
									page("explanation/campaign-framework/03-module-creation/module-creation-process", "Creation Process"),
									page("explanation/campaign-framework/03-module-creation/module-types-templates", "Module Types & Templates"),
									page("explanation/campaign-framework/03-module-creation/module-running", "Running Your Module"),
									page("explanation/campaign-framework/03-module-creation/module-first", "Your First Module"),
								],
							},
							{
								label: "Running the Game",
								collapsed: true,
								items: [
									page("explanation/campaign-framework/04-session-management", "Overview"),
									page("explanation/campaign-framework/04-session-management/session-before", "Pre-Session Preparation"),
									page("explanation/campaign-framework/04-session-management/session-during", "Running the Session"),
									page("explanation/campaign-framework/04-session-management/session-after", "Post-Session & Continuity"),
									page("explanation/campaign-framework/04-session-management/session-interlude", "Between Sessions"),
								],
							},
							page("explanation/campaign-framework/05-scaling", "Scaling Your Campaign"),
							{
								label: "Templates",
								collapsed: true,
								items: [
									page("explanation/campaign-framework/06-templates", "Overview"),
									{
										label: "Template Library",
										collapsed: true,
										items: [
											page("explanation/campaign-framework/06-templates/templates", "Overview"),
											page("explanation/campaign-framework/06-templates/templates/campaign_pitch", "Campaign Pitch"),
											page("explanation/campaign-framework/06-templates/templates/campaign_bible", "Campaign Bible"),
											page("explanation/campaign-framework/06-templates/templates/document_tracker", "Campaign Document Tracker"),
											page("explanation/campaign-framework/06-templates/templates/character_guidelines", "Character Guidelines"),
											page("explanation/campaign-framework/06-templates/templates/character_integration", "Character Integration Sheet"),
											page("explanation/campaign-framework/06-templates/templates/clue_tracker", "Clue Tracker"),
											page("explanation/campaign-framework/06-templates/templates/faction_overview", "Faction Overview"),
											page("explanation/campaign-framework/06-templates/templates/faction_template", "Faction Template"),
											page("explanation/campaign-framework/06-templates/templates/house_rules", "House Rules"),
											page("explanation/campaign-framework/06-templates/templates/major_npc_tracker", "Major NPC Tracker"),
											page("explanation/campaign-framework/06-templates/templates/module_overview", "Module Overview"),
											page("explanation/campaign-framework/06-templates/templates/module_dungeon", "Dungeon Module"),
											page("explanation/campaign-framework/06-templates/templates/module_heist", "Heist Module"),
											page("explanation/campaign-framework/06-templates/templates/module_horror", "Horror Module"),
											page("explanation/campaign-framework/06-templates/templates/module_mystery", "Mystery Module"),
											page("explanation/campaign-framework/06-templates/templates/module_political", "Political Module"),
											page("explanation/campaign-framework/06-templates/templates/pc_arc_tracker", "PC Arc Tracker"),
											page("explanation/campaign-framework/06-templates/templates/play_notes", "Play Notes"),
											page("explanation/campaign-framework/06-templates/templates/player_secrets", "Player Secrets"),
											page("explanation/campaign-framework/06-templates/templates/quick_npc_reference", "Quick NPC Reference"),
											page("explanation/campaign-framework/06-templates/templates/quick_start_kit", "Quick Start Kit"),
											page("explanation/campaign-framework/06-templates/templates/region_overview", "Region Overview"),
											page("explanation/campaign-framework/06-templates/templates/safety_tools", "Safety Tools & Boundaries"),
											page("explanation/campaign-framework/06-templates/templates/starting_scenario", "Starting Scenario"),
											page("explanation/campaign-framework/06-templates/templates/table_expectations", "Table Expectations & Logistics"),
											page("explanation/campaign-framework/06-templates/templates/world_events_timeline", "World Events Timeline"),
											page("explanation/campaign-framework/06-templates/templates/world_primer", "World Primer"),
											page("explanation/campaign-framework/06-templates/templates/archived/session_outline", "Session Outline (Archived)"),
										],
									},
								],
							},
						],
					},
				],
			},
			],
		}),
	],
});
