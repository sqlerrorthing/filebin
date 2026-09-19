<script lang="ts">
    import type {Row} from "$lib/components/files-table/types";

    let { row = $bindable() }: {
        row: Row
    } = $props();
</script>

<p>{row.kind}</p>
