import { setContext, getContext } from "svelte";
import type { FolderContext } from "./folder.svelte";
import type { FilesContext } from "./files.svelte";
import { fileService } from "$lib/services/file.service";

export type UploadTaskState =
    | { case: "uploading"; progress: number; speed: number; eta: number }
    | { case: "completed" }
    | { case: "error"; message: string }
    | { case: "cancelled" };

export interface UploadTask {
    id: string;
    file: File;
    state: UploadTaskState;
    controller: AbortController;
}

export class UploadContext {
    tasks = $state<UploadTask[]>([]);
    private folderCtx: FolderContext;
    private filesCtx: FilesContext;

    constructor(folderCtx: FolderContext, filesCtx: FilesContext) {
        this.folderCtx = folderCtx;
        this.filesCtx = filesCtx;
    }

    async uploadFiles(files: File[]) {
        for (const file of files) {
            await this.uploadFile(file);
        }
    }

    async uploadFile(file: File) {
        if (!this.folderCtx?.token || !this.folderCtx?.folder?.id) return;

        const taskId = Math.random().toString(36).substring(2, 9);
        const controller = new AbortController();

        const task: UploadTask = {
            id: taskId,
            file,
            state: { case: "uploading", progress: 0, speed: 0, eta: 0 },
            controller,
        };

        this.tasks.push(task);

        try {
            await fileService.upload(
                this.folderCtx.folder.id,
                this.folderCtx.token,
                this.folderCtx.key,
                file,
                (progress, speed, eta) => {
                    const t = this.tasks.find((item) => item.id === taskId);
                    if (t && t.state.case === "uploading") {
                        t.state = { case: "uploading", progress, speed, eta };
                    }
                },
                controller.signal
            );

            const t = this.tasks.find((item) => item.id === taskId);
            if (t) {
                t.state = { case: "completed" };
            }

            await this.filesCtx.loadFiles();
            this.removeTask(taskId);
        } catch (e: any) {
            const t = this.tasks.find((item) => item.id === taskId);
            if (t) {
                if (controller.signal.aborted) {
                    t.state = { case: "cancelled" };
                } else {
                    t.state = { case: "error", message: e?.message || "Upload failed" };
                }
            }
        }
    }

    async retryTask(taskId: string) {
        const task = this.tasks.find((t) => t.id === taskId);
        if (!task) return;
        task.controller = new AbortController();
        task.state = { case: "uploading", progress: 0, speed: 0, eta: 0 };

        try {
            await fileService.upload(
                this.folderCtx.folder.id!,
                this.folderCtx.token!,
                this.folderCtx.key,
                task.file,
                (progress, speed, eta) => {
                    const t = this.tasks.find((item) => item.id === taskId);
                    if (t && t.state.case === "uploading") {
                        t.state = { case: "uploading", progress, speed, eta };
                    }
                },
                task.controller.signal
            );

            const t = this.tasks.find((item) => item.id === taskId);
            if (t) {
                t.state = { case: "completed" };
            }

            await this.filesCtx.loadFiles();
            this.removeTask(taskId);
        } catch (e: any) {
            const t = this.tasks.find((item) => item.id === taskId);
            if (t) {
                if (task.controller.signal.aborted) {
                    t.state = { case: "cancelled" };
                } else {
                    t.state = { case: "error", message: e?.message || "Upload failed" };
                }
            }
        }
    }

    cancelTask(taskId: string) {
        const task = this.tasks.find((t) => t.id === taskId);
        if (task) {
            task.controller.abort();
            task.state = { case: "cancelled" };
        }
    }

    removeTask(taskId: string) {
        this.tasks = this.tasks.filter((t) => t.id !== taskId);
    }
}

const UPLOAD_KEY = Symbol("UPLOAD_KEY");

export const setUploadContext = (ctx: UploadContext) => setContext(UPLOAD_KEY, ctx);
export const getUploadContext = () => getContext<UploadContext>(UPLOAD_KEY);
