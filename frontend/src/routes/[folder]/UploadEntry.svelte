<script lang="ts">
    import { type UploadItem, uploadStore } from '$lib/stores/upload.svelte';
    import { formatBytes } from '$lib/utils';
    import { LoaderCircle, RotateCcw, X } from '@lucide/svelte';
    import { activeFolder } from '$lib/stores/folder.svelte';
    import type { FileView } from '$lib/grpc/gen/folder/v1/files_pb';

    let {
        upload = $bindable(),
        onComplete = $bindable(),
    }: {
        upload: UploadItem;
        onComplete: (file: FileView) => void;
    } = $props();

    function retryUpload() {
        if (activeFolder.ownedRef && activeFolder.key) {
            uploadStore.retry(
                upload.id!!,
                activeFolder.key,
                activeFolder.ownedRef
            );
        }
    }

    $effect(() => {
        if (upload.status.case === 'completed') {
            onComplete(upload.status.file);
            uploadStore.removeUpload(upload.id!!);
        }
    });
</script>

<div class="flex w-full gap-2">
    <div class="relative grid h-6 w-6 place-items-center">
        {#if upload.status.case === 'error'}
            <button class="cursor-pointer" onclick={retryUpload}>
                <RotateCcw class="col-start-1 row-start-1 h-full w-5" />
            </button>
        {:else}
            <LoaderCircle
                class="col-start-1 row-start-1 h-full w-full animate-spin"
            />
            <span class="col-start-1 row-start-1 text-xs">
                {#if upload.status.case === 'uploading'}
                    {Math.round(upload.status.progress)}
                {:else}
                    0
                {/if}
            </span>
        {/if}
    </div>

    <div class="min-w-0 flex-1">
        <div class="">
            {upload.file!!.name}
        </div>

        {#if upload.status.case === 'error'}
            <span class="text-destructive-foreground"
                >{upload.status.error}</span
            >
        {/if}
    </div>

    <div class="flex gap-2">
        <span class="text-muted-foreground text-sm">
            {formatBytes(upload.file!!.size)}
        </span>

        <div>
            <button
                class="flex cursor-pointer justify-center"
                onclick={() => uploadStore.removeUpload(upload.id!!)}
            >
                <X class="h-4 w-4" />
            </button>
        </div>
    </div>
</div>
