<script lang="ts">
    import './layout.css';
    import favicon from '$lib/assets/favicon.ico';
    import {limitsStore} from "$lib/stores/limits.svelte";
    import {onMount} from "svelte";
    import * as m from "$lib/paraglide/messages";
    import {ParaglideMessage} from '@inlang/paraglide-js-svelte';
    import LanguageSwitch from "./LanguageSwitch.svelte";

    let {children} = $props();

    onMount(async () => {
        if (!limitsStore.data && !limitsStore.loading) {
            await limitsStore.fetch();
        }
    });
</script>

<svelte:head>
    <link rel="icon" href={favicon}/>
</svelte:head>

<div class="flex justify-center p-4">
    <div class="bg-card sm:min-w-lg min-w-full shadow">
        <div class="flex flex-col items-center justify-center sm:flex-row gap-4 text-center sm:text-left border-b-2 border-dashed border-muted pb-2 p-4">
            <img src={favicon} class="sm:w-24 w-32 h-auto object-cover" alt="">
            <h2 class="whitespace-pre-line text-md">{m['head.title']()}</h2>
        </div>
        <div class="p-4">
            {@render children()}
        </div>
        <div class="p-4 border-t-2 border-dashed border-muted flex justify-center flex-col items-center">
            <LanguageSwitch />

            <span class="text-muted-foreground text-sm">
              <ParaglideMessage message={m["footer.made"]} inputs={{author: "s", authorUrl: "https://github.com/sqlerrorthing"}}>
                {#snippet link({ children, options })}
                    <a href={options.to as string} class="text-foreground hover:underline hover:text-primary" target="_blank">
                        {@render children?.()}
                    </a>
                {/snippet}
              </ParaglideMessage>
            </span>

            <span class="text-muted-foreground text-sm">
              <ParaglideMessage message={m["footer.source"]} inputs={{url: "https://github.com/sqlerrorthing/filebin", platform: "Github"}}>
                {#snippet link({ children, options })}
                    <a href={options.to as string} class="text-foreground hover:underline hover:text-primary" target="_blank">
                        {@render children?.()}
                    </a>
                {/snippet}
              </ParaglideMessage>
            </span>
        </div>
    </div>
</div>

<!--{#if limitsStore.loading}-->
<!--    <div class="flex items-center justify-center h-screen">-->
<!--        <LoaderCircle class="h-12 w-12 animate-spin text-primary" />-->
<!--    </div>-->
<!--{:else if limitsStore.error}-->
<!--    <div class="flex items-center justify-center h-screen text-destructive-foreground">-->
<!--        <p>{m.error({message: limitsStore.error.message})}</p>-->
<!--    </div>-->
<!--{:else if limitsStore.data}-->

<!--{/if}-->
