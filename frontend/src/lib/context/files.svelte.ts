import { setContext, getContext } from "svelte";
import type { FileId } from "$lib/grpc/gen/folder/v1/common_pb";
import { fileService, type FileItem } from "$lib/services/file.service";
import type { FolderContext } from "./folder.svelte";
import type { FileView } from "$lib/grpc/gen/folder/v1/files_pb";

export type FilesState =
    | { case: "loading" }
    | { case: "error"; message: string }
    | { case: "success"; files: FileItem[] };

export class FilesContext {
    state: FilesState = $state({ case: "loading" });
    currentPath = $state<string[]>([]);
    deletingFileId = $state<FileId | null>(null);
    downloadingFileId = $state<FileId | null>(null);

    private folderCtx: FolderContext;

    constructor(folderCtx: FolderContext) {
        this.folderCtx = folderCtx;
    }

    async loadFiles() {
        if (!this.folderCtx?.folder?.id) return;
        if (this.state.case !== "success") {
            this.state = { case: "loading" };
        }
        try {
            const files = await fileService.list(this.folderCtx.key, this.folderCtx.folder.id);
            this.state = { case: "success", files };
        } catch (e: any) {
            if (this.state.case !== "success") {
                this.state = { case: "error", message: e?.message || "Failed to load files" };
            }
        }
    }

    async deleteFile(fileId: FileId) {
        if (!this.folderCtx?.token || !this.folderCtx?.folder?.id) return;
        this.deletingFileId = fileId;
        try {
            await fileService.delete(this.folderCtx.folder.id, this.folderCtx.token, fileId);
            if (this.state.case === "success") {
                this.state = {
                    case: "success",
                    files: this.state.files.filter((f) => f.id.value !== fileId.value),
                };
            }
        } catch (e: any) {
            console.error("Failed to delete file", e);
        } finally {
            this.deletingFileId = null;
        }
    }

    async downloadFile(fileId: FileId, fileName: string) {
        if (!this.folderCtx?.folder?.id) return;
        this.downloadingFileId = fileId;
        try {
            await fileService.download(this.folderCtx.folder.id, fileId, fileName);
        } catch (e: any) {
            console.error("Failed to download file", e);
        } finally {
            this.downloadingFileId = null;
        }
    }

    get currentItems() {
        if (this.state.case !== "success") return { subfolders: [], files: [] };
        const subfoldersSet = new Set<string>();
        const currentFiles: Array<{
            id: FileId;
            name: string;
            path: string;
            size: number | bigint;
            view: FileView;
        }> = [];

        for (const f of this.state.files) {
            const segments = f.path.split("/").filter(Boolean);
            let matches = true;
            for (let i = 0; i < this.currentPath.length; i++) {
                if (segments[i] !== this.currentPath[i]) {
                    matches = false;
                    break;
                }
            }

            if (!matches) continue;

            const remaining = segments.slice(this.currentPath.length);
            if (remaining.length === 0) continue;

            if (remaining.length === 1) {
                currentFiles.push({
                    id: f.id,
                    name: remaining[0],
                    path: f.path,
                    size: f.size,
                    view: f.view,
                });
            } else {
                subfoldersSet.add(remaining[0]);
            }
        }

        const subfolders = Array.from(subfoldersSet).sort();
        currentFiles.reverse();

        return { subfolders, files: currentFiles };
    }
}

const FILES_KEY = Symbol("FILES_KEY");

export const setFilesContext = (ctx: FilesContext) => setContext(FILES_KEY, ctx);
export const getFilesContext = () => getContext<FilesContext>(FILES_KEY);
