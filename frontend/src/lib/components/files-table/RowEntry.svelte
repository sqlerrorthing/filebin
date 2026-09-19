<script lang="ts">
    import type {Row} from "$lib/components/files-table/types";
    import {getFilesContext} from "$lib/context/files.svelte";
    import {formatBytes} from "$lib/utils";
    import {Download, LoaderCircle, ChevronRight, Trash} from "@lucide/svelte";
    import Action from "$lib/components/files-table/Action.svelte";
    import {getFolderContext} from "$lib/context/folder.svelte";
    import * as m from "$lib/paraglide/messages";

    let {row = $bindable()}: {
        row: Row
    } = $props();

    const ctx = getFilesContext();
    const folderCtx = getFolderContext();

    const isDownloading = $derived(row.kind === "file" && ctx.isDownloading(row.id));
    const isDeleting = $derived(row.kind === "file" && ctx.isDeleting(row.id));

    const name = $derived(row.kind === "goUp" ? ".." : row.name);
    const rowType = $derived(
        row.kind === "goUp"
            ? m["folder.contents.parent"]()
            : row.kind === "folder"
                ? m["folder.contents.folder"]()
                : row.type || m["folder.contents.file"]()
    )

    function handleClick() {
        if (row.kind === "goUp") {
            ctx.goUp();
        } else if (row.kind === "folder") {
            ctx.enterFolder(row.name);
        } else if (row.kind === "file") {
            void ctx.downloadFile(row);
        }
    }
</script>

<div
        role="button"
        tabindex="0"
        onclick={handleClick}
        onkeydown={(e) => e.key === "Enter" && handleClick()}
        class="contents group cursor-pointer select-none"
>
    <div class="flex items-center gap-3 py-3 pr-4 overflow-hidden">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-muted/60 text-foreground group-hover:bg-accent group-hover:text-accent-foreground transition-colors">
            <row.icon class="h-5 w-5" />
        </div>
        <div class="flex flex-col min-w-0">
            <span class="font-medium truncate text-foreground group-hover:text-accent-foreground transition-colors">
                {name}
            </span>
            <span class="text-xs text-muted-foreground">
                {rowType}
            </span>
        </div>
    </div>

    <div class="py-3 text-sm text-muted-foreground font-mono">
        {#if (row as any).size === undefined}
            —
        {:else}
            {formatBytes((row as any).size)}
        {/if}
    </div>

    <div class="flex items-center justify-end gap-1 py-3">
        {#if row.kind === "file"}
            <Action class="hover:bg-muted hover:text-foreground" onclick={() => void ctx.downloadFile(row)} disabled={isDownloading || isDeleting}>
                {#if isDownloading}
                    <LoaderCircle class="h-full w-full animate-spin"/>
                {:else}
                    <Download class="h-full w-full"/>
                {/if}
            </Action>

            {#if folderCtx().token}
                <Action class="hover:bg-destructive/10 hover:text-destructive" onclick={() => void ctx.deleteFile(row)} disabled={isDownloading || isDeleting}>
                    {#if isDeleting}
                        <LoaderCircle class="h-full w-full animate-spin"/>
                    {:else}
                        <Trash class="h-full w-full"/>
                    {/if}
                </Action>
            {/if}
        {:else}
            <div class="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground group-hover:text-foreground transition-colors">
                <ChevronRight class="h-4 w-4"/>
            </div>
        {/if}
    </div>
</div>
