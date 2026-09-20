import type {FileId} from "$lib/grpc/gen/folder/v1/common_pb";
import type {FolderContext} from "$lib/context/folder.svelte";
import {getContext, setContext} from "svelte";
import {filesService} from "$lib/services/files.service";
import * as m from "$lib/paraglide/messages";

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

    private deleting = $state(new Set<FileId>());
    private downloading = $state(new Set<FileId>());

    downloadingZip = $state(false);
    zipProgress = $state<{ current: number; total: number; status: string } | null>(null);
    private zipAbortController: AbortController | null = null;

    uploadingFiles = $state<Map<string, {
        key: string;
        name: string;
        size: number;
        loaded: number;
        progress: number;
        speed: number;
        eta: number;
        status: string;
        currentChunk: number;
        totalChunks: number;
    }>>(new Map());

    uploadBatch = $state<{
        totalFiles: number;
        completedFiles: number;
        totalBytes: number;
        uploadedBytes: number;
        progress: number;
        speed: number;
        eta: number;
        isUploading: boolean;
    } | null>(null);

    private uploadControllers = new Map<string, AbortController>();

    failedUploads = $state<Map<string, {
        key: string;
        name: string;
        file: File;
        path: string;
        error: string;
    }>>(new Map());

    retryUpload(key: string) {
        const failed = this.failedUploads.get(key);
        if (!failed) return;
        this.failedUploads.delete(key);
        void this.uploadFiles([{ file: failed.file, path: failed.path }]);
    }

    retryAllFailed() {
        const items = Array.from(this.failedUploads.values()).map(f => ({ file: f.file, path: f.path }));
        this.failedUploads.clear();
        if (items.length > 0) {
            void this.uploadFiles(items);
        }
    }

    dismissFailed(key: string) {
        this.failedUploads.delete(key);
    }

    dismissAllFailed() {
        this.failedUploads.clear();
    }

    cancelUpload(key: string) {
        const controller = this.uploadControllers.get(key);
        if (controller) {
            controller.abort();
            this.uploadControllers.delete(key);
        }
        this.uploadingFiles.delete(key);
    }

    cancelAllUploads() {
        for (const controller of this.uploadControllers.values()) {
            controller.abort();
        }
        this.uploadControllers.clear();
        this.uploadingFiles.clear();
        this.uploadBatch = null;
    }

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
        this.state.files = new Map(this.state.files);
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
        this.downloading = new Set(this.downloading);
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
            this.downloading = new Set(this.downloading);
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
            const blob = await filesService.downloadZip(
                folderId,
                key,
                filesToZip,
                prefix,
                (progress) => {
                    this.zipProgress = progress;
                },
                signal
            );

            const folderName = this.currentPath.length
                ? this.currentPath[this.currentPath.length - 1]
                : defaultFolderName;
            
            const zipFileName = `${folderName}.zip`;

            const url = window.URL.createObjectURL(blob);
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

    async uploadFiles(inputFiles: { file: File; path: string }[] | FileList | File[]) {
        const folderId = this.folderCtx()?.folder?.id;
        const token = this.folderCtx()?.token;
        const key = this.folderCtx()?.key;

        if (!folderId || !token || !key) return;

        const prefix = this.currentPath.length
            ? this.currentPath.join("/") + "/"
            : "";

        let fileItems: { file: File; path: string }[];

        if (Array.isArray(inputFiles)) {
            if (inputFiles.length > 0 && typeof inputFiles[0] === 'object' && inputFiles[0] !== null && 'file' in inputFiles[0]) {
                fileItems = inputFiles as { file: File; path: string }[];
            } else {
                fileItems = (inputFiles as File[]).map(f => ({
                    file: f,
                    path: (f as any).webkitRelativePath || f.name
                }));
            }
        } else {
            fileItems = Array.from(inputFiles).map(f => ({
                file: f,
                path: (f as any).webkitRelativePath || f.name
            }));
        }

        if (fileItems.length === 0) return;

        const newFilesTotalBytes = fileItems.reduce((acc, item) => acc + item.file.size, 0);

        if (!this.uploadBatch) {
            this.uploadBatch = {
                totalFiles: fileItems.length,
                completedFiles: 0,
                totalBytes: newFilesTotalBytes,
                uploadedBytes: 0,
                progress: 0,
                speed: 0,
                eta: 0,
                isUploading: true,
            };
        } else {
            this.uploadBatch.totalFiles += fileItems.length;
            this.uploadBatch.totalBytes += newFilesTotalBytes;
        }

        const batchStartTime = Date.now();
        let batchLastTime = batchStartTime;
        let batchSpeed = 0;

        for (const item of fileItems) {
            const relPath = item.path || item.file.name;
            const fullPath = prefix + relPath;
            const uploadKey = fullPath;
            const fileSize = item.file.size;

            const controller = new AbortController();
            this.uploadControllers.set(uploadKey, controller);

            this.uploadingFiles.set(uploadKey, {
                key: uploadKey,
                name: item.file.name,
                size: fileSize,
                loaded: 0,
                progress: 0,
                speed: 0,
                eta: 0,
                status: m["file.loading"](),
                currentChunk: 0,
                totalChunks: 1,
            });

            try {
                const uploaded = await filesService.uploadFile(
                    folderId,
                    token,
                    key,
                    item.file,
                    fullPath,
                    (progressInfo) => {
                        const current = this.uploadingFiles.get(uploadKey);
                        if (current) {
                            this.uploadingFiles.set(uploadKey, {
                                ...current,
                                loaded: progressInfo.loaded,
                                progress: progressInfo.progress,
                                speed: progressInfo.speed,
                                eta: progressInfo.eta,
                                currentChunk: progressInfo.currentChunk,
                                totalChunks: progressInfo.totalChunks,
                                status: progressInfo.progress >= 100 ? m["common.actions.done"]() : m["file.uploading"]({progress: progressInfo.progress})
                            });
                        }

                        if (this.uploadBatch) {
                            let totalLoadedSoFar = 0;
                            for (const up of this.uploadingFiles.values()) {
                                totalLoadedSoFar += up.loaded;
                            }
                            const avgSize = this.uploadBatch.totalBytes / this.uploadBatch.totalFiles || 0;
                            const completedWork = this.uploadBatch.completedFiles * (avgSize * 2 + 16);
                            const totalWorkExpected = this.uploadBatch.totalBytes * 2 + (this.uploadBatch.totalFiles * 16);

                            this.uploadBatch.uploadedBytes = Math.min(totalWorkExpected, completedWork + totalLoadedSoFar);

                            const now = Date.now();
                            const dt = (now - batchLastTime) / 1000;
                            if (dt >= 0.2) {
                                if (dt > 0) {
                                    batchSpeed = this.uploadBatch.uploadedBytes / ((now - batchStartTime) / 1000);
                                }
                                batchLastTime = now;
                            }
                            this.uploadBatch.speed = batchSpeed;
                            this.uploadBatch.progress = totalWorkExpected > 0 ? Math.min(100, Math.round((this.uploadBatch.uploadedBytes / totalWorkExpected) * 100)) : 100;
                            const rem = totalWorkExpected - this.uploadBatch.uploadedBytes;
                            this.uploadBatch.eta = batchSpeed > 0 ? rem / batchSpeed : 0;
                        }
                    },
                    controller.signal
                );
                this.push(uploaded);
            } catch (e: any) {
                if (e.message !== "Aborted") {
                    console.error("Failed to upload file:", e);
                    this.failedUploads.set(uploadKey, {
                        key: uploadKey,
                        name: item.file.name,
                        file: item.file,
                        path: fullPath,
                        error: e.toString()
                    });
                }
            } finally {
                this.uploadControllers.delete(uploadKey);
                this.uploadingFiles.delete(uploadKey);
                if (this.uploadBatch) {
                    this.uploadBatch.completedFiles++;
                    if (this.uploadBatch.completedFiles >= this.uploadBatch.totalFiles) {
                        this.uploadBatch = null;
                    }
                }
            }
        }
    }

    async deleteFile(file: FileItem) {
        const folderId = this.folderCtx()?.folder?.id;
        const token = this.folderCtx()?.token;

        if (!folderId || !token) return;

        this.deleting.add(file.id);
        this.deleting = new Set(this.deleting);
        try {
            await filesService.delete(folderId, token, file.id);
            if (this.state.case === "loaded") {
                this.state.files.delete(file.path);
                this.state.files = new Map(this.state.files);
            }
        } catch (e: any) {
            console.error("Failed to delete file:", e);
        } finally {
            this.deleting.delete(file.id);
            this.deleting = new Set(this.deleting);
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
