<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import MoveUpRight from "@lucide/svelte/icons/move-up-right";
    import ActionButton from "./ActionButton.svelte";
    import FilesSelector from "$lib/components/upload/FilesSelector.svelte";
    import { SelectedBy } from "$lib/types/files";
    import { cn } from "$lib/utils";
    import { LoaderCircle } from "@lucide/svelte";
    import { encryptBlob, exportKey, generateCryptoKey } from "$lib/crypt/key";
    import { folderClient } from "$lib/grpc";
    import { create } from "@bufbuild/protobuf";
    import { FolderNameSchema } from "$lib/grpc/gen/folder/v1/common_pb";
    import { goto } from "$app/navigation";
    import { localizeHref } from "$lib/paraglide/runtime";

    let isDragging = $state(false);
    let selector = $state<FilesSelector | null>(null);
    let loading = $state(false);

    const createFolderAndQueueFiles = async (
        files: FileList | File[],
        by: SelectedBy
    ) => {
        if (by === SelectedBy.DROP) {
            isDragging = false;
        }

        if (!files || files.length === 0) {
            return;
        }

        loading = true;

        try {
            const key = await generateCryptoKey();
            const createFolder = await folderClient.createFolder({
                name: create(FolderNameSchema, {
                    value: await encryptBlob(
                        key,
                        new TextEncoder().encode(m["folder.default_name"]())
                    ),
                }),
            });

            const folder = createFolder.folder;
            const exportedKey = await exportKey(key);

            if (folder) {
                await goto(localizeHref(`${folder.id!!.value}#${exportedKey}`), {
                    state: {
                        folder: folder,
                        key: key,
                        token: createFolder.token,
                        pendingFiles: Array.from(files),
                    },
                });
            }
        } catch (e: any) {
            console.log(e);
            // todo: toast
        } finally {
            loading = false;
        }
    };

    const handleDragOver = (event: DragEvent) => {
        event.preventDefault();
        isDragging = true;
    };

    const handleDragLeave = () => {
        isDragging = false;
    };
</script>

<svelte:window ondragover={handleDragOver} ondragleave={handleDragLeave} />

<FilesSelector bind:this={selector} onSelect={createFolderAndQueueFiles} />

<ActionButton
    displayName={m["index.actions.upload.name"]}
    description={isDragging
        ? m["index.actions.upload.release-to-upload"]
        : m["index.actions.upload.description"]}
    highlight={true}
    class={cn(
        "relative",
        isDragging &&
            `before:border-accent before:pointer-events-none before:absolute
            before:inset-1 before:rounded-lg before:border-4
            before:border-dashed`
    )}
    onclick={() => selector?.select()}
    disabled={loading}
>
    {#snippet icon()}
        {#if loading}
            <LoaderCircle class="animate-spin" />
        {:else}
            <MoveUpRight class="h-full w-full" />
        {/if}
    {/snippet}
</ActionButton>
