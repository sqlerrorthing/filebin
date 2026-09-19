<script lang="ts">
    import {getFolderContext} from "$lib/context/folder.svelte";
    import {getFilesContext} from "$lib/context/files.svelte";

    const ctx = getFilesContext();

    const items = $derived.by(() => {
        return ctx.currentItems;
    })

    const folders = $derived.by(() => {
        return items.folders;
    })

    const files = $derived.by(() => {
        return items.files;
    })
</script>

<div class="overflow-hidden border border-solid border-border bg-card rounded-2xl">
    <div class="grid grid-cols-[1fr_120px_100px] items-center border-b border-border px-4 py-3 text-sm text-muted-foreground">
        <span>File</span>
        <span>Size</span>
        <span class="text-right">Actions</span>
    </div>

    {#each files as file}
        <div
                class="grid grid-cols-[1fr_120px_100px] items-center px-4 py-3 transition-colors hover:bg-muted/50"
        >
            <div class="flex min-w-0 items-center gap-3">
                <div
                        class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-muted"
                >
                    📄
                </div>

                <div class="min-w-0">
                    <p class="truncate text-sm font-medium">
                        {file.name}
                    </p>
                </div>
            </div>

            <span class="text-sm text-muted-foreground">
                {file.size}
            </span>

            <div class="flex justify-end gap-1">

            </div>
        </div>
    {/each}
</div>
