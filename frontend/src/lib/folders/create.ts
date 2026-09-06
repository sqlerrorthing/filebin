import {encryptBlob} from "$lib/crypt";
import {folderClient, useGrpc} from "$lib/grpc";
import {create} from "@bufbuild/protobuf";
import {FolderNameSchema, type OwnedFolder} from "$lib/grpc/gen/folder/v1/common_pb";

export async function createFolder(
    key: CryptoKey,
    name: string
): Promise<OwnedFolder> {
    const createFolder = useGrpc(folderClient.createFolder);

    const folder_name = create(FolderNameSchema, {
        value: await encryptBlob(key, new TextEncoder().encode(name))
    });

    await createFolder.call({
        name: folder_name,
    })

    if (createFolder.error) {
        throw createFolder.error
    }

    return createFolder.data!!
}
