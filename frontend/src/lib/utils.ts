import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export function formatBytes(
    bytes: number | bigint,
    decimals: number = 2
): string {
    if (bytes === 0) return "0 Bytes";
    const numBytes = Number(bytes ?? 0);

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const i = Math.floor(Math.log(numBytes) / Math.log(k));

    const unit = i === 0 ? "Bytes" : `${"KMGTPEZY"[i - 1]}B`;

    return `${parseFloat((numBytes / Math.pow(k, i)).toFixed(dm))} ${unit}`;
}

export function formatSpeed(bytesPerSecond: number): string {
    if (!bytesPerSecond || bytesPerSecond <= 0) return "0 B/s";
    return `${formatBytes(bytesPerSecond)}/s`;
}

export function formatEta(seconds: number): string {
    if (!seconds || !isFinite(seconds) || seconds <= 0) return "0s";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    if (mins === 0) return `${secs}s`;
    return `${mins}m ${secs}s`;
}
