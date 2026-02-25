use crate::tools::create_default_registry;
use anyhow::Result;
use serde_json::Value;

pub fn run_tools() -> Result<()> {
    let registry = create_default_registry();
    let definitions = registry.get_all_definitions();

    println!("Available Tools:");
    println!("{}", "=".repeat(60));

    for tool in definitions {
        println!("\n📦 {}", tool.function.name);
        println!("{}", "-".repeat(40));
        println!("{}", tool.function.description);
        
        if let Some(props) = tool.function.parameters.get("properties") {
            if let Value::Object(properties) = props {
                println!("\nParameters:");
                for (name, schema) in properties {
                    let type_str = schema.get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("unknown");
                    let desc = schema.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");
                    println!("  {} ({}) - {}", name, type_str, desc);
                }
            }
        }
    }

    Ok(())
}
