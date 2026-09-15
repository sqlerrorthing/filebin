import { decryptBlobAsString, encryptBlob } from "$lib/crypt/key";
import { fileClient } from "$lib/grpc";
import { create } from "@bufbuild/protobuf";
import {
    OwnedFolderRefSchema,
    type FolderId,
    type FolderToken,
    type FileId,
} from "$lib/grpc/gen/folder/v1/common_pb";
import { FileMetadataSchema, type FileView } from "$lib/grpc/gen/folder/v1/files_pb";

export interface FileItem {
    id: FileId;
    path: string;
    size: number | bigint;
    view: FileView;
}

export const fileService = {
    async list(key: CryptoKey, folderId: FolderId): Promise<FileItem[]> {
        const res = await fileClient.listFiles({ folder: folderId });
        const decryptedFiles = await Promise.all(
            res.files.map(async (file) => {
                if (!file.id || !file.metadata?.value) return null;
                try {
                    const metaStr = await decryptBlobAsString(key, file.metadata.value);
                    let filePath = metaStr;
                    try {
                        const parsed = JSON.parse(metaStr);
                        if (parsed && typeof parsed.path === "string") {
                            filePath = parsed.path;
                        }
                    } catch {}
                    return {
                        id: file.id,
                        path: filePath,
                        size: file.size,
                        view: file,
                    };
                } catch {
                    return null;
                }
            })
        );
        return decryptedFiles.filter((f): f is NonNullable<typeof f> => f !== null);
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

    async download(folderId: FolderId, fileId: FileId, fileName: string): Promise<void> {
        const stream = fileClient.download({
            folder: folderId,
            file: fileId,
        });

        const chunks: Uint8Array[] = [];
        for await (const chunk of stream) {
            if (chunk.blob?.part) {
                chunks.push(chunk.blob.part);
            }
        }

        const totalLength = chunks.reduce((acc, c) => acc + c.length, 0);
        const combined = new Uint8Array(totalLength);
        let offset = 0;
        for (const c of chunks) {
            combined.set(c, offset);
            offset += c.length;
        }

        const blob = new Blob([combined]);
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = fileName;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    },

    async upload(
        folderId: FolderId,
        token: FolderToken,
        key: CryptoKey,
        file: File,
        onProgress: (progress: number, speed: number, eta: number) => void,
        signal: AbortSignal
    ): Promise<FileView> {
        const metaObj = {
            path: file.name,
            type: file.type || "application/octet-stream",
        };
        const metadataEncrypted = await encryptBlob(
            key,
            new TextEncoder().encode(JSON.stringify(metaObj))
        );

        if (signal.aborted) throw new Error("Cancelled");

        const initRes = await fileClient.initiateUpload(
            {
                metadata: create(FileMetadataSchema, {
                    value: metadataEncrypted,
                }),
                folder: create(OwnedFolderRefSchema, {
                    folderId,
                    token,
                }),
            },
            { signal }
        );

        const uploadId = initRes.uploadId;
        const chunkSize = initRes.chunkSize;
        const fileSize = file.size;

        let bytesUploaded = 0;
        const startTime = Date.now();
        let lastTime = startTime;
        let lastBytes = 0;

        if (fileSize === 0) {
            if (signal.aborted) throw new Error("Cancelled");
            const emptyEncrypted = await encryptBlob(key, new Uint8Array(0));
            const res = await fileClient.uploadChunk(
                {
                    uploadId,
                    chunkData: emptyEncrypted.data,
                    vault: emptyEncrypted.meta,
                },
                { signal }
            );
            if (res.result.case === "file" && res.result.value) {
                return res.result.value;
            }
            throw new Error("Upload completion failed");
        }

        let offset = 0;
        let lastRes: FileView | null = null;

        while (offset < fileSize) {
            if (signal.aborted) throw new Error("Cancelled");

            const chunkBlob = file.slice(offset, offset + chunkSize);
            const arrayBuffer = await chunkBlob.arrayBuffer();
            const chunkBytes = new Uint8Array(arrayBuffer);

            const isLast = offset + chunkSize >= fileSize;
            const encryptedChunk = await encryptBlob(key, chunkBytes);

            let res;
            if (!isLast) {
                if (chunkBytes.length < chunkSize) {
                    res = await fileClient.uploadChunk(
                        {
                            uploadId,
                            chunkData: encryptedChunk.data,
                            vault: encryptedChunk.meta,
                        },
                        { signal }
                    );
                    bytesUploaded = fileSize;
                    break;
                }
                res = await fileClient.uploadChunk(
                    {
                        uploadId,
                        chunkData: encryptedChunk.data,
                        vault: undefined,
                    },
                    { signal }
                );
            } else {
                res = await fileClient.uploadChunk(
                    {
                        uploadId,
                        chunkData: encryptedChunk.data,
                        vault: encryptedChunk.meta,
                    },
                    { signal }
                );
            }

            if (res.result.case === "file" && res.result.value) {
                lastRes = res.result.value;
            }

            offset += chunkSize;
            bytesUploaded = Math.min(offset, fileSize);

            const now = Date.now();
            const durationSec = (now - lastTime) / 1000;
            let speed = 0;
            if (durationSec > 0.2) {
                speed = (bytesUploaded - lastBytes) / durationSec;
                lastTime = now;
                lastBytes = bytesUploaded;
            } else {
                const totalSec = (now - startTime) / 1000;
                speed = totalSec > 0 ? bytesUploaded / totalSec : 0;
            }

            const progress = Math.round((bytesUploaded / fileSize) * 100);
            const remainingBytes = fileSize - bytesUploaded;
            const eta = speed > 0 ? Math.ceil(remainingBytes / speed) : 0;

            onProgress(progress, speed, eta);
        }

        if (lastRes) {
            return lastRes;
        }

        throw new Error("Upload completed without file view result");
    },
};
