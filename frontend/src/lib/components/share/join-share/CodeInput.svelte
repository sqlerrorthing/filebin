<script lang="ts">
    import { cn } from "$lib/utils";

    let {
        length = 6,
        pattern = /[0-9]$/,
        uppercase = true,
        onComplete,
    }: {
        length?: number;
        pattern?: RegExp;
        uppercase?: boolean;
        onComplete: (code: string) => boolean | Promise<boolean>;
    } = $props();

    let values = $state<string[]>([]);

    $effect(() => {
        values = Array(length).fill("");
    });

    let inputRefs = $state<HTMLInputElement[]>([]);

    function handleInput(index: number, e: Event) {
        const target = e.target as HTMLInputElement;
        let val = target.value.slice(-1);

        if (!val) {
            values[index] = "";
            return;
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

        checkCompletion(index + 1);
    }

    function handleKeyDown(index: number, e: KeyboardEvent) {
        if (e.key === "Backspace") {
            e.preventDefault();

            if (values[index]) {
                values[index] = "";
            } else if (index > 0) {
                inputRefs[index - 1]?.focus();
                inputRefs[index - 1]?.select();
            }
        } else if (e.key === "ArrowLeft") {
            const canMove = index > 0 || e.shiftKey;
            if (!canMove) return;

            e.preventDefault();

            if (e.shiftKey) {
                values.push(...values.splice(0, 1));
            }

            const nextIndex = e.shiftKey
                ? (index - 1 + length) % length
                : index - 1;
            inputRefs[nextIndex]?.focus();
            inputRefs[nextIndex]?.select();
        } else if (e.key === "ArrowRight") {
            const canMove = index < length - 1 || e.shiftKey;
            if (!canMove) return;

            e.preventDefault();

            if (e.shiftKey) {
                values.unshift(...values.splice(-1));
            }

            const nextIndex = e.shiftKey ? (index + 1) % length : index + 1;
            inputRefs[nextIndex]?.focus();
            inputRefs[nextIndex]?.select();
        }
    }

    function handlePaste(startIndex: number, e: ClipboardEvent) {
        e.stopPropagation();
        e.preventDefault();
        let pasteData = e.clipboardData?.getData("text").trim() || "";

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

        checkCompletion(nextFocus);
    }

    function handleFocus(e: FocusEvent) {
        const target = e.target as HTMLInputElement;
        target.select();
    }

    async function checkCompletion(next_idx: number) {
        const fullCode = values.join("");
        if (fullCode.length === length) {
            const success = await onComplete(fullCode);
            if (success) {
                values = Array(length).fill("");
                inputRefs[0]?.focus();
            }
        } else if (next_idx >= values.length) {
            for (let i = 0; i < values.length; i++) {
                if (values[i].trim().length == 0) {
                    inputRefs[i]?.focus();
                    break;
                }
            }
        }
    }
</script>

<div class="mx-auto flex w-full max-w-sm justify-center gap-1.5 sm:gap-2">
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
                `border-muted-foreground bg-background text-foreground
                focus:border-primary aspect-5/6 max-w-10 min-w-0 flex-1 border
                border-dashed p-0 text-center font-mono
                text-[clamp(1rem,4vw,1.125rem)] tracking-widest outline-none
                focus:border-solid`,
                (values[i]?.trim().length ?? 0) > 0 &&
                    "border-primary border-solid",
                uppercase && "uppercase"
            )}
        />
    {/each}
</div>
