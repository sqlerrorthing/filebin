<script lang="ts">
    import Container from "$lib/components/container/Container.svelte";

    let { children } = $props();
</script>

<Container>
    {@render children()}
</Container>
