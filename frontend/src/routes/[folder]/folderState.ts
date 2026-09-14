import type { FolderId, FolderToken } from "$lib/grpc/gen/folder/v1/common_pb";

export interface FolderPageState {
    folder: FolderId;
    key: CryptoKey;
    token?: FolderToken;
    pendingFiles?: File[];
}
