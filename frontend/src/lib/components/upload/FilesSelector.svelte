<script lang="ts">
    import { SelectedBy } from "$lib/types/files";
    import { onMount } from "svelte";

    let {
        onSelect,
    }: {
        onSelect: (
            files: { file: File; path: string }[] | FileList | File[],
            by: SelectedBy
        ) => void | Promise<void>;
    } = $props();

    let fileInput = $state<HTMLInputElement | null>(null);
    let folderInput = $state<HTMLInputElement | null>(null);

    const getExtensionFromMime = (mime: string): string => {
        switch (mime) {
            case "image/png":
                return "png";
            case "image/jpeg":
                return "jpg";
            case "image/gif":
                return "gif";
            case "image/webp":
                return "webp";
            case "image/svg+xml":
                return "svg";
            case "video/mp4":
                return "mp4";
            case "video/webm":
                return "webm";
            case "video/ogg":
                return "ogv";
            case "text/plain":
                return "txt";
            default:
                return "bin";
        }
    };

    const isInput = (target: HTMLElement): boolean => {
        const isInput =
            target.tagName === "INPUT" || target.tagName === "TEXTAREA";
        const isEditable =
            target.hasAttribute("contenteditable") || target.isContentEditable;

        return isInput || isEditable;
    };

    const handlePaste = (event: ClipboardEvent) => {
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
            if (item.kind === "file") {
                const file = item.getAsFile();
                if (!file) continue;

                if (
                    file.type.startsWith("video/") ||
                    file.type.startsWith("image/")
                ) {
                    let fileName = file.name;
                    if (
                        !fileName ||
                        fileName === "image.png" ||
                        fileName === "blob"
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
                        file.name && file.name !== "blob"
                            ? file.name
                            : `clipboard.${ext}`;
                    const binFile = new File([file], fileName, {
                        type: file.type || "application/octet-stream",
                    });
                    files.push(binFile);
                }
            } else if (item.kind === "string" && item.type === "text/plain") {
                item.getAsString((text) => {
                    if (!text) return;
                    const textFile = new File([text], `clipboard.txt`, {
                        type: "text/plain",
                    });
                    onSelect([textFile], SelectedBy.PASTE);
                });
            }
        }

        if (files.length > 0) {
            onSelect(files, SelectedBy.PASTE);
        }
    };

    const onFileSelect = (event: Event) => {
        const input = event.target as HTMLInputElement;
        if (input.files) {
            const items = Array.from(input.files).map(f => ({
                file: f,
                path: (f as any).webkitRelativePath || f.name
            }));
            onSelect(items, SelectedBy.CHOOSE);
            input.value = "";
        }
    };

    const onFolderSelect = (event: Event) => {
        const input = event.target as HTMLInputElement;
        if (input.files) {
            const items = Array.from(input.files).map(f => ({
                file: f,
                path: (f as any).webkitRelativePath || f.name
            }));
            onSelect(items, SelectedBy.CHOOSE);
            input.value = "";
        }
    };

    const getAllFileEntries = async (dataTransferItemList: DataTransferItemList): Promise<{ file: File; path: string }[]> => {
        let files: { file: File; path: string }[] = [];
        
        const readEntry = async (entry: FileSystemEntry, path = ""): Promise<void> => {
            if (entry.isFile) {
                const fileEntry = entry as FileSystemFileEntry;
                await new Promise<void>((resolve) => {
                    fileEntry.file((file) => {
                        files.push({ file, path: path + file.name });
                        resolve();
                    });
                });
            } else if (entry.isDirectory) {
                const dirEntry = entry as FileSystemDirectoryEntry;
                const reader = dirEntry.createReader();
                const readBatch = async (): Promise<void> => {
                    return new Promise<void>((resolve) => {
                        reader.readEntries(async (entries) => {
                            if (entries.length === 0) {
                                resolve();
                                return;
                            }
                            for (const entry of entries) {
                                await readEntry(entry, path + dirEntry.name + "/");
                            }
                            await readBatch();
                            resolve();
                        });
                    });
                };
                await readBatch();
            }
        };

        for (let i = 0; i < dataTransferItemList.length; i++) {
            const item = dataTransferItemList[i];
            if (item.kind === "file") {
                const entry = (item as any).webkitGetAsEntry?.();
                if (entry) {
                    await readEntry(entry, "");
                } else {
                    const file = item.getAsFile();
                    if (file) {
                        files.push({ file, path: file.name });
                    }
                }
            }
        }

        return files;
    };

    const handleDrop = async (event: DragEvent) => {
        event.preventDefault();
        if (event.dataTransfer?.items) {
            const items = await getAllFileEntries(event.dataTransfer.items);
            if (items.length > 0) {
                onSelect(items, SelectedBy.DROP);
            }
        } else if (event.dataTransfer?.files) {
            const items = Array.from(event.dataTransfer.files).map(f => ({
                file: f,
                path: (f as any).webkitRelativePath || f.name
            }));
            onSelect(items, SelectedBy.DROP);
        }
    };

    export const selectFiles = () => {
        fileInput?.click();
    };

    export const selectFolder = () => {
        folderInput?.click();
    };

    export const select = () => {
        fileInput?.click();
    };

    onMount(() => {
        document.addEventListener("paste", handlePaste);

        return () => {
            document.removeEventListener("paste", handlePaste);
        };
    });
</script>

<svelte:window ondrop={handleDrop} />

<input
    type="file"
    multiple
    bind:this={fileInput}
    onchange={onFileSelect}
    class="hidden"
/>

<input
    type="file"
    webkitdirectory
    multiple
    bind:this={folderInput}
    onchange={onFolderSelect}
    class="hidden"
/>
