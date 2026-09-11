<script lang="ts">
    import { activeFolder } from "$lib/stores/folder.svelte";
    import { shareClient } from "$lib/grpc";
    import { create } from "@bufbuild/protobuf";
    import { ShareRequestSchema, SendKeyRequestSchema } from "$lib/grpc/gen/folder/v1/share_pb";
    import { generateEcdhKeyPair, exportPublicKey, deriveSharedSecretKey, generateSasEmojis, encryptFolderKey } from "$lib/crypt/share";
    import { LoaderCircle, Check, ShieldCheck, Copy, X } from "@lucide/svelte";
    import { onDestroy } from "svelte";
    import * as m from "$lib/paraglide/messages";

    let { onClose } = $props<{ onClose: () => void }>();

    type ShareState =
        | { step: "loading" }
        | { step: "waiting"; code: string; ttl: number }
        | { step: "sas"; emojis: string[]; sharedSecretKey: CryptoKey; sessionId: string }
        | { step: "success" }
        | { step: "error"; message: string };

    let shareState = $state<ShareState>({ step: "loading"});
    let copied = $state(false);

    const abortController = new AbortController();
    onDestroy(() => abortController.abort());

    function handleClose() {
        abortController.abort();
        onClose();
    }

    function showError(message: string) {
        shareState = { step: "error", message };
    }

    async function startSharing() {
        if (!activeFolder.id || !activeFolder.key) {
            return showError("No active folder or key");
        }

        try {
            shareState = { step: "loading" };
            const keyPair = await generateEcdhKeyPair();
            const pubKeyBytes = await exportPublicKey(keyPair.publicKey);

            const stream = shareClient.share(create(ShareRequestSchema, {
                folderId: activeFolder.id,
                publicKey: pubKeyBytes,
            }), { signal: abortController.signal });

            for await (const resp of stream) {
                const sessionId = resp.sessionId;

                switch (resp.event.case) {
                    case "codeRotated":
                        if (shareState.step === "loading" || shareState.step === "waiting") {
                            shareState = {
                                step: "waiting",
                                code: resp.event.value.code,
                                ttl: resp.event.value.ttlSeconds
                            }
                        }
                        break;

                    case "receiverJoined": {
                        const sharedSecretKey = await deriveSharedSecretKey(
                            keyPair.privateKey,
                            resp.event.value.receiverPublicKey
                        );
                        const emojis = await generateSasEmojis(sharedSecretKey);

                        shareState = {
                            step: "sas",
                            emojis,
                            sharedSecretKey,
                            sessionId,
                        };
                        break;
                    }

                    case "sessionClosed":
                        if ((shareState.step as ShareState["step"]) !== "success") handleClose();
                        return;
                }
            }
        } catch (e: any) {
            if (abortController.signal.aborted) return;
            showError(e?.message || String(e));
        }
    }

    async function confirmAndSendKey() {
        if (shareState.step !== "sas" || !activeFolder.key) return;

        const { sharedSecretKey, sessionId } = shareState;

        try {
            shareState = { step: "loading" };
            const encryptedKey = await encryptFolderKey(sharedSecretKey, activeFolder.key);

            await shareClient.sendKey(
                create(SendKeyRequestSchema, {
                    sessionId,
                    encryptedFolderKey: encryptedKey,
                })
            );

            shareState = { step: "success" };
        } catch (e: any) {
            if (abortController.signal.aborted) return;
            showError(e?.message || String(e));
        }
    }

    async function copyCode(code: string) {
        await navigator.clipboard.writeText(code);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }

    startSharing();
</script>

<div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
    <div class="bg-card border shadow-lg p-6 max-w-md w-full relative flex flex-col gap-4">
        <button class="absolute top-4 right-4 text-muted-foreground hover:text-foreground cursor-pointer" onclick={handleClose}>
            <X class="w-5 h-5"/>
        </button>

        <h2 class="text-xl font-bold flex items-center gap-2">
            <ShieldCheck class="w-6 h-6 text-primary"/>
            {m["share.title"]()}
        </h2>

        {#if shareState.step === "loading"}
            <div class="flex flex-col items-center justify-center py-8 gap-3">
                <LoaderCircle class="animate-spin w-8 h-8 text-primary"/>
                <p class="text-muted-foreground">{m["share.init"]()}</p>
            </div>
        {:else if shareState.step === "waiting"}
            {@const waitingState = shareState}
            <div class="flex flex-col items-center justify-center py-6 gap-4">
                <p class="text-sm text-muted-foreground text-center">
                    {m["share.share"]()}
                </p>
                <div class="bg-muted p-4 flex items-center gap-3">
                    <span class="text-3xl font-mono tracking-widest font-bold">{waitingState.code}</span>
                    <button class="p-2 hover:bg-background border cursor-pointer" onclick={() => copyCode(waitingState.code)} title={m["share.copy-code"]()}>
                        {#if copied}
                            <Check class="w-5 h-5 text-green-500"/>
                        {:else}
                            <Copy class="w-5 h-5"/>
                        {/if}
                    </button>
                </div>
                <div class="flex items-center gap-2 text-xs text-muted-foreground">
                    <LoaderCircle class="animate-spin w-3 h-3"/>
                    {m["share.rotate"]()}
                </div>
            </div>
        {:else if shareState.step === "sas"}
            <div class="flex flex-col items-center justify-center py-4 gap-4">
                <div class="bg-primary/10 text-primary p-3 text-sm text-center">
                    {m["share.connected-and-verify"]()}
                </div>
                <div class="flex flex-wrap justify-center items-center gap-2 sm:gap-4 p-4 bg-muted max-w-full">
                    {#each shareState.emojis as emoji}
                        <span class="text-2xl sm:text-4xl select-none">{emoji}</span>
                    {/each}
                </div>
                <p class="text-xs text-muted-foreground text-center">
                    {m["share.if-match"]()}
                </p>
                <button
                        class="w-full bg-primary text-primary-foreground py-2 font-medium hover:opacity-90 flex items-center justify-center gap-2 cursor-pointer"
                        onclick={confirmAndSendKey}
                >
                    <Check class="w-4 h-4"/>
                    {m["share.match-btn"]()}
                </button>
            </div>
        {:else if shareState.step === "success"}
            <div class="flex flex-col items-center justify-center py-8 gap-4">
                <div class="w-12 h-12 text-green-500 flex items-center justify-center">
                    <Check class="w-6 h-6"/>
                </div>
                <p class="font-medium text-center">{m["share.success"]()}</p>
                <button class="px-4 py-2 bg-muted text-sm hover:bg-muted/80 cursor-pointer" onclick={handleClose}>
                    {m["general.close"]()}
                </button>
            </div>
        {:else if shareState.step === "error"}
            <div class="flex flex-col items-center justify-center py-6 gap-4">
                <p class="text-destructive text-center">{shareState.message}</p>
                <button class="px-4 py-2 bg-muted text-sm hover:bg-muted/80 cursor-pointer" onclick={handleClose}>
                    {m["general.close"]()}
                </button>
            </div>
        {/if}
    </div>
</div>