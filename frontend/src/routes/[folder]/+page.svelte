<script lang="ts">
    import {activeFolder} from "$lib/stores/folder.svelte";
    import {page} from "$app/state";
    import {folderClient, useGrpc, useStreamGrpc} from "$lib/grpc";
    import {onMount} from "svelte";
    import * as m from "$lib/paraglide/messages";
    import {exportKey, importKeyFromUrlSafe} from "$lib/crypt";
    import ErrorBanner from "$lib/components/error/ErrorBanner.svelte";
    import {LoaderCircle} from "@lucide/svelte";
    import {create} from "@bufbuild/protobuf";
    import {GetFolderRequestSchema} from "$lib/grpc/gen/folder/v1/folder_pb";
    import {FolderIdSchema} from "$lib/grpc/gen/folder/v1/common_pb";
    import {Code, ConnectError} from "@connectrpc/connect";
    import FolderName from "./FolderName.svelte";
    import { goto } from "$app/navigation";
    import {localizeHref} from "$lib/paraglide/runtime";

    const getFolder = useGrpc(folderClient.getFolder);
    const updatesStream = useStreamGrpc(folderClient.updates);

    let localLoading = $state(true);
    let localError = $state<string | null>(null);

    const isLoading = $derived(localLoading || activeFolder.isDecrypting);
    const errorMessage = $derived(localError || activeFolder.error);

    const routeFolderId = $derived(page.params.folder);
    onMount(async () => {
        const keyString = page.url.hash.replace("#", "");

        if (
            activeFolder.key !== null
            && activeFolder?.id?.value === routeFolderId
            && await exportKey(activeFolder.key) === keyString) {
            localLoading = false;
            return;
        }

        try {
            localLoading = true;
            const key = await importKeyFromUrlSafe(keyString);

            await getFolder.call(create(GetFolderRequestSchema, {
                id: create(FolderIdSchema, {
                    value: routeFolderId
                })
            }));

            if (getFolder.error instanceof ConnectError) {
                if (getFolder.error.code === Code.NotFound) {
                    localError = m["folders.not-found"]()
                } else if (getFolder.error.code === Code.InvalidArgument) {
                    localError = m["folders.incorrect-id"]()
                } else {
                    localError = getFolder.error.toString()
                }
            }

            if (getFolder.data) {
                await activeFolder.set(
                    getFolder.data,
                    key,
                    undefined
                )

                startListening().then()
            }
        } catch (e: any) {
            localError = e.message;
        } finally {
            localLoading = false;
        }
    })

    async function startListening() {
        if (activeFolder.id === null) return;

        for await (const updateMsg of updatesStream.call({id: activeFolder.id})) {
            const update = updateMsg.update;
            switch (update.case) {
                case "folderDeleted": {
                    await goto(localizeHref("/"))
                }
            }
        }
    }
</script>

{#if isLoading}
    <div class="flex justify-center">
        <LoaderCircle class="animate-spin w-8 h-auto"/>
    </div>
{:else if errorMessage}
    <ErrorBanner error={errorMessage}/>
{:else if activeFolder.decrypted}
    <FolderName/>
{/if}
