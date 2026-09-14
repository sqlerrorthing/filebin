import {setContext, getContext} from 'svelte';
import type {FolderId, FolderToken} from "$lib/grpc/gen/folder/v1/common_pb";

export class EncryptedFolderContext {
    folder: FolderId = $state()!;
    key: CryptoKey = $state()!;
    token = $state<FolderToken | null>(null);
    pendingFiles: File[] = $state([]);

    constructor(folder: FolderId, key: CryptoKey, token?: FolderToken, pendingFiles: File[] = []) {
        this.folder = folder;
        this.key = key;
        this.token = token ?? null;
        this.pendingFiles = pendingFiles;
    }
}

const KEY = Symbol('ECNRYPTED_FOLDER_CONTEXT');

export const setEncryptedFolderContext = (initial: {
    folder: FolderId;
    key: CryptoKey;
    token?: FolderToken,
    pendingFiles?: File[]
}) => {
    return setContext(KEY, new EncryptedFolderContext(initial.folder, initial.key, initial.token, initial.pendingFiles));
}

export const useFolderContext = () => {
    return getContext<EncryptedFolderContext>(KEY);
}