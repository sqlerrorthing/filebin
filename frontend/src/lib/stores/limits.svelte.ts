import { folderClient, useGrpc } from "$lib/grpc";

class LimitsStore {
    #grpc = useGrpc(folderClient.limits);

    get data() {
        return this.#grpc.data;
    }

    get loading() {
        return this.#grpc.loading;
    }

    get error() {
        return this.#grpc.error;
    }

    async fetch() {
        await this.#grpc.call({});
    }
}

export const limitsStore = new LimitsStore();
