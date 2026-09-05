<script lang="ts">
    import {type UploadItem, uploadStore} from "$lib/stores/upload.svelte";
    import {formatBytes} from "$lib/utils";
    import {LoaderCircle, RotateCcw, X} from "@lucide/svelte";
    import {activeFolder} from "$lib/stores/folder.svelte";
    import type {FileView} from "$lib/grpc/gen/folder/v1/files_pb";

    let {
        upload = $bindable(),
        onComplete = $bindable()
    }: {
        upload: UploadItem,
        onComplete: (file: FileView) => void
    } = $props();

    function retryUpload() {
        if (activeFolder.ownedRef && activeFolder.key) {
            uploadStore.retry(upload.id!!, activeFolder.key, activeFolder.ownedRef)
        }
    }

    $effect(() => {
        if (upload.status.case === "completed") {
            onComplete(upload.status.file);
            uploadStore.removeUpload(upload.id!!)
        }
    })
</script>

<div class="flex gap-2 w-full">
    <div class="w-6 h-6 relative grid place-items-center">
        {#if upload.status.case === "error"}
            <button class="cursor-pointer" onclick={retryUpload}>
                <RotateCcw class="w-5 h-full col-start-1 row-start-1" />
            </button>
        {:else}
            <LoaderCircle class="w-full h-full animate-spin col-start-1 row-start-1"/>
            <span class="col-start-1 row-start-1 text-xs">
                {#if upload.status.case === "uploading"}
                    {Math.round(upload.status.progress)}
                {:else}
                    0
                {/if}
            </span>
        {/if}
    </div>

    <div class="flex-1 min-w-0">
        <div class="">
            {upload.file!!.name}
        </div>

        {#if upload.status.case === "error"}
            <span class="text-destructive-foreground">{upload.status.error}</span>
        {/if}
    </div>

    <div class="flex gap-2">
        <span class="text-muted-foreground text-sm">
            {formatBytes(upload.file!!.size)}
        </span>

        <div>
            <button class="cursor-pointer flex justify-center" onclick={() => uploadStore.removeUpload(upload.id!!)}>
                <X class="w-4 h-4" />
            </button>
        </div>
    </div>
</div>