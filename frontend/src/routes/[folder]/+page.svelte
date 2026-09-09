<script lang="ts">
    import {activeFolder} from "$lib/stores/folder.svelte";
    import {page} from "$app/state";
    import {folderClient, useGrpc, useStreamGrpc} from "$lib/grpc";
    import {onMount} from "svelte";
    import * as m from "$lib/paraglide/messages";
    import {exportKey, importKeyFromUrlSafe} from "$lib/crypt";
    import ErrorBanner from "$lib/components/error/ErrorBanner.svelte";
    import {Check, LoaderCircle, Share2} from "@lucide/svelte";
    import ShareModal from "$lib/components/share/ShareModal.svelte";
    import {create} from "@bufbuild/protobuf";
    import {DeleteFolderRequestSchema, GetFolderRequestSchema} from "$lib/grpc/gen/folder/v1/folder_pb";
    import {FolderIdSchema} from "$lib/grpc/gen/folder/v1/common_pb";
    import {Code, ConnectError} from "@connectrpc/connect";
    import FolderName from "./FolderName.svelte";
    import {goto} from "$app/navigation";
    import {localizeHref} from "$lib/paraglide/runtime";
    import FileList from "./FileList.svelte";
    import {uploadStore} from "$lib/stores/upload.svelte";
    import {limitsStore} from "$lib/stores/limits.svelte";
    import Upload from "$lib/components/upload/Upload.svelte";
    import {Trash} from "@lucide/svelte";
    import {cn} from "$lib/utils";

    const useGetFolder = useGrpc(folderClient.getFolder);
    const useDeleteFolder = useGrpc(folderClient.deleteFolder);

    const updatesStream = useStreamGrpc(folderClient.updates);

    let filesCount = $state(0);
    let filesList: FileList | null = $state(null);

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
            && await exportKey(activeFolder.key) === keyString)
        {
            localLoading = false;
            return;
        }

        try {
            localLoading = true;
            const key = await importKeyFromUrlSafe(keyString);

            await useGetFolder.call(create(GetFolderRequestSchema, {
                id: create(FolderIdSchema, {
                    value: routeFolderId
                })
            }));

            if (useGetFolder.error instanceof ConnectError) {
                if (useGetFolder.error.code === Code.NotFound) {
                    localError = m["folders.not-found"]()
                } else if (useGetFolder.error.code === Code.InvalidArgument) {
                    localError = m["folders.incorrect-id"]()
                } else {
                    localError = useGetFolder.error.toString()
                }
            }

            if (useGetFolder.data) {
                await activeFolder.set(
                    useGetFolder.data,
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
                    activeFolder.clear();
                    await goto(localizeHref("/"), {replaceState: true});
                    break
                }
                case "folderNameChanged": {
                    break
                }
                case "newFile": {
                    if (filesList) {
                        await filesList.addFile(update.value)
                    }

                    break
                }
            }
        }
    }

    async function deleteFolder() {
        if (!activeFolder.ownedRef) {
            return
        }

        localLoading = true;

        try {
            let resp = await useDeleteFolder.call(create(DeleteFolderRequestSchema, {ownedFolder: activeFolder.ownedRef}));

            if (resp) {
                activeFolder.clear();
                await goto(localizeHref("/"), {replaceState: true});
            }
        } finally {
            localLoading = false;
        }
    }

    let isConfirming = $state(false);
    let canConfirm = $state(false);
    let isSharing = $state(false);

    let cooldownTimer: number | null = null;
    let totalTimer: number | null = null;

    function startCooldown() {
        canConfirm = false;
        if (cooldownTimer) clearTimeout(cooldownTimer);

        cooldownTimer = setTimeout(() => {
            canConfirm = true;
        }, 500);
    }

    function handleDeleteClick() {
        if (!isConfirming) {
            isConfirming = true;
            startCooldown();

            totalTimer = setTimeout(() => {
                resetState();
            }, 2000);
        } else {
            if (!canConfirm) {
                startCooldown();
            } else {
                clearAllTimers();
                resetState();
                deleteFolder();
            }
        }
    }

    function resetState() {
        isConfirming = false;
        canConfirm = false;
        clearAllTimers();
    }

    function clearAllTimers() {
        if (cooldownTimer) clearTimeout(cooldownTimer);
        if (totalTimer) clearTimeout(totalTimer);
        cooldownTimer = null;
        totalTimer = null;
    }
</script>

{#if isLoading}
    <div class="flex justify-center">
        <LoaderCircle class="animate-spin w-8 h-auto"/>
    </div>
{:else if errorMessage}
    <ErrorBanner error={errorMessage}/>
{:else if activeFolder.decrypted}
    <div class="flex gap-3 flex-col">
        <div class="flex">
            <div class="flex-1 min-w-0">
                <FolderName/>
            </div>

            <div class="flex items-center gap-1">
                <button
                        class="cursor-pointer flex items-center gap-2 px-3 py-2 hover:bg-muted"
                        onclick={() => isSharing = true}
                        title="Share folder"
                >
                    <Share2 class="w-4 h-4" />
                </button>
                {#if activeFolder.token}
                    <button
                            class={cn(
                                "cursor-pointer flex items-center gap-2 px-3 py-2",
                                isConfirming ? "bg-destructive text-primary-foreground" : "hover:bg-muted",
                                isConfirming && !canConfirm && "pointer-events-none bg-muted text-muted-foreground"
                            )}
                            onclick={handleDeleteClick}
                            disabled={isConfirming && !canConfirm}
                    >
                        {#if isConfirming}
                            <Check class="w-4 h-4" />
                            <span class="text-sm/2">{m["files.delete-confirm"]()}</span>
                        {:else}
                            <Trash class="w-4 h-4" />
                        {/if}
                    </button>
                {/if}
            </div>
        </div>
        <div>
            <FileList
                    bind:id={activeFolder.id!!}
                    bind:uploading={uploadStore.items}
                    bind:key={activeFolder.key!!}
                    bind:token={activeFolder.token}
                    bind:filesCount
                    bind:this={filesList}
            />
        </div>

        {#if activeFolder.token && filesCount < (limitsStore.data?.maxFilesPerFolder ?? 0)}
            <div class="flex justify-center">
                <Upload/>
            </div>
        {/if}
    </div>

    {#if isSharing}
        <ShareModal onClose={() => isSharing = false}/>
    {/if}
{/if}
