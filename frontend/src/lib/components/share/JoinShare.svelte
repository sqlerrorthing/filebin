<script lang="ts">
    import { shareClient } from '$lib/grpc';
    import { create } from '@bufbuild/protobuf';
    import { JoinRequestSchema } from '$lib/grpc/gen/folder/v1/share_pb';
    import {
        generateEcdhKeyPair,
        exportPublicKey,
        deriveSharedSecretKey,
        generateSasEmojis,
        decryptFolderKey,
    } from '$lib/crypt/share';
    import { exportKey } from '$lib/crypt';
    import { LoaderCircle, KeyRound } from '@lucide/svelte';
    import { goto } from '$app/navigation';
    import { localizeHref } from '$lib/paraglide/runtime';
    import { onDestroy } from 'svelte';
    import * as m from '$lib/paraglide/messages';
    import CodeInput from '$lib/components/share/join-share/CodeInput.svelte';

    type JoinState =
        | { step: 'input'; errorMessage?: string }
        | { step: 'connecting' }
        | { step: 'sas'; emojis: string[] }
        | { step: 'decrypting' }
        | { step: 'error'; message: string };

    let joinState = $state<JoinState>({ step: 'input' });

    let abortController: AbortController | null = null;

    onDestroy(() => cancelSession());

    function cancelSession() {
        if (abortController) {
            abortController.abort();
            abortController = null;
        }
        joinState = { step: 'input' };
    }

    function showError(message: string) {
        joinState = { step: 'error', message };
    }

    async function handleJoin(code: string): Promise<boolean> {
        try {
            joinState = { step: 'connecting' };
            abortController = new AbortController();

            const keyPair = await generateEcdhKeyPair();
            const pubKeyBytes = await exportPublicKey(keyPair.publicKey);

            const stream = shareClient.join(
                create(JoinRequestSchema, {
                    code,
                    publicKey: pubKeyBytes,
                }),
                { signal: abortController.signal }
            );

            let sharedSecretKey: CryptoKey | null = null;

            for await (const resp of stream) {
                switch (resp.event.case) {
                    case 'sessionConnected': {
                        const senderPk = resp.event.value.senderPublicKey;
                        sharedSecretKey = await deriveSharedSecretKey(
                            keyPair.privateKey,
                            senderPk
                        );
                        const emojis = await generateSasEmojis(sharedSecretKey);
                        joinState = { step: 'sas', emojis };
                        break;
                    }

                    case 'keyReceived': {
                        joinState = { step: 'decrypting' };

                        const { encryptedFolderKey, folderId } =
                            resp.event.value;

                        const folderKey = await decryptFolderKey(
                            sharedSecretKey!!,
                            encryptedFolderKey
                        );
                        const keyString = await exportKey(folderKey);

                        await goto(
                            localizeHref(`/${folderId!!.value}#${keyString}`)
                        );
                        return true;
                    }
                }
            }
        } catch (err: any) {
            if (abortController?.signal.aborted) return false;
            showError(err?.message || String(err));

            return false;
        }

        return true;
    }
</script>

<div
    class="bg-card border-muted-foreground flex w-full flex-col gap-3 border border-dashed p-4"
>
    <h3 class="text-md flex items-center gap-2 font-semibold">
        <KeyRound class="text-primary h-5 w-5" />
        {m['share.modal.title']()}
    </h3>

    {#if joinState.step === 'input' || joinState.step === 'error'}
        {@const s = joinState}

        <div class="flex flex-col items-center justify-center gap-2">
            <p class="text-sm font-medium">{m['share.modal.enter-code']()}</p>

            <CodeInput length={6} onComplete={handleJoin} />

            {#if s.step === 'input' && s.errorMessage}
                <span class="text-destructive text-sm">{s.errorMessage}</span>
            {:else if s.step === 'error'}
                <span class="text-destructive text-sm">{s.message}</span>
            {/if}
        </div>
    {:else if joinState.step === 'connecting'}
        <div
            class="text-muted-foreground flex flex-col items-center justify-center gap-3 py-4"
        >
            <div class="flex items-center gap-2">
                <LoaderCircle class="text-primary h-5 w-5 animate-spin" />
                {m['share.session.connecting']()}
            </div>
            <button
                type="button"
                class="bg-muted hover:bg-muted/80 text-foreground cursor-pointer px-3 py-1 text-xs"
                onclick={cancelSession}
            >
                {m['common.actions.close']()}
            </button>
        </div>
    {:else if joinState.step === 'sas'}
        {@const s = joinState}
        <div class="flex flex-col items-center justify-center gap-3 py-2">
            <div class="text-muted-foreground text-center text-xs">
                {m['share.verification.verify-sender']()}
            </div>
            <div
                class="bg-muted flex max-w-full flex-wrap items-center justify-center gap-2 p-4 sm:gap-4"
            >
                {#each s.emojis as emoji}
                    <span class="text-2xl select-none sm:text-4xl">{emoji}</span
                    >
                {/each}
            </div>
            <div class="text-primary flex items-center gap-2 text-xs">
                <LoaderCircle class="h-3 w-3 animate-spin" />
                {m['share.session.waiting-sender']()}
            </div>
            <button
                type="button"
                class="bg-muted hover:bg-muted/80 text-foreground mt-2 cursor-pointer px-3 py-1 text-xs"
                onclick={cancelSession}
            >
                {m['common.actions.close']()}
            </button>
        </div>
    {:else if joinState.step === 'decrypting'}
        <div
            class="text-muted-foreground flex items-center justify-center gap-2 py-4"
        >
            <LoaderCircle class="text-primary h-5 w-5 animate-spin" />
            {m['share.session.decrypting']()}
        </div>
    {/if}
</div>
