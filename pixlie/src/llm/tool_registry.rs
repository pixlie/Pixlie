// pixlie/src/llm/tool_registry.rs

use super::Tool;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn get_all_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }
}
