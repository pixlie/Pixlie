export type HnItem = {
    id: bigint;
    item_type: string;
    by: string | null;
    time: string;
    text: string | null;
    url: string | null;
    score: number | null;
    title: string | null;
    parent: number | null;
    kids: string | null;
    descendants: number | null;
    deleted: boolean;
    dead: boolean;
    created_at: string;
};
