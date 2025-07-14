use std::fs;
use tempfile::tempdir;

#[test]
fn test_cross_directory_import_fixes() {
    // Create a temporary directory structure
    let temp_dir = tempdir().unwrap();
    let base_dir = temp_dir.path().join("types");
    let api_dir = base_dir.join("api");
    fs::create_dir_all(&api_dir).unwrap();

    // Create a test file with incorrect imports
    let test_file = api_dir.join("TestResponse.ts");
    let content_with_wrong_imports = r#"// This file is auto-generated
import type { Entity } from "./Entity";
import type { DownloadStats } from "./DownloadStats";
import type { ModelInfo } from "./ModelInfo";

export type TestResponse = { entity: Entity, stats: DownloadStats, model: ModelInfo };
"#;
    fs::write(&test_file, content_with_wrong_imports).unwrap();

    // Apply the same fix logic as in export_types.rs
    let fixes = vec![
        ("./DownloadStats", "../database/DownloadStats"),
        ("./Entity", "../database/Entity"),
        ("./EntityReference", "../database/EntityReference"),
        ("./EntityRelation", "../database/EntityRelation"),
        ("./ExtractionStats", "../database/ExtractionStats"),
        ("./HnItem", "../database/HnItem"),
        ("./ModelInfo", "../extraction/ModelInfo"),
    ];

    let content = fs::read_to_string(&test_file).unwrap();
    let mut fixed_content = content.clone();

    for (old_import, new_import) in &fixes {
        let old_pattern = format!("\"{old_import}\"");
        let new_pattern = format!("\"{new_import}\"");
        fixed_content = fixed_content.replace(&old_pattern, &new_pattern);
    }

    fs::write(&test_file, fixed_content).unwrap();

    // Verify the imports were fixed correctly
    let final_content = fs::read_to_string(&test_file).unwrap();
    assert!(final_content.contains("\"../database/Entity\""));
    assert!(final_content.contains("\"../database/DownloadStats\""));
    assert!(final_content.contains("\"../extraction/ModelInfo\""));
    assert!(!final_content.contains("\"./Entity\""));
    assert!(!final_content.contains("\"./DownloadStats\""));
    assert!(!final_content.contains("\"./ModelInfo\""));
}

#[test]
fn test_no_changes_when_imports_already_correct() {
    // Create a temporary directory structure
    let temp_dir = tempdir().unwrap();
    let base_dir = temp_dir.path().join("types");
    let api_dir = base_dir.join("api");
    fs::create_dir_all(&api_dir).unwrap();

    // Create a test file with already correct imports
    let test_file = api_dir.join("TestResponse.ts");
    let content_with_correct_imports = r#"// This file is auto-generated
import type { Entity } from "../database/Entity";
import type { DownloadStats } from "../database/DownloadStats";

export type TestResponse = { entity: Entity, stats: DownloadStats };
"#;
    fs::write(&test_file, content_with_correct_imports).unwrap();

    // Apply the same fix logic as in export_types.rs
    let fixes = vec![
        ("./DownloadStats", "../database/DownloadStats"),
        ("./Entity", "../database/Entity"),
        ("./EntityReference", "../database/EntityReference"),
        ("./EntityRelation", "../database/EntityRelation"),
        ("./ExtractionStats", "../database/ExtractionStats"),
        ("./HnItem", "../database/HnItem"),
        ("./ModelInfo", "../extraction/ModelInfo"),
    ];

    let original_content = fs::read_to_string(&test_file).unwrap();
    let mut fixed_content = original_content.clone();

    for (old_import, new_import) in &fixes {
        let old_pattern = format!("\"{old_import}\"");
        let new_pattern = format!("\"{new_import}\"");
        fixed_content = fixed_content.replace(&old_pattern, &new_pattern);
    }

    // Content should remain unchanged
    assert_eq!(original_content, fixed_content);
}
