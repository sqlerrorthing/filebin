<script lang="ts">
    import {type FileItem, getFilesContext} from "$lib/context/files.svelte";
    import {
        File,
        FileArchive,
        FileCode, FileHeadphone,
        FileImage, FilePlay,
        FileSpreadsheet,
        FileText,
        Folder, type LucideProps
    } from "@lucide/svelte";
    import type {Component} from "svelte";
    import type {Row} from "$lib/components/files-table/types";
    import RowEntry from "$lib/components/files-table/RowEntry.svelte";

    const ctx = getFilesContext();

    const items = $derived(ctx.currentItems);

    const rows = $derived.by(() => {
        let result: Row[] = [];

        if (!ctx.isRoot) {
            result.push({
                kind: "goUp" as const,
                icon: Folder
            });
        }

        for (const folder of items.folders) {
            result.push({
                kind: "folder" as const,
                icon: Folder,
                ...folder
            })
        }

        for (const file of items.files) {
            result.push({
                kind: "file" as const,
                icon: getFileIcon(file.type),
                ...file,
            });
        }

        return result;
    })

    const getFileIcon = (mimeType: string): Component<LucideProps> => {
        if (mimeType.startsWith("image/")) return FileImage;
        if (mimeType.startsWith("video/")) return FilePlay;
        if (mimeType.startsWith("audio/")) return FileHeadphone;
        if (mimeType.includes("pdf") || mimeType.includes("text/"))
            return FileText;
        if (
            mimeType.includes("zip") ||
            mimeType.includes("tar") ||
            mimeType.includes("compressed")
        )
            return FileArchive;
        if (
            mimeType.includes("json") ||
            mimeType.includes("javascript") ||
            mimeType.includes("html") ||
            mimeType.includes("xml")
        )
            return FileCode;
        if (
            mimeType.includes("sheet") ||
            mimeType.includes("excel") ||
            mimeType.includes("csv")
        )
            return FileSpreadsheet;

        return File;
    }
</script>

<div class="overflow-hidden border border-solid border-border bg-card rounded-2xl">
    <div class="grid grid-cols-[1fr_120px_100px] items-center border-b border-border px-4 py-3 text-sm text-muted-foreground">
        <span>File</span>
        <span>Size</span>
        <span class="text-right">Actions</span>
    </div>

    {#each rows as _, i}
        <div
                class="grid grid-cols-[1fr_120px_100px] items-center px-4 py-3 transition-colors hover:bg-muted/50"
        >
            <RowEntry bind:row={rows[i]} />
        </div>
    {/each}
</div>
