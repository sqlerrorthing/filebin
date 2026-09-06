<script lang="ts">
    import { activeFolder } from "$lib/stores/folder.svelte";
    import { shareClient } from "$lib/grpc";
    import { create } from "@bufbuild/protobuf";
    import { ShareRequestSchema, SendKeyRequestSchema } from "$lib/grpc/gen/folder/v1/share_pb";
    import { generateEcdhKeyPair, exportPublicKey, deriveSharedSecretKey, generateSasEmojis, encryptFolderKey } from "$lib/crypt/share";
    import { LoaderCircle, Check, ShieldCheck, Copy, X } from "@lucide/svelte";
    import { onDestroy } from "svelte";

    let { onClose } = $props<{ onClose: () => void }>();

    let step = $state<"loading" | "waiting" | "sas" | "success" | "error">("loading");
    let code = $state("");
    let ttl = $state(0);
    let errorMessage = $state<string | null>(null);
    let emojis = $state<string[]>([]);
    let sessionId = $state("");
    let sharedSecretKey = $state<CryptoKey | null>(null);
    let privateKey = $state<CryptoKey | null>(null);
    let copied = $state(false);
    import * as m from "$lib/paraglide/messages";

    let abortController = new AbortController();

    onDestroy(() => {
        abortController.abort();
    });

    async function startSharing() {
        if (!activeFolder.id || !activeFolder.key) {
            errorMessage = "No active folder or key";
            step = "error";
            return;
        }

        try {
            step = "loading";
            const keyPair = await generateEcdhKeyPair();
            privateKey = keyPair.privateKey;
            const pubKeyBytes = await exportPublicKey(keyPair.publicKey);

            const req = create(ShareRequestSchema, {
                folderId: activeFolder.id,
                publicKey: pubKeyBytes,
            });

            const stream = shareClient.share(req, { signal: abortController.signal });

            for await (const resp of stream) {
                sessionId = resp.sessionId;
                if (resp.event.case === "codeRotated") {
                    code = resp.event.value.code;
                    ttl = resp.event.value.ttlSeconds;
                    if (step === "loading") {
                        step = "waiting";
                    }
                } else if (resp.event.case === "receiverJoined") {
                    const receiverPk = resp.event.value.receiverPublicKey;
                    if (privateKey) {
                        sharedSecretKey = await deriveSharedSecretKey(privateKey, receiverPk);
                        emojis = await generateSasEmojis(sharedSecretKey);
                        step = "sas";
                    }
                } else if (resp.event.case === "sessionFailed") {
                    errorMessage = resp.event.value.errorMessage;
                    step = "error";
                }
            }
        } catch (e: any) {
            if (abortController.signal.aborted) return;
            errorMessage = e.message || String(e);
            step = "error";
        }
    }

    async function confirmAndSendKey() {
        if (!sharedSecretKey || !sessionId || !activeFolder.key) return;

        try {
            step = "loading";
            const encryptedKey = await encryptFolderKey(sharedSecretKey, activeFolder.key);

            await shareClient.sendKey(create(SendKeyRequestSchema, {
                sessionId,
                encryptedFolderKey: encryptedKey,
            }));

            step = "success";
        } catch (e: any) {
            errorMessage = e.message || String(e);
            step = "error";
        }
    }

    function copyCode() {
        navigator.clipboard.writeText(code);
        copied = true;
        setTimeout(() => copied = false, 2000);
    }

    startSharing();
</script>

<div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
    <div class="bg-card border rounded-lg shadow-lg p-6 max-w-md w-full relative flex flex-col gap-4">
        <button class="absolute top-4 right-4 text-muted-foreground hover:text-foreground cursor-pointer" onclick={onClose}>
            <X class="w-5 h-5"/>
        </button>

        <h2 class="text-xl font-bold flex items-center gap-2">
            <ShieldCheck class="w-6 h-6 text-primary"/>
            {m["share.title"]()}
        </h2>

        {#if step === "loading"}
            <div class="flex flex-col items-center justify-center py-8 gap-3">
                <LoaderCircle class="animate-spin w-8 h-8 text-primary"/>
                <p class="text-muted-foreground">{m["share.init"]()}</p>
            </div>
        {:else if step === "waiting"}
            <div class="flex flex-col items-center justify-center py-6 gap-4">
                <p class="text-sm text-muted-foreground text-center">
                    {m["share.share"]()}
                </p>
                <div class="bg-muted p-4 rounded-lg flex items-center gap-3">
                    <span class="text-3xl font-mono tracking-widest font-bold">{code}</span>
                    <button class="p-2 hover:bg-background rounded border cursor-pointer" onclick={copyCode} title={m["share.copy-code"]()}>
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
        {:else if step === "sas"}
            <div class="flex flex-col items-center justify-center py-4 gap-4">
                <div class="bg-primary/10 text-primary p-3 rounded-lg text-sm text-center">
                    {m["share.connected-and-verify"]()}
                </div>
                <div class="flex gap-4 p-4 bg-muted rounded-xl">
                    {#each emojis as emoji}
                        <span class="text-4xl">{emoji}</span>
                    {/each}
                </div>
                <p class="text-xs text-muted-foreground text-center">
                    {m["share.if-match"]()}
                </p>
                <button
                    class="w-full bg-primary text-primary-foreground py-2 rounded-lg font-medium hover:opacity-90 flex items-center justify-center gap-2 cursor-pointer"
                    onclick={confirmAndSendKey}
                >
                    <Check class="w-4 h-4"/>
                    {m["share.match-btn"]()}
                </button>
            </div>
        {:else if step === "success"}
            <div class="flex flex-col items-center justify-center py-8 gap-4">
                <div class="w-12 h-12 bg-green-500/20 text-green-500 rounded-full flex items-center justify-center">
                    <Check class="w-6 h-6"/>
                </div>
                <p class="font-medium text-center">{m["share.success"]()}</p>
                <button
                    class="px-4 py-2 bg-muted rounded-lg text-sm hover:bg-muted/80 cursor-pointer"
                    onclick={onClose}
                >
                    {m["general.close"]()}
                </button>
            </div>
        {:else if step === "error"}
            <div class="flex flex-col items-center justify-center py-6 gap-4">
                <p class="text-destructive text-center">{errorMessage || "An error occurred"}</p>
                <button
                    class="px-4 py-2 bg-muted rounded-lg text-sm hover:bg-muted/80 cursor-pointer"
                    onclick={onClose}
                >
                    {m["general.close"]()}
                </button>
            </div>
        {/if}
    </div>
</div>
