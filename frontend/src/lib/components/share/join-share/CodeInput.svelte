<script lang="ts">
    import {cn} from "$lib/utils";

    let {
        length = 6,
        pattern = /[0-9]$/,
        uppercase = true,
        onComplete
    }: {
        length?: number;
        pattern?: RegExp;
        uppercase?: boolean;
        onComplete: (code: string) => boolean | Promise<boolean>;
    } = $props();

    let values = $state<string[]>([]);

    $effect(() => {
        values = Array(length).fill('');
    });

    let inputRefs = $state<HTMLInputElement[]>([]);

    function handleInput(index: number, e: Event) {
        const target = e.target as HTMLInputElement;
        let val = target.value.slice(-1);

        if (!val) {
            values[index] = '';
            return
        }

        if (uppercase) {
            val = val.toUpperCase();
        }

        if (pattern.test(val)) {
            values[index] = val;
            if (index < length - 1) {
                inputRefs[index + 1]?.focus();
                inputRefs[index + 1]?.select();
            }
        } else {
            target.value = values[index];
        }

        checkCompletion();
    }

    function handleKeyDown(index: number, e: KeyboardEvent) {
        if (e.key === 'Backspace') {
            e.preventDefault();

            if (values[index]) {
                values[index] = '';
            } else if (index > 0) {
                inputRefs[index - 1]?.focus();
                inputRefs[index - 1]?.select();
            }
        } else if (e.key === 'ArrowLeft' && index > 0) {
            e.preventDefault();
            inputRefs[index - 1]?.focus();
            inputRefs[index - 1]?.select();
        } else if (e.key === 'ArrowRight' && index < length - 1) {
            e.preventDefault();
            inputRefs[index + 1]?.focus();
            inputRefs[index + 1]?.select();
        }
    }

    function handlePaste(startIndex: number, e: ClipboardEvent) {
        e.stopPropagation();
        e.preventDefault();
        let pasteData = e.clipboardData?.getData('text').trim() || '';

        if (uppercase) {
            pasteData = pasteData.toUpperCase();
        }

        let charIndex = startIndex;
        for (let i = 0; i < pasteData.length && charIndex < length; i++) {
            const char = pasteData[i];
            if (pattern.test(char)) {
                values[charIndex] = char;
                charIndex++;
            }
        }

        const nextFocus = Math.min(charIndex, length - 1);
        inputRefs[nextFocus]?.focus();
        inputRefs[nextFocus]?.select();

        checkCompletion();
    }

    function handleFocus(e: FocusEvent) {
        const target = e.target as HTMLInputElement;
        target.select();
    }

    async function checkCompletion() {
        const fullCode = values.join('');
        if (fullCode.length === length) {
            const success = await onComplete(fullCode);
            if (success) {
                values = Array(length).fill('');
                inputRefs[0]?.focus();
            }
        }
    }
</script>

<div class="flex justify-center gap-1.5 sm:gap-2 w-full max-w-sm mx-auto">
    {#each Array(length) as _, i}
        <input
                bind:this={inputRefs[i]}
                type="text"
                inputmode="text"
                maxlength="1"
                value={values[i]}
                oninput={(e) => handleInput(i, e)}
                onkeydown={(e) => handleKeyDown(i, e)}
                onfocus={handleFocus}
                onpaste={(e) => handlePaste(i, e)}
                class={cn(
                    "flex-1 min-w-0 aspect-5/6 max-w-10 text-center border border-dashed border-muted-foreground bg-background text-foreground tracking-widest font-mono text-[clamp(1rem,4vw,1.125rem)] p-0 outline-none focus:border-solid focus:border-primary",
                    (values[i]?.trim().length ?? 0) > 0 && "border-solid border-primary",
                    uppercase && "uppercase"
                )}
        />
    {/each}
</div>