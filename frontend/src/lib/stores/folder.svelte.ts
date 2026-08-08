import  {type Folder, type FolderId, type FolderToken, FolderTokenSchema, OwnedFolderRefSchema} from "$lib/grpc/gen/folder/v1/common_pb";
import {decryptBlobAsString} from "$lib/crypt";
import * as m from "$lib/paraglide/messages";
import {create} from "@bufbuild/protobuf";
import type {DateTime} from "$lib/grpc/gen/google/type/datetime_pb";

export interface DecryptedFolder {
    name: string;
    createdAt: DateTime,
    expiredAt: DateTime
}

class ActiveFolderStore {
    id = $state<FolderId | null>(null);
    key = $state<CryptoKey | null>(null);
    token = $state<FolderToken | null>(null);

    decrypted = $state<DecryptedFolder | null>(null);
    isDecrypting = $state(false);

    error = $state<string | null>(null);

    private async decryptFolder(key: CryptoKey, folder: Folder) {
        this.isDecrypting = true;
        this.error = null;

        try {
            const name = await decryptBlobAsString(key, folder?.name?.value!!);

            this.decrypted = {
                name,
                expiredAt: folder.expiredAt!!,
                createdAt: folder.createdAt!!
            };
        } catch (e) {
            console.error("Failed to decrypt folder:", e);
            this.error = m["crypt.errors.decrypt"]();
        } finally {
            this.isDecrypting = false;
        }
    }

    private isTokenValid(token: FolderToken): boolean {
        try {
            const base64Url = token.value.split('.')[1];
            if (!base64Url) return false;

            const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
            const jsonPayload = decodeURIComponent(
                atob(base64)
                    .split('')
                    .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
                    .join('')
            );

            const payload = JSON.parse(jsonPayload);

            if (payload && typeof payload.exp === 'number') {
                const currentTime = Math.floor(Date.now() / 1000);
                return payload.exp > currentTime;
            }

            return false;
        } catch (error) {
            console.error('Failed to parse JWT token:', error);
            return false;
        }
    }

    async set(folder: Folder, cryptoKey: CryptoKey, folderToken: FolderToken | null | undefined) {
        const tokenItem = `${folder.id?.value}_token`;

        this.id = folder.id!!;
        this.key = cryptoKey;

        if (folderToken === null) {
            this.token = null;
            localStorage.removeItem(tokenItem)
        } else if (folderToken !== undefined) {
            if (this.isTokenValid(folderToken)) {
                this.token = folderToken;
                localStorage.setItem(tokenItem, folderToken.value)
            }
        } else {
            if (!this.token) {
                const rawSaved = localStorage.getItem(tokenItem);
                const savedToken = rawSaved !== null
                    ? create(FolderTokenSchema, { value: rawSaved }) : null;

                if (savedToken && this.isTokenValid(savedToken)) {
                    this.token = savedToken;
                } else if (rawSaved != null) {
                    localStorage.removeItem(rawSaved)
                }
            }
        }

        await this.decryptFolder(cryptoKey, folder);
    }

    get ownedRef() {
        if (this.token && this.id) {
            return create(OwnedFolderRefSchema, {
                folderId: this.id,
                token: this.token
            })
        } else {
            return null
        }
    }

    clear() {
        this.id = null;
        this.key = null;
        this.token = null;
        this.decrypted = null;
        this.error = null;
    }
}

export const activeFolder = new ActiveFolderStore();
