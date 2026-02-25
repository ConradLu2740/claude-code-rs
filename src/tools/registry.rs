use crate::llm::ToolDefinition;
use crate::tools::ToolExecutor;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolExecutor>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn ToolExecutor>) {
        let name = tool.schema().name.clone();
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn ToolExecutor>> {
        self.tools.get(name)
    }

    pub fn get_all_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| {
                let schema = tool.schema();
                ToolDefinition::new(
                    schema.name.clone(),
                    schema.description.clone(),
                    schema.parameters.clone(),
                )
            })
            .collect()
    }

    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    registry.register(Box::new(crate::tools::builtin::read::ReadTool::new()));
    registry.register(Box::new(crate::tools::builtin::write::WriteTool::new()));
    registry.register(Box::new(crate::tools::builtin::edit::EditTool::new()));
    registry.register(Box::new(crate::tools::builtin::glob::GlobTool::new()));
    registry.register(Box::new(crate::tools::builtin::grep::GrepTool::new()));
    registry.register(Box::new(crate::tools::builtin::shell::ShellTool::new()));
    registry.register(Box::new(crate::tools::builtin::ls::LsTool::new()));
    registry.register(Box::new(crate::tools::builtin::web::WebSearchTool::new()));
    registry.register(Box::new(crate::tools::builtin::web::WebFetchTool::new()));
    
    registry
}
