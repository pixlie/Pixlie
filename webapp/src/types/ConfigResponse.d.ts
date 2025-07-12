import type { DownloadStats } from "./DownloadStats";
export type ConfigResponse = {
    config_path: string;
    data_folder: string | null;
    download_stats: DownloadStats;
};
