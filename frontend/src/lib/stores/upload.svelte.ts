import { bufferToBase64, encryptBlob, exportKeyToArray } from '$lib/crypt';
import { fileClient, useGrpc } from '$lib/grpc';
import { create } from '@bufbuild/protobuf';
import {
    FileMetadataSchema,
    type FileView,
    InitiateUploadRequestSchema,
    UploadChunkRequestSchema,
} from '$lib/grpc/gen/folder/v1/files_pb';
import type { OwnedFolderRef } from '$lib/grpc/gen/folder/v1/common_pb';
import { ctr } from '@noble/ciphers/aes.js';
import { ghash } from '@noble/ciphers/_polyval.js';
import {
    AlgorithmSchema,
    EncryptedVaultSchema,
    VersionSchema,
} from '$lib/grpc/gen/folder/v1/encryption_pb';

type Status =
    | {
          case: 'pending';
      }
    | {
          case: 'uploading';
          progress: number;
          controller: AbortController;
      }
    | {
          case: 'completed';
          file: FileView;
      }
    | {
          case: 'error';
          error: string;
      }
    | {
          case: 'canceled';
      };

export class UploadItem {
    id = $state<string>();
    file = $state<File>();
    status = $state<Status>({ case: 'pending' });

    constructor(file: File) {
        this.file = file;
    }
}

class UploadStore {
    items = $state<UploadItem[]>([]);

    addFiles(
        files: FileList | File[],
        key: CryptoKey,
        ownedFolderRef: OwnedFolderRef
    ) {
        const newItems = Array.from(files).map((file) => {
            const item = new UploadItem(file);
            item.id = crypto.randomUUID();
            item.status.case = 'pending';
            return item;
        });

        this.items = [...this.items, ...newItems];

        for (const item of newItems) {
            this.#uploadFile(item, key, ownedFolderRef).then();
        }
    }

    async retry(id: string, key: CryptoKey, ownedFolderRef: OwnedFolderRef) {
        const item = this.items.find((i) => i.id === id);

        if (item) {
            if (item.status.case === 'uploading') {
                item.status.controller.abort();
            }

            item.status.case = 'pending';
            await this.#uploadFile(item, key, ownedFolderRef);
        }
    }

    removeUpload(id: string) {
        const item = this.items.find((i) => i.id === id);
        if (item) {
            if (item.status.case === 'uploading') {
                item.status.controller.abort();
            }

            item.status.case = 'canceled';
            this.items = this.items.filter((i) => i.id !== id);
        }
    }

    cancelAll() {
        for (const item of this.items) {
            if (item.status.case === 'uploading') {
                this.removeUpload(item.id!!);
            }
        }
    }

    async #uploadFile(
        item: UploadItem,
        key: CryptoKey,
        ownedFolderRef: OwnedFolderRef
    ) {
        const controller = new AbortController();

        item.status = {
            case: 'uploading',
            controller,
            progress: 0,
        };

        const signal = controller.signal;

        try {
            const encryptedFileMeta = await encryptBlob(
                key,
                new TextEncoder().encode(
                    JSON.stringify({
                        path: item.file!!.name,
                        type: item.file!!.type,
                    })
                )
            );

            const useInitUpload = useGrpc(
                fileClient.initiateUpload.bind(fileClient)
            );

            const initResponse = await useInitUpload.call(
                create(InitiateUploadRequestSchema, {
                    metadata: create(FileMetadataSchema, {
                        value: encryptedFileMeta,
                    }),
                    folder: ownedFolderRef,
                }),
                { signal }
            );

            if (!initResponse) {
                item.status = {
                    case: 'error',
                    error:
                        useInitUpload.error?.message ||
                        'Upload initiated failed',
                };

                return;
            }

            const { uploadId, chunkSize: rawChunkSize } = initResponse;
            const chunkSize = Number(rawChunkSize);

            if (!chunkSize || chunkSize <= 0) {
                throw new Error(`Invalid chunkSize received: ${rawChunkSize}`);
            }

            const keyArray = await exportKeyToArray(key);

            const rawCtrH = ctr(keyArray, new Uint8Array(16));
            const hKey = rawCtrH.encrypt(new Uint8Array(16));

            const iv = window.crypto.getRandomValues(new Uint8Array(12));
            const j0 = new Uint8Array(16);
            j0.set(iv, 0);
            j0[15] = 1;

            const rawCtrMask = ctr(keyArray, j0);
            const tagMask = rawCtrMask.encrypt(new Uint8Array(16));

            const initialCounter = new Uint8Array(16);
            initialCounter.set(iv, 0);
            initialCounter[15] = 2;

            const currentCounter = new Uint8Array(initialCounter);

            const hasher = ghash.create(hKey);

            const totalSize = item.file!!.size;
            let uploadedBytes = 0;
            let offset = 0;
            let chunkIndex = 0;

            const useUploadChunk = useGrpc(
                fileClient.uploadChunk.bind(fileClient)
            );

            function addBlocksToCounter(counter: Uint8Array, blocks: number) {
                const view = new DataView(
                    counter.buffer,
                    counter.byteOffset,
                    counter.byteLength
                );
                const high = view.getUint32(8, false);
                const low = view.getUint32(12, false);

                const totalLow = low + blocks;
                const carry = Math.floor(totalLow / 0x100000000);
                view.setUint32(12, totalLow >>> 0, false);

                if (carry > 0) {
                    const totalHigh = high + carry;
                    view.setUint32(8, totalHigh >>> 0, false);
                }
            }

            while (offset < totalSize) {
                if (signal.aborted)
                    throw new DOMException('Aborted', 'AbortError');

                const currentChunkSize = Math.min(
                    chunkSize,
                    totalSize - offset
                );

                const chunkBlob = item.file!!.slice(
                    offset,
                    offset + currentChunkSize
                );
                const rawChunk = new Uint8Array(await chunkBlob.arrayBuffer());

                offset += currentChunkSize;
                const isLastChunk = offset >= totalSize;

                const cipher = ctr(keyArray, currentCounter);
                const encryptedChunkData = cipher.encrypt(rawChunk);

                const blocksUsed = Math.ceil(rawChunk.length / 16);
                addBlocksToCounter(currentCounter, blocksUsed);

                hasher.update(encryptedChunkData);

                uploadedBytes += rawChunk.length;

                item.status.progress = Math.min(
                    Math.round((uploadedBytes / totalSize) * 100),
                    99
                );

                if (!isLastChunk) {
                    await useUploadChunk.call(
                        create(UploadChunkRequestSchema, {
                            uploadId,
                            chunkData: encryptedChunkData,
                        }),
                        { signal }
                    );
                } else {
                    const lenBlock = new Uint8Array(16);
                    const view = new DataView(lenBlock.buffer);

                    const bitLen = BigInt(uploadedBytes) * 8n;
                    view.setBigUint64(8, bitLen, false);

                    hasher.update(lenBlock);
                    const ghashResult = hasher.digest();

                    const tag = new Uint8Array(16);
                    for (let i = 0; i < 16; i++) {
                        tag[i] = ghashResult[i] ^ tagMask[i];
                    }

                    const vault = create(EncryptedVaultSchema, {
                        iv: bufferToBase64(iv),
                        tag: bufferToBase64(tag),
                        version: create(VersionSchema, { value: 1 }),
                        algo: create(AlgorithmSchema, { value: 'aes-256-gcm' }),
                    });

                    await useUploadChunk.call(
                        create(UploadChunkRequestSchema, {
                            uploadId,
                            chunkData: encryptedChunkData,
                            vault: vault,
                        }),
                        { signal }
                    );
                }

                if (useUploadChunk.error) {
                    console.error(
                        `[Upload] Error on chunk #${chunkIndex}:`,
                        useUploadChunk.error
                    );
                    item.status = {
                        case: 'error',
                        error: useUploadChunk.error.message,
                    };
                    return;
                }

                chunkIndex++;
            }

            if (useUploadChunk.data?.result.case !== 'file') {
                item.status = {
                    case: 'error',
                    error: 'Server didnt return file',
                };

                return;
            }

            item.status = {
                case: 'completed',
                file: useUploadChunk.data?.result?.value!!,
            };
            console.log('[Upload] 14. Upload completed successfully');
        } catch (error: any) {
            console.error('[Upload] Caught error in catch block:', error);
            if (signal.aborted || error?.name === 'AbortError') {
                item.status = { case: 'canceled' };
                console.log('Upload was canceled by signal');
            } else {
                item.status = {
                    case: 'error',
                    error: error?.message || error || 'Unknown upload error',
                };
            }
        }
    }
}

export const uploadStore = new UploadStore();
