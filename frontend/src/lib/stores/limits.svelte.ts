import { folderClient, useGrpc } from "$lib/grpc";

class LimitsStore {
    #grpc = useGrpc(folderClient.limits);
    #hasCalled = $state(false);

    get data() {
        return this.#grpc.data;
    }

    get loading() {
        return !this.#hasCalled || this.#grpc.loading;
    }

    get hasCalled() {
        return this.#hasCalled
    }

    get error() {
        return this.#grpc.error;
    }

    async fetch() {
        this.#hasCalled = true;
        await this.#grpc.call({});
    }
}

export const limitsStore = new LimitsStore();
