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
        FileCode, FileSpreadsheet
    } from '@lucide/svelte';
    import type {DecryptedFileView} from "./FileList.svelte";
    import {formatBytes} from "$lib/utils";

    let {
        file = $bindable(),
        onDelete = $bindable()
    }: {
        file: DecryptedFileView,
        onDelete?: () => Promise<void>;
        onDownload: () => Promise<void>;
    } = $props();

    function getFileIcon(mimeType: string = '') {
        if (mimeType.startsWith('image/')) return FileImage;
        if (mimeType.startsWith('video/')) return FileVideo;
        if (mimeType.startsWith('audio/')) return FileAudio;
        if (mimeType.includes('pdf') || mimeType.includes('text/')) return FileText;
        if (mimeType.includes('zip') || mimeType.includes('tar') || mimeType.includes('compressed')) return FileArchive;
        if (mimeType.includes('json') || mimeType.includes('javascript') || mimeType.includes('html') || mimeType.includes('xml')) return FileCode;
        if (mimeType.includes('sheet') || mimeType.includes('excel') || mimeType.includes('csv')) return FileSpreadsheet;

        return File;
    }

    let FileIcon = $derived(getFileIcon(file.metadata.type));
</script>

<div class="flex gap-2 w-full h-full">
    <div class="w-6 h-6 flex justify-center items-center">
        <FileIcon class="w-5 h-full"/>
    </div>

    <div class="flex-1 min-w-0">
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
            <button class="cursor-pointer flex justify-center">
                <Download class="w-4 h-4" />
            </button>
        </div>
    </div>
</div>