<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.ico";
    import { ModeWatcher } from "mode-watcher";
    import Header from "$lib/components/header/Header.svelte";

    let { children } = $props();


</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />

<div class="bg-background text-foreground min-h-screen">
    <header class="border-border h-16 border-b">
        <Header />
    </header>

    <main class="flex min-h-[calc(100vh-4rem)] flex-col">
        {@render children()}
    </main>
</div>
