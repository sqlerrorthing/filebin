<script lang="ts">
    import { activeFolder } from "$lib/stores/folder.svelte";
    import { LoaderCircle, Pencil } from "@lucide/svelte";
    import {encryptBlob} from "$lib/crypt";
    import {folderClient, useGrpc} from "$lib/grpc";
    import {create} from "@bufbuild/protobuf";
    import {RenameRequestSchema} from "$lib/grpc/gen/folder/v1/folder_pb";
    import {FolderNameSchema, OwnedFolderRefSchema} from "$lib/grpc/gen/folder/v1/common_pb";

    let isSyncing = $state(false);
    let syncError = $state<string | null>(null);
    let timeout: ReturnType<typeof setTimeout>;

    const renameFolder = useGrpc(folderClient.rename);


    async function saveName(newName: string) {
        if (!activeFolder.key || !activeFolder.ownedRef || !activeFolder.decrypted) return;

        isSyncing = true;
        syncError = null;

        try {
            const encryptedName = await encryptBlob(activeFolder.key, new TextEncoder().encode(newName));

            await renameFolder.call(create(RenameRequestSchema, {
                ownedFolder: activeFolder.ownedRef,
                name: create(FolderNameSchema, {
                    value: encryptedName
                })
            }))

            if (renameFolder.error) {
                syncError = renameFolder.error.message;
            }
        } finally {
            isSyncing = false;
        }
    }

    function handleInput(e: Event) {
        const target = e.target as HTMLInputElement;

        if (target.value.length > 32) {
            target.value = target.value.substring(0, 32);
        }

        const newName = target.value;

        clearTimeout(timeout);

        timeout = setTimeout(() => {
            syncError = null;
            saveName(newName);
        }, 750);
    }
</script>

<div class="flex items-center gap-3">
    {#if activeFolder.decrypted}
        {#if activeFolder.token}
            <div class="relative flex items-center">
                <input
                        type="text"
                        bind:value={activeFolder.decrypted.name}
                        oninput={handleInput}
                        class="text-2xl font-bold bg-transparent border-b-2 border-transparent hover:border-gray-300 focus:border-blue-500 outline-none transition-colors px-1"
                />

                {#if isSyncing}
                    <LoaderCircle class="animate-spin w-4 h-4 text-gray-500 absolute -right-6" />
                {:else}
                    <Pencil class="w-4 h-4 text-gray-400 absolute -right-6 opacity-0 hover:opacity-100" />
                {/if}
            </div>

            {#if syncError}
                <span class="text-red-500 text-sm ml-4">{syncError}</span>
            {/if}
        {:else}
            <h1 class="text-2xl font-bold px-1">
                {activeFolder.decrypted.name}
            </h1>
        {/if}
    {/if}
</div>