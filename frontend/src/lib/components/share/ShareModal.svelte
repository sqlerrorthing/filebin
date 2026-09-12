<script lang="ts">
    import { activeFolder } from '$lib/stores/folder.svelte';
    import { shareClient } from '$lib/grpc';
    import { create } from '@bufbuild/protobuf';
    import {
        ShareRequestSchema,
        SendKeyRequestSchema,
    } from '$lib/grpc/gen/folder/v1/share_pb';
    import {
        generateEcdhKeyPair,
        exportPublicKey,
        deriveSharedSecretKey,
        generateSasEmojis,
        encryptFolderKey,
    } from '$lib/crypt/share';
    import { LoaderCircle, Check, ShieldCheck, Copy, X } from '@lucide/svelte';
    import { onDestroy } from 'svelte';
    import * as m from '$lib/paraglide/messages';

    let { onClose } = $props<{ onClose: () => void }>();

    type ShareState =
        | { step: 'loading' }
        | { step: 'waiting'; code: string; ttl: number }
        | {
              step: 'sas';
              emojis: string[];
              sharedSecretKey: CryptoKey;
              sessionId: string;
          }
        | { step: 'success' }
        | { step: 'error'; message: string };

    let shareState = $state<ShareState>({ step: 'loading' });
    let copied = $state(false);

    let secondsLeft = $state(0);

    $effect(() => {
        if (shareState.step !== 'waiting') return;

        secondsLeft = shareState.ttl;

        const interval = setInterval(() => {
            secondsLeft = Math.max(0, secondsLeft - 1);
            if (secondsLeft === 0) {
                clearInterval(interval);
            }
        }, 1000);

        return () => clearInterval(interval);
    });

    let formattedTime = $derived(() => {
        const minutes = Math.floor(secondsLeft / 60);
        const seconds = secondsLeft % 60;
        return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
    });

    const abortController = new AbortController();
    onDestroy(() => abortController.abort());

    function handleClose() {
        abortController.abort();
        onClose();
    }

    function showError(message: string) {
        shareState = { step: 'error', message };
    }

    async function startSharing() {
        if (!activeFolder.id || !activeFolder.key) {
            return showError('No active folder or key');
        }

        try {
            shareState = { step: 'loading' };
            const keyPair = await generateEcdhKeyPair();
            const pubKeyBytes = await exportPublicKey(keyPair.publicKey);

            const stream = shareClient.share(
                create(ShareRequestSchema, {
                    folderId: activeFolder.id,
                    publicKey: pubKeyBytes,
                }),
                { signal: abortController.signal }
            );

            for await (const resp of stream) {
                const sessionId = resp.sessionId;

                switch (resp.event.case) {
                    case 'codeRotated':
                        if (
                            shareState.step === 'loading' ||
                            shareState.step === 'waiting'
                        ) {
                            shareState = {
                                step: 'waiting',
                                code: resp.event.value.code,
                                ttl: resp.event.value.ttlSeconds,
                            };
                        }
                        break;

                    case 'receiverJoined': {
                        const sharedSecretKey = await deriveSharedSecretKey(
                            keyPair.privateKey,
                            resp.event.value.receiverPublicKey
                        );
                        const emojis = await generateSasEmojis(sharedSecretKey);

                        shareState = {
                            step: 'sas',
                            emojis,
                            sharedSecretKey,
                            sessionId,
                        };
                        break;
                    }

                    case 'sessionClosed':
                        if (
                            (shareState.step as ShareState['step']) !==
                            'success'
                        )
                            handleClose();
                        return;
                }
            }
        } catch (e: any) {
            if (abortController.signal.aborted) return;
            showError(e?.message || String(e));
        }
    }

    async function confirmAndSendKey() {
        if (shareState.step !== 'sas' || !activeFolder.key) return;

        const { sharedSecretKey, sessionId } = shareState;

        try {
            shareState = { step: 'loading' };
            const encryptedKey = await encryptFolderKey(
                sharedSecretKey,
                activeFolder.key
            );

            await shareClient.sendKey(
                create(SendKeyRequestSchema, {
                    sessionId,
                    encryptedFolderKey: encryptedKey,
                })
            );

            shareState = { step: 'success' };
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

<div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
>
    <div
        class="bg-card relative flex w-full max-w-md flex-col gap-4 border p-6 shadow-lg"
    >
        <button
            class="text-muted-foreground hover:text-foreground absolute top-4 right-4 cursor-pointer"
            onclick={handleClose}
        >
            <X class="h-5 w-5" />
        </button>

        <h2 class="flex items-center gap-2 text-xl font-bold">
            <ShieldCheck class="text-primary h-6 w-6" />
            {m["share.modal.title"]()}
        </h2>

        {#if shareState.step === 'loading'}
            <div class="flex flex-col items-center justify-center gap-3 py-8">
                <LoaderCircle class="text-primary h-8 w-8 animate-spin" />
                <p class="text-muted-foreground">{m["share.session.init"]()}</p>
            </div>
        {:else if shareState.step === 'waiting'}
            {@const waitingState = shareState}
            <div class="flex flex-col items-center justify-center gap-4 py-6">
                <p class="text-muted-foreground text-center text-sm">
                    {m["share.modal.action-hint"]()}
                </p>
                <div class="bg-muted flex items-center gap-3 p-4">
                    <span class="font-mono text-3xl font-bold tracking-widest"
                        >{waitingState.code}</span
                    >
                    <button
                        class="hover:bg-background cursor-pointer border p-2"
                        onclick={() => copyCode(waitingState.code)}
                        title={m["common.actions.copy-code"]()}
                    >
                        {#if copied}
                            <Check class="h-5 w-5 text-green-500" />
                        {:else}
                            <Copy class="h-5 w-5" />
                        {/if}
                    </button>
                </div>
                <div
                    class="text-muted-foreground flex items-center gap-2 text-xs"
                >
                    <LoaderCircle class="h-3 w-3 animate-spin" />
                    <span>{formattedTime()}</span>
                </div>
            </div>
        {:else if shareState.step === 'sas'}
            <div class="flex flex-col items-center justify-center gap-4 py-4">
                <div class="bg-primary/10 text-primary p-3 text-center text-sm">
                    {m["share.verification.match-prompt"]()}
                </div>
                <div
                    class="bg-muted flex max-w-full flex-wrap items-center justify-center gap-2 p-4 sm:gap-4"
                >
                    {#each shareState.emojis as emoji}
                        <span class="text-2xl select-none sm:text-4xl"
                            >{emoji}</span
                        >
                    {/each}
                </div>
                <p class="text-muted-foreground text-center text-xs">
                    {m["share.verification.instruction"]()}
                </p>
                <button
                    class="bg-primary text-primary-foreground flex w-full cursor-pointer items-center justify-center gap-2 py-2 font-medium hover:opacity-90"
                    onclick={confirmAndSendKey}
                >
                    <Check class="h-4 w-4" />
                    {m["share.verification.confirm-btn"]()}
                </button>
            </div>
        {:else if shareState.step === 'success'}
            <div class="flex flex-col items-center justify-center gap-4 py-8">
                <div
                    class="flex h-12 w-12 items-center justify-center text-green-500"
                >
                    <Check class="h-6 w-6" />
                </div>
                <p class="text-center font-medium">{m["share.session.success"]()}</p>
                <button
                    class="bg-muted hover:bg-muted/80 cursor-pointer px-4 py-2 text-sm"
                    onclick={handleClose}
                >
                    {m["common.actions.close"]()}
                </button>
            </div>
        {:else if shareState.step === 'error'}
            <div class="flex flex-col items-center justify-center gap-4 py-6">
                <p class="text-destructive text-center">{shareState.message}</p>
                <button
                    class="bg-muted hover:bg-muted/80 cursor-pointer px-4 py-2 text-sm"
                    onclick={handleClose}
                >
                    {m["common.actions.close"]()}
                </button>
            </div>
        {/if}
    </div>
</div>
