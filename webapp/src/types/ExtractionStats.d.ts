export type ExtractionStats = {
    total_entities: number;
    entities_by_type: Record<string, number>;
    total_items_processed: number;
    items_remaining: number;
    is_extracting: boolean;
    last_extraction_time: string | null;
};
