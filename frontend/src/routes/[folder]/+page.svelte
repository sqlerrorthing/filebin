<script lang="ts">
    import { activeFolder } from "$lib/stores/folder.svelte";
    import {page} from "$app/state";
    import {folderClient, useGrpc} from "$lib/grpc";
    import {onMount} from "svelte";
    import * as m from "$lib/paraglide/messages";
    import {decryptBlobAsString, exportKey, importKeyFromUrlSafe} from "$lib/crypt";
    import ErrorBanner from "$lib/components/error/ErrorBanner.svelte";
    import { LoaderCircle } from "@lucide/svelte";

    let localLoading = $state(true);
    let localError = $state<string | null>(null);

    const isLoading = $derived(localLoading || activeFolder.isDecrypting);
    const errorMessage = $derived(localError || activeFolder.error);

    const routeFolderId = $derived(page.params.folder);

    const getFolder = useGrpc(folderClient.getFolder);

    onMount(async () => {
        const keyString = page.url.hash.replace("#", "");

        if (
            activeFolder.key !== null
            && activeFolder?.folder?.id?.value === routeFolderId
            && await exportKey(activeFolder.key) === keyString)
        {
            localLoading = false;
            return;
        }

        try {
            localLoading = true;
            const key = await importKeyFromUrlSafe(keyString);

            // todo call getFolder
        } catch (e: any) {
            localError = e.message;
        } finally {
            localLoading = false;
        }
    })
</script>

{#if isLoading}
    <div class="flex justify-center">
        <LoaderCircle class="animate-spin w-8 h-auto" />
    </div>
{:else if errorMessage}
    <ErrorBanner error={errorMessage} />
{:else if activeFolder.decrypted}
    <h1>{activeFolder.decrypted.name}</h1>
{/if}
