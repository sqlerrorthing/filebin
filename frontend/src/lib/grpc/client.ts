import { createClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { PUBLIC_BACKEND_URL } from '$env/static/public';
import {FolderService} from "$lib/grpc/gen/folder/v1/folder_pb";
import {FilesService} from "$lib/grpc/gen/folder/v1/files_pb";


const transport = createGrpcWebTransport({
    baseUrl: PUBLIC_BACKEND_URL,
});

export const folderClient = createClient(FolderService, transport);
export const fileClient = createClient(FilesService, transport);
