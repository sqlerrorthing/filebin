import { setContext, getContext } from "svelte";
import type { Folder, FolderToken } from "$lib/grpc/gen/folder/v1/common_pb";
import { folderService } from "$lib/services/folder.service";

export class FolderContext {
    folder: Folder = $state()!;
    key: CryptoKey = $state()!;
    token = $state<FolderToken | null>(null);
    pendingFiles: File[] = $state([]);
    folderName = $state("");

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
        this.initName();
    }

    private async initName() {
        this.folderName = await folderService.decryptName(this.key, this.folder);
    }

    async rename(newName: string) {
        if (!this.token || !this.folder?.id) return;
        const newFolderName = await folderService.rename(
            this.key,
            this.folder.id,
            this.token,
            newName
        );
        this.folder.name = newFolderName;
        this.folderName = newName;
    }
}

export type EncryptedFolderContext = FolderContext;

const FOLDER_KEY = Symbol("FOLDER_KEY");

export const setFolderContext = (ctx: FolderContext) => setContext(FOLDER_KEY, ctx);
export const getFolderContext = () => getContext<FolderContext>(FOLDER_KEY);

export const setEncryptedFolderContext = setFolderContext;
export const getEncryptedFolderContext = getFolderContext;
