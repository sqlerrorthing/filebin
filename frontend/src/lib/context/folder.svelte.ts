import {setContext} from "svelte";
import type {
    Folder,
    FolderToken,
} from "$lib/grpc/gen/folder/v1/common_pb";
import {folderService} from "$lib/services/folder.service";

export class EncryptedFolderContext {
    folder: Folder = $state()!;
    key: CryptoKey = $state()!;
    token = $state<FolderToken | null>(null);
    decrypted: {
        state: "loading"
    } | {
        state: "error",
        error: string
    } | {
        state: "decrypted",
        ctx: DecryptedFolderContext
    } = $state({state: "loading"});

    constructor(
        folder: Folder,
        key: CryptoKey,
        token?: FolderToken,
    ) {
        this.folder = folder;
        this.key = key;
        this.token = token ?? null;

        void this.startDecryption();
    }

    private showDecryptionError(error: string) {
        this.decrypted = {
            state: "error",
            error
        }
    }

    async startDecryption() {
        this.decrypted = {state: "loading"};
        try {
            let name = await folderService.decryptName(this.key, this.folder);

            if (!name) {
                return this.showDecryptionError("Invalid folder name");
            }

            this.decrypted = {
                state: "decrypted",
                ctx: new DecryptedFolderContext(
                    name
                )
            }
        } catch (e: any) {
            return this.showDecryptionError(e.toString())
        }
    }
}

export class DecryptedFolderContext {
    name: string = $state()!;

    constructor(name: string) {
        this.name = name;
    }
}

const EFC = Symbol("EFC");

export const setEncryptedFolderContext = (ctx: EncryptedFolderContext) => {
    return setContext(EFC, ctx);
};
