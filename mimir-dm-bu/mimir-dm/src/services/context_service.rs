use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub id: String,
    #[serde(rename = "type")]
    pub window_type: String,
    pub title: String,
    pub focused: bool,
    pub route: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    pub timestamp: u64,
    #[serde(rename = "type")]
    pub action_type: String,
    pub description: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignContext {
    pub id: Option<String>,
    pub name: Option<String>,
    pub current_stage: Option<String>,
    pub current_document: Option<String>,
    pub modules: Option<Vec<ModuleInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleContext {
    pub id: Option<String>,
    pub name: Option<String>,
    pub campaign_id: Option<String>,
    pub current_stage: Option<String>,
    pub sessions: Option<Vec<SessionInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub id: Option<String>,
    pub name: Option<String>,
    pub module_id: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingContext {
    pub current_book: Option<String>,
    pub current_section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogContext {
    pub selected_category: Option<String>,
    pub selected_items: Option<Vec<String>>,
    pub search_query: Option<String>,
    pub selected_sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceContext {
    pub active_tab: Option<String>,
    pub reading: Option<ReadingContext>,
    pub catalog: Option<CatalogContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    pub campaign: Option<CampaignContext>,
    pub module: Option<ModuleContext>,
    pub session: Option<SessionContext>,
    pub reference: Option<ReferenceContext>,
    pub windows: HashMap<String, WindowState>,
    pub recent_actions: Vec<UserAction>,
    pub context_usage: Option<usize>,
}

pub struct ContextService {
    context: Arc<Mutex<SharedContext>>,
}

impl Default for ContextService {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextService {
    pub fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(SharedContext {
                campaign: None,
                module: None,
                session: None,
                reference: None,
                windows: HashMap::new(),
                recent_actions: Vec::new(),
                context_usage: None,
            })),
        }
    }

    pub fn update_context(&self, context_type: &str, data: &str) -> Result<(), String> {
        let mut context = self.context.lock().map_err(|e| e.to_string())?;

        match context_type {
            "campaign" => {
                let campaign_data: CampaignContext = serde_json::from_str(data)
                    .map_err(|e| format!("Failed to parse campaign context: {}", e))?;
                context.campaign = Some(campaign_data);
                debug!("Updated campaign context");
            }
            "module" => {
                let module_data: ModuleContext = serde_json::from_str(data)
                    .map_err(|e| format!("Failed to parse module context: {}", e))?;
                context.module = Some(module_data);
                debug!("Updated module context");
            }
            "session" => {
                let session_data: SessionContext = serde_json::from_str(data)
                    .map_err(|e| format!("Failed to parse session context: {}", e))?;
                context.session = Some(session_data);
                debug!("Updated session context");
            }
            "reference" => {
                let reference_data: ReferenceContext = serde_json::from_str(data)
                    .map_err(|e| format!("Failed to parse reference context: {}", e))?;
                context.reference = Some(reference_data);
                debug!("Updated reference context");
            }
            "windows" => {
                let windows_data: Vec<WindowState> = serde_json::from_str(data)
                    .map_err(|e| format!("Failed to parse windows context: {}", e))?;
                context.windows = windows_data
                    .into_iter()
                    .map(|w| (w.id.clone(), w))
                    .collect();
                debug!("Updated windows context");
            }
            "actions" => {
                let actions_data: Vec<UserAction> = serde_json::from_str(data)
                    .map_err(|e| format!("Failed to parse actions context: {}", e))?;
                context.recent_actions = actions_data;
                debug!("Updated recent actions");
            }
            _ => {
                return Err(format!("Unknown context type: {}", context_type));
            }
        }

        Ok(())
    }

    pub fn get_full_context(&self) -> Result<String, String> {
        let context = self.context.lock().map_err(|e| e.to_string())?;
        serde_json::to_string(&*context).map_err(|e| format!("Failed to serialize context: {}", e))
    }

    pub fn register_window(
        &self,
        window_id: &str,
        window_type: &str,
        title: &str,
    ) -> Result<(), String> {
        let mut context = self.context.lock().map_err(|e| e.to_string())?;

        let window_state = WindowState {
            id: window_id.to_string(),
            window_type: window_type.to_string(),
            title: title.to_string(),
            focused: false,
            route: None,
        };

        context.windows.insert(window_id.to_string(), window_state);
        info!("Registered window: {} ({})", window_id, window_type);

        Ok(())
    }

    pub fn unregister_window(&self, window_id: &str) -> Result<(), String> {
        let mut context = self.context.lock().map_err(|e| e.to_string())?;
        context.windows.remove(window_id);
        info!("Unregistered window: {}", window_id);
        Ok(())
    }

    pub fn clear_context(&self) -> Result<(), String> {
        let mut context = self.context.lock().map_err(|e| e.to_string())?;
        *context = SharedContext {
            campaign: None,
            module: None,
            session: None,
            reference: None,
            windows: context.windows.clone(), // Keep windows registered
            recent_actions: Vec::new(),
            context_usage: None,
        };
        info!("Cleared shared context");
        Ok(())
    }

    pub fn update_context_usage(&self, usage: usize) -> Result<(), String> {
        let mut context = self.context.lock().map_err(|e| e.to_string())?;
        context.context_usage = Some(usage);
        Ok(())
    }

    pub fn get_context_for_llm(&self) -> Result<String, String> {
        let context = self.context.lock().map_err(|e| e.to_string())?;

        // Build a structured context string optimized for LLM consumption
        let mut llm_context = Vec::new();

        if let Some(ref campaign) = context.campaign {
            llm_context.push(format!(
                "Current Campaign: {} (ID: {})",
                campaign.name.as_ref().unwrap_or(&"Unknown".to_string()),
                campaign.id.as_ref().unwrap_or(&"None".to_string())
            ));

            if let Some(ref stage) = campaign.current_stage {
                llm_context.push(format!("Campaign Stage: {}", stage));
            }

            if let Some(ref doc) = campaign.current_document {
                llm_context.push(format!("Current Document: {}", doc));
            }
        }

        if let Some(ref module) = context.module {
            llm_context.push(format!(
                "Active Module: {} (ID: {})",
                module.name.as_ref().unwrap_or(&"Unknown".to_string()),
                module.id.as_ref().unwrap_or(&"None".to_string())
            ));
        }

        if let Some(ref session) = context.session {
            llm_context.push(format!(
                "Current Session: {} (Status: {})",
                session.name.as_ref().unwrap_or(&"Unknown".to_string()),
                session.status.as_ref().unwrap_or(&"Unknown".to_string())
            ));
        }

        if let Some(ref reference) = context.reference {
            if let Some(ref active_tab) = reference.active_tab {
                llm_context.push(format!("Reference Mode: {}", active_tab));
            }

            if let Some(ref reading) = reference.reading {
                if let Some(ref book) = reading.current_book {
                    llm_context.push(format!("Reading Book: {}", book));
                }
                if let Some(ref section) = reading.current_section {
                    llm_context.push(format!("Current Section: {}", section));
                }
            }

            if let Some(ref catalog) = reference.catalog {
                if let Some(ref category) = catalog.selected_category {
                    llm_context.push(format!("Catalog Category: {}", category));
                }
                if let Some(ref sources) = catalog.selected_sources {
                    if !sources.is_empty() {
                        llm_context.push(format!("Selected Sources: {}", sources.join(", ")));
                    }
                }
                if let Some(ref query) = catalog.search_query {
                    if !query.is_empty() {
                        llm_context.push(format!("Search Query: {}", query));
                    }
                }
                if let Some(ref items) = catalog.selected_items {
                    if !items.is_empty() {
                        llm_context.push(format!("Selected Items: {}", items.join(", ")));
                    }
                }
            }
        }

        if !context.recent_actions.is_empty() {
            llm_context.push("Recent Actions:".to_string());
            for action in context.recent_actions.iter().take(5) {
                llm_context.push(format!("  - {}", action.description));
            }
        }

        Ok(llm_context.join("\n"))
    }
}

// Tauri state wrapper
pub struct ContextState(pub Arc<ContextService>);

impl Default for ContextState {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextState {
    pub fn new() -> Self {
        Self(Arc::new(ContextService::new()))
    }
}
