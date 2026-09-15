import { decryptBlobAsString, encryptBlob } from "$lib/crypt/key";
import { folderClient } from "$lib/grpc";
import { create } from "@bufbuild/protobuf";
import {
    FolderNameSchema,
    OwnedFolderRefSchema,
    type Folder,
    type FolderId,
    type FolderName,
    type FolderToken,
} from "$lib/grpc/gen/folder/v1/common_pb";

export const folderService = {
    async decryptName(key: CryptoKey, folder: Folder): Promise<string> {
        if (!folder.name?.value) return "Folder";
        try {
            return await decryptBlobAsString(key, folder.name.value);
        } catch {
            return "Folder";
        }
    },

    async rename(
        key: CryptoKey,
        folderId: FolderId,
        token: FolderToken,
        newName: string
    ): Promise<FolderName> {
        const encrypted = await encryptBlob(key, new TextEncoder().encode(newName));
        const folderName = create(FolderNameSchema, { value: encrypted });

        await folderClient.rename({
            ownedFolder: create(OwnedFolderRefSchema, {
                folderId,
                token,
            }),
            name: folderName,
        });

        return folderName;
    },
};
