//! Campaign board definition.

use super::{BoardDefinition, StageMetadata};

/// Campaign workflow board with progression stages.
pub struct CampaignBoard;

impl Default for CampaignBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl CampaignBoard {
    /// Creates a new campaign board.
    pub fn new() -> Self {
        Self
    }
}

impl BoardDefinition for CampaignBoard {
    fn board_type(&self) -> &str {
        "campaign"
    }

    fn stages(&self) -> Vec<&str> {
        vec![
            "concept",
            "session_zero",
            "integration",
            "active",
            "concluding",
            "completed",
        ]
    }

    fn can_transition(&self, from: &str, to: &str) -> bool {
        match (from, to) {
            // Forward progression
            ("concept", "session_zero") => true,
            ("session_zero", "integration") => true,
            ("integration", "active") => true,
            ("active", "concluding") => true,
            ("concluding", "completed") => true,

            _ => false,
        }
    }

    fn required_documents(&self, stage: &str) -> Vec<&str> {
        match stage {
            "concept" => vec!["campaign_pitch"],
            "session_zero" => vec![
                "starting_scenario",
                "world_primer",
                "character_guidelines",
                "table_expectations",
                "character_integration",
            ],
            "integration" => vec!["campaign_bible"],
            "active" => vec![], // No required documents
            "concluding" => vec![],
            "completed" => vec![],
            _ => vec![],
        }
    }

    fn optional_documents(&self, stage: &str) -> Vec<&str> {
        match stage {
            "concept" => vec![], // No optional documents - notes and inspiration are working tools, not artifacts
            "session_zero" => vec!["safety_tools", "house_rules"],
            "integration" => vec!["player_secrets", "faction_overview"],
            "active" => vec![], // No documents in active stage - managed through session boards
            _ => vec![],
        }
    }

    fn next_stage(&self, current: &str) -> Option<&str> {
        match current {
            "concept" => Some("session_zero"),
            "session_zero" => Some("integration"),
            "integration" => Some("active"),
            "active" => Some("concluding"),
            "concluding" => Some("completed"),
            _ => None,
        }
    }

    fn stage_metadata(&self, stage: &str) -> StageMetadata {
        match stage {
            "concept" => StageMetadata {
                display_name: "Concept".to_string(),
                description: "Initial campaign planning and pitch development".to_string(),
                completion_message: Some(
                    "Great! Your campaign pitch is complete. Next, you'll prepare materials for Session Zero."
                        .to_string()
                ),
                transition_prompt: Some(
                    "You can always edit this document later, but make sure your players have a chance to read the pitch and provide initial feedback before Session Zero."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Phase 1: The Spark - Transform Your Idea into a Campaign</h3>
<p>Every campaign begins with a spark—that initial idea that excites you enough to build a world around it. This phase captures and refines that inspiration into a workable campaign concept. Total time investment: 5-8 hours across a week.</p>

<h4>Finding and Refining Your Spark</h4>
<p>Your spark might come from:</p>
<ul>
  <li><strong>Media Inspiration</strong>: "What if Game of Thrones met Aliens?"</li>
  <li><strong>Mechanical Interest</strong>: "I want to run a naval exploration campaign"</li>
  <li><strong>Thematic Question</strong>: "What does it mean to be heroic in a morally gray world?"</li>
  <li><strong>Visual Imagery</strong>: "A city built on the back of a massive, sleeping dragon"</li>
  <li><strong>Player Request</strong>: "We want to play sky pirates!"</li>
</ul>

<h4>Define Your Big Three</h4>
<p>Transform your spark into concrete campaign elements:</p>
<ol>
  <li><strong>Core Conflict</strong>
    <p>The fundamental tension driving the campaign. What's at stake? What happens if no one acts? This should create urgency and agency.</p>
  </li>
  <li><strong>Unique Element</strong>
    <p>What makes this different from generic fantasy? (Though there's nothing wrong with generic fantasy—sometimes you just need to save a princess from a dragon!)</p>
  </li>
  <li><strong>Player Role</strong>
    <p>How do the PCs fit into this world? Are they hired investigators, reluctant heroes, ambitious mercenaries? Give them a clear starting position.</p>
  </li>
</ol>

<h4>Create Your Starting Point</h4>
<p>Define where and how the campaign begins:</p>
<ul>
  <li><strong>Physical Location</strong>: Specific place where Session 1 occurs</li>
  <li><strong>Initial Situation</strong>: The immediate problem or opportunity</li>
  <li><strong>Party Connection</strong>: Why these specific PCs are together</li>
  <li><strong>First Adventure</strong>: Plan a 3-4 session mini-arc to establish tone</li>
</ul>

<h4>Your Campaign Pitch Document</h4>
<p>Write a one-page pitch that excites potential players while setting expectations:</p>
<ul>
  <li><strong>The Hook</strong>: 2-3 sentences that sell the concept</li>
  <li><strong>What Makes It Special</strong>: 3-4 unique aspects</li>
  <li><strong>The Tone</strong>: Reference media or themes they'll recognize</li>
  <li><strong>What You Need From Players</strong>: Commitment level, play style, character requirements</li>
  <li><strong>Practical Details</strong>: System, starting level, session frequency</li>
</ul>

<h4>Phase 1 Timeline</h4>
<ul>
  <li><strong>Days 1-2</strong>: Brainstorm and refine your spark (2 hours)</li>
  <li><strong>Day 3</strong>: Define your Big Three (1 hour)</li>
  <li><strong>Day 4</strong>: Create starting point details (1 hour)</li>
  <li><strong>Day 5</strong>: Write campaign pitch (1 hour)</li>
  <li><strong>Day 6</strong>: Outline first adventure (2 hours)</li>
  <li><strong>Day 7</strong>: Review and polish (1 hour)</li>
</ul>

<p class="integration-note">Remember: Your spark should excite you enough to sustain months or years of play. If you're not enthusiastic about it, your players won't be either. But also remember—campaigns evolve. Your initial concept is a starting point, not a prison.</p>"#
                        .to_string()
                ),
            },
            "session_zero" => StageMetadata {
                display_name: "Session Zero".to_string(),
                description: "Preparing materials for the collaborative session zero".to_string(),
                completion_message: Some(
                    "Excellent! Your Session Zero materials are ready. After your Session Zero, you'll move to the Integration stage."
                        .to_string()
                ),
                transition_prompt: Some(
                    "Remember to share these documents with your players before Session Zero. Take notes during the session as you'll need them for the Integration stage."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Phase 2: Session Zero Preparation</h3>
<p>Session Zero creates the bridge between your vision and your players' expectations. This phase prepares you to run an effective Session Zero that aligns everyone and generates excitement. Time investment: 6-9 hours across a week.</p>

<h4>The Session Zero Packet - Progressive Disclosure</h4>
<p>Create player-facing documents using information layering to prevent overload:</p>

<ol>
  <li><strong>Starting Scenario (1-2 pages)</strong>
    <p>Send T-7 days before Session Zero. Sets the immediate situation:</p>
    <ul>
      <li>Where the characters are right now</li>
      <li>Recent events they might have witnessed</li>
      <li>The immediate opportunity that brings them together</li>
      <li>Questions to consider for character creation</li>
    </ul>
  </li>
  
  <li><strong>World Primer (2-3 pages)</strong>
    <p>Send with full packet at T-3 days. Provides deeper context:</p>
    <ul>
      <li>Brief history (5-6 key events maximum)</li>
      <li>Current situation (2-3 paragraphs)</li>
      <li>Major factions/powers (3-4 with one-line descriptions)</li>
      <li>Common knowledge vs. mysteries</li>
      <li>Geography and important locations</li>
    </ul>
  </li>
  
  <li><strong>Character Guidelines</strong>
    <p>Include with packet. Helps players create appropriate characters:</p>
    <ul>
      <li>Allowed races, classes, and backgrounds</li>
      <li>Starting level and equipment</li>
      <li>Required connections to the setting</li>
      <li>Character concept examples</li>
      <li>What makes a good fit for this campaign</li>
    </ul>
  </li>
  
  <li><strong>Table Expectations</strong>
    <p>Essential for alignment. Cover:</p>
    <ul>
      <li>Tone and themes (with content warnings if needed)</li>
      <li>Player agency vs. railroad expectations</li>
      <li>Combat vs. roleplay vs. exploration balance</li>
      <li>House rules and table etiquette</li>
      <li>Scheduling and attendance expectations</li>
    </ul>
  </li>
  
  <li><strong>Character Integration Worksheet</strong>
    <p>Collaborative tool for Session Zero:</p>
    <ul>
      <li>Three NPCs your character knows</li>
      <li>One secret about the setting</li>
      <li>A rumor they've heard (true or false)</li>
      <li>Connection to at least one other PC</li>
      <li>Personal stake in the opening scenario</li>
    </ul>
  </li>
</ol>

<h4>Session Zero Structure Plan</h4>
<p>Design your 3-4 hour session:</p>
<ul>
  <li><strong>Hour 1</strong>: Introductions, expectations, safety tools</li>
  <li><strong>Hour 2</strong>: Character creation/refinement</li>
  <li><strong>Hour 3</strong>: Party connections and dynamics</li>
  <li><strong>Hour 4</strong>: World building contributions, Q&A</li>
</ul>

<h4>Preparation Timeline</h4>
<ul>
  <li><strong>T-7 days</strong>: Send Starting Scenario to generate excitement</li>
  <li><strong>T-6 days</strong>: Create World Primer (2 hours)</li>
  <li><strong>T-5 days</strong>: Write Character Guidelines (1 hour)</li>
  <li><strong>T-4 days</strong>: Develop Table Expectations (1 hour)</li>
  <li><strong>T-3 days</strong>: Send complete packet to players</li>
  <li><strong>T-2 days</strong>: Create integration worksheets (1 hour)</li>
  <li><strong>T-1 day</strong>: Review and prepare session materials</li>
</ul>

<p class="integration-note">Session Zero is not about playing the game—it's about ensuring everyone wants to play the same game. Take time to listen to player ideas and concerns. The best campaigns grow from this collaborative foundation.</p>"#
                        .to_string()
                ),
            },
            "integration" => StageMetadata {
                display_name: "Integration".to_string(),
                description: "Integrating player feedback and characters into the campaign".to_string(),
                completion_message: Some(
                    "Perfect! Your campaign is fully integrated and ready to begin. Time to start your adventure!"
                        .to_string()
                ),
                transition_prompt: Some(
                    "These documents will be your reference throughout the campaign. Make sure everything from Session Zero has been incorporated."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Phase 4: Integration and Launch</h3>
<p>Transform Session Zero output into your first module. This phase weaves player contributions into playable content while observing what excites your specific group. Time investment: 6-9 hours.</p>

<h4>Your First Module Is Special</h4>
<p>Design it as a testing ground:</p>
<ul>
  <li><strong>Short Duration</strong>: 2-3 sessions (6-10 hours of play)</li>
  <li><strong>Flexible Paths</strong>: Multiple approaches to explore player preferences</li>
  <li><strong>Observable Moments</strong>: Built-in decision points reveal what players enjoy</li>
  <li><strong>Integrated Elements</strong>: Weaves in Session Zero discoveries</li>
</ul>

<h4>Character Integration Process</h4>
<p>For each PC, identify and document:</p>
<ol>
  <li><strong>Immediate Connection</strong>
    <p>Why does this opening problem matter to them personally? Use their backstory, goals, or relationships from Session Zero.</p>
  </li>
  <li><strong>Unique Advantage</strong>
    <p>What skills, knowledge, or connections do they bring that others lack? Plan at least one scene where this shines.</p>
  </li>
  <li><strong>Personal Stakes</strong>
    <p>What do they gain or lose from success or failure? Make it specific to their character, not generic rewards.</p>
  </li>
  <li><strong>Growth Opportunity</strong>
    <p>How does this adventure challenge their beliefs or push them toward their goals?</p>
  </li>
</ol>

<h4>Mining Session Zero</h4>
<p>Catalog player contributions to weave into your module:</p>
<ul>
  <li><strong>NPCs Created</strong>: Use the Character Wizard to create NPCs with full stats, or note simpler NPCs in the Campaign Bible</li>
  <li><strong>Locations Mentioned</strong>: Note places from character histories that could become scenes</li>
  <li><strong>Backstory Elements</strong>: Track rivals, mentors, debts, and obligations to reference</li>
  <li><strong>Stated Interests</strong>: Remember what excited players during discussion</li>
</ul>
<p class="tip"><strong>Tip:</strong> Go to Characters and click Create Character, then select NPC to create NPCs with ability scores, equipment, and backstory.</p>

<h4>First Module Design Goals</h4>
<ul>
  <li><strong>Test Engagement</strong>: Include combat, investigation, and social scenes to see what resonates</li>
  <li><strong>Branch Early</strong>: Offer meaningful choices by end of first session</li>
  <li><strong>Feature Everyone</strong>: Each PC gets at least one spotlight moment per session</li>
  <li><strong>Plant Seeds</strong>: Introduce 3-4 potential future plots without committing</li>
  <li><strong>Stay Flexible</strong>: Don't lock into one campaign direction yet</li>
</ul>

<h4>Creating Your Campaign Bible</h4>
<p>Consolidate everything into your reference document:</p>
<ul>
  <li><strong>Character Profiles</strong>: One page per PC with connections, goals, and likely approaches</li>
  <li><strong>NPC Roster</strong>: Quick reference of all NPCs with relationships mapped</li>
  <li><strong>Location Guide</strong>: Key places with who controls them and what happens there</li>
  <li><strong>Faction Summary</strong>: Who wants what and how PCs relate to each</li>
  <li><strong>Open Threads</strong>: List of mysteries, conflicts, and opportunities to develop</li>
</ul>

<h4>Major NPC Development</h4>
<p>For significant NPCs who may need stats in combat, create them using the Character Wizard (Characters > Create Character > NPC). For all NPCs, consider:</p>
<ul>
  <li><strong>Appearance</strong>: One memorable physical trait</li>
  <li><strong>Voice</strong>: Speech pattern or verbal tic</li>
  <li><strong>Want</strong>: What they're trying to achieve</li>
  <li><strong>Leverage</strong>: What they can offer or threaten</li>
  <li><strong>Secret</strong>: Something not immediately obvious</li>
</ul>

<h4>Integration Timeline</h4>
<ul>
  <li><strong>T+1 day after Session Zero</strong>: Review notes while fresh (1 hour)</li>
  <li><strong>Day 2</strong>: Create character integration profiles (2 hours)</li>
  <li><strong>Day 3</strong>: Develop Campaign Bible structure (2 hours)</li>
  <li><strong>Day 4</strong>: Design major NPCs (2 hours)</li>
  <li><strong>Day 5-6</strong>: Create first module using Module Creation process</li>
  <li><strong>Day 7</strong>: Final review and polish</li>
</ul>

<p class="integration-note">Your first module is a testing ground, not a commitment. Watch what makes players lean forward, take notes on what they engage with, and use these observations to shape the campaign's true direction. The best campaigns grow from this patient observation.</p>"#
                        .to_string()
                ),
            },
            "active" => StageMetadata {
                display_name: "Active".to_string(),
                description: "Campaign is actively being played".to_string(),
                completion_message: Some(
                    "Your campaign has been an amazing journey! Time to bring it to a conclusion."
                        .to_string()
                ),
                transition_prompt: Some(
                    "Is your campaign approaching its finale? Move to concluding when you're ready to wrap up the story."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Campaign Active: Managing Your Living World</h3>
<p>Your campaign is now active! Create modules below to structure your adventures while maintaining narrative momentum.</p>

<h4>Quick Reference: Module Rhythm</h4>
<ul>
  <li><strong>Planning Time</strong>: 8-12 hours creates 12-20 hours of play</li>
  <li><strong>Session Prep</strong>: 1 hour creates 4 hours of play</li>
  <li><strong>Between Modules</strong>: Take a week to assess and plan next arc</li>
</ul>

<h4>When to Create New Modules</h4>
<ul>
  <li>Current module has 1-2 sessions remaining</li>
  <li>Story reaches a natural transition point</li>
  <li>Players achieve major goal requiring new direction</li>
  <li>Campaign shifts between major acts or themes</li>
</ul>

<h4>Health Check</h4>
<p><strong>Green Flags:</strong> Players discuss game between sessions, reference past events, make long-term plans</p>
<p><strong>Warning Signs:</strong> Aimless sessions, DM burnout, scattered focus, attendance issues</p>

<p class="integration-note">Remember: Sustainable pacing beats perfect preparation. Use the module system below to maintain structure while leaving room for emergent storytelling.</p>"#
                        .to_string()
                ),
            },
            "concluding" => StageMetadata {
                display_name: "Concluding".to_string(),
                description: "Campaign is wrapping up its final story arcs".to_string(),
                completion_message: Some(
                    "Congratulations! Your campaign has reached its epic conclusion."
                        .to_string()
                ),
                transition_prompt: Some(
                    "Have all story arcs been resolved? Mark the campaign as completed to archive it."
                        .to_string()
                ),
                help_text: None,
                content: Some(
                    r#"<h3>Phase 5: Campaign Conclusion</h3>
<p>Your campaign is approaching its natural end. Following the Campaign Genesis framework, Phase 5 guides you through crafting a memorable finale.</p>

<h4>The 8-Session Countdown</h4>
<p>Work backwards from your intended conclusion:</p>
<ul>
  <li><strong>8 sessions before</strong>: Announce the campaign is entering its final arc</li>
  <li><strong>6 sessions before</strong>: Begin resolving secondary storylines</li>
  <li><strong>4 sessions before</strong>: Focus entirely on the primary story arc</li>
  <li><strong>2 sessions before</strong>: Set up the climactic confrontation</li>
  <li><strong>Final session</strong>: The climax (consider extending to 5-6 hours)</li>
  <li><strong>Epilogue session</strong>: Optional but recommended - jump forward in time</li>
</ul>

<h4>What Needs Resolution</h4>
<p>From the documentation on campaign conclusions:</p>
<ul>
  <li><strong>Must Resolve</strong>: Primary conflict, PC personal arcs, major NPCs, world changes</li>
  <li><strong>Can Leave Open</strong>: Minor mysteries, secondary NPCs, sequel hooks</li>
</ul>

<h4>Designing Your Final Module</h4>
<ul>
  <li>Callback to earlier events and Session 1</li>
  <li>Showcase how PCs have grown</li>
  <li>Make it personal to the characters</li>
  <li>Let player choices shape the ending</li>
</ul>

<h4>Common Pitfalls</h4>
<ul>
  <li>Don't introduce new plot elements</li>
  <li>Avoid deus ex machina - let PCs be the heroes</li>
  <li>Don't rush - take the sessions you need</li>
  <li>Remember: plot serves characters, not vice versa</li>
</ul>

<p class="integration-note">See the full Phase 5 documentation for detailed guidance on preserving your campaign, running epilogue sessions, and transitioning to your next campaign.</p>"#
                        .to_string()
                ),
            },
            "completed" => StageMetadata {
                display_name: "Completed".to_string(),
                description: "Campaign has been completed and archived".to_string(),
                completion_message: None,
                transition_prompt: None,
                help_text: None,
                content: Some(
                    r#"<h3>Campaign Complete</h3>
<p>Your campaign has reached its conclusion. This archive preserves all your campaign materials for future reference.</p>

<h4>What's Preserved</h4>
<ul>
  <li>All modules and adventures</li>
  <li>Session notes and records</li>
  <li>Character progression documents</li>
  <li>World building materials</li>
  <li>Campaign reference documents</li>
</ul>

<h4>Using This Archive</h4>
<p>Your completed campaign materials can be:</p>
<ul>
  <li>Referenced for sequel campaigns in the same world</li>
  <li>Mined for successful modules to reuse</li>
  <li>Reviewed to understand what worked well</li>
  <li>Shared with other DMs as examples</li>
</ul>

<h4>Starting Your Next Campaign</h4>
<p>When you're ready for a new campaign:</p>
<ul>
  <li>Take a break to let this campaign settle</li>
  <li>Discuss with players what they want next</li>
  <li>Apply lessons learned from this experience</li>
  <li>Begin fresh with a new Campaign Genesis process</li>
</ul>

<p class="integration-note">Completing a campaign is a significant achievement. Your shared story will be remembered and referenced for years to come.</p>"#
                        .to_string()
                ),
            },
            _ => StageMetadata {
                display_name: stage.to_string(),
                description: format!("The {} stage", stage),
                completion_message: None,
                transition_prompt: None,
                help_text: None,
                content: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_type() {
        let board = CampaignBoard::new();
        assert_eq!(board.board_type(), "campaign");
    }

    #[test]
    fn test_stages_order() {
        let board = CampaignBoard::new();
        let stages = board.stages();

        assert_eq!(stages.len(), 6);
        assert_eq!(stages[0], "concept");
        assert_eq!(stages[1], "session_zero");
        assert_eq!(stages[2], "integration");
        assert_eq!(stages[3], "active");
        assert_eq!(stages[4], "concluding");
        assert_eq!(stages[5], "completed");
    }

    #[test]
    fn test_valid_forward_transitions() {
        let board = CampaignBoard::new();

        // Test all valid forward transitions
        assert!(board.can_transition("concept", "session_zero"));
        assert!(board.can_transition("session_zero", "integration"));
        assert!(board.can_transition("integration", "active"));
        assert!(board.can_transition("active", "concluding"));
        assert!(board.can_transition("concluding", "completed"));
    }

    #[test]
    fn test_invalid_transitions() {
        let board = CampaignBoard::new();

        // Test backward transitions (not allowed)
        assert!(!board.can_transition("session_zero", "concept"));
        assert!(!board.can_transition("integration", "session_zero"));
        assert!(!board.can_transition("active", "integration"));

        // Test skip transitions (not allowed)
        assert!(!board.can_transition("concept", "integration"));
        assert!(!board.can_transition("concept", "active"));
        assert!(!board.can_transition("session_zero", "active"));

        // Test self-transitions (not allowed)
        assert!(!board.can_transition("concept", "concept"));
        assert!(!board.can_transition("active", "active"));

        // Test from completed (no transitions allowed)
        assert!(!board.can_transition("completed", "concept"));
        assert!(!board.can_transition("completed", "active"));

        // Test invalid stage names
        assert!(!board.can_transition("invalid", "concept"));
        assert!(!board.can_transition("concept", "invalid"));
    }

    #[test]
    fn test_required_documents_per_stage() {
        let board = CampaignBoard::new();

        // Concept stage
        let concept_docs = board.required_documents("concept");
        assert_eq!(concept_docs.len(), 1);
        assert_eq!(concept_docs[0], "campaign_pitch");

        // Session Zero stage
        let session_zero_docs = board.required_documents("session_zero");
        assert_eq!(session_zero_docs.len(), 5);
        assert!(session_zero_docs.contains(&"starting_scenario"));
        assert!(session_zero_docs.contains(&"world_primer"));
        assert!(session_zero_docs.contains(&"character_guidelines"));
        assert!(session_zero_docs.contains(&"table_expectations"));
        assert!(session_zero_docs.contains(&"character_integration"));

        // Integration stage
        let integration_docs = board.required_documents("integration");
        assert_eq!(integration_docs.len(), 1);
        assert!(integration_docs.contains(&"campaign_bible"));

        // Active stage (no required documents)
        assert_eq!(board.required_documents("active").len(), 0);
        assert_eq!(board.required_documents("concluding").len(), 0);
        assert_eq!(board.required_documents("completed").len(), 0);

        // Invalid stage
        assert_eq!(board.required_documents("invalid").len(), 0);
    }

    #[test]
    fn test_optional_documents_per_stage() {
        let board = CampaignBoard::new();

        // Concept stage - no optional documents (notes and inspiration are tools, not artifacts)
        let concept_optional = board.optional_documents("concept");
        assert_eq!(concept_optional.len(), 0);

        // Session Zero stage
        let session_zero_optional = board.optional_documents("session_zero");
        assert_eq!(session_zero_optional.len(), 2);
        assert!(session_zero_optional.contains(&"safety_tools"));
        assert!(session_zero_optional.contains(&"house_rules"));

        // Integration stage
        let integration_optional = board.optional_documents("integration");
        assert_eq!(integration_optional.len(), 2);
        assert!(integration_optional.contains(&"player_secrets"));
        assert!(integration_optional.contains(&"faction_overview"));

        // Active stage - no documents (managed through session boards)
        let active_optional = board.optional_documents("active");
        assert_eq!(active_optional.len(), 0);

        // Stages with no optional documents
        assert_eq!(board.optional_documents("concluding").len(), 0);
        assert_eq!(board.optional_documents("completed").len(), 0);
        assert_eq!(board.optional_documents("invalid").len(), 0);
    }

    #[test]
    fn test_next_stage_progression() {
        let board = CampaignBoard::new();

        assert_eq!(board.next_stage("concept"), Some("session_zero"));
        assert_eq!(board.next_stage("session_zero"), Some("integration"));
        assert_eq!(board.next_stage("integration"), Some("active"));
        assert_eq!(board.next_stage("active"), Some("concluding"));
        assert_eq!(board.next_stage("concluding"), Some("completed"));
        assert_eq!(board.next_stage("completed"), None);
        assert_eq!(board.next_stage("invalid"), None);
    }

    #[test]
    fn test_stage_metadata_completeness() {
        let board = CampaignBoard::new();

        // Test that all stages have metadata
        for stage in board.stages() {
            let metadata = board.stage_metadata(stage);
            assert!(!metadata.display_name.is_empty());
            assert!(!metadata.description.is_empty());
        }

        // Test specific metadata for concept stage
        let concept_meta = board.stage_metadata("concept");
        assert_eq!(concept_meta.display_name, "Concept");
        assert!(concept_meta.description.contains("planning"));
        assert!(concept_meta.completion_message.is_some());
        assert!(concept_meta.transition_prompt.is_some());
        assert!(concept_meta.content.is_some()); // Check for content field instead of help_text

        // Test specific metadata for session_zero stage
        let session_zero_meta = board.stage_metadata("session_zero");
        assert_eq!(session_zero_meta.display_name, "Session Zero");
        assert!(session_zero_meta.description.contains("collaborative"));
        assert!(session_zero_meta.completion_message.is_some());
        assert!(session_zero_meta.transition_prompt.is_some());
        assert!(session_zero_meta.content.is_some()); // Check for content field instead of help_text

        // Test specific metadata for integration stage
        let integration_meta = board.stage_metadata("integration");
        assert_eq!(integration_meta.display_name, "Integration");
        assert!(integration_meta.description.contains("player feedback"));
        assert!(integration_meta.completion_message.is_some());
        assert!(integration_meta.transition_prompt.is_some());
        assert!(integration_meta.content.is_some()); // Check for content field instead of help_text

        // Test specific metadata for active stage
        let active_meta = board.stage_metadata("active");
        assert_eq!(active_meta.display_name, "Active");
        assert!(active_meta.description.contains("actively being played"));
        assert!(active_meta.completion_message.is_some());
        assert!(active_meta.transition_prompt.is_some());
        assert!(active_meta.content.is_some());

        // Test fallback metadata for unknown stage
        let unknown_meta = board.stage_metadata("unknown");
        assert_eq!(unknown_meta.display_name, "unknown");
        assert_eq!(unknown_meta.description, "The unknown stage");
        assert!(unknown_meta.completion_message.is_none());
        assert!(unknown_meta.transition_prompt.is_none());
        assert!(unknown_meta.content.is_none());
    }

    #[test]
    fn test_stage_progression_completeness() {
        let board = CampaignBoard::new();
        let stages = board.stages();

        // Verify that each stage (except the last) has a next stage
        for i in 0..stages.len() - 1 {
            let current = stages[i];
            let expected_next = stages[i + 1];
            assert_eq!(board.next_stage(current), Some(expected_next));
        }

        // Verify the last stage has no next stage
        assert_eq!(board.next_stage(stages[stages.len() - 1]), None);
    }

    #[test]
    fn test_transition_consistency_with_next_stage() {
        let board = CampaignBoard::new();

        // For each stage that has a next stage, verify can_transition agrees
        for stage in board.stages() {
            if let Some(next) = board.next_stage(stage) {
                assert!(
                    board.can_transition(stage, next),
                    "Stage {} should be able to transition to next stage {}",
                    stage,
                    next
                );
            }
        }
    }

    #[test]
    fn test_no_orphaned_transitions() {
        let board = CampaignBoard::new();
        let valid_stages: Vec<&str> = board.stages();

        // Test that can_transition only returns true for valid stage pairs
        for from in &valid_stages {
            for to in &valid_stages {
                if board.can_transition(from, to) {
                    // If transition is allowed, verify it matches next_stage
                    assert_eq!(
                        board.next_stage(from),
                        Some(*to),
                        "Transition from {} to {} is allowed but doesn't match next_stage",
                        from,
                        to
                    );
                }
            }
        }
    }
}
