<script lang="ts">
    import {getFolderContext} from "$lib/context/folder.svelte";
    import {getFilesContext} from "$lib/context/files.svelte";
    import * as m from "$lib/paraglide/messages";
    import Breadcrumbs from "./Breadcrumbs.svelte";
    import {folderClient} from "$lib/grpc";
    import {create} from "@bufbuild/protobuf";
    import {OwnedFolderRefSchema} from "$lib/grpc/gen/folder/v1/common_pb";
    import {goto} from "$app/navigation";
    import {localizeHref} from "$lib/paraglide/runtime";
    import {Download, Trash2, LoaderCircle, X} from "@lucide/svelte";

    const ctx = getFolderContext();
    const filesCtx = getFilesContext();

    const decrypted = $derived.by(() => {
        return ctx().decrypted;
    });

    const count = $derived.by(() => {
        const items = filesCtx.currentItems;
        const totalFiles = items.files.length;
        const totalFolders = items.folders.length;
        return totalFiles + totalFolders;
    });

    const downloadTitle = $derived.by(() => {
        if (filesCtx.isRoot) {
            return m["folder.download.all_as_zip"]();
        }

        return m["folder.download.as_zip"]({path: filesCtx.path.join("/")});
    });

    let deletingFolder = $state(false);

    async function handleDeleteFolder() {
        const folderCtx = ctx();
        if (!folderCtx.folder?.id || !folderCtx.token) return;

        if (!confirm(m["folder.delete.confirm_folder"]())) return;

        deletingFolder = true;
        try {
            await folderClient.deleteFolder({
                ownedFolder: create(OwnedFolderRefSchema, {
                    folderId: folderCtx.folder.id,
                    token: folderCtx.token,
                }),
            });
            await goto(localizeHref("/"));
        } catch (e: any) {
            console.error("Failed to delete folder:", e);
        } finally {
            deletingFolder = false;
        }
    }
</script>

{#if decrypted.state === "decrypted"}
    {@const dCtx = decrypted.ctx}

    <div>
        <div class="flex items-center gap-4">
            <div class="flex flex-col flex-1">
                <span class="text-3xl font-bold">{dCtx.name}</span>
                <span class="text-muted-foreground">
                    {#if filesCtx.isLoading}
                        Loading...
                    {:else}
                        {m["folder.files_count"]({count: count.toString()})}
                    {/if}
                </span>
            </div>

            <div class="flex items-center gap-2">
                {#if filesCtx.downloadingZip}
                    <div class="flex items-center gap-3 px-3 py-2 text-xs font-medium rounded-xl bg-card border border-border shadow-sm">
                        <LoaderCircle class="h-4 w-4 animate-spin text-primary shrink-0"/>
                        <div class="flex flex-col min-w-32.5">
                            <span class="font-medium truncate text-foreground">
                                {filesCtx.zipProgress?.status || "..."}
                            </span>
                            {#if filesCtx.zipProgress && filesCtx.zipProgress.total > 0}
                                <div class="w-full bg-muted rounded-full h-1 mt-1 overflow-hidden">
                                    <div
                                            class="bg-primary h-1 transition-all duration-200"
                                            style="width: {(filesCtx.zipProgress.current / filesCtx.zipProgress.total) * 100}%"
                                    ></div>
                                </div>
                            {/if}
                        </div>
                        <button
                                type="button"
                                title={m["common.actions.cancel"]()}
                                onclick={() => filesCtx.cancelZipDownload()}
                                class="flex h-6 w-6 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted hover:text-destructive transition-colors cursor-pointer shrink-0"
                        >
                            <X class="h-3.5 w-3.5"/>
                        </button>
                    </div>
                {:else}
                    <button
                            type="button"
                            title={downloadTitle}
                            onclick={() => void filesCtx.downloadZip(dCtx.name)}
                            class="flex items-center gap-2 px-3.5 py-2 text-sm font-medium rounded-xl bg-card border border-border text-foreground hover:bg-muted transition-colors cursor-pointer shadow-sm"
                    >
                        <Download class="h-4 w-4"/>
                        <span class="hidden sm:inline">Zip</span>
                    </button>
                {/if}

                {#if ctx().token !== null}
                    <button
                            type="button"
                            title={m["folder.delete.folder"]()}
                            onclick={handleDeleteFolder}
                            disabled={deletingFolder}
                            class="flex items-center gap-2 px-3.5 py-2 text-sm font-medium rounded-xl bg-card border border-border text-destructive hover:bg-destructive/10 disabled:opacity-50 transition-colors cursor-pointer shadow-sm"
                    >
                        {#if deletingFolder}
                            <LoaderCircle class="h-4 w-4 animate-spin"/>
                        {:else}
                            <Trash2 class="h-4 w-4"/>
                        {/if}

                        <span class="hidden sm:inline">{m["common.actions.delete"]()}</span>
                    </button>
                {/if}
            </div>
        </div>

        <Breadcrumbs/>
    </div>
{/if}
