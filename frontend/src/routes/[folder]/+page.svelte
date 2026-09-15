<script lang="ts">
    import { page } from "$app/state";
    import {
        setEncryptedFolderContext,
        EncryptedFolderContext,
    } from "$lib/context/folder.svelte";
    import { create } from "@bufbuild/protobuf";
    import {
        type Folder,
        type FolderId,
        FolderIdSchema,
        type FolderToken,
        FolderTokenSchema,
    } from "$lib/grpc/gen/folder/v1/common_pb";
    import { importKeyFromUrlSafe } from "$lib/crypt/key";
    import FolderOverview from "./FolderOverview.svelte";
    import { onMount } from "svelte";
    import { folderClient } from "$lib/grpc";
    import * as m from "$lib/paraglide/messages";
    import { Code, type ConnectError } from "@connectrpc/connect";

    const pageState = page.state as
        | {
              folder?: Folder;
              key?: CryptoKey;
              token?: FolderToken;
              pendingFiles?: File[];
          }
        | undefined;

    const folderId = $derived.by(() => {
        let id = page.params.folder;
        return create(FolderIdSchema, {
            value: id,
        });
    });

    let state:
        | {
              case: "loading";
          }
        | {
              case: "error";
              error: string;
          }
        | {
              case: "ctx";
              ctx: EncryptedFolderContext;
          } = $state({ case: "loading" });

    const loadFolder = async (folderId: FolderId): Promise<Folder | null> => {
        try {
            return await folderClient.getFolder({
                id: folderId,
            });
        } catch (e: ConnectError) {
            if (e.code === Code.NotFound) {
                return null;
            }

            throw e;
        }
    };

    const showError = (error: string) => {
        state = {
            case: "error",
            error,
        };
    };

    onMount(async () => {
        try {
            state = { case: "loading" };

            let folder = pageState?.folder;
            let key = pageState?.key;
            let token = pageState?.token;
            const pendingFiles = pageState?.pendingFiles ?? [];

            if (!folder) {
                let loadedFolder = await loadFolder(folderId);

                if (!loadedFolder) {
                    return showError(m["folder.errors.not_found"]());
                }

                folder = loadedFolder;
            }

            if (token) {
                localStorage.setItem(
                    `folder_token_${folder.id!!.value}`,
                    token.value
                );
            } else {
                const storedTokenValue = localStorage.getItem(
                    `folder_token_${folder.id!!.value}`
                );
                if (storedTokenValue) {
                    token = create(FolderTokenSchema, {
                        value: storedTokenValue,
                    });
                }
            }

            if (!key) {
                const hash = window.location.hash;
                if (hash.startsWith("#")) {
                    const keyStr = hash.slice(1);
                    if (keyStr) {
                        key = await importKeyFromUrlSafe(keyStr);
                    }
                }
            }

            if (!key) {
                return showError(m["common.errors.key.missing"]());
            }

            state = {
                case: "ctx",
                ctx: new EncryptedFolderContext(
                    folder,
                    key,
                    token,
                    pendingFiles
                ),
            };
        } catch (e: any) {
            showError(m["common.errors.generic"]({ error: e }));
        }
    });
</script>

{#if state.case === "loading"}
    <p>Loading...</p>
{:else if state.case === "error"}
    <p>Init error: {state.error}</p>
{:else}
    <p>{JSON.stringify(state.ctx)}</p>
{/if}
