<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.ico";

    let { children } = $props();
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

<div class="bg-background text-foreground">
    <main class="min-h-screen">
        {@render children()}
    </main>
</div>
