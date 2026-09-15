<script lang="ts">
    import { getUploadContext } from "$lib/context/upload.svelte";
    import { formatBytes } from "$lib/utils";
    import { File as FileIcon, X, CheckCircle2, AlertCircle, Ban } from "@lucide/svelte";

    const uploadCtx = getUploadContext();
</script>

{#if uploadCtx.tasks.length > 0}
    <div class="flex flex-col gap-3 p-4 rounded-xl border bg-card text-card-foreground shadow-xs">
        <h3 class="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
            Uploads ({uploadCtx.tasks.length})
        </h3>
        <div class="flex flex-col gap-3">
            {#each uploadCtx.tasks as task (task.id)}
                <div class="flex flex-col gap-1.5 p-3 rounded-lg bg-muted/30 border">
                    <div class="flex items-center justify-between gap-2">
                        <div class="flex items-center gap-2 min-w-0">
                            <FileIcon class="size-4 text-muted-foreground shrink-0" />
                            <span class="text-sm font-medium truncate">{task.file.name}</span>
                            <span class="text-xs text-muted-foreground shrink-0">
                                ({formatBytes(task.file.size)})
                            </span>
                        </div>
                        <div class="flex items-center gap-2 shrink-0">
                            {#if task.state.case === "uploading"}
                                <button
                                    onclick={() => uploadCtx.cancelTask(task.id)}
                                    class="p-1 rounded-md hover:bg-muted text-muted-foreground hover:text-destructive transition-colors cursor-pointer"
                                    title="Cancel upload"
                                >
                                    <X class="size-4" />
                                </button>
                            {:else}
                                <button
                                    onclick={() => uploadCtx.removeTask(task.id)}
                                    class="p-1 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                                    title="Dismiss"
                                >
                                    <X class="size-4" />
                                </button>
                            {/if}
                        </div>
                    </div>

                    <!-- Progress bar & details -->
                    {#if task.state.case === "uploading"}
                        <div class="flex flex-col gap-1">
                            <div class="w-full h-2 rounded-full bg-muted overflow-hidden">
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
                                        Calculating...
                                    {/if}
                                </span>
                            </div>
                        </div>
                    {:else if task.state.case === "completed"}
                        <div class="flex items-center gap-1.5 text-xs text-green-600 font-medium">
                            <CheckCircle2 class="size-4" />
                            <span>Completed</span>
                        </div>
                    {:else if task.state.case === "error"}
                        <div class="flex items-center gap-1.5 text-xs text-destructive font-medium">
                            <AlertCircle class="size-4" />
                            <span>{task.state.message}</span>
                        </div>
                    {:else if task.state.case === "cancelled"}
                        <div class="flex items-center gap-1.5 text-xs text-muted-foreground font-medium">
                            <Ban class="size-4" />
                            <span>Cancelled</span>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    </div>
{/if}
