<script lang="ts">
    import { getFilesContext } from "$lib/context/files.svelte";
    import { getFolderContext } from "$lib/context/folder.svelte";
    import { getUploadContext } from "$lib/context/upload.svelte";
    import { formatBytes } from "$lib/utils";
    import {
        Folder,
        File as FileIcon,
        ChevronRight,
        Download,
        Trash2,
        LoaderCircle,
        X,
        RotateCw,
        AlertCircle,
    } from "@lucide/svelte";
    import * as m from "$lib/paraglide/messages";

    const filesCtx = getFilesContext();
    const folderCtx = getFolderContext();
    const uploadCtx = getUploadContext();

    let items = $derived(filesCtx.currentItems);
    let uploadingTasks = $derived(
        uploadCtx.tasks.filter((t) => t.state.case !== "completed")
    );
</script>

<div class="rounded-xl border bg-card text-card-foreground shadow-xs overflow-hidden">
    {#if filesCtx.state.case === "loading" && items.files.length === 0 && uploadingTasks.length === 0}
        <div class="flex items-center justify-center p-12 text-muted-foreground gap-2">
            <LoaderCircle class="size-5 animate-spin" />
            <span>{m["folder.status.loading"]()}</span>
        </div>
    {:else if filesCtx.state.case === "error" && items.files.length === 0}
        <div class="flex items-center justify-center p-12 text-destructive gap-2">
            <span>{filesCtx.state.message}</span>
        </div>
    {:else if items.subfolders.length === 0 && items.files.length === 0 && uploadingTasks.length === 0}
        <div class="flex flex-col items-center justify-center p-12 text-muted-foreground gap-2">
            <Folder class="size-12 opacity-40" />
            <p>{m["folder.empty"]()}</p>
        </div>
    {:else}
        <div class="overflow-x-auto">
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b bg-muted/50 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                        <th class="py-3 px-4">{m["folder.table.name"]()}</th>
                        <th class="py-3 px-4">{m["folder.table.size"]()}</th>
                        <th class="py-3 px-4 text-right">{m["folder.table.actions"]()}</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-border">
                    <!-- Uploading / Failed Tasks (Always at the top) -->
                    {#each uploadingTasks as task (task.id)}
                        <tr class="hover:bg-muted/30 transition-colors bg-primary/5">
                            <td class="py-3 px-4">
                                <div class="flex flex-col gap-1.5 min-w-0 max-w-md">
                                    <div class="flex items-center gap-2.5 min-w-0">
                                        <FileIcon class="size-5 text-primary shrink-0 animate-pulse" />
                                        <span class="font-medium truncate">{task.file.name}</span>
                                    </div>
                                    {#if task.state.case === "uploading"}
                                        <div class="flex flex-col gap-1 w-full">
                                            <div class="w-full h-1.5 rounded-full bg-muted overflow-hidden">
                                                <div
                                                    class="h-full bg-primary transition-all duration-300"
                                                    style="width: {task.state.progress}%"
                                                ></div>
                                            </div>
                                            <div class="flex items-center justify-between text-xs text-muted-foreground">
                                                <span>{task.state.progress}% • {formatBytes(task.state.speed)}/s</span>
                                                <span>
                                                    {#if task.state.eta > 0}
                                                        ETA: {task.state.eta}s
                                                    {:else}
                                                        {m["folder.status.uploading"]()}
                                                    {/if}
                                                </span>
                                            </div>
                                        </div>
                                    {:else if task.state.case === "error"}
                                        <div class="flex items-center gap-1.5 text-xs text-destructive">
                                            <AlertCircle class="size-3.5" />
                                            <span>{task.state.message}</span>
                                        </div>
                                    {:else if task.state.case === "cancelled"}
                                        <div class="text-xs text-muted-foreground">{m["folder.status.cancelled"]()}</div>
                                    {/if}
                                </div>
                            </td>
                            <td class="py-3 px-4 text-sm text-muted-foreground">
                                {formatBytes(task.file.size)}
                            </td>
                            <td class="py-3 px-4 text-right">
                                <div class="flex items-center justify-end gap-2">
                                    {#if task.state.case === "uploading"}
                                        <button
                                            onclick={() => uploadCtx.cancelTask(task.id)}
                                            class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-destructive transition-colors cursor-pointer"
                                            title={m["folder.actions.cancel"]()}
                                        >
                                            <X class="size-4" />
                                        </button>
                                    {:else if task.state.case === "error" || task.state.case === "cancelled"}
                                        <button
                                            onclick={() => uploadCtx.retryTask(task.id)}
                                            class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-primary transition-colors cursor-pointer flex items-center gap-1 text-xs font-medium"
                                            title={m["folder.actions.retry"]()}
                                        >
                                            <RotateCw class="size-4" />
                                            <span>{m["folder.actions.retry"]()}</span>
                                        </button>
                                        <button
                                            onclick={() => uploadCtx.removeTask(task.id)}
                                            class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                                            title={m["folder.actions.dismiss"]()}
                                        >
                                            <X class="size-4" />
                                        </button>
                                    {/if}
                                </div>
                            </td>
                        </tr>
                    {/each}

                    <!-- Subfolders -->
                    {#each items.subfolders as folder}
                        <tr class="hover:bg-muted/30 transition-colors">
                            <td class="py-3 px-4">
                                <button
                                    onclick={() => (filesCtx.currentPath = [...filesCtx.currentPath, folder])}
                                    class="flex items-center gap-3 text-left group cursor-pointer w-full"
                                >
                                    <Folder class="size-5 text-blue-500 fill-blue-500/20 shrink-0" />
                                    <span class="font-medium group-hover:text-foreground">{folder}</span>
                                </button>
                            </td>
                            <td class="py-3 px-4 text-sm text-muted-foreground">—</td>
                            <td class="py-3 px-4 text-right">
                                <ChevronRight class="size-4 text-muted-foreground inline" />
                            </td>
                        </tr>
                    {/each}

                    <!-- Files -->
                    {#each items.files as file}
                        <tr class="hover:bg-muted/30 transition-colors">
                            <td class="py-3 px-4">
                                <div class="flex items-center gap-3 min-w-0">
                                    <FileIcon class="size-5 text-muted-foreground shrink-0" />
                                    <span class="font-medium truncate">{file.name}</span>
                                </div>
                            </td>
                            <td class="py-3 px-4 text-sm text-muted-foreground">
                                {formatBytes(file.size)}
                            </td>
                            <td class="py-3 px-4 text-right">
                                <div class="flex items-center justify-end gap-2">
                                    <button
                                        onclick={() => filesCtx.downloadFile(file.id, file.name)}
                                        disabled={filesCtx.downloadingFileId?.value === file.id.value}
                                        class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50 cursor-pointer"
                                        title={m["folder.actions.download"]()}
                                    >
                                        {#if filesCtx.downloadingFileId?.value === file.id.value}
                                            <LoaderCircle class="size-4 animate-spin" />
                                        {:else}
                                            <Download class="size-4" />
                                        {/if}
                                    </button>
                                    {#if folderCtx?.token}
                                        <button
                                            onclick={() => filesCtx.deleteFile(file.id)}
                                            disabled={filesCtx.deletingFileId?.value === file.id.value}
                                            class="p-2 rounded-lg hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors disabled:opacity-50 cursor-pointer"
                                            title={m["folder.actions.delete"]()}
                                        >
                                            {#if filesCtx.deletingFileId?.value === file.id.value}
                                                <LoaderCircle class="size-4 animate-spin" />
                                            {:else}
                                                <Trash2 class="size-4" />
                                            {/if}
                                        </button>
                                    {/if}
                                </div>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
