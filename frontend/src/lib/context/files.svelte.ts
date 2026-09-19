import type {FileId} from "$lib/grpc/gen/folder/v1/common_pb";
import type {FolderContext} from "$lib/context/folder.svelte";
import {getContext, setContext} from "svelte";
import {filesService} from "$lib/services/files.service";
import JSZip from "jszip";

export type FilesState =
    | { case: "loading" }
    | { case: "error"; message: string }
    | { case: "loaded"; files: Map<string, FileItem> }

export type FileItem = {
    id: FileId,
    name: string,
    path: string,
    type: string,
    size: bigint
}

export class FilesContext {
    private state = $state<FilesState>({case: "loading"})
    private currentPath = $state<string[]>([]);

    private readonly deleting = $state(new Set<FileId>());
    private readonly downloading = $state(new Set<FileId>());

    downloadingZip = $state(false);
    zipProgress = $state<{ current: number; total: number; status: string } | null>(null);
    private zipAbortController: AbortController | null = null;

    constructor(private readonly folderCtx: () => FolderContext) {
        void this.loadFiles()
    }

    get path(): string[] {
        return this.currentPath;
    }

    get currentItems() {
        if (this.state.case !== "loaded") {
            return {
                folders: [],
                files: [],
            };
        }

        const prefix = this.currentPath.length
            ? this.currentPath.join("/") + "/"
            : "";

        const folders = new Map<string, bigint>();
        const files: FileItem[] = [];

        for (const file of this.state.files.values()) {
            if (!file.path.startsWith(prefix)) {
                continue;
            }

            const rest = file.path.slice(prefix.length);
            const slash = rest.indexOf("/");

            const folderName = rest.slice(0, slash);

            if (slash === -1) {
                files.push(file);
            } else {
                folders.set(folderName, (folders.get(folderName) ?? 0n) + file.size);
            }
        }

        return {
            folders: [...folders.entries()]
                .map(([name, size]) => ({ name, size }))
                .sort((a, b) => a.name.localeCompare(b.name)),
            files: files.reverse(),
        };
    }

    get isLoading(): boolean {
        return this.state.case === "loading";
    }

    get error(): string | null {
        return this.state.case === "error" ? this.state.message : null;
    }

    isDeleting(id: FileId): boolean {
        return this.deleting.has(id);
    }

    isDownloading(id: FileId): boolean {
        return this.downloading.has(id);
    }

    push(file: FileItem) {
        if (this.state.case !== "loaded") {
            return;
        }

        this.state.files.set(file.path, file);
    }

    get isRoot(): boolean {
        return this.currentPath.length === 0;
    }

    enterFolder(name: string) {
        this.currentPath.push(name);
    }

    goUp() {
        this.currentPath.pop();
    }

    resetPath() {
        this.currentPath = [];
    }

    async downloadFile(file: FileItem) {
        const folderId = this.folderCtx()?.folder?.id;
        const key = this.folderCtx()?.key;

        if (!folderId || !key) return;

        this.downloading.add(file.id);
        try {
            const decrypted = await filesService.download(folderId, file.id, key);
            const blob = new Blob([new Uint8Array(decrypted)], { type: file.type || "application/octet-stream" });
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = file.name;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            window.URL.revokeObjectURL(url);
        } catch (e: any) {
            console.error("Failed to download file:", e);
        } finally {
            this.downloading.delete(file.id);
        }
    }

    cancelZipDownload() {
        if (this.zipAbortController) {
            this.zipAbortController.abort();
            this.zipAbortController = null;
        }
        this.downloadingZip = false;
        this.zipProgress = null;
    }

    async downloadZip(defaultFolderName: string = "archive") {
        if (this.state.case !== "loaded") return;
        if (this.downloadingZip) return;

        const folderId = this.folderCtx()?.folder?.id;
        const key = this.folderCtx()?.key;
        if (!folderId || !key) return;

        const prefix = this.currentPath.length
            ? this.currentPath.join("/") + "/"
            : "";

        const filesToZip: FileItem[] = [];
        for (const file of this.state.files.values()) {
            if (file.path.startsWith(prefix)) {
                filesToZip.push(file);
            }
        }

        if (filesToZip.length === 0) {
            return;
        }

        this.downloadingZip = true;
        this.zipAbortController = new AbortController();
        const signal = this.zipAbortController.signal;

        try {
            const zip = new JSZip();
            let current = 0;
            const total = filesToZip.length;

            this.zipProgress = { current, total, status: "Подготовка файлов..." };

            for (const file of filesToZip) {
                if (signal.aborted) {
                    throw new Error("Aborted");
                }

                this.zipProgress = {
                    current: ++current,
                    total,
                    status: `Загрузка ${file.name} (${current}/${total})`,
                };

                const decrypted = await filesService.download(folderId, file.id, key);
                const relativePath = file.path.slice(prefix.length);
                zip.file(relativePath, new Uint8Array(decrypted));
            }

            if (signal.aborted) {
                throw new Error("Aborted");
            }

            this.zipProgress = { current: total, total, status: "Сжатие в ZIP..." };

            const content = await zip.generateAsync(
                {
                    type: "blob",
                    compression: "DEFLATE",
                    compressionOptions: { level: 6 }
                },
                (metadata) => {
                    if (signal.aborted) return;
                    this.zipProgress = {
                        current: total,
                        total,
                        status: `Сжатие... ${Math.round(metadata.percent)}%`,
                    };
                }
            );

            if (signal.aborted) {
                throw new Error("Aborted");
            }

            const folderName = this.currentPath.length
                ? this.currentPath[this.currentPath.length - 1]
                : defaultFolderName;
            
            const zipFileName = `${folderName}.zip`;

            const url = window.URL.createObjectURL(content);
            const a = document.createElement("a");
            a.href = url;
            a.download = zipFileName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            window.URL.revokeObjectURL(url);
        } catch (e: any) {
            if (e.message !== "Aborted") {
                console.error("Failed to generate ZIP:", e);
            }
        } finally {
            this.downloadingZip = false;
            this.zipProgress = null;
            this.zipAbortController = null;
        }
    }

    async deleteFile(file: FileItem) {
        const folderId = this.folderCtx()?.folder?.id;
        const token = this.folderCtx()?.token;

        if (!folderId || !token) return;

        this.deleting.add(file.id);
        try {
            await filesService.delete(folderId, token, file.id);
            if (this.state.case === "loaded") {
                this.state.files.delete(file.path);
            }
        } catch (e: any) {
            console.error("Failed to delete file:", e);
        } finally {
            this.deleting.delete(file.id);
        }
    }

    async loadFiles() {
        const folderId = this.folderCtx()?.folder?.id;

        if (!folderId) return;

        if (this.state.case === "loaded") {
            this.state = {case: "loading"};
        }

        try {
            const files = await filesService.list(folderId, this.folderCtx().key)

            this.state = {
                case: "loaded",
                files
            }
        } catch (e: any) {
            this.state = {
                case: "error",
                message: e.toString()
            }
        }
    }
}

const FILES_KEY = Symbol("FILES_KEY");

export function setFilesContext(ctx: FilesContext) {
    setContext(FILES_KEY, ctx);
}

export function getFilesContext() {
    return getContext<FilesContext>(FILES_KEY);
}
