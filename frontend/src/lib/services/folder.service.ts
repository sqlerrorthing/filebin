import type {Folder} from "$lib/grpc/gen/folder/v1/common_pb";
import {decryptBlobAsString} from "$lib/crypt/key";

export const folderService = {
    async decryptName(key: CryptoKey, folder: Folder): Promise<string | null> {
        if (!folder.name?.value) return null;
        return await decryptBlobAsString(key, folder.name.value);
    },
}
