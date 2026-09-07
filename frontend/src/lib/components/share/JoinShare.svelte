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

    type JoinState =
        | { step: "input"; errorMessage?: string }
        | { step: "connecting" }
        | { step: "sas"; emojis: string[] }
        | { step: "decrypting" }
        | { step: "error"; message: string };

    let joinState = $state<JoinState>({ step: "input" });
    let code = $state("");

    let abortController: AbortController | null = null;

    onDestroy(() => cancelSession());

    function cancelSession() {
        if (abortController) {
            abortController.abort();
            abortController = null;
        }
        joinState = { step: "input" };
        code = "";
    }

    function showError(message: string) {
        joinState = { step: "error", message };
    }

    async function handleJoin(e: Event) {
        e.preventDefault();
        const trimmedCode = code.trim();

        if (trimmedCode.length !== 6) {
            joinState = { step: "input", errorMessage: m["share.invalid-code"]({digits: 6}) };
            return;
        }

        try {
            joinState = { step: "connecting" };
            abortController = new AbortController();

            const keyPair = await generateEcdhKeyPair();
            const pubKeyBytes = await exportPublicKey(keyPair.publicKey);

            const stream = shareClient.join(
                create(JoinRequestSchema, {
                    code: trimmedCode,
                    publicKey: pubKeyBytes,
                }),
                { signal: abortController.signal }
            );

            let sharedSecretKey: CryptoKey | null = null;

            for await (const resp of stream) {
                switch (resp.event.case) {
                    case "sessionConnected": {
                        const senderPk = resp.event.value.senderPublicKey;
                        sharedSecretKey = await deriveSharedSecretKey(keyPair.privateKey, senderPk);
                        const emojis = await generateSasEmojis(sharedSecretKey);
                        joinState = { step: "sas", emojis };
                        break;
                    }

                    case "keyReceived": {
                        joinState = { step: "decrypting" };

                        const { encryptedFolderKey, folderId } = resp.event.value;

                        const folderKey = await decryptFolderKey(sharedSecretKey!!, encryptedFolderKey);
                        const keyString = await exportKey(folderKey);

                        await goto(localizeHref(`/${folderId!!.value}#${keyString}`));
                        return;
                    }
                }
            }
        } catch (err: any) {
            if (abortController?.signal.aborted) return;
            showError(err?.message || String(err));
        }
    }
</script>

<div class="bg-card border-muted-foreground border border-dashed p-4 w-full flex flex-col gap-3">
    <h3 class="font-semibold flex items-center gap-2 text-md">
        <KeyRound class="w-5 h-5 text-primary" />
        {m["share.title"]()}
    </h3>

    {#if joinState.step === "input" || joinState.step === "error"}
        {@const s = joinState}
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

        {#if s.step === "input" && s.errorMessage}
            <span class="text-destructive text-sm">{s.errorMessage}</span>
        {:else if s.step === "error"}
            <span class="text-destructive text-sm">{s.message}</span>
        {/if}
    {:else if joinState.step === "connecting"}
        <div class="flex flex-col items-center justify-center py-4 gap-3 text-muted-foreground">
            <div class="flex items-center gap-2">
                <LoaderCircle class="animate-spin w-5 h-5 text-primary" />
                {m["share.connecting"]()}
            </div>
            <button
                    type="button"
                    class="px-3 py-1 text-xs bg-muted hover:bg-muted/80 text-foreground cursor-pointer"
                    onclick={cancelSession}
            >
                {m["general.close"]()}
            </button>
        </div>
    {:else if joinState.step === "sas"}
        {@const s = joinState}
        <div class="flex flex-col items-center justify-center py-2 gap-3">
            <div class="text-xs text-muted-foreground text-center">
                {m["share.verify-emojis"]()}
            </div>
            <div class="flex gap-3 p-3 bg-muted">
                {#each s.emojis as emoji}
                    <span class="text-3xl">{emoji}</span>
                {/each}
            </div>
            <div class="flex items-center gap-2 text-xs text-primary">
                <LoaderCircle class="animate-spin w-3 h-3" />
                {m["share.waiting-confirm"]()}
            </div>
            <button
                    type="button"
                    class="px-3 py-1 text-xs bg-muted hover:bg-muted/80 text-foreground cursor-pointer mt-2"
                    onclick={cancelSession}
            >
                {m["general.close"]()}
            </button>
        </div>
    {:else if joinState.step === "decrypting"}
        <div class="flex items-center justify-center py-4 gap-2 text-muted-foreground">
            <LoaderCircle class="animate-spin w-5 h-5 text-primary" />
            {m["share.decrypting-and-open"]()}
        </div>
    {/if}
</div>
