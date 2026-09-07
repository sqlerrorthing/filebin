<script lang="ts">
    import {shareClient} from "$lib/grpc";
    import {create} from "@bufbuild/protobuf";
    import {JoinRequestSchema} from "$lib/grpc/gen/folder/v1/share_pb";
    import {
        generateEcdhKeyPair,
        exportPublicKey,
        deriveSharedSecretKey,
        generateSasEmojis,
        decryptFolderKey
    } from "$lib/crypt/share";
    import {exportKey} from "$lib/crypt";
    import {LoaderCircle, KeyRound} from "@lucide/svelte";
    import {goto} from "$app/navigation";
    import {localizeHref} from "$lib/paraglide/runtime";
    import {onDestroy} from "svelte";
    import * as m from "$lib/paraglide/messages";

    let code = $state("");
    let step = $state<"input" | "connecting" | "sas" | "decrypting" | "error">("input");
    let errorMessage = $state<string | null>(null);
    let emojis = $state<string[]>([]);
    let abortController: AbortController | null = null;

    onDestroy(() => {
        if (abortController) abortController.abort();
    });

    async function handleJoin(e: Event) {
        e.preventDefault();
        if (!code || code.length !== 6) {
            errorMessage = "Please enter a valid 6-digit code";
            return;
        }

        try {
            step = "connecting";
            errorMessage = null;
            abortController = new AbortController();

            const keyPair = await generateEcdhKeyPair();
            const pubKeyBytes = await exportPublicKey(keyPair.publicKey);

            const stream = shareClient.join(create(JoinRequestSchema, {
                code: code.trim(),
                publicKey: pubKeyBytes,
            }), {signal: abortController.signal});

            let sharedSecretKey: CryptoKey | null = null;

            for await (const resp of stream) {
                if (resp.event.case === "sessionConnected") {
                    const senderPk = resp.event.value.senderPublicKey;
                    sharedSecretKey = await deriveSharedSecretKey(keyPair.privateKey, senderPk);
                    emojis = await generateSasEmojis(sharedSecretKey);
                    step = "sas";
                } else if (resp.event.case === "keyReceived") {
                    if (!sharedSecretKey) {
                        throw new Error("Shared secret not established");
                    }
                    step = "decrypting";
                    const encryptedFolderKey = resp.event.value.encryptedFolderKey;
                    const folderIdObj = resp.event.value.folderId;
                    if (!folderIdObj) {
                        throw new Error("Missing folder ID");
                    }

                    const folderKey = await decryptFolderKey(sharedSecretKey, encryptedFolderKey);
                    const keyString = await exportKey(folderKey);
                    const folderId = folderIdObj.value;

                    await goto(localizeHref(`/${folderId}#${keyString}`));
                    return;
                } else if (resp.event.case === "sessionFailed") {
                    errorMessage = resp.event.value.errorMessage;
                    step = "error";
                }
            }
        } catch (err: any) {
            if (abortController?.signal.aborted) return;
            errorMessage = err.message || String(err);
            step = "error";
        }
    }
</script>

<!-- fixme: pasting the code causes upload. -->

<div class="bg-card border-muted-foreground border border-dashed p-4 w-full flex flex-col gap-3">
    <h3 class="font-semibold flex items-center gap-2 text-md">
        <KeyRound class="w-5 h-5 text-primary"/>
        {m["share.title"]()}
    </h3>

    {#if step === "input" || step === "error"}
        <form onsubmit={handleJoin} class="flex gap-2 flex-col sm:flex-row">
            <input
                    type="text"
                    placeholder={m["share.enter-code"]()}
                    bind:value={code}
                    class="flex-1 px-3 py-2 border bg-background text-foreground tracking-widest font-mono uppercase outline-none focus:border-primary"
            />
            <button
                    type="submit"
                    class="px-4 py-2 bg-primary text-primary-foreground font-medium hover:bg-accent-foreground cursor-pointer"
            >
                {m["share.join"]()}
            </button>
        </form>
        {#if errorMessage}
            <span class="text-destructive text-sm">{errorMessage}</span>
        {/if}
    {:else if step === "connecting"}
        <div class="flex items-center justify-center py-4 gap-2 text-muted-foreground">
            <LoaderCircle class="animate-spin w-5 h-5 text-primary"/>
            {m["share.connecting"]()}
        </div>
    {:else if step === "sas"}
        <div class="flex flex-col items-center justify-center py-2 gap-3">
            <div class="text-xs text-muted-foreground text-center">
                {m["share.verify-emojis"]()}
            </div>
            <div class="flex gap-3 p-3 bg-muted">
                {#each emojis as emoji}
                    <span class="text-3xl">{emoji}</span>
                {/each}
            </div>
            <div class="flex items-center gap-2 text-xs text-primary">
                <LoaderCircle class="animate-spin w-3 h-3"/>
                {m["share.waiting-confirm"]()}
            </div>
        </div>
    {:else if step === "decrypting"}
        <div class="flex items-center justify-center py-4 gap-2 text-muted-foreground">
            <LoaderCircle class="animate-spin w-5 h-5 text-primary"/>
            {m["share.decrypting-and-open"]()}
        </div>
    {/if}
</div>
