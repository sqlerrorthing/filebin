<script lang="ts">
    import type {FileId, FolderId, FolderToken} from "$lib/grpc/gen/folder/v1/common_pb";
    import {fileClient, useGrpc} from "$lib/grpc";
    import {LoaderCircle} from "@lucide/svelte";
    import {create} from "@bufbuild/protobuf";
    import {type FileView, ListFilesRequestSchema} from "$lib/grpc/gen/folder/v1/files_pb";
    import type {UploadItem} from "$lib/stores/upload.svelte";
    import ErrorBanner from "$lib/components/error/ErrorBanner.svelte";
    import FileEntry from "./FileEntry.svelte";
    import {decryptBlob} from "$lib/crypt";
    import * as m from "$lib/paraglide/messages";
    import type {NewFile} from "$lib/grpc/gen/folder/v1/updates_pb";
    import UploadEntry from "./UploadEntry.svelte";
    import {limitsStore} from "$lib/stores/limits.svelte";

    let {
        id = $bindable(),
        uploading = $bindable(),
        key = $bindable(),
        token = $bindable(),
        filesCount = $bindable()
    }: {
        id: FolderId,
        uploading: UploadItem[],
        key: CryptoKey,
        token: FolderToken | null,
        filesCount: number
    } = $props();

    export interface DecryptedFileView {
        id: FileId,
        metadata: Metadata,
        size: bigint
    }

    let localLoading = $state(true);
    let localError = $state<string | null>(null);

    let files = $state<DecryptedFileView[]>([]);

    const useListFiles = useGrpc(fileClient.listFiles);
    const isLoading = $derived(localLoading || useListFiles.loading);

    const activeUploads = $derived(uploading.filter(u => u.status.case !== "canceled"));

    export interface Metadata {
        path: string,
        type: string
    }

    async function decryptMetadata(file: FileView): Promise<DecryptedFileView | null> {
        if (!key || !file?.metadata?.value || !file.id) return null;

        const decryptedBytes: Uint8Array = await decryptBlob(key, file.metadata.value);

        const decoder = new TextDecoder();
        const jsonString = decoder.decode(decryptedBytes);

        return {
            id: file.id,
            metadata: JSON.parse(jsonString) as Metadata,
            size: file.size!!
        };
    }

    export async function addFile(file: NewFile) {
        if (!file.file) {
            return;
        }

        await pushFileView(file.file);
    }

    async function pushFileView(file: FileView) {
        if (!files.find(f => f.id == file.id)) {
            let decrypted = await decryptMetadata(file);

            if (!decrypted) {
                return;
            }

            files = [...files, decrypted];
        }
    }

    $effect(() => {
        filesCount = files.length;
    });

    $effect(() => {
        (async () => {
            try {
                await useListFiles.call(create(
                    ListFilesRequestSchema, {
                        folder: id
                    }
                ));

                if (useListFiles.error) {
                    localError = useListFiles.error.message;
                }

                if (useListFiles.data) {
                    files = (await Promise.all(
                        useListFiles.data.files!!.map(async (file) => {
                            const decrypted = await decryptMetadata(file);
                            if (!decrypted) return null;
                            return decrypted;
                        })
                    )).filter((f): f is DecryptedFileView => f !== null);
                }
            } finally {
                localLoading = false
            }
        })()
    })
</script>

{#if isLoading}
    <div class="flex items-center justify-center">
        <LoaderCircle class="animate-spin w-8 h-auto"/>
    </div>
{:else if localError}
    <ErrorBanner error={localError}/>
{:else if files}
    <div class="flex flex-col gap-1">
        <span>{m["files.files"]({ count: files.length, max: limitsStore.data?.maxFilesPerFolder ?? '...' })}</span>
        {#each files as _, i}
            <FileEntry bind:file={files[i]} onDownload={async () => {}} />
        {/each}
    </div>

    {#if activeUploads.length >= 1}
        {#if files.length >= 1}
            <div class="w-full h-3 flex items-center">
                <div class="w-full h-0.5 bg-muted"></div>
            </div>
        {/if}

        <div class="flex flex-col">
            {#each activeUploads as _, i}
                <UploadEntry bind:upload={activeUploads[i]} onComplete={pushFileView}/>
            {/each}
        </div>
    {/if}
{/if}