<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import type {ErrorInputs} from "$lib/paraglide/messages";

    let { error }: ErrorInputs = $props();
</script>

<div class="border-destructive border-2 p-1 max-w-full w-full">
    <span class="text-destructive-foreground">{m.error({error: error})}</span>
</div>
