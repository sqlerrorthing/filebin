import type {Folder, FolderId} from "$lib/grpc/gen/folder/v1/common_pb";
import {decryptBlob, decryptBlobAsString} from "$lib/crypt/key";
import type {FileItem} from "$lib/context/files.svelte";
import {fileClient} from "$lib/grpc";

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
}
