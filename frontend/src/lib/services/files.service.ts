import type {Folder, FolderId, FileId, FolderToken} from "$lib/grpc/gen/folder/v1/common_pb";
import {OwnedFolderRefSchema} from "$lib/grpc/gen/folder/v1/common_pb";
import {decryptBlob, decryptBlobAsString} from "$lib/crypt/key";
import type {FileItem} from "$lib/context/files.svelte";
import {fileClient} from "$lib/grpc";
import {create} from "@bufbuild/protobuf";
import {EncryptedBlobsSchema} from "$lib/grpc/gen/folder/v1/encryption_pb";

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
