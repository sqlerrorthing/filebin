<script lang="ts">
    import {
        File,
        Download,
        Trash,
        FileImage,
        FileVideo,
        FileAudio,
        FileText,
        FileArchive,
        FileCode,
        FileSpreadsheet,
    } from '@lucide/svelte';
    import type { DecryptedFileView } from './FileList.svelte';
    import { formatBytes } from '$lib/utils';

    let {
        file = $bindable(),
        onDelete = $bindable(),
        onDownload = $bindable(),
    }: {
        file: DecryptedFileView;
        onDelete?: () => Promise<void>;
        onDownload: (onProgress: (percent: number) => void) => Promise<void>;
    } = $props();

    let isDownloading = $state(false);
    let progress = $state(0);

    const size = 16;
    const strokeWidth = 2;

    const radius = (size - strokeWidth) / 2;
    const circumference = 2 * Math.PI * radius;

    let clampedProgress = $derived(Math.min(95, Math.max(2, progress)));

    let strokeDashoffset = $derived(
        circumference - (clampedProgress / 100) * circumference
    );

    async function clickDownload() {
        try {
            progress = 0;
            isDownloading = true;
            await onDownload((p) => (progress = p));
        } finally {
            isDownloading = false;
        }
    }

    function getFileIcon(mimeType: string = '') {
        if (mimeType.startsWith('image/')) return FileImage;
        if (mimeType.startsWith('video/')) return FileVideo;
        if (mimeType.startsWith('audio/')) return FileAudio;
        if (mimeType.includes('pdf') || mimeType.includes('text/'))
            return FileText;
        if (
            mimeType.includes('zip') ||
            mimeType.includes('tar') ||
            mimeType.includes('compressed')
        )
            return FileArchive;
        if (
            mimeType.includes('json') ||
            mimeType.includes('javascript') ||
            mimeType.includes('html') ||
            mimeType.includes('xml')
        )
            return FileCode;
        if (
            mimeType.includes('sheet') ||
            mimeType.includes('excel') ||
            mimeType.includes('csv')
        )
            return FileSpreadsheet;

        return File;
    }

    let FileIcon = $derived(getFileIcon(file.metadata.type));
</script>

<div class="flex h-full w-full gap-2">
    <div class="flex h-6 w-6 items-center justify-center">
        <FileIcon class="h-full w-5" />
    </div>

    <div class="min-w-0 flex-1">
        {file.metadata.path}
    </div>

    <div class="flex items-center gap-2 text-sm">
        <span class="text-muted-foreground">
            {formatBytes(file.size)}
        </span>

        <div>
            {#if onDelete}
                <button>
                    <Trash />
                </button>
            {/if}
            <button
                class="flex cursor-pointer justify-center"
                onclick={clickDownload}
                disabled={isDownloading}
                title={isDownloading ? `${progress}%` : ''}
            >
                {#if isDownloading}
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 {size} {size}"
                        fill="none"
                        stroke="currentColor"
                        stroke-width={strokeWidth}
                        stroke-linecap="round"
                        style="width: {size}px; height: {size}px; min-width: {size}px;"
                    >
                        <circle
                            cx={size / 2}
                            cy={size / 2}
                            r={radius}
                            class="opacity-20"
                        />

                        <circle
                            cx={size / 2}
                            cy={size / 2}
                            r={radius}
                            stroke-dasharray={circumference}
                            stroke-dashoffset={strokeDashoffset}
                            class="origin-center -rotate-90 animate-spin transition-all duration-200 ease-out"
                        />
                    </svg>
                {:else}
                    <Download class="h-4 w-4" />
                {/if}
            </button>
        </div>
    </div>
</div>
