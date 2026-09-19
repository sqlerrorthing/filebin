<script lang="ts">
    import {getFilesContext} from "$lib/context/files.svelte";
    import {
        File,
        FileArchive,
        FileCode,
        FileHeadphone,
        FileImage,
        FilePlay,
        FileSpreadsheet,
        FileText,
        Folder,
        FolderOpen,
        Inbox,
        LoaderCircle,
        type LucideProps
    } from "@lucide/svelte";
    import type {Component} from "svelte";
    import type {Row} from "$lib/components/files-table/types";
    import RowEntry from "$lib/components/files-table/RowEntry.svelte";
    import * as m from "$lib/paraglide/messages";
    import {getFolderContext} from "$lib/context/folder.svelte";

    const ctx = getFilesContext();
    const folderCtx = getFolderContext();

    const items = $derived(ctx.currentItems);

    const rows = $derived.by(() => {
        let result: Row[] = [];

        if (!ctx.isRoot) {
            result.push({
                kind: "goUp" as const,
                icon: FolderOpen
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
    <div class="grid grid-cols-[1fr_120px_100px] items-center border-b border-border px-4 py-3 text-xs font-semibold tracking-wider uppercase text-muted-foreground bg-muted/30">
        <span>{m["folder.contents.name"]()}</span>
        <span>{m["folder.contents.size"]()}</span>
        <span class="text-right">{m["folder.contents.actions"]()}</span>
    </div>

    {#if ctx.isLoading}
        <div class="flex flex-col items-center justify-center py-16 text-muted-foreground gap-3">
            <LoaderCircle class="h-6 w-6 animate-spin" />
            <span class="text-sm font-medium">{m["folder.loading"]()}</span>
        </div>
    {:else if ctx.error}
        <div class="flex flex-col items-center justify-center py-16 text-destructive gap-2 px-4 text-center">
            <span class="text-sm font-medium">{m["folder.errors.failed_to_load_files"]()}</span>
            <span class="text-xs text-muted-foreground">{ctx.error}</span>
        </div>
    {:else if rows.length === 0}
        <div class="flex flex-col items-center justify-center py-16 text-muted-foreground gap-3">
            <div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-muted text-muted-foreground">
                <Inbox class="h-6 w-6" />
            </div>
            <div class="flex flex-col items-center gap-1">
                <span class="text-sm font-medium text-foreground">{m["folder.empty"]()}</span>
                {#if folderCtx().token}
                    <span class="text-xs">{m["folder.upload_to_get_started"]()}</span>
                {/if}
            </div>
        </div>
    {:else}
        <div class="divide-y divide-border/60">
            {#each rows as _, i}
                <div
                    class="grid grid-cols-[1fr_120px_100px] items-center px-4 transition-colors hover:bg-muted/50"
                >
                    <RowEntry bind:row={rows[i]} />
                </div>
            {/each}
        </div>
    {/if}
</div>
