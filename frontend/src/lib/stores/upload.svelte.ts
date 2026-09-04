class UploadStore {
    #perding = $state<Array<File>>([]);

    get pending() {
        return this.#perding;
    }
}

export const uploadStore = new UploadStore();