import { createClient } from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { env } from '$env/dynamic/public';
import { FolderService } from '$lib/grpc/gen/folder/v1/folder_pb';
import { FilesService } from '$lib/grpc/gen/folder/v1/files_pb';
import { ShareService } from '$lib/grpc/gen/folder/v1/share_pb';

const transport = createGrpcWebTransport({
    baseUrl: env.PUBLIC_BACKEND_URL,
});

export const folderClient = createClient(FolderService, transport);
export const fileClient = createClient(FilesService, transport);
export const shareClient = createClient(ShareService, transport);
