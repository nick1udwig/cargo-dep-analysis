use cargo_metadata::{Dependency, MetadataCommand};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get cargo metadata
    let metadata = MetadataCommand::new().exec()?;
    let package = metadata.root_package().unwrap();

    // Collect all dependencies and their underscore variants
    let mut deps = HashMap::new();
    let mut name_mappings = HashMap::new();
    for dep in &package.dependencies {
        let underscore_name = dep.name.replace('-', "_");
        name_mappings.insert(underscore_name, dep.name.clone());
        deps.insert(dep.name.clone(), analyze_dependency(&dep));
    }

    // Scan source files for usage
    let mut used_crates = HashSet::new();
    for entry in WalkDir::new("src") {
        let entry = entry?;
        if entry.path().extension().map_or(false, |ext| ext == "rs") {
            let content = std::fs::read_to_string(entry.path())?;
            scan_for_usage(&content, &mut used_crates, &name_mappings);
        }
    }

    // Compare and report
    println!("\nDependency Analysis Report:");
    println!("==========================");

    for (name, info) in deps {
        let underscore_name = name.replace('-', "_");
        let usage_status = if used_crates.contains(&name) || used_crates.contains(&underscore_name)
        {
            continue;
        } else {
            "POTENTIALLY UNUSED"
        };

        println!("\n{} ({})", name, usage_status);
        println!("Version: {}", info.version);
        println!("Feature flags: {:?}", info.features);

        if usage_status == "POTENTIALLY UNUSED" {
            println!("⚠️  This dependency might be removable. Verify:");
            println!("  1. Check for macro usage");
            println!("  2. Look for #[derive(...)] usage");
            println!("  3. Review build.rs dependencies");
            println!("  4. Check conditional compilation flags");
        }
    }

    Ok(())
}

#[derive(Debug)]
struct DependencyInfo {
    version: String,
    features: Vec<String>,
}

fn analyze_dependency(dep: &Dependency) -> DependencyInfo {
    DependencyInfo {
        version: dep.req.to_string(),
        features: dep.features.clone(),
    }
}

fn scan_for_usage(
    content: &str,
    used_crates: &mut HashSet<String>,
    name_mappings: &HashMap<String, String>,
) {
    let patterns = [
        // Basic use statements
        r#"use\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*::"#,
        // Qualified use statements
        r#"use\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s+as\s+[a-zA-Z_][a-zA-Z0-9_]*)?\s*;"#,
        // Extern crate statements
        r#"extern\s+crate\s+([a-zA-Z_][a-zA-Z0-9_]*)"#,
        // Derive macros
        r#"#\[derive\(([^)]*)\)\]"#,
        // Direct crate references
        r#"([a-zA-Z_][a-zA-Z0-9_]*)::\w+"#,
        // Macro usage
        r#"([a-zA-Z_][a-zA-Z0-9_]*)!\s*[({]"#,
        // Module declarations
        r#"mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;"#,
        // Type annotations
        r#":\s*([a-zA-Z_][a-zA-Z0-9_]*)::"#,
    ];

    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        for cap in re.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                for name in m.as_str().split(',') {
                    let clean_name = name
                        .trim()
                        .trim_start_matches("crate::")
                        .trim_start_matches("self::")
                        .trim_start_matches("::");

                    if !clean_name.is_empty()
                        && !clean_name.starts_with("super")
                        && !clean_name.starts_with("crate")
                    {
                        // Check both underscore and hyphenated versions
                        if let Some(original_name) = name_mappings.get(clean_name) {
                            used_crates.insert(original_name.clone());
                        } else {
                            used_crates.insert(clean_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Special case for common macro-only crates
    let macro_crates = ["anyhow", "thiserror", "lazy_static", "serde"];
    for macro_crate in macro_crates {
        if content.contains(&format!("{}!", macro_crate))
            || content.contains(&format!("use {}::", macro_crate))
        {
            used_crates.insert(macro_crate.to_string());
        }
    }
}
