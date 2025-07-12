export type ModelInfo = {
    name: string;
    size_mb: bigint;
    download_url: string;
    is_downloaded: boolean;
    local_path: string | null;
};
