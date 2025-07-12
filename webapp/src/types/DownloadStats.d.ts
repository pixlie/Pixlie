export type DownloadStats = {
    total_items: number;
    total_users: number;
    last_download_time: string | null;
    items_downloaded_today: number;
    download_errors: number;
    is_downloading: boolean;
};
