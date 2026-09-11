<script lang="ts">
    import './layout.css';
    import favicon from '$lib/assets/favicon.ico';
    import { limitsStore } from '$lib/stores/limits.svelte';
    import { onMount } from 'svelte';
    import * as m from '$lib/paraglide/messages';
    import { ParaglideMessage } from '@inlang/paraglide-js-svelte';
    import LanguageSwitch from './LanguageSwitch.svelte';

    let { children } = $props();

    onMount(async () => {
        if (!limitsStore.data && !limitsStore.hasCalled) {
            await limitsStore.fetch();
        }
    });
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

<div class="flex justify-center p-4">
    <div class="bg-card min-w-full shadow sm:min-w-lg">
        <div
            class="border-muted flex flex-col items-center justify-center gap-4 border-b-2 border-dashed p-4 pb-2 text-center sm:flex-row sm:text-left"
        >
            <img
                src={favicon}
                class="h-auto w-32 min-w-32 object-cover sm:w-24 sm:min-w-24"
                alt=""
            />
            <h2 class="text-md whitespace-pre-line">{m['head.title']()}</h2>
        </div>
        <div class="p-4">
            {@render children()}
        </div>
        <div
            class="border-muted flex flex-col items-center justify-center border-t-2 border-dashed p-4"
        >
            <LanguageSwitch />

            <span class="text-muted-foreground text-sm">
                <ParaglideMessage
                    message={m['footer.made']}
                    inputs={{
                        author: 's',
                        authorUrl: 'https://github.com/sqlerrorthing',
                    }}
                >
                    {#snippet link({ children, options })}
                        <a
                            href={options.to as string}
                            class="text-foreground hover:text-primary hover:underline"
                            target="_blank"
                        >
                            {@render children?.()}
                        </a>
                    {/snippet}
                </ParaglideMessage>
            </span>

            <span class="text-muted-foreground text-sm">
                <ParaglideMessage
                    message={m['footer.source']}
                    inputs={{
                        url: 'https://github.com/sqlerrorthing/filebin',
                        platform: 'Github',
                    }}
                >
                    {#snippet link({ children, options })}
                        <a
                            href={options.to as string}
                            class="text-foreground hover:text-primary hover:underline"
                            target="_blank"
                        >
                            {@render children?.()}
                        </a>
                    {/snippet}
                </ParaglideMessage>
            </span>
        </div>
    </div>
</div>
