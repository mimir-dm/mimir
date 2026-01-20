//! Module board definition.

use super::{BoardDefinition, StageMetadata};

/// Module workflow board with progression stages.
pub struct ModuleBoard;

impl Default for ModuleBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleBoard {
    /// Creates a new module board.
    pub fn new() -> Self {
        Self
    }
}

impl BoardDefinition for ModuleBoard {
    fn board_type(&self) -> &str {
        "module"
    }

    fn stages(&self) -> Vec<&str> {
        vec!["planning", "development", "ready", "active", "completed"]
    }

    fn can_transition(&self, from: &str, to: &str) -> bool {
        match (from, to) {
            // Forward progression
            ("planning", "development") => true,
            ("development", "ready") => true,
            ("ready", "active") => true,
            ("active", "completed") => true,

            // Allow moving back
            ("development", "planning") => true,
            ("ready", "development") => true,

            _ => false,
        }
    }

    fn required_documents(&self, stage: &str) -> Vec<&str> {
        match stage {
            "planning" => vec!["module_overview"],
            "development" => vec!["quick_npc_reference"],
            "ready" => vec!["session_outline"],
            "active" => vec!["document_tracker"],
            _ => vec![],
        }
    }

    fn optional_documents(&self, stage: &str) -> Vec<&str> {
        match stage {
            "planning" => vec![],
            "development" => vec!["major_npc_tracker", "faction_template"],
            "ready" => vec!["clue_tracker", "region_overview"],
            "active" => vec![],
            _ => vec![],
        }
    }

    fn next_stage(&self, current: &str) -> Option<&str> {
        match current {
            "planning" => Some("development"),
            "development" => Some("ready"),
            "ready" => Some("active"),
            "active" => Some("completed"),
            _ => None,
        }
    }

    fn stage_metadata(&self, stage: &str) -> StageMetadata {
        match stage {
            "planning" => StageMetadata {
                display_name: "Planning".to_string(),
                description: "Developing the module concept and structure".to_string(),
                completion_message: Some(
                    "Module concept is solid! Time to develop the details."
                        .to_string()
                ),
                transition_prompt: Some(
                    "Have you completed the module outline? Moving to development will begin creating encounters and NPCs."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Module Planning: Define Your Core Concept</h3>
<p>Transform your initial idea into an actionable module concept. This phase typically takes 2-3 hours and creates the blueprint for everything that follows.</p>

<h4>Essential Elements to Define:</h4>
<ol>
  <li><strong>Central Conflict</strong>
    <p>What's the main problem, mystery, or challenge? This should connect to your campaign themes and be something players can meaningfully impact. Make it personal to at least one character.</p>
  </li>
  <li><strong>Stakes and Consequences</strong>
    <p>What happens if players succeed? What if they fail? What if they don't engage at all? All three outcomes should be interesting and move the campaign forward.</p>
  </li>
  <li><strong>The Hook</strong>
    <p>How do players learn about this situation? Why must they act now rather than later? The best hooks combine external pressure with personal investment.</p>
  </li>
  <li><strong>Module Scope</strong>
    <p>First module after Session Zero: 2 sessions (testing ground for player preferences)<br>
    Standard modules: 3-4 sessions (most sustainable pace)<br>
    Major arc modules: 5-6 sessions (campaign turning points)</p>
  </li>
</ol>

<p class="integration-note">Your first module is special: Keep it to 2 sessions with varied content types and explicit decision points to discover what your specific group enjoys. This investment in observation pays off in all future modules.</p>"#
                        .to_string()
                ),
            },
            "development" => StageMetadata {
                display_name: "Development".to_string(),
                description: "Creating encounters, NPCs, and locations".to_string(),
                completion_message: Some(
                    "Module content is complete! Now finalize everything for play."
                        .to_string()
                ),
                transition_prompt: Some(
                    "Is all content created? Moving to ready means the module is fully prepared for play."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Module Development: Populate Your Structure</h3>
<p>Fill your module skeleton with engaging content. This phase takes 3-4 hours and transforms your outline into playable material.</p>

<h4>The Five-Room Dungeon Structure</h4>
<p>This is a pacing framework, not literal rooms. Every module type benefits from these dramatic beats:</p>
<ol>
  <li><strong>Entrance/Guardian</strong>: Initial challenge that establishes tone</li>
  <li><strong>Puzzle/Roleplay</strong>: Non-combat challenge requiring thought</li>
  <li><strong>Setback/Twist</strong>: Complication that changes the situation</li>
  <li><strong>Climax/Boss</strong>: Major confrontation or decision point</li>
  <li><strong>Revelation/Reward</strong>: Payoff and hooks for future modules</li>
</ol>

<h4>NPC Creation Priority Order:</h4>
<ol>
  <li><strong>Quest Giver</strong>: Who presents the module's central problem?</li>
  <li><strong>Primary Antagonist</strong>: Who actively opposes the players?</li>
  <li><strong>Key Informant</strong>: Who provides crucial information?</li>
  <li><strong>Wild Card</strong>: Who complicates matters unexpectedly?</li>
  <li><strong>Supporting Cast</strong>: 2-3 minor but memorable NPCs</li>
</ol>

<h4>Encounter Design Balance:</h4>
<ul>
  <li><strong>40% Combat</strong>: Varied difficulty and enemy types</li>
  <li><strong>30% Social</strong>: Negotiations, investigations, persuasion</li>
  <li><strong>20% Exploration</strong>: Discovery, problem-solving, navigation</li>
  <li><strong>10% Unique</strong>: Something specific to this module's theme</li>
</ul>

<h4>Information Architecture:</h4>
<p>Every essential clue needs multiple sources. Players will miss things, NPCs will die unexpectedly, and locations might be skipped. Build redundancy into your information flow. Include 1-2 red herrings maximum, and they should be interesting dead ends, not frustrating wastes of time.</p>"#
                        .to_string()
                ),
            },
            "ready" => StageMetadata {
                display_name: "Ready".to_string(),
                description: "Module is complete and ready to run".to_string(),
                completion_message: Some(
                    "Module is ready to run! You can start it whenever you're ready."
                        .to_string()
                ),
                transition_prompt: Some(
                    "Ready to run this module? Moving to active means you'll begin playing it in your next session."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Module Ready: Final Pressure Testing</h3>
<p>Your module is complete. Now ensure it can survive contact with players. This final review takes 1-2 hours and prevents common failure points.</p>

<h4>The Three-Path Test</h4>
<p>Verify players can complete your module through different approaches:</p>
<ul>
  <li><strong>Combat Path</strong>: Can they fight through every obstacle?</li>
  <li><strong>Social Path</strong>: Can they negotiate and investigate their way through?</li>
  <li><strong>Stealth Path</strong>: Can they avoid direct confrontation entirely?</li>
</ul>
<p>Not every encounter needs all three options, but the module as a whole should support different play styles.</p>

<h4>Failure State Planning</h4>
<p>What happens when things go wrong? Consider these scenarios:</p>
<ul>
  <li>Players miss crucial information despite redundancy</li>
  <li>Key NPCs die before revealing important details</li>
  <li>Players completely skip major locations</li>
  <li>Time pressure expires before objectives are met</li>
</ul>
<p>Each failure should lead somewhere interesting, not to a dead stop.</p>

<h4>Session Breakdown Check</h4>
<ul>
  <li><strong>Session 1</strong>: Hook established, initial challenges faced, clear objectives</li>
  <li><strong>Middle Sessions</strong>: Investigation, development, rising complications</li>
  <li><strong>Penultimate Session</strong>: Point of no return, final preparations</li>
  <li><strong>Final Session</strong>: Climax, resolution, future seeds planted</li>
</ul>

<p class="integration-note">Remember: Each session should end with players eager for the next one. Build in cliffhangers and decision points that matter.</p>"#
                        .to_string()
                ),
            },
            "active" => StageMetadata {
                display_name: "Active".to_string(),
                description: "Module is currently being played".to_string(),
                completion_message: None,
                transition_prompt: Some(
                    "Has the module concluded? Mark it complete to archive it and move on to the next module."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Module Active: Running Your Sessions</h3>
<p>Your module is in play. Use the session tracker above to manage your progress.</p>

<h4>Quick Session Prep Timeline</h4>
<ul>
  <li><strong>T+0</strong>: Capture raw notes immediately (20 min)</li>
  <li><strong>T-6 days</strong>: Sort into Must/Nice/Skip prep tasks (30 min)</li>
  <li><strong>T-4 days</strong>: Core prep work - see checklist below (60-90 min)</li>
  <li><strong>T-3 days</strong>: Create one-page reference (30 min)</li>
</ul>

<h4>The 8-Step Prep Checklist</h4>
<ol>
  <li>Review character status and who needs spotlight</li>
  <li>Create strong opening scene with immediate hook</li>
  <li>Outline 3-5 potential scenes with timing</li>
  <li>Ensure critical info has multiple discovery paths</li>
  <li>Prep key locations with interactive elements</li>
  <li>Define NPC appearances, voices, and wants</li>
  <li>Select encounters with tactics in mind</li>
  <li>Plan rewards that advance character arcs</li>
</ol>

<h4>Session Endings That Work</h4>
<p><strong>Cliffhanger</strong> (door opens) • <strong>Revelation</strong> (everything changes) • <strong>Decision</strong> (what do you do?) • <strong>Victory Cost</strong> (but at what price?)</p>

<p class="integration-note">Remember: When players surprise you, ask "How does this make the story better?" not "How do I get back on track?"</p>"#
                        .to_string()
                ),
            },
            "completed" => StageMetadata {
                display_name: "Completed".to_string(),
                description: "Module has been played and completed".to_string(),
                completion_message: None,
                transition_prompt: None,
                help_text: None,
                content: Some(
                    r#"<h3>Module Completed: Archive and Analysis</h3>
<p>This module's story has concluded. The sessions have been played, the challenges overcome (or not), and the consequences are now part of your campaign's history.</p>

<h4>Capture Key Outcomes</h4>
<p>Document these elements while they're fresh:</p>
<ul>
  <li><strong>Major Decisions</strong>: What choices did players make at crucial moments?</li>
  <li><strong>Surviving NPCs</strong>: Who lived, who died, and how did relationships change?</li>
  <li><strong>Unresolved Threads</strong>: What questions remain unanswered?</li>
  <li><strong>Player Reactions</strong>: What moments generated the strongest responses?</li>
</ul>

<h4>Module Post-Mortem Questions</h4>
<ul>
  <li>Did the module achieve its narrative purpose in the campaign?</li>
  <li>Which prepared content saw use, and what was bypassed entirely?</li>
  <li>How accurate was your session count estimate?</li>
  <li>What would you do differently if running this module again?</li>
</ul>

<h4>Mining for Future Content</h4>
<p>Every completed module provides material for future adventures:</p>
<ul>
  <li><strong>Consequences</strong>: How do this module's outcomes affect the world?</li>
  <li><strong>Callbacks</strong>: Which NPCs or locations could return meaningfully?</li>
  <li><strong>Themes</strong>: What themes resonated that you could explore deeper?</li>
  <li><strong>Preferences</strong>: What did you learn about your players' interests?</li>
</ul>

<p class="integration-note">Session notes and player choices from this module become valuable reference material when players inevitably ask, "Whatever happened to that merchant we saved?" or when you need to show that their actions had lasting impact.</p>"#
                        .to_string()
                ),
            },
            _ => StageMetadata {
                display_name: stage.to_string(),
                description: format!("Module in {} stage", stage),
                completion_message: None,
                transition_prompt: None,
                help_text: None,
                content: None,
            },
        }
    }

    fn no_completion_required_documents(&self, stage: &str) -> Vec<&str> {
        match stage {
            "active" => vec!["document_tracker"],
            _ => vec![],
        }
    }
}
