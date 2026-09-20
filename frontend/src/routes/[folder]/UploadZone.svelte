<script lang="ts">
    import {getFilesContext} from "$lib/context/files.svelte";
    import {getFolderContext} from "$lib/context/folder.svelte";
    import {Upload, FolderUp, LoaderCircle} from "@lucide/svelte";
    import FilesSelector from "$lib/components/upload/FilesSelector.svelte";
    import {formatBytes, formatSpeed, formatEta} from "$lib/utils";
    import * as m from "$lib/paraglide/messages";

    const filesCtx = getFilesContext();
    const folderCtx = getFolderContext();

    let selector = $state<FilesSelector | null>(null);
    let showMenu = $state(false);

    const hasToken = $derived(folderCtx()?.token !== null);

    const handleSelect = (files: { file: File; path: string }[] | FileList | File[]) => {
        void filesCtx.uploadFiles(files);
    };

</script>

{#if hasToken}
    <FilesSelector bind:this={selector} onSelect={handleSelect}/>

    <div class="mb-4 relative">
        <button
                type="button"
                onclick={() => showMenu = !showMenu}
                class="w-full border-2 border-dashed border-border hover:border-primary/50 bg-card/50 hover:bg-muted/50 rounded-2xl p-6 text-center transition-all cursor-pointer flex flex-col items-center justify-center gap-2 group"
        >
            <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-muted text-muted-foreground group-hover:bg-primary group-hover:text-primary-foreground transition-colors">
                <Upload class="h-6 w-6"/>
            </div>
            <div class="flex flex-col items-center gap-0.5">
                <span class="text-sm font-semibold text-foreground">
                    {m["folder.upload.title"]()}
                </span>
            </div>
        </button>

        {#if showMenu}
            <div class="absolute left-0 right-0 top-full mt-2 z-20 bg-card border border-border rounded-xl shadow-lg p-1.5 flex flex-col gap-1">
                <button
                        type="button"
                        onclick={() => { showMenu = false; selector?.selectFiles(); }}
                        class="flex items-center gap-3 px-4 py-2.5 rounded-lg hover:bg-muted text-sm font-medium text-foreground transition-colors cursor-pointer text-left w-full"
                >
                    <Upload class="h-4 w-4 text-primary"/>
                    <span>{m["common.actions.choose.files"]()}</span>
                </button>
                <button
                        type="button"
                        onclick={() => { showMenu = false; selector?.selectFolder(); }}
                        class="flex items-center gap-3 px-4 py-2.5 rounded-lg hover:bg-muted text-sm font-medium text-foreground transition-colors cursor-pointer text-left w-full"
                >
                    <FolderUp class="h-4 w-4 text-primary"/>
                    <span>{m["common.actions.choose.folder"]()}</span>
                </button>
            </div>
        {/if}
    </div>

    {#if filesCtx.uploadBatch || filesCtx.uploadingFiles.size > 0}
        <div class="mb-4 flex flex-col gap-3">
            {#if filesCtx.uploadBatch}
                <div class="px-4 py-3.5 rounded-2xl bg-card border-2 border-primary/20 shadow-sm flex flex-col gap-2">
                    <div class="flex items-center justify-between text-xs">
                        <span class="font-semibold text-foreground flex items-center gap-2">
                            <LoaderCircle class="h-4 w-4 animate-spin text-primary"/>
                            {m["folder.upload.overall_progress"]({
                                completed: filesCtx.uploadBatch.completedFiles,
                                total: filesCtx.uploadBatch.totalFiles
                            })}
                        </span>
                        <div class="flex items-center gap-3 font-mono text-muted-foreground">
                            <span>{formatSpeed(filesCtx.uploadBatch.speed)}</span>
                            <span>ETA: {formatEta(filesCtx.uploadBatch.eta)}</span>
                            <span class="font-bold text-primary">{filesCtx.uploadBatch.progress}%</span>
                            <button
                                    type="button"
                                    onclick={() => filesCtx.cancelAllUploads()}
                                    class="text-xs text-destructive hover:underline ml-2 cursor-pointer font-sans"
                            >
                                {m["common.actions.cancel_all"]()}
                            </button>
                        </div>
                    </div>
                    <div class="w-full bg-muted rounded-full h-2.5 overflow-hidden">
                        <div
                                class="bg-primary h-2.5 transition-all duration-300"
                                style="width: {filesCtx.uploadBatch.progress}%"
                        ></div>
                    </div>
                </div>
            {/if}

            {#each filesCtx.uploadingFiles.values() as upload}
                <div class="flex flex-col gap-2 px-4 py-3 rounded-xl bg-card border border-border shadow-sm">
                    <div class="flex items-center justify-between text-xs">
                        <div class="flex items-center gap-2 min-w-0">
                            <LoaderCircle class="h-3.5 w-3.5 animate-spin text-primary shrink-0"/>
                            <span class="font-medium truncate text-foreground">{upload.key}</span>
                        </div>
                        <div class="flex items-center gap-2 font-mono text-muted-foreground shrink-0">
                            <span>{formatBytes(upload.loaded)} / {formatBytes(upload.size * 2)}</span>
                            <span>{formatSpeed(upload.speed)}</span>
                            <span>ETA: {formatEta(upload.eta)}</span>
                            <span class="text-primary font-semibold">{upload.progress}%</span>
                            <button
                                    type="button"
                                    onclick={() => filesCtx.cancelUpload(upload.key)}
                                    class="text-xs text-destructive hover:underline ml-1 cursor-pointer font-sans font-semibold px-1.5 py-0.5 rounded bg-destructive/10"
                            >
                                ✕
                            </button>
                        </div>
                    </div>
                    <div class="w-full bg-muted rounded-full h-1.5 overflow-hidden">
                        <div
                                class="bg-primary h-1.5 transition-all duration-300"
                                style="width: {upload.progress}%"
                        ></div>
                    </div>
                    <div class="flex items-center justify-between text-[11px] text-muted-foreground">
                        <span>{upload.status}</span>
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    {#if filesCtx.failedUploads.size > 0}
        <div class="mb-4 px-4 py-3.5 rounded-2xl bg-destructive/10 border border-destructive/20 shadow-sm flex flex-col gap-3">
            <div class="flex items-center justify-between text-xs">
                <span class="font-semibold text-destructive flex items-center gap-2">
                    {m["folder.upload.failed.total"]({count: filesCtx.failedUploads.size})}
                </span>
                <div class="flex items-center gap-2">
                    <button
                            type="button"
                            onclick={() => filesCtx.retryAllFailed()}
                            class="text-xs text-foreground bg-background hover:bg-muted px-2.5 py-1 rounded-md font-medium transition-colors cursor-pointer border border-border"
                    >
                        {m["common.actions.retry_all"]()}
                    </button>
                    <button
                            type="button"
                            onclick={() => filesCtx.dismissAllFailed()}
                            class="text-xs text-muted-foreground hover:text-foreground px-2 py-1 transition-colors cursor-pointer"
                    >
                        {m["common.actions.clear"]()}
                    </button>
                </div>
            </div>

            {#if filesCtx.failedUploads.size <= 5}
                <div class="flex flex-col gap-2">
                    {#each filesCtx.failedUploads.values() as failed}
                        <div class="flex items-center justify-between bg-background/80 px-3 py-2 rounded-lg text-xs border border-border">
                            <span class="font-medium truncate text-foreground flex-1 mr-2">{failed.key}</span>
                            <div class="flex items-center gap-2 shrink-0">
                                <button
                                        type="button"
                                        onclick={() => filesCtx.retryUpload(failed.key)}
                                        class="text-xs text-primary hover:underline font-semibold cursor-pointer"
                                >
                                    {m["common.actions.retry"]()}
                                </button>
                                <button
                                        type="button"
                                        onclick={() => filesCtx.dismissFailed(failed.key)}
                                        class="text-xs text-muted-foreground hover:text-foreground cursor-pointer"
                                >
                                    ✕
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="flex flex-col gap-2">
                    <div class="text-xs text-muted-foreground">
                        {m["folder.upload.failed.total"]({count: filesCtx.failedUploads.size})}
                    </div>
                    <div class="flex flex-col gap-1.5 max-h-40 overflow-y-auto pr-1">
                        {#each Array.from(filesCtx.failedUploads.values()).slice(0, 5) as failed}
                            <div class="flex items-center justify-between bg-background/80 px-3 py-1.5 rounded-lg text-xs border border-border">
                                <span class="font-medium truncate text-foreground flex-1 mr-2">{failed.key}</span>
                                <button
                                        type="button"
                                        onclick={() => filesCtx.retryUpload(failed.key)}
                                        class="text-xs text-primary hover:underline font-semibold cursor-pointer shrink-0"
                                >
                                    {m["common.actions.retry"]()}
                                </button>
                            </div>
                        {/each}
                    </div>
                    <div class="text-[11px] text-muted-foreground text-center">
                        {m["folder.upload.failed.and"]({count: filesCtx.failedUploads.size - 5})}
                    </div>
                </div>
            {/if}
        </div>
    {/if}
{/if}
