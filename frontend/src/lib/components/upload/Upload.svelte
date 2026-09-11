<script lang="ts">
    import { FileUp, LoaderCircle } from '@lucide/svelte';
    import * as m from '$lib/paraglide/messages';
    import { exportKey, generateCryptoKey } from '$lib/crypt';
    import { createFolder } from '$lib/folders/create';
    import { cn } from '$lib/utils';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { activeFolder } from '$lib/stores/folder.svelte.js';
    import { localizeHref } from '$lib/paraglide/runtime';
    import { uploadStore } from '$lib/stores/upload.svelte.js';
    import { portal } from '$lib/actions/portal';

    let loading = $state(false);
    let error = $state<string | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);
    let isDragging = $state(false);

    async function handleUpload(files: FileList | File[]) {
        if (!files || files.length === 0) return;

        loading = true;
        error = null;

        try {
            if (activeFolder.ownedRef && activeFolder.key) {
                uploadStore.addFiles(
                    files,
                    activeFolder.key,
                    activeFolder.ownedRef
                );
            } else {
                const key = await generateCryptoKey();
                const folder = await createFolder(
                    key,
                    m['folders.base-name']()
                );

                await activeFolder.set(folder.folder!!, key, folder.token!!);

                uploadStore.addFiles(files, key, activeFolder.ownedRef!!);

                const exportedKey = await exportKey(key);
                await goto(
                    `${localizeHref(folder?.folder?.id?.value!!)}#${exportedKey}`
                );
            }
        } catch (e: any) {
            error = e.toString();
        } finally {
            loading = false;
        }
    }

    function getExtensionFromMime(mime: string): string {
        switch (mime) {
            case 'image/png':
                return 'png';
            case 'image/jpeg':
                return 'jpg';
            case 'image/gif':
                return 'gif';
            case 'image/webp':
                return 'webp';
            case 'image/svg+xml':
                return 'svg';
            case 'video/mp4':
                return 'mp4';
            case 'video/webm':
                return 'webm';
            case 'video/ogg':
                return 'ogv';
            case 'text/plain':
                return 'txt';
            default:
                return 'bin';
        }
    }

    function isInput(target: HTMLElement): boolean {
        const isInput =
            target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';
        const isEditable =
            target.hasAttribute('contenteditable') || target.isContentEditable;

        return isInput || isEditable;
    }

    function handlePaste(event: ClipboardEvent) {
        const target = event.target as HTMLElement | null;

        if (target && isInput(target)) {
            return;
        }

        const activeEl = document.activeElement as HTMLElement | null;
        if (activeEl && activeEl !== document.body && isInput(activeEl)) {
            return;
        }

        const items = event.clipboardData?.items;
        if (!items) return;

        const files: File[] = [];

        for (const item of items) {
            if (item.kind === 'file') {
                const file = item.getAsFile();
                if (!file) continue;

                if (
                    file.type.startsWith('video/') ||
                    file.type.startsWith('image/')
                ) {
                    let fileName = file.name;
                    if (
                        !fileName ||
                        fileName === 'image.png' ||
                        fileName === 'blob'
                    ) {
                        const ext = getExtensionFromMime(file.type);
                        fileName = `clipboard.${ext}`;
                    }

                    const validFile =
                        fileName !== file.name
                            ? new File([file], fileName, { type: file.type })
                            : file;

                    files.push(validFile);
                } else {
                    const ext = getExtensionFromMime(file.type);
                    const fileName =
                        file.name && file.name !== 'blob'
                            ? file.name
                            : `clipboard.${ext}`;
                    const binFile = new File([file], fileName, {
                        type: file.type || 'application/octet-stream',
                    });
                    files.push(binFile);
                }
            } else if (item.kind === 'string' && item.type === 'text/plain') {
                item.getAsString((text) => {
                    if (!text) return;
                    const textFile = new File([text], `clipboard.txt`, {
                        type: 'text/plain',
                    });
                    handleUpload([textFile]);
                });
            }
        }

        if (files.length > 0) {
            handleUpload(files);
        }
    }

    function onFileSelect(event: Event) {
        const input = event.target as HTMLInputElement;
        if (input.files) {
            handleUpload(input.files);
        }
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        isDragging = false;
        if (event.dataTransfer?.files) {
            handleUpload(event.dataTransfer.files);
        }
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault();
        isDragging = true;
    }

    function handleDragLeave() {
        isDragging = false;
    }

    onMount(() => {
        loading = false;
        error = null;

        document.addEventListener('paste', handlePaste);
        return () => {
            document.removeEventListener('paste', handlePaste);
        };
    });
</script>

<svelte:window
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
/>

<input
    type="file"
    multiple
    bind:this={fileInput}
    onchange={onFileSelect}
    class="hidden"
/>

{#if error}
    <p class="text-destructive text-center">{error}</p>
{/if}

<div class="relative">
    {#if isDragging}
        <div
            use:portal
            class="bg-primary/10 border-primary pointer-events-none fixed inset-0 z-50 flex items-center justify-center border-2 border-dashed"
        >
            <p
                class="bg-background text-primary rounded px-4 py-2 font-medium shadow"
            >
                {m['files.drop-here']()}
            </p>
        </div>
    {/if}

    <button
        class={cn("bg-primary text-primary-foreground hover:bg-accent-foreground flex cursor-pointer justify-center gap-3 px-4 py-2 shadow-sm sm:w-48",
            loading && "bg-muted-foreground"
        )}
        onclick={() => fileInput?.click()}
        disabled={loading}
    >
        {#if loading}
            <LoaderCircle class="animate-spin" />
        {:else}
            <FileUp />
        {/if}
        {m['files.upload']()}
    </button>
</div>
