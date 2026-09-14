import {setContext, getContext} from "svelte";
import type {Folder, FolderId, FolderToken} from "$lib/grpc/gen/folder/v1/common_pb";

export class EncryptedFolderContext {
    folder: Folder = $state()!;
    key: CryptoKey = $state()!;
    token = $state<FolderToken | null>(null);
    pendingFiles: File[] = $state([]);

    constructor(
        folder: Folder,
        key: CryptoKey,
        token?: FolderToken,
        pendingFiles: File[] = []
    ) {
        this.folder = folder;
        this.key = key;
        this.token = token ?? null;
        this.pendingFiles = pendingFiles;
    }
}

const EFC = Symbol("EFC");

export const setEncryptedFolderContext = (ctx: EncryptedFolderContext) => {
    return setContext(
        EFC,
        ctx
    );
};

export const useFolderContext = () => {
    return getContext<EncryptedFolderContext>(EFC);
};

export class FolderContext {
    #encryptedCtx: EncryptedFolderContext;

    constructor(encryptedCtx: EncryptedFolderContext) {
        this.#encryptedCtx = encryptedCtx
    }


}