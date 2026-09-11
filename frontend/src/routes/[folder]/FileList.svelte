<script lang="ts">
    import type {
        FileId,
        FolderId,
        FolderToken,
    } from '$lib/grpc/gen/folder/v1/common_pb';
    import { fileClient, useGrpc, useStreamGrpc } from '$lib/grpc';
    import { LoaderCircle } from '@lucide/svelte';
    import { create } from '@bufbuild/protobuf';
    import {
        DownloadRequestSchema,
        type FileView,
        ListFilesRequestSchema,
    } from '$lib/grpc/gen/folder/v1/files_pb';
    import type { UploadItem } from '$lib/stores/upload.svelte';
    import ErrorBanner from '$lib/components/error/ErrorBanner.svelte';
    import FileEntry from './FileEntry.svelte';
    import { decryptBlob } from '$lib/crypt';
    import * as m from '$lib/paraglide/messages';
    import type { NewFile } from '$lib/grpc/gen/folder/v1/updates_pb';
    import UploadEntry from './UploadEntry.svelte';
    import { limitsStore } from '$lib/stores/limits.svelte';
    import {
        EncryptedBlobsSchema,
        type EncryptedVault,
    } from '$lib/grpc/gen/folder/v1/encryption_pb';

    let {
        id = $bindable(),
        uploading = $bindable(),
        key = $bindable(),
        token = $bindable(),
        filesCount = $bindable(),
    }: {
        id: FolderId;
        uploading: UploadItem[];
        key: CryptoKey;
        token: FolderToken | null;
        filesCount: number;
    } = $props();

    export interface DecryptedFileView {
        id: FileId;
        metadata: Metadata;
        size: bigint;
    }

    let localLoading = $state(true);
    let localError = $state<string | null>(null);

    let files = $state(new Map<string, DecryptedFileView>());
    let filesArray = $derived([...files.values()]);

    const useListFiles = useGrpc(fileClient.listFiles);
    const isLoading = $derived(localLoading || useListFiles.loading);

    const activeUploads = $derived(
        uploading.filter((u) => u.status.case !== 'canceled')
    );

    export interface Metadata {
        path: string;
        type: string;
    }

    async function decryptMetadata(
        file: FileView
    ): Promise<DecryptedFileView | null> {
        if (!key || !file?.metadata?.value || !file.id) return null;

        const decryptedBytes: Uint8Array = await decryptBlob(
            key,
            file.metadata.value
        );

        const decoder = new TextDecoder();
        const jsonString = decoder.decode(decryptedBytes);

        return {
            id: file.id,
            metadata: JSON.parse(jsonString) as Metadata,
            size: file.size!!,
        };
    }

    export async function addFile(file: NewFile) {
        if (!file.file) {
            return;
        }

        await pushFileView(file.file);
    }

    async function pushFileView(file: FileView) {
        let decrypted = await decryptMetadata(file);

        if (!decrypted) {
            return;
        }

        files.set(decrypted.id.value, decrypted);
        files = new Map(files);
    }

    function concatUint8Arrays(arrays: Uint8Array[]): Uint8Array {
        const totalLength = arrays.reduce((acc, arr) => acc + arr.length, 0);

        const result = new Uint8Array(totalLength);

        let offset = 0;
        for (const arr of arrays) {
            result.set(arr, offset);
            offset += arr.length;
        }

        return result;
    }

    async function downloadFile(
        file: DecryptedFileView,
        onProgress: (percent: number) => void
    ) {
        let useDownload = useStreamGrpc(fileClient.download);

        const chunks: Uint8Array[] = [];
        let vault: EncryptedVault | null = null;
        const totalSize = Number(file.size) || 0;
        let receivedLength = 0;

        for await (const chunk of useDownload.call(
            create(DownloadRequestSchema, {
                file: file.id,
                folder: id,
            })
        )) {
            if (chunk.blob && chunk.blob.part) {
                chunks.push(chunk.blob.part);
                receivedLength += chunk.blob.part.length;

                const percent = Math.min(
                    100,
                    Math.round((receivedLength / totalSize) * 100)
                );
                onProgress(percent);
            }

            if (chunk.vault && !vault) {
                vault = chunk.vault;
            }
        }

        const totalLength = chunks.reduce((acc, val) => acc + val.length, 0);
        const ciphertext = new Uint8Array(totalLength);
        let offset = 0;
        for (const c of chunks) {
            ciphertext.set(c, offset);
            offset += c.length;
        }

        let decryptedData: Uint8Array;
        try {
            decryptedData = await decryptBlob(
                key,
                create(EncryptedBlobsSchema, {
                    meta: vault!!,
                    data: concatUint8Arrays(chunks),
                })
            );
        } catch (e) {
            console.error('Error file decryption: ', e);
            return;
        }

        const blob = new Blob([decryptedData as Uint8Array<ArrayBuffer>]);
        const url = URL.createObjectURL(blob);

        const a = document.createElement('a');
        a.style.display = 'none';
        a.href = url;
        a.download = file.metadata.path || 'downloaded_file';

        document.body.appendChild(a);
        a.click();

        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    $effect(() => {
        filesCount = files.size;
    });

    $effect(() => {
        (async () => {
            try {
                await useListFiles.call(
                    create(ListFilesRequestSchema, {
                        folder: id,
                    })
                );

                if (useListFiles.error) {
                    localError = useListFiles.error.message;
                }

                if (useListFiles.data) {
                    let results = (
                        await Promise.all(
                            useListFiles.data.files!!.map(async (file) => {
                                const decrypted = await decryptMetadata(file);
                                if (!decrypted) return null;
                                return decrypted;
                            })
                        )
                    ).filter((f): f is DecryptedFileView => f !== null);

                    for (const decrypted of results) {
                        files.set(decrypted.id.value, decrypted);
                    }
                }
            } finally {
                localLoading = false;
            }
        })();
    });
</script>

{#if isLoading}
    <div class="flex items-center justify-center">
        <LoaderCircle class="h-auto w-8 animate-spin" />
    </div>
{:else if localError}
    <ErrorBanner error={localError} />
{:else if files}
    <div class="flex flex-col gap-1">
        <span
            >{m['files.files']({
                count: files.size,
                max: limitsStore.data?.maxFilesPerFolder ?? '...',
            })}</span
        >
        {#each filesArray as file (file.id)}
            <FileEntry
                {file}
                onDownload={async (o) => await downloadFile(file, o)}
            />
        {/each}
    </div>

    {#if activeUploads.length >= 1}
        {#if files.size >= 1}
            <div class="flex h-3 w-full items-center">
                <div class="bg-muted h-0.5 w-full"></div>
            </div>
        {/if}

        <div class="flex flex-col">
            {#each activeUploads as _, i}
                <UploadEntry
                    bind:upload={activeUploads[i]}
                    onComplete={pushFileView}
                />
            {/each}
        </div>
    {/if}
{/if}
