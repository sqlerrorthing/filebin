<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import ActionButton from "./ActionButton.svelte";
    import UploadButton from "./UploadButton.svelte";
    import Container from "$lib/components/container/Container.svelte";
    import { MoveRight } from "@lucide/svelte";
</script>

<svelte:head>
    <title>{m["page.index.title"]()} — Filebin</title>
    <meta name="description" content={m["page.index.description"]()} />

    <meta property="og:title" content={m["page.index.title"]()} />
    <meta property="og:description" content={m["page.index.description"]()} />
</svelte:head>

<Container class="md:my-auto">
    <div
        class="flex w-full flex-col gap-4 md:flex-row md:items-center
            md:justify-between"
    >
        <div class="flex flex-col gap-4">
            <h2 class="text-muted-foreground text-sm uppercase">
                {m["index.subtitle"]()}
            </h2>

            <h1
                class="text-[clamp(46px,7vw,82px)] leading-[.94] font-bold
                    tracking-[-.065em] whitespace-pre-line"
            >
                {m["index.title"]()}
            </h1>

            <p class="text-md text-muted-foreground max-w-lg font-bold">
                {m["index.description"]()}
            </p>
        </div>

        <div
            class="border-card-foreground flex flex-col gap-2 rounded-2xl border
                border-solid p-2 shadow-xl"
        >
            <UploadButton />

            <ActionButton
                displayName={m["index.actions.join.name"]}
                description={m["index.actions.join.description"]}
            >
                {#snippet icon()}
                    <MoveRight class="h-full w-full" />
                {/snippet}
            </ActionButton>
        </div>
    </div>
</Container>
