import type {Folder, FolderId, FileId, FolderToken} from "$lib/grpc/gen/folder/v1/common_pb";
import {OwnedFolderRefSchema, FileIdSchema} from "$lib/grpc/gen/folder/v1/common_pb";
import {decryptBlob, decryptBlobAsString, encryptBlob, bufferToBase64} from "$lib/crypt/key";
import {
    AlgorithmSchema,
    EncryptedBlobsSchema,
    EncryptedVaultSchema,
    VersionSchema
} from "$lib/grpc/gen/folder/v1/encryption_pb";
import type {FileItem} from "$lib/context/files.svelte";
import {fileClient} from "$lib/grpc";
import {create, toBinary, fromBinary} from "@bufbuild/protobuf";
import {UploadChunkRequestSchema, UploadChunkResponseSchema} from "$lib/grpc/gen/folder/v1/files_pb";
import {env} from "$env/dynamic/public";
import JSZip from "jszip";
import * as m from "$lib/paraglide/messages";

type DecryptedFileMetadata = {
    path: string;
    type: string;
};

export const filesService = {
    async list(folderId: FolderId, key: CryptoKey): Promise<Map<string, FileItem>> {
        const response = await fileClient.listFiles({
            folder: folderId,
        });

        const items = await Promise.all(
            response.files.map(async (view): Promise<FileItem> => {
                if (!view.id) {
                    throw new Error("FileView is missing id");
                }

                if (!view.metadata?.value) {
                    throw new Error(
                        `File ${view.id.value} is missing encrypted metadata`,
                    );
                }

                const decrypted = await decryptBlob(
                    key,
                    view.metadata.value,
                );

                const metadata = JSON.parse(
                    new TextDecoder().decode(decrypted),
                ) as DecryptedFileMetadata;

                return {
                    id: view.id,
                    path: metadata.path,
                    name: metadata.path.split("/").pop()!,
                    type: metadata.type,
                    size: view.size,
                };
            }),
        );

        return new Map(
            items.map((file) => [file.path, file]),
        );
    },

    async download(folderId: FolderId, fileId: FileId, key: CryptoKey): Promise<Uint8Array> {
        const stream = fileClient.download({
            folder: folderId,
            file: fileId,
        });

        let vault: any = null;
        const parts: Uint8Array[] = [];

        for await (const chunk of stream) {
            if (chunk.vault && !vault) {
                vault = chunk.vault;
            }
            if (chunk.blob?.part) {
                parts.push(chunk.blob.part);
            }
        }

        if (!vault) {
            throw new Error("Missing encryption vault in download stream");
        }

        const totalLength = parts.reduce((acc, p) => acc + p.length, 0);
        const ciphertext = new Uint8Array(totalLength);
        let offset = 0;
        for (const p of parts) {
            ciphertext.set(p, offset);
            offset += p.length;
        }

        const encryptedBlob = create(EncryptedBlobsSchema, {
            meta: vault,
            data: ciphertext,
        });

        return await decryptBlob(key, encryptedBlob);
    },

    async downloadZip(
        folderId: FolderId,
        key: CryptoKey,
        filesToZip: FileItem[],
        prefix: string,
        onProgress?: (progress: { current: number; total: number; status: string }) => void,
        signal?: AbortSignal
    ): Promise<Blob> {
        const zip = new JSZip();
        let current = 0;
        const total = filesToZip.length;

        onProgress?.({current, total, status: m["folder.download.preparing"]()});

        for (const file of filesToZip) {
            if (signal?.aborted) {
                throw new Error("Aborted");
            }

            onProgress?.({
                current: ++current,
                total,
                status: m["folder.download.downloading"]({
                    name: file.name,
                    current: current.toString(),
                    total: total.toString(),
                }),
            });

            const decrypted = await this.download(folderId, file.id, key);
            const relativePath = file.path.slice(prefix.length);
            zip.file(relativePath, new Uint8Array(decrypted));
        }

        if (signal?.aborted) {
            throw new Error("Aborted");
        }

        onProgress?.({current: total, total, status: m["folder.download.compressing"]()});

        const content = await zip.generateAsync(
            {
                type: "blob",
                compression: "DEFLATE",
                compressionOptions: {level: 6}
            },
            (metadata) => {
                if (signal?.aborted) return;
                const pct = Math.round(metadata.percent);
                onProgress?.({
                    current: total,
                    total,
                    status: m["folder.download.compressing_percent"]({
                        percent: pct.toString(),
                    }),
                });
            }
        );

        if (signal?.aborted) {
            throw new Error("Aborted");
        }

        return content;
    },

    async uploadFile(
        folderId: FolderId,
        token: FolderToken,
        key: CryptoKey,
        file: File,
        path: string,
        onProgress?: (progressInfo: {
            loaded: number;
            total: number;
            progress: number;
            speed: number;
            eta: number;
            currentChunk: number;
            totalChunks: number;
        }) => void,
        signal?: AbortSignal
    ): Promise<FileItem> {
        const metadataBytes = new TextEncoder().encode(
            JSON.stringify({
                path,
                type: file.type || "application/octet-stream",
            })
        );
        const encryptedMetadata = await encryptBlob(key, metadataBytes);

        const initResponse = await fileClient.initiateUpload({
            folder: create(OwnedFolderRefSchema, {
                folderId,
                token,
            }),
            metadata: {
                value: encryptedMetadata,
            },
        });

        const uploadId = initResponse.uploadId;
        const chunkSize = initResponse.chunkSize || 1024 * 1024 * 5;

        const totalSize = file.size;
        const cipherTotalEst = totalSize > 0 ? totalSize + 16 : 16;
        const totalWorkBytes = totalSize + cipherTotalEst;
        const totalChunks = Math.ceil(totalSize / chunkSize) || 1;

        let lastTime = Date.now();
        let lastLoaded = 0;
        let smoothedSpeed = 0;

        let bytesEncrypted = 0;
        let bytesUploaded = 0;

        const report = (loaded: number, currentChunk: number, totalC: number) => {
            const now = Date.now();
            const dt = (now - lastTime) / 1000;
            const bytesDelta = loaded - lastLoaded;

            if (dt >= 0.2 || loaded >= totalWorkBytes) {
                if (dt > 0) {
                    const instSpeed = bytesDelta / dt;
                    smoothedSpeed = smoothedSpeed === 0 ? instSpeed : (smoothedSpeed * 0.7 + instSpeed * 0.3);
                }
                lastTime = now;
                lastLoaded = loaded;
            }

            const progress = totalWorkBytes > 0 ? Math.min(100, Math.round((loaded / totalWorkBytes) * 100)) : 100;
            const remaining = totalWorkBytes - loaded;
            const eta = smoothedSpeed > 0 ? remaining / smoothedSpeed : 0;

            onProgress?.({
                loaded,
                total: totalSize,
                progress,
                speed: smoothedSpeed,
                eta,
                currentChunk,
                totalChunks: totalC,
            });
        };

        const iv = window.crypto.getRandomValues(new Uint8Array(12));
        let offset = 0;
        let fileView: any = null;

        const cipherParts: Uint8Array[] = [];
        let totalCipherLength = 0;

        let chunkIndex = 0;
        while (offset < totalSize) {
            if (signal?.aborted) throw new Error("Aborted");

            const slice = file.slice(offset, offset + chunkSize);
            const chunkBuffer = new Uint8Array(await slice.arrayBuffer());

            const encryptedChunk = await window.crypto.subtle.encrypt(
                {name: "AES-GCM", iv},
                key,
                chunkBuffer
            );

            const encBytes = new Uint8Array(encryptedChunk);
            cipherParts.push(encBytes);
            totalCipherLength += encBytes.length;

            offset += chunkSize;
            chunkIndex++;
            bytesEncrypted = offset > totalSize ? totalSize : offset;
            report(bytesEncrypted, chunkIndex, totalChunks);
        }

        if (totalSize === 0) {
            const encryptedChunk = await window.crypto.subtle.encrypt(
                {name: "AES-GCM", iv},
                key,
                new Uint8Array(0)
            );
            const encBytes = new Uint8Array(encryptedChunk);
            cipherParts.push(encBytes);
            totalCipherLength += encBytes.length;
            bytesEncrypted = 0;
            report(0, 1, 1);
        }

        const fullEncrypted = new Uint8Array(totalCipherLength);
        let pos = 0;
        for (const part of cipherParts) {
            fullEncrypted.set(part, pos);
            pos += part.length;
        }

        const ciphertext = fullEncrypted.slice(0, fullEncrypted.length - 16);
        const tag = fullEncrypted.slice(fullEncrypted.length - 16);

        const version = create(VersionSchema, {value: 1});
        const algo = create(AlgorithmSchema, {value: "aes-256-gcm"});
        const encryptedVault = create(EncryptedVaultSchema, {
            iv: bufferToBase64(iv),
            tag: bufferToBase64(tag),
            version,
            algo,
        });

        let cipherOffset = 0;
        const cipherTotal = ciphertext.length;
        let uploadChunkIndex = 0;
        const totalUploadChunks = Math.ceil(cipherTotal / chunkSize) || 1;

        while (cipherOffset < cipherTotal) {
            if (signal?.aborted) throw new Error("Aborted");

            const isLast = cipherOffset + chunkSize >= cipherTotal;
            const chunkData = isLast
                ? ciphertext.slice(cipherOffset)
                : ciphertext.slice(cipherOffset, cipherOffset + chunkSize);

            const vault = isLast ? encryptedVault : undefined;

            const res = await fileClient.uploadChunk({
                    uploadId,
                    vault,
                    chunkData,
                }, {
                    signal
                }
            );

            if (res.result?.case === "file") {
                fileView = res.result.value;
            }

            cipherOffset += chunkData.length;
            uploadChunkIndex++;
            bytesUploaded = cipherOffset;
            const totalLoaded = Math.min(totalWorkBytes, totalSize + bytesUploaded);
            report(totalLoaded, uploadChunkIndex, totalUploadChunks);
        }

        if (cipherTotal === 0) {
            const res = await fileClient.uploadChunk({
                uploadId,
                vault: encryptedVault,
                chunkData: new Uint8Array(0),
            }, { signal });

            if (res.result?.case === "file") {
                fileView = res.result.value;
            }
            report(totalWorkBytes, 1, 1);
        }

        if (!fileView || !fileView.id) {
            throw new Error("Upload completed but FileView was not returned");
        }

        return {
            id: fileView.id,
            path,
            name: path.split("/").pop()!,
            type: file.type || "application/octet-stream",
            size: BigInt(file.size),
        };
    },

    async delete(folderId: FolderId, token: FolderToken, fileId: FileId): Promise<void> {
        await fileClient.delete({
            folder: create(OwnedFolderRefSchema, {
                folderId,
                token,
            }),
            fileId,
        });
    },
}
