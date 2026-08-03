import type {Folder, FolderToken} from "$lib/grpc/gen/folder/v1/common_pb";
import {decryptBlobAsString} from "$lib/crypt";

export interface DecryptedFolder {
    name: string;
}

class ActiveFolderStore {
    folder = $state<Folder | null>(null);
    key = $state<CryptoKey | null>(null);
    token = $state<FolderToken | null>(null);

    decrypted = $state<DecryptedFolder | null>(null);

    isDecrypting = $state(false);
    error = $state<string | null>(null);

    private async decryptCurrentFolder() {
        if (!this.folder || !this.key) return;

        this.isDecrypting = true;
        this.error = null;

        try {
            const name = await decryptBlobAsString(this.key, this.folder?.name?.value!!);


            this.decrypted = { name };
        } catch (e) {
            console.error("Failed to decrypt folder:", e);
            this.error = "Ошибка расшифровки";
        } finally {
            this.isDecrypting = false;
        }
    }

    set(folderId: Folder, cryptoKey: CryptoKey, folderToken: FolderToken | null) {
        this.folder = folderId;
        this.key = cryptoKey;
        this.token = folderToken;

        this.decryptCurrentFolder();
    }

    clear() {
        this.folder = null;
        this.key = null;
        this.token = null;
        this.decrypted = null;
        this.error = null;
    }
}

export const activeFolder = new ActiveFolderStore();
