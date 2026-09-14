<script lang="ts">
    import { page } from "$app/state";
    import { setEncryptedFolderContext } from "$lib/context/folder.svelte";
    import type { FolderPageState } from "./folderState.ts";

    const state = page.state as FolderPageState;

    const eFolderCtx = setEncryptedFolderContext({
        folder: state.folder,
        key: state.key,
        token: state.token,
        pendingFiles: state.pendingFiles ?? [],
    });
</script>

<p>
    {JSON.stringify(eFolderCtx)}
</p>
