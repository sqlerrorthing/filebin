<script lang="ts">
    import * as m from "$lib/paraglide/messages";

    let {
        basicKey,
    }: {
        basicKey: string;
    } = $props();
</script>

<button class="w-full sm:w-72" title={m[`${basicKey}.name`]()}>
    <span>
        {m[`${basicKey}.name`]()}
    </span>
    <span>
        {m[`${basicKey}.description`]()}
    </span>
</button>
