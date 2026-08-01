<script lang="ts">
    import {folderClient, useGrpc} from "$lib/grpc";
    import {setLocale, getLocale, locales} from '$lib/paraglide/runtime';
    import {cn} from "$lib/utils";

    const limits = useGrpc(folderClient.limits);
    const currentLocale = getLocale();

    import * as m from '$lib/paraglide/messages';
</script>

<button onclick={() => limits.call({})} disabled={limits.loading}>
    {limits.loading ? 'Loading...' : m.example_message({username: "penis"})}
</button>

<div class="language-switcher">
    {#each locales as l}
        <button
            class={cn(
              "flex items-center px-4 py-2 rounded-lg font-medium transition-colors", // Общие базовые классы
              l === currentLocale ? "bg-black text-white" : "bg-gray-100 text-gray-700 hover:bg-gray-200" // Условия
            )}
            onclick={() => setLocale(l)}
        >
            {l.toUpperCase()}
        </button>
    {/each}
</div>

{#if limits.error}
    <p style="color: red">{limits.error.message}</p>
{/if}

{#if limits.data}
    <pre>{limits.data.maxFileSize.toString()}</pre>
{/if}
