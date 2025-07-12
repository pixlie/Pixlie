import type { HnItem } from "./HnItem";
export type GetItemsResponse = {
    items: Array<HnItem>;
    total_count: number;
    page: number;
    limit: number;
    total_pages: number;
};
