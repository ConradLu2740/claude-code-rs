use serde_json::json;

#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl ToolSchema {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }

    pub fn simple(
        name: impl Into<String>,
        description: impl Into<String>,
        properties: &[(&str, &str, bool)],
    ) -> Self {
        let props: serde_json::Map<String, serde_json::Value> = properties
            .iter()
            .map(|(name, desc, _required)| {
                (
                    name.to_string(),
                    json!({
                        "type": "string",
                        "description": desc
                    }),
                )
            })
            .collect();

        let required: Vec<&str> = properties
            .iter()
            .filter(|(_, _, req)| *req)
            .map(|(name, _, _)| *name)
            .collect();

        Self {
            name: name.into(),
            description: description.into(),
            parameters: json!({
                "type": "object",
                "properties": props,
                "required": required
            }),
        }
    }
}

pub fn file_path_schema() -> serde_json::Value {
    json!({
        "type": "string",
        "description": "The absolute path to the file to read"
    })
}

pub fn glob_pattern_schema() -> serde_json::Value {
    json!({
        "type": "string",
        "description": "The glob pattern to match files against (e.g., \"**/*.rs\")"
    })
}

pub fn search_pattern_schema() -> serde_json::Value {
    json!({
        "type": "string",
        "description": "The regular expression pattern to search for"
    })
}
