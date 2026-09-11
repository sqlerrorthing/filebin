<script lang="ts">
    import * as m from '$lib/paraglide/messages';
    import type { ErrorInputs } from '$lib/paraglide/messages';

    let { error }: ErrorInputs = $props();
</script>

<div class="border-destructive w-full max-w-full border-2 p-1">
    <span class="text-destructive-foreground">{m.error({ error: error })}</span>
</div>
