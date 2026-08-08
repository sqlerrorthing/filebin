import {type CallOptions, ConnectError} from "@connectrpc/connect";

export function useGrpc<Req, Res>(
    rpcMethod: (request: Req, options?: CallOptions) => Promise<Res>
) {
    let data = $state<Res | null>(null);
    let loading = $state(false);
    let error = $state<ConnectError | Error | null>(null);

    let abortController: AbortController | null = null;

    async function call(request: Req, options?: CallOptions): Promise<Res | null> {
        if (abortController) {
            abortController.abort("Cancelled by a new request");
        }

        const controller = new AbortController();
        abortController = controller;
        const currentSignal = controller.signal;

        const mergedOptions: CallOptions = {
            ...options,
            signal: currentSignal
        }

        loading = true;
        error = null;

        try {
            const response = await rpcMethod(request, mergedOptions);
            if (currentSignal.aborted) return null;

            data = response;
            return data;
        } catch (err: unknown) {
            if (currentSignal.aborted) {
                return null;
            }

            console.error("gRPC Error:", err);

            if (err instanceof ConnectError) {
                error = err;
            } else if (err instanceof Error) {
                error = err;
            } else {
                error = new Error(String(err));
            }

            data = null;
            return null;
        } finally {
            if (abortController === controller) {
                loading = false;
                abortController = null;
            }
        }
    }

    function abort() {
        if (abortController) {
            abortController.abort("Manual abort");
            abortController = null;
        }
        loading = false;
    }

    function reset() {
        abort();
        data = null;
        error = null;
    }

    return {
        get data() { return data; },
        get error() { return error; },
        get loading() { return loading; },
        call,
        reset,
    };
}

export function useStreamGrpc<Req, Res>(
    rpcMethod: (request: Req, options?: CallOptions) => AsyncIterable<Res>
){
    let latestData = $state<Res | null>(null);
    let loading = $state(false);
    let error = $state<ConnectError | Error | null>(null);

    let abortController: AbortController | null = null;

    async function* call(request: Req, options?: CallOptions): AsyncGenerator<Res, void, unknown> {
        if (abortController) {
            abortController.abort("Cancelled by a new stream request");
        }

        const controller = new AbortController();
        abortController = controller;
        const currentSignal = controller.signal;

        const mergedOptions: CallOptions = {
            ...options,
            signal: currentSignal
        };

        loading = true;
        error = null;

        try {
            const stream = rpcMethod(request, mergedOptions);
            loading = false;

            for await (const chunk of stream) {
                if (currentSignal.aborted) break;

                latestData = chunk;

                yield chunk;
            }
        } catch (err: unknown) {
            if (currentSignal.aborted) return;

            console.error("gRPC Stream Error:", err);
            if (err instanceof ConnectError || err instanceof Error) {
                error = err;
            } else {
                error = new Error(String(err));
            }
        } finally {
            if (abortController === controller) {
                loading = false;
                abortController = null;
            }
        }
    }

    function abort() {
        if (abortController) {
            abortController.abort("Manual abort");
            abortController = null;
        }
        loading = false;
    }

    function reset() {
        abort();
        latestData = null;
        error = null;
    }

    return {
        get latestData() { return latestData; },
        get error() { return error; },
        get loading() { return loading; },
        call,
        reset,
        abort
    };
}
