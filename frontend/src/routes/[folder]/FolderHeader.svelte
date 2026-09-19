<script lang="ts">
    import {getFolderContext} from "$lib/context/folder.svelte";
    import {getFilesContext} from "$lib/context/files.svelte";
    import * as m from "$lib/paraglide/messages";
    import Breadcrumbs from "./Breadcrumbs.svelte";

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
</script>

{#if decrypted.state === "decrypted"}
    {@const dCtx = decrypted.ctx}

    <div>
        <div>
            <div class="flex flex-col">
                <span class="text-3xl font-bold">{dCtx.name}</span>
                <span class="text-muted-foreground">
                    {#if filesCtx.isLoading}
                        Loading...
                    {:else}
                        {m["folder.files_count"]({ count: count.toString() })}
                    {/if}
                </span>
            </div>
        </div>

        <Breadcrumbs />
    </div>
{/if}
