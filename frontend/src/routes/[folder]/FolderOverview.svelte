<script lang="ts">
    import Container from "$lib/components/container/Container.svelte";
    import type { EncryptedFolderContext } from "$lib/context/folder.svelte";
    import { FolderContext, setFolderContext } from "$lib/context/folder.svelte";
    import { FilesContext, setFilesContext } from "$lib/context/files.svelte";
    import { UploadContext, setUploadContext } from "$lib/context/upload.svelte";
    import FolderHeader from "./FolderHeader.svelte";
    import Breadcrumbs from "./Breadcrumbs.svelte";
    import FileTable from "./FileTable.svelte";
    import { onMount } from "svelte";

    let { ctx }: { ctx: EncryptedFolderContext } = $props();

    const folderCtx = new FolderContext(
        ctx.folder,
        ctx.key,
        ctx.token ?? undefined,
        ctx.pendingFiles
    );
    setFolderContext(folderCtx);

    const filesCtx = new FilesContext(folderCtx);
    setFilesContext(filesCtx);

    const uploadCtx = new UploadContext(folderCtx, filesCtx);
    setUploadContext(uploadCtx);

    onMount(async () => {
        await filesCtx.loadFiles();
        if (folderCtx.pendingFiles && folderCtx.pendingFiles.length > 0) {
            const pending = [...folderCtx.pendingFiles];
            folderCtx.pendingFiles = [];
            await uploadCtx.uploadFiles(pending);
        }
    });
</script>

<Container>
    <div class="flex flex-col gap-6">
        <FolderHeader />
        <Breadcrumbs />
        <FileTable />
    </div>
</Container>
