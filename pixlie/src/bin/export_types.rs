use pixlie::database::{
    DownloadStats, Entity, EntityReference, EntityRelation, ExtractionStats, HnItem,
};
use pixlie::entity_extraction::ModelInfo;
use pixlie::handlers::{
    ConfigResponse, DownloadModelRequest, DownloadStatusResponse, EntityDetailResponse,
    EntityItemWithHighlights, ExtractionStatusResponse, GetEntitiesRequest, GetEntitiesResponse,
    GetEntityItemsRequest, GetEntityItemsResponse, GetEntityReferencesRequest,
    GetEntityReferencesResponse, GetItemsRequest, GetItemsResponse, GetRelationsRequest,
    GetRelationsResponse, ModelsResponse, SearchEntitiesRequest, SearchEntitiesResponse,
    SetDataFolderRequest, StartDownloadRequest, StartExtractionRequest,
};
use pixlie::tools::data_query::{SearchItemsParams, SearchItemsResponse};
use std::fs;
use std::path::Path;
use ts_rs::TS;

fn export_type<T: TS + 'static>(output_dir: &Path) -> std::io::Result<()> {
    let type_name = T::name();
    let output_path = output_dir.join(format!("{}.ts", type_name));
    let content = T::export_to_string().unwrap();
    fs::write(&output_path, content)?;
    println!("✅ TypeScript type exported to {:?}", output_path);
    Ok(())
}

fn export_tool_types(output_dir: &Path) -> std::io::Result<()> {
    let tool_output_dir = output_dir.join("tools");
    if !tool_output_dir.exists() {
        fs::create_dir_all(&tool_output_dir)?;
    }

    // Manually export tool-related types that have the #[ts(export)] attribute
    let search_items_params_path = tool_output_dir.join("SearchItemsParams.ts");
    fs::write(search_items_params_path, SearchItemsParams::export_to_string().unwrap())?;
    let search_items_response_path = tool_output_dir.join("SearchItemsResponse.ts");
    fs::write(search_items_response_path, SearchItemsResponse::export_to_string().unwrap())?;

    println!(
        "\n✅ All TypeScript tool types exported successfully to '{}'.",
        tool_output_dir.display()
    );

    Ok(())
}

fn main() -> std::io::Result<()> {
    let output_dir = Path::new("../webapp/src/types");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Handlers
    export_type::<ConfigResponse>(output_dir)?;
    export_type::<SetDataFolderRequest>(output_dir)?;
    export_type::<DownloadStatusResponse>(output_dir)?;
    export_type::<StartDownloadRequest>(output_dir)?;
    export_type::<ModelsResponse>(output_dir)?;
    export_type::<DownloadModelRequest>(output_dir)?;
    export_type::<ExtractionStatusResponse>(output_dir)?;
    export_type::<StartExtractionRequest>(output_dir)?;
    export_type::<GetItemsRequest>(output_dir)?;
    export_type::<GetItemsResponse>(output_dir)?;
    export_type::<GetEntitiesRequest>(output_dir)?;
    export_type::<GetEntitiesResponse>(output_dir)?;
    export_type::<GetRelationsRequest>(output_dir)?;
    export_type::<GetRelationsResponse>(output_dir)?;
    export_type::<SearchEntitiesRequest>(output_dir)?;
    export_type::<SearchEntitiesResponse>(output_dir)?;
    export_type::<EntityDetailResponse>(output_dir)?;
    export_type::<GetEntityReferencesRequest>(output_dir)?;
    export_type::<GetEntityReferencesResponse>(output_dir)?;
    export_type::<GetEntityItemsRequest>(output_dir)?;
    export_type::<GetEntityItemsResponse>(output_dir)?;
    export_type::<EntityItemWithHighlights>(output_dir)?;
    export_type::<Entity>(output_dir)?;
    export_type::<EntityReference>(output_dir)?;
    export_type::<EntityRelation>(output_dir)?;

    // Database
    export_type::<DownloadStats>(output_dir)?;
    export_type::<ExtractionStats>(output_dir)?;
    export_type::<HnItem>(output_dir)?;

    // Entity Extraction
    export_type::<ModelInfo>(output_dir)?;

    // Tools
    export_tool_types(output_dir)?;

    println!(
        "\n✅ All TypeScript types exported successfully to '{}'.",
        output_dir.display()
    );

    Ok(())
}
