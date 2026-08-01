import {type CallOptions, ConnectError} from "@connectrpc/connect";

export function useGrpc<Req, Res>(
    rpcMethod: (request: Req, options?: CallOptions) => Promise<Res>
) {
    let data = $state<Res | null>(null);
    let loading = $state(false);
    let error = $state<ConnectError | Error | null>(null);

    async function call(request: Req, options?: CallOptions): Promise<Res | null> {
        loading = true;
        error = null;

        try {
            data = await rpcMethod(request, options);
            return data;
        } catch (err: any) {
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
            loading = false;
        }
    }

    function reset() {
        data = null;
        error = null;
        loading = false;
    }

    return {
        get data() { return data; },
        get error() { return error; },
        get loading() { return loading; },
        call,
        reset,
    };
}
