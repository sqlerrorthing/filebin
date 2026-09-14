<script lang="ts">
    import { page } from "$app/state";
    import { setEncryptedFolderContext, EncryptedFolderContext } from "$lib/context/folder.svelte";
    import { create } from "@bufbuild/protobuf";
    import { FolderTokenSchema } from "$lib/grpc/gen/folder/v1/common_pb";
    import { importKeyFromUrlSafe } from "$lib/crypt/key";
    import FolderOverview from "./FolderOverview.svelte";
    import {onMount} from "svelte";

    const pageState = page.state as any;
    const folderId = page.params.folder;

    let initialFolder = pageState?.folder;
    let initialToken = pageState?.token;

    const ctx = new EncryptedFolderContext(
        initialFolder,
        pageState?.key,
        initialToken,
        pageState?.pendingFiles ?? []
    );

    setEncryptedFolderContext(ctx);

    let isLoading = $state(true);
    let error = $state<string | null>(null);

    onMount(async () => {
        try {
            if (initialToken) {
                localStorage.setItem(`folder_token_${initialFolder.id.value}`, initialToken.id.value);
            } else {
                const storedTokenVal = localStorage.getItem(`folder_token_${initialFolder.id.value}`);
                if (storedTokenVal) {
                    ctx.token = create(FolderTokenSchema, { value: storedTokenVal });
                }
            }

            if (!ctx.key) {
                const hash = window.location.hash;
                if (hash && hash.startsWith("#")) {
                    const keyStr = hash.slice(1);
                    ctx.key = await importKeyFromUrlSafe(keyStr);
                }
            }

            if (!ctx.key) {
                error = "Encryption key is missing";
                return;
            }
        } finally {
            isLoading = false;
        }
    })
</script>

{#if error}
    <div>
        <p>Init error: {error}</p>
    </div>
{:else if isLoading}
    <div>
        <p>Loading...</p>
    </div>
{:else}
    <FolderOverview />
{/if}