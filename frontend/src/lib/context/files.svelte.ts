import type {FileId} from "$lib/grpc/gen/folder/v1/common_pb";
import type {FolderContext} from "$lib/context/folder.svelte";
import {getContext, setContext} from "svelte";
import {filesService} from "$lib/services/files.service";

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

    constructor(private readonly folderCtx: () => FolderContext) {
        void this.loadFiles()
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