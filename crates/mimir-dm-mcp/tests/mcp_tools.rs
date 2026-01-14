//! MCP tool integration tests.
//!
//! Tests for MCP tool handlers to verify they work correctly
//! with proper state management and error handling.

mod common;

mod campaign_tools {
    use super::common::TestMcpEnv;
    use mimir_dm_mcp::tools::{ListCampaignsInput, SetActiveCampaignInput};

    #[tokio::test]
    async fn test_list_campaigns_empty() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let input = ListCampaignsInput::default();
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list campaigns");

        assert!(result.is_empty(), "Should have no campaigns initially");
    }

    #[tokio::test]
    async fn test_list_campaigns_with_campaigns() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        // Create some campaigns
        env.create_campaign("Campaign One").expect("Failed to create campaign 1");
        env.create_campaign("Campaign Two").expect("Failed to create campaign 2");

        let input = ListCampaignsInput::default();
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list campaigns");

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.name == "Campaign One"));
        assert!(result.iter().any(|c| c.name == "Campaign Two"));
    }

    #[tokio::test]
    async fn test_list_campaigns_excludes_archived_by_default() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        // Create campaigns
        let _ = env.create_campaign("Active Campaign").expect("Failed to create campaign");
        let (campaign2_id, _) = env.create_campaign("Archived Campaign").expect("Failed to create campaign");

        // Archive one campaign
        {
            let mut conn = env.context.get_connection().expect("Failed to get connection");
            let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);
            service.archive_campaign(campaign2_id).expect("Failed to archive");
        }

        let input = ListCampaignsInput { include_archived: false };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list campaigns");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Active Campaign");
    }

    #[tokio::test]
    async fn test_list_campaigns_includes_archived_when_requested() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        // Create campaigns
        let _ = env.create_campaign("Active Campaign").expect("Failed to create campaign");
        let (campaign2_id, _) = env.create_campaign("Archived Campaign").expect("Failed to create campaign");

        // Archive one campaign
        {
            let mut conn = env.context.get_connection().expect("Failed to get connection");
            let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);
            service.archive_campaign(campaign2_id).expect("Failed to archive");
        }

        let input = ListCampaignsInput { include_archived: true };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list campaigns");

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_set_active_campaign_success() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");

        let input = SetActiveCampaignInput { campaign_id };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to set active campaign");

        assert!(result.success);
        assert_eq!(result.campaign_id, campaign_id);
        assert_eq!(result.campaign_name, "Test Campaign");

        // Verify context was updated
        let active = env.context.get_active_campaign().await;
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, campaign_id);
    }

    #[tokio::test]
    async fn test_set_active_campaign_not_found() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let input = SetActiveCampaignInput { campaign_id: 99999 };
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail for non-existent campaign");
    }

    #[tokio::test]
    async fn test_tool_definitions() {
        // Verify tool definitions are correctly structured
        let list_tool = ListCampaignsInput::tool();
        assert_eq!(list_tool.name, "list_campaigns");
        assert!(list_tool.description.is_some());

        let set_tool = SetActiveCampaignInput::tool();
        assert_eq!(set_tool.name, "set_active_campaign");
        assert!(set_tool.description.is_some());
    }
}

mod module_tools {
    use super::common::TestMcpEnv;
    use mimir_dm_mcp::tools::{CreateModuleInput, GetModuleDetailsInput, ListModulesInput};

    #[tokio::test]
    async fn test_list_modules_requires_active_campaign() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let input = ListModulesInput::default();
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail without active campaign");
    }

    #[tokio::test]
    async fn test_list_modules_empty() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = ListModulesInput::default();
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list modules");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_create_module() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = CreateModuleInput {
            name: "Lost Mines of Phandelver".to_string(),
            module_type: None,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to create module");

        assert!(result.module_id > 0);
        assert_eq!(result.name, "Lost Mines of Phandelver");
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_create_module_with_type() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = CreateModuleInput {
            name: "Dragon's Lair".to_string(),
            module_type: Some("dungeon".to_string()),
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to create module");

        assert!(result.module_id > 0);
        assert_eq!(result.name, "Dragon's Lair");
    }

    #[tokio::test]
    async fn test_list_modules_with_modules() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create modules
        let input1 = CreateModuleInput {
            name: "Module One".to_string(),
            module_type: None,
        };
        input1.execute(env.context.clone()).await.expect("Failed to create");

        let input2 = CreateModuleInput {
            name: "Module Two".to_string(),
            module_type: None,
        };
        input2.execute(env.context.clone()).await.expect("Failed to create");

        let list_input = ListModulesInput::default();
        let result = list_input
            .execute(env.context.clone())
            .await
            .expect("Failed to list modules");

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_get_module_details() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create a module
        let create_input = CreateModuleInput {
            name: "Detailed Module".to_string(),
            module_type: None,
        };
        let created = create_input
            .execute(env.context.clone())
            .await
            .expect("Failed to create");

        // Get details
        let input = GetModuleDetailsInput {
            module_id: created.module_id,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to get details");

        assert_eq!(result.id, created.module_id);
        assert_eq!(result.name, "Detailed Module");
        // Module should have some default documents created
        assert!(!result.documents.is_empty());
    }

    #[tokio::test]
    async fn test_get_module_details_not_found() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = GetModuleDetailsInput { module_id: 99999 };
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail for non-existent module");
    }
}

mod document_tools {
    use super::common::TestMcpEnv;
    use mimir_dm_mcp::tools::{
        CreateModuleInput, CreateUserDocumentInput, EditDocumentInput, ListDocumentsInput,
        ReadDocumentInput,
    };

    #[tokio::test]
    async fn test_list_documents_requires_active_campaign() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let input = ListDocumentsInput::default();
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail without active campaign");
    }

    #[tokio::test]
    async fn test_list_documents_for_module() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create a module (which creates default documents)
        let create_input = CreateModuleInput {
            name: "Doc Test Module".to_string(),
            module_type: None,
        };
        let module = create_input
            .execute(env.context.clone())
            .await
            .expect("Failed to create");

        let input = ListDocumentsInput {
            level: Some("module".to_string()),
            module_id: Some(module.module_id),
            session_id: None,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list documents");

        // Module creation should have created some system documents
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_create_user_document() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create a module first
        let module_input = CreateModuleInput {
            name: "User Doc Module".to_string(),
            module_type: None,
        };
        let module = module_input
            .execute(env.context.clone())
            .await
            .expect("Failed to create module");

        // Create a user document
        let input = CreateUserDocumentInput {
            title: "Custom Notes".to_string(),
            content: Some("# My Notes\n\nInitial content.".to_string()),
            module_id: Some(module.module_id),
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to create document");

        assert!(result.document_id > 0);
        assert_eq!(result.title, "Custom Notes");
    }

    #[tokio::test]
    async fn test_read_document() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create a module (which creates documents)
        let module_input = CreateModuleInput {
            name: "Read Doc Module".to_string(),
            module_type: None,
        };
        let module = module_input
            .execute(env.context.clone())
            .await
            .expect("Failed to create module");

        // List documents to get an ID
        let list_input = ListDocumentsInput {
            level: Some("module".to_string()),
            module_id: Some(module.module_id),
            session_id: None,
        };
        let docs = list_input
            .execute(env.context.clone())
            .await
            .expect("Failed to list");

        assert!(!docs.is_empty(), "Should have documents");

        // Read the first document
        let input = ReadDocumentInput {
            document_id: docs[0].id,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to read document");

        assert_eq!(result.id, docs[0].id);
    }

    #[tokio::test]
    async fn test_read_document_not_found() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = ReadDocumentInput { document_id: 99999 };
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail for non-existent document");
    }

    #[tokio::test]
    async fn test_edit_document() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create a user document that we can edit
        let create_doc = CreateUserDocumentInput {
            title: "Editable Doc".to_string(),
            content: Some("# Editable\n\nOriginal content here.".to_string()),
            module_id: None,
        };
        let doc = create_doc
            .execute(env.context.clone())
            .await
            .expect("Failed to create doc");

        // Edit the document
        let edit_input = EditDocumentInput {
            document_id: doc.document_id,
            search: "Original content here.".to_string(),
            replace: "Updated content!".to_string(),
            replace_all: false,
        };
        let result = edit_input
            .execute(env.context.clone())
            .await
            .expect("Failed to edit document");

        assert!(result.success);
        assert_eq!(result.replacements_made, 1);

        // Verify the edit
        let read_input = ReadDocumentInput {
            document_id: doc.document_id,
        };
        let read_result = read_input
            .execute(env.context.clone())
            .await
            .expect("Failed to read");

        assert!(read_result.content.contains("Updated content!"));
    }
}

mod character_tools {
    use super::common::TestMcpEnv;
    use mimir_dm_mcp::tools::{
        CreateNpcInput, GetCharacterInput, ListCharactersInput, UpdateCharacterCurrencyInput,
    };

    #[tokio::test]
    async fn test_list_characters_requires_active_campaign() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let input = ListCharactersInput::default();
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail without active campaign");
    }

    #[tokio::test]
    async fn test_list_characters_empty() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = ListCharactersInput::default();
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list characters");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_create_npc() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = CreateNpcInput {
            name: "Shopkeeper Bob".to_string(),
            race: "Human".to_string(),
            class: None,
            role: Some("Merchant".to_string()),
            location: Some("Market Square".to_string()),
            faction: None,
            notes: None,
            alignment: None,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to create NPC");

        assert!(result.character_id > 0);
        assert_eq!(result.name, "Shopkeeper Bob");
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_list_characters_with_npcs() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create NPCs
        let npc1 = CreateNpcInput {
            name: "Guard Captain".to_string(),
            race: "Human".to_string(),
            class: None,
            role: Some("Guard".to_string()),
            location: None,
            faction: None,
            notes: None,
            alignment: None,
        };
        npc1.execute(env.context.clone()).await.expect("Failed to create");

        let npc2 = CreateNpcInput {
            name: "Innkeeper".to_string(),
            race: "Halfling".to_string(),
            class: None,
            role: Some("Merchant".to_string()),
            location: Some("Tavern".to_string()),
            faction: None,
            notes: None,
            alignment: None,
        };
        npc2.execute(env.context.clone()).await.expect("Failed to create");

        let input = ListCharactersInput::default();
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list");

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_list_characters_filter_npcs() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create an NPC
        let npc = CreateNpcInput {
            name: "Test NPC".to_string(),
            race: "Elf".to_string(),
            class: None,
            role: None,
            location: None,
            faction: None,
            notes: None,
            alignment: None,
        };
        npc.execute(env.context.clone()).await.expect("Failed to create");

        // Filter for NPCs only
        let input = ListCharactersInput {
            character_type: Some("npc".to_string()),
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to list");

        assert_eq!(result.len(), 1);
        assert!(result[0].is_npc);
    }

    #[tokio::test]
    async fn test_get_character() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create an NPC
        let npc = CreateNpcInput {
            name: "Detailed NPC".to_string(),
            race: "Dwarf".to_string(),
            class: None,
            role: Some("Blacksmith".to_string()),
            location: Some("Forge".to_string()),
            faction: None,
            notes: None,
            alignment: None,
        };
        let created = npc.execute(env.context.clone()).await.expect("Failed to create");

        // Get character details
        let input = GetCharacterInput {
            character_id: created.character_id,
            include_versions: false,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to get character");

        assert_eq!(result.id, created.character_id);
        assert_eq!(result.name, "Detailed NPC");
        assert_eq!(result.race, "Dwarf");
    }

    #[tokio::test]
    async fn test_get_character_not_found() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        let input = GetCharacterInput {
            character_id: 99999,
            include_versions: false,
        };
        let result = input.execute(env.context.clone()).await;

        assert!(result.is_err(), "Should fail for non-existent character");
    }

    #[tokio::test]
    async fn test_update_character_currency() {
        let env = TestMcpEnv::new().expect("Failed to create test environment");

        let (campaign_id, _) = env.create_campaign("Test Campaign").expect("Failed to create campaign");
        env.set_active_campaign(campaign_id).await.expect("Failed to set active");

        // Create an NPC
        let npc = CreateNpcInput {
            name: "Wealthy NPC".to_string(),
            race: "Human".to_string(),
            class: None,
            role: None,
            location: None,
            faction: None,
            notes: None,
            alignment: None,
        };
        let created = npc.execute(env.context.clone()).await.expect("Failed to create");

        // Update currency
        let input = UpdateCharacterCurrencyInput {
            character_id: created.character_id,
            copper: 50,
            silver: 25,
            electrum: 0,
            gold: 100,
            platinum: 5,
        };
        let result = input
            .execute(env.context.clone())
            .await
            .expect("Failed to update currency");

        assert!(result.success);
        assert_eq!(result.current_currency.copper, 50);
        assert_eq!(result.current_currency.silver, 25);
        assert_eq!(result.current_currency.gold, 100);
        assert_eq!(result.current_currency.platinum, 5);
    }
}

mod tool_discovery {
    use mimir_dm_mcp::tools::{
        CreateModuleInput, CreateNpcInput, CreateUserDocumentInput, EditDocumentInput,
        GetCharacterInput, GetModuleDetailsInput, ListCampaignsInput, ListCharactersInput,
        ListDocumentsInput, ListModulesInput, ReadDocumentInput, SearchItemsInput,
        SearchMonstersInput, SearchTrapsInput, SetActiveCampaignInput, UpdateCharacterCurrencyInput,
    };

    #[tokio::test]
    async fn test_tool_definitions_campaign() {
        let list = ListCampaignsInput::tool();
        assert_eq!(list.name, "list_campaigns");
        assert!(list.description.is_some());

        let set_active = SetActiveCampaignInput::tool();
        assert_eq!(set_active.name, "set_active_campaign");
    }

    #[tokio::test]
    async fn test_tool_definitions_module() {
        let create = CreateModuleInput::tool();
        assert_eq!(create.name, "create_module");

        let list = ListModulesInput::tool();
        assert_eq!(list.name, "list_modules");

        let details = GetModuleDetailsInput::tool();
        assert_eq!(details.name, "get_module_details");
    }

    #[tokio::test]
    async fn test_tool_definitions_document() {
        let list = ListDocumentsInput::tool();
        assert_eq!(list.name, "list_documents");

        let read = ReadDocumentInput::tool();
        assert_eq!(read.name, "read_document");

        let edit = EditDocumentInput::tool();
        assert_eq!(edit.name, "edit_document");

        let create = CreateUserDocumentInput::tool();
        assert_eq!(create.name, "create_user_document");
    }

    #[tokio::test]
    async fn test_tool_definitions_character() {
        let list = ListCharactersInput::tool();
        assert_eq!(list.name, "list_characters");

        let get = GetCharacterInput::tool();
        assert_eq!(get.name, "get_character");

        let create_npc = CreateNpcInput::tool();
        assert_eq!(create_npc.name, "create_npc");

        let currency = UpdateCharacterCurrencyInput::tool();
        assert_eq!(currency.name, "update_character_currency");
    }

    #[tokio::test]
    async fn test_tool_definitions_catalog() {
        let monsters = SearchMonstersInput::tool();
        assert_eq!(monsters.name, "search_monsters");

        let items = SearchItemsInput::tool();
        assert_eq!(items.name, "search_items");

        let traps = SearchTrapsInput::tool();
        assert_eq!(traps.name, "search_traps");
    }
}
