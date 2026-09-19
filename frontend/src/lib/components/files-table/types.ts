import {Folder, type LucideProps} from "@lucide/svelte";
import type {Component} from "svelte";
import type {FileItem} from "$lib/context/files.svelte";

export type Row = {
    icon: Component<LucideProps>,
} & Kind;

export type Kind =
    | { kind: "goUp" }
    | { kind: "folder", name: string, size: bigint }
    | ({ kind: "file" } & FileItem);
