<script lang="ts">
    import { getFolderContext } from "$lib/context/folder.svelte";
    import { getUploadContext } from "$lib/context/upload.svelte";
    import { Folder, Pencil, Check, X, LoaderCircle, Upload } from "@lucide/svelte";
    import * as m from "$lib/paraglide/messages";

    const folderCtx = getFolderContext();
    const uploadCtx = getUploadContext();

    let isEditing = $state(false);
    let newFolderName = $state("");
    let saving = $state(false);
    let errorMessage = $state<string | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);

    $effect(() => {
        if (folderCtx.folderName) {
            newFolderName = folderCtx.folderName;
        }
    });

    const handleRename = async () => {
        if (!newFolderName.trim()) return;
        saving = true;
        errorMessage = null;
        try {
            await folderCtx.rename(newFolderName.trim());
            isEditing = false;
        } catch (e: any) {
            errorMessage = e?.message || m["folder.errors.rename_failed"]();
        } finally {
            saving = false;
        }
    };

    const handleFileSelect = async (e: Event) => {
        const target = e.target as HTMLInputElement;
        if (!target.files || target.files.length === 0) return;
        const selectedFiles = Array.from(target.files);
        target.value = "";
        await uploadCtx.uploadFiles(selectedFiles);
    };

    const formattedCreatedAt = $derived.by(() => {
        const dt = folderCtx.folder?.createdAt;
        if (!dt || !dt.year || !dt.month || !dt.day) return "";
        const date = new Date(
            Date.UTC(dt.year, dt.month - 1, dt.day, dt.hours || 0, dt.minutes || 0, dt.seconds || 0)
        );
        return date.toLocaleDateString(undefined, {
            year: "numeric",
            month: "short",
            day: "numeric",
        });
    });
</script>

<div class="flex flex-col gap-2">
    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b pb-4">
        <div class="flex items-center gap-3">
            <Folder class="size-8 text-primary" />
            {#if isEditing}
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        bind:value={newFolderName}
                        class="px-3 py-1.5 rounded-lg border bg-input text-foreground text-lg font-semibold focus:outline-none focus:ring-2 focus:ring-ring"
                        placeholder={m["folder.default_name"]()}
                    />
                    <button
                        onclick={handleRename}
                        disabled={saving}
                        class="p-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50 cursor-pointer"
                        title={m["common.actions.confirm"]()}
                    >
                        {#if saving}
                            <LoaderCircle class="size-4 animate-spin" />
                        {:else}
                            <Check class="size-4" />
                        {/if}
                    </button>
                    <button
                        onclick={() => {
                            isEditing = false;
                            newFolderName = folderCtx.folderName;
                        }}
                        disabled={saving}
                        class="p-2 rounded-lg border hover:bg-muted cursor-pointer"
                        title={m["common.actions.close"]()}
                    >
                        <X class="size-4" />
                    </button>
                </div>
            {:else}
                <div class="flex flex-col">
                    <div class="flex items-center gap-2">
                        <h1 class="text-2xl font-bold tracking-tight">{folderCtx.folderName}</h1>
                        {#if folderCtx?.token}
                            <button
                                onclick={() => {
                                    isEditing = true;
                                    newFolderName = folderCtx.folderName;
                                }}
                                class="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
                                title="Rename folder"
                            >
                                <Pencil class="size-4" />
                            </button>
                        {/if}
                    </div>
                    {#if formattedCreatedAt}
                        <span class="text-xs text-muted-foreground mt-0.5">
                            {m["folder.created_at"]({ date: formattedCreatedAt })}
                        </span>
                    {/if}
                </div>
            {/if}
        </div>

        {#if folderCtx?.token}
            <div class="flex items-center gap-2">
                <input
                    type="file"
                    multiple
                    class="hidden"
                    bind:this={fileInput}
                    onchange={handleFileSelect}
                />
                <button
                    onclick={() => fileInput?.click()}
                    class="flex items-center gap-2 px-4 py-2 rounded-xl bg-primary text-primary-foreground font-medium hover:opacity-90 transition-all cursor-pointer shadow-xs"
                >
                    <Upload class="size-4" />
                    <span>{m["folder.actions.upload"]()}</span>
                </button>
            </div>
        {/if}
    </div>

    {#if errorMessage}
        <div class="p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            {errorMessage}
        </div>
    {/if}
</div>
