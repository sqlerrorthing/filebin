<script lang="ts">
    import {getEncryptedFolderContext} from "$lib/context/folder.svelte";
    import * as m from "$lib/paraglide/messages";
    import Breadcrumbs from "./Breadcrumbs.svelte";

    const ctx = getEncryptedFolderContext();

    const decrypted = $derived.by(() => {
        return ctx().decrypted;
    })
</script>

{#if decrypted.state === "decrypted"}
    {@const dCtx = decrypted.ctx}

    <div>
        <div>
            <div class="flex flex-col">
                <span class="text-3xl font-bold">{dCtx.name}</span>
                <span class="text-muted-foreground">{m["folder.files_count"]({ count: "..." })}</span>
            </div>
        </div>

        <Breadcrumbs />
    </div>
{/if}
