//! Workspace-specific configuration for Pixlie TUI application
//!
//! Workspace configurations allow per-project customization of settings
//! that override global defaults when working within specific workspaces.

use super::settings::*;
use crate::error::{ErrorContext, ErrorContextExt, PixlieError, Result};
use serde::{Deserialize, Serialize};

/// Workspace-specific configuration
///
/// This configuration is stored in `.pixlie-workspace.toml` within each workspace
/// and provides project-specific overrides for global settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Workspace metadata
    #[serde(default)]
    pub metadata: WorkspaceMetadata,

    /// UI configuration overrides
    pub ui: Option<UiConfig>,

    /// Session configuration overrides
    pub session: Option<SessionConfig>,

    /// LLM configuration overrides
    pub llm: Option<LlmConfig>,

    /// Database configuration overrides
    pub database: Option<DatabaseConfig>,

    /// Shortcuts configuration overrides
    pub shortcuts: Option<ShortcutsConfig>,

    /// Workspace-specific settings
    #[serde(default)]
    pub workspace: WorkspaceSettings,
}

/// Workspace metadata and identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// Workspace name
    pub name: Option<String>,

    /// Workspace description
    pub description: Option<String>,

    /// Workspace version/tag
    pub version: Option<String>,

    /// Creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last modified timestamp
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,

    /// Workspace tags for organization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Default database file path relative to workspace
    pub default_database: Option<String>,

    /// Workspace-specific environment variables
    #[serde(default)]
    pub environment: std::collections::HashMap<String, String>,
}

/// Workspace-specific settings that don't override global config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// Pinned objectives that persist across sessions
    #[serde(default)]
    pub pinned_objectives: Vec<PinnedObjective>,

    /// Workspace-specific templates for common queries
    #[serde(default)]
    pub query_templates: Vec<QueryTemplate>,

    /// Workspace-specific analysis workflows
    #[serde(default)]
    pub workflows: Vec<AnalysisWorkflow>,

    /// Custom tool configurations for this workspace
    #[serde(default)]
    pub custom_tools: std::collections::HashMap<String, ToolConfig>,

    /// Workspace-specific data source configurations
    #[serde(default)]
    pub data_sources: Vec<DataSourceConfig>,

    /// Auto-load objectives on workspace startup
    #[serde(default)]
    pub auto_load_objectives: bool,

    /// Workspace backup settings
    #[serde(default)]
    pub backup: WorkspaceBackupConfig,
}

/// A pinned objective that persists across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedObjective {
    /// Unique identifier for the objective
    pub id: uuid::Uuid,

    /// Objective title/name
    pub title: String,

    /// Objective description or query
    pub description: String,

    /// Priority level (high, medium, low)
    #[serde(default = "default_priority")]
    pub priority: String,

    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Whether to auto-load this objective on workspace startup
    #[serde(default)]
    pub auto_load: bool,

    /// Estimated completion time in minutes
    pub estimated_duration: Option<u32>,
}

/// A reusable query template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTemplate {
    /// Template name
    pub name: String,

    /// Template description
    pub description: Option<String>,

    /// SQL query template with placeholders
    pub query: String,

    /// Template parameters with descriptions
    #[serde(default)]
    pub parameters: Vec<TemplateParameter>,

    /// Template category for organization
    pub category: Option<String>,

    /// Template tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A parameter for a query template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,

    /// Parameter description
    pub description: Option<String>,

    /// Parameter type (string, number, date, etc.)
    pub param_type: String,

    /// Default value
    pub default_value: Option<String>,

    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
}

/// An analysis workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisWorkflow {
    /// Workflow name
    pub name: String,

    /// Workflow description
    pub description: Option<String>,

    /// Ordered list of steps in the workflow
    pub steps: Vec<WorkflowStep>,

    /// Workflow triggers (manual, auto, scheduled)
    #[serde(default)]
    pub triggers: Vec<String>,

    /// Workflow category
    pub category: Option<String>,
}

/// A step in an analysis workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,

    /// Step description
    pub description: Option<String>,

    /// Tool to execute for this step
    pub tool: String,

    /// Parameters for the tool
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, serde_json::Value>,

    /// Whether to continue on error
    #[serde(default)]
    pub continue_on_error: bool,

    /// Conditions for step execution
    pub conditions: Option<String>,
}

/// Custom tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool type or class
    pub tool_type: String,

    /// Tool configuration parameters
    #[serde(default)]
    pub config: std::collections::HashMap<String, serde_json::Value>,

    /// Tool description
    pub description: Option<String>,

    /// Whether the tool is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// Data source name
    pub name: String,

    /// Data source type (sqlite, csv, json, etc.)
    pub source_type: String,

    /// Connection string or file path
    pub connection: String,

    /// Data source description
    pub description: Option<String>,

    /// Whether the data source is read-only
    #[serde(default = "default_true")]
    pub read_only: bool,

    /// Data source-specific configuration
    #[serde(default)]
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

/// Workspace backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBackupConfig {
    /// Enable automatic backups
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Backup frequency in hours
    #[serde(default = "default_backup_frequency_hours")]
    pub frequency_hours: u32,

    /// Maximum number of backups to keep
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,

    /// Backup compression level (0-9)
    #[serde(default = "default_compression_level")]
    pub compression_level: u8,

    /// Include session history in backups
    #[serde(default = "default_true")]
    pub include_history: bool,
}

// Default value functions

fn default_priority() -> String {
    "medium".to_string()
}
fn default_true() -> bool {
    true
}
fn default_backup_frequency_hours() -> u32 {
    24
}
fn default_max_backups() -> u32 {
    7
}
fn default_compression_level() -> u8 {
    6
}

// Default implementations

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            metadata: WorkspaceMetadata::default(),
            ui: None,
            session: None,
            llm: None,
            database: None,
            shortcuts: None,
            workspace: WorkspaceSettings::default(),
        }
    }
}

impl Default for WorkspaceMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            version: None,
            created_at: Some(chrono::Utc::now()),
            last_modified: Some(chrono::Utc::now()),
            tags: Vec::new(),
            default_database: None,
            environment: std::collections::HashMap::new(),
        }
    }
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            pinned_objectives: Vec::new(),
            query_templates: Vec::new(),
            workflows: Vec::new(),
            custom_tools: std::collections::HashMap::new(),
            data_sources: Vec::new(),
            auto_load_objectives: false,
            backup: WorkspaceBackupConfig::default(),
        }
    }
}

impl Default for WorkspaceBackupConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            frequency_hours: default_backup_frequency_hours(),
            max_backups: default_max_backups(),
            compression_level: default_compression_level(),
            include_history: default_true(),
        }
    }
}

// Validation implementations

impl WorkspaceConfig {
    /// Validate workspace configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Workspace configuration validation");

        // Validate UI config override if present
        if let Some(ui) = &self.ui {
            ui.validate().with_context(|| context.clone())?;
        }

        // Validate session config override if present
        if let Some(session) = &self.session {
            session.validate().with_context(|| context.clone())?;
        }

        // Validate LLM config override if present
        if let Some(llm) = &self.llm {
            llm.validate().with_context(|| context.clone())?;
        }

        // Validate database config override if present
        if let Some(database) = &self.database {
            database.validate().with_context(|| context.clone())?;
        }

        // Validate shortcuts config override if present
        if let Some(shortcuts) = &self.shortcuts {
            shortcuts.validate().with_context(|| context.clone())?;
        }

        // Validate workspace-specific settings
        self.workspace.validate().with_context(|| context)?;

        Ok(())
    }

    /// Update the last modified timestamp
    pub fn touch(&mut self) {
        self.metadata.last_modified = Some(chrono::Utc::now());
    }

    /// Add a pinned objective
    pub fn add_pinned_objective(&mut self, title: String, description: String) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();
        let objective = PinnedObjective {
            id,
            title,
            description,
            priority: default_priority(),
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            auto_load: false,
            estimated_duration: None,
        };

        self.workspace.pinned_objectives.push(objective);
        self.touch();
        id
    }

    /// Remove a pinned objective by ID
    pub fn remove_pinned_objective(&mut self, id: uuid::Uuid) -> bool {
        let initial_len = self.workspace.pinned_objectives.len();
        self.workspace.pinned_objectives.retain(|obj| obj.id != id);
        let removed = self.workspace.pinned_objectives.len() < initial_len;

        if removed {
            self.touch();
        }

        removed
    }

    /// Add a query template
    pub fn add_query_template(&mut self, template: QueryTemplate) {
        self.workspace.query_templates.push(template);
        self.touch();
    }

    /// Add a data source
    pub fn add_data_source(&mut self, data_source: DataSourceConfig) {
        self.workspace.data_sources.push(data_source);
        self.touch();
    }
}

impl WorkspaceSettings {
    /// Validate workspace settings
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Workspace settings validation");

        // Validate pinned objectives
        for objective in &self.pinned_objectives {
            objective.validate().with_context(|| context.clone())?;
        }

        // Validate query templates
        for template in &self.query_templates {
            template.validate().with_context(|| context.clone())?;
        }

        // Validate workflows
        for workflow in &self.workflows {
            workflow.validate().with_context(|| context.clone())?;
        }

        // Validate data sources
        for data_source in &self.data_sources {
            data_source.validate().with_context(|| context.clone())?;
        }

        // Validate backup config
        self.backup.validate().with_context(|| context)?;

        Ok(())
    }
}

impl PinnedObjective {
    /// Validate pinned objective
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Pinned objective validation");

        if self.title.trim().is_empty() {
            return Err(PixlieError::validation(
                "pinned_objective.title",
                "Objective title cannot be empty",
                context,
            ));
        }

        if !["high", "medium", "low"].contains(&self.priority.as_str()) {
            return Err(PixlieError::validation(
                "pinned_objective.priority",
                "Priority must be 'high', 'medium', or 'low'",
                context,
            ));
        }

        Ok(())
    }
}

impl QueryTemplate {
    /// Validate query template
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Query template validation");

        if self.name.trim().is_empty() {
            return Err(PixlieError::validation(
                "query_template.name",
                "Template name cannot be empty",
                context,
            ));
        }

        if self.query.trim().is_empty() {
            return Err(PixlieError::validation(
                "query_template.query",
                "Template query cannot be empty",
                context,
            ));
        }

        Ok(())
    }
}

impl AnalysisWorkflow {
    /// Validate analysis workflow
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Analysis workflow validation");

        if self.name.trim().is_empty() {
            return Err(PixlieError::validation(
                "workflow.name",
                "Workflow name cannot be empty",
                context,
            ));
        }

        if self.steps.is_empty() {
            return Err(PixlieError::validation(
                "workflow.steps",
                "Workflow must have at least one step",
                context,
            ));
        }

        Ok(())
    }
}

impl DataSourceConfig {
    /// Validate data source configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Data source validation");

        if self.name.trim().is_empty() {
            return Err(PixlieError::validation(
                "data_source.name",
                "Data source name cannot be empty",
                context,
            ));
        }

        if self.connection.trim().is_empty() {
            return Err(PixlieError::validation(
                "data_source.connection",
                "Data source connection cannot be empty",
                context,
            ));
        }

        Ok(())
    }
}

impl WorkspaceBackupConfig {
    /// Validate backup configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Backup configuration validation");

        if self.frequency_hours > 168 {
            // 1 week
            return Err(PixlieError::validation(
                "backup.frequency_hours",
                "Backup frequency cannot exceed 168 hours (1 week)",
                context,
            ));
        }

        if self.max_backups > 100 {
            return Err(PixlieError::validation(
                "backup.max_backups",
                "Maximum backups cannot exceed 100",
                context,
            ));
        }

        if self.compression_level > 9 {
            return Err(PixlieError::validation(
                "backup.compression_level",
                "Compression level must be between 0 and 9",
                context,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();
        assert!(config.ui.is_none());
        assert!(config.workspace.pinned_objectives.is_empty());
        assert!(config.workspace.backup.enabled);
    }

    #[test]
    fn test_add_pinned_objective() {
        let mut config = WorkspaceConfig::default();
        let id = config
            .add_pinned_objective("Test Objective".to_string(), "Test description".to_string());

        assert_eq!(config.workspace.pinned_objectives.len(), 1);
        assert_eq!(config.workspace.pinned_objectives[0].id, id);
        assert_eq!(
            config.workspace.pinned_objectives[0].title,
            "Test Objective"
        );
    }

    #[test]
    fn test_remove_pinned_objective() {
        let mut config = WorkspaceConfig::default();
        let id = config
            .add_pinned_objective("Test Objective".to_string(), "Test description".to_string());

        assert!(config.remove_pinned_objective(id));
        assert!(config.workspace.pinned_objectives.is_empty());

        // Try to remove non-existent objective
        assert!(!config.remove_pinned_objective(uuid::Uuid::new_v4()));
    }

    #[test]
    fn test_pinned_objective_validation() {
        let mut objective = PinnedObjective {
            id: uuid::Uuid::new_v4(),
            title: "Valid Title".to_string(),
            description: "Valid description".to_string(),
            priority: "medium".to_string(),
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            auto_load: false,
            estimated_duration: None,
        };

        assert!(objective.validate().is_ok());

        objective.title = "".to_string();
        assert!(objective.validate().is_err());

        objective.title = "Valid Title".to_string();
        objective.priority = "invalid".to_string();
        assert!(objective.validate().is_err());
    }

    #[test]
    fn test_workspace_serialization() {
        let config = WorkspaceConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: WorkspaceConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            config.workspace.backup.enabled,
            parsed.workspace.backup.enabled
        );
    }
}
