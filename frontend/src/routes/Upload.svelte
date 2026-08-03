<script lang="ts">
    import {FileUp, LoaderCircle} from "@lucide/svelte";
    import * as m from "$lib/paraglide/messages";
    import {exportKey, generateCryptoKey} from "$lib/crypt";
    import {createFolder} from "$lib/folders/create";
    import {cn} from "$lib/utils";
    import {goto} from "$app/navigation";
    import {onMount} from "svelte";
    import {activeFolder} from "$lib/stores/folder.svelte";

    let loading = $state(false);
    let error = $state<string | null>(null);

    async function handleUpload() {
        loading = true;
        error = null;

        try {
            const key = await generateCryptoKey();
            const folder = await createFolder(key, m["folders.base-name"]());

            activeFolder.set(
                folder.folder!!,
                key,
                folder.token!!
            )

            const exportedKey = await exportKey(key);
            await goto(`${folder?.folder?.id?.value!!}#${exportedKey}`)
        } catch (e: any) {
            error = e.toString();
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loading = false;
        error = null;
    })
</script>

{#if error}
    <p class="text-destructive text-center">{error}</p>
{/if}

<button
        class={cn("\
            cursor-pointer bg-primary text-primary-foreground px-4 py-2 shadow-sm flex gap-3 justify-center \
            sm:w-48 hover:bg-accent-foreground",
            loading && "bg-muted-foreground"
        )}
        onclick={handleUpload}
        disabled={loading}
>
    {#if loading}
        <LoaderCircle class="animate-spin" />
    {:else}
        <FileUp/>
    {/if}
    {m["files.upload"]()}
</button>
