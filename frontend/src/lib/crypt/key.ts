import {
    AlgorithmSchema,
    type EncryptedBlobs,
    EncryptedBlobsSchema,
    EncryptedVaultSchema,
    VersionSchema,
} from "$lib/grpc/gen/folder/v1/encryption_pb";
import { create } from "@bufbuild/protobuf";

export const generateCryptoKey = async (): Promise<CryptoKey> => {
    return await window.crypto.subtle.generateKey(
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
};

export const exportKey = async (key: CryptoKey): Promise<string> => {
    return (await exportKeyToArray(key)).toBase64({
        alphabet: "base64url",
        omitPadding: true,
    });
};

export const exportKeyToArray = async (key: CryptoKey): Promise<Uint8Array> => {
    const exportedKey = await window.crypto.subtle.exportKey("raw", key);
    return new Uint8Array(exportedKey);
};

export const bufferToBase64 = (buffer: ArrayBuffer | Uint8Array): string => {
    const bytes =
        buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    let binary = "";
    for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
};

export const importKeyFromUrlSafe = async (
    urlSafeString: string
): Promise<CryptoKey> => {
    const bytes = Uint8Array.fromBase64(urlSafeString, {
        alphabet: "base64url",
    });

    return await window.crypto.subtle.importKey(
        "raw",
        bytes,
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
};

export const encryptBlob = async (
    key: CryptoKey,
    array: Uint8Array<ArrayBuffer>
): Promise<EncryptedBlobs> => {
    const iv = window.crypto.getRandomValues(new Uint8Array(12));

    const encryptedName = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv },
        key,
        array
    );

    const encryptedBytes = new Uint8Array(encryptedName);
    const ciphertext = encryptedBytes.slice(0, encryptedBytes.length - 16);
    const tag = encryptedBytes.slice(encryptedBytes.length - 16);

    const version = create(VersionSchema, {
        value: 1,
    });

    const algo = create(AlgorithmSchema, {
        value: "aes-256-gcm",
    });

    const encryptedVault = create(EncryptedVaultSchema, {
        iv: bufferToBase64(iv),
        tag: bufferToBase64(tag),
        version,
        algo,
    });

    return create(EncryptedBlobsSchema, {
        meta: encryptedVault,
        data: ciphertext,
    });
};

const base64ToBuffer = (base64: string): Uint8Array => {
    return Uint8Array.fromBase64(base64, { alphabet: "base64" });
};

export const decryptBlob = async (
    key: CryptoKey,
    encryptedBlob: EncryptedBlobs
): Promise<Uint8Array> => {
    if (!encryptedBlob.meta) {
        throw new Error("Missing encrypted blob metadata");
    }

    const { iv: ivBase64, tag: tagBase64 } = encryptedBlob.meta;
    const ciphertext = encryptedBlob.data;

    if (!ivBase64 || !tagBase64 || !ciphertext) {
        throw new Error(
            "Invalid encrypted blob structure: missing iv, tag or data"
        );
    }

    const iv = base64ToBuffer(ivBase64);
    const tag = base64ToBuffer(tagBase64);

    const encryptedData = new Uint8Array(ciphertext.length + tag.length);
    encryptedData.set(ciphertext, 0);
    encryptedData.set(tag, ciphertext.length);

    const decryptedBuffer = await window.crypto.subtle.decrypt(
        { name: "AES-GCM", iv: new Uint8Array(iv) },
        key,
        encryptedData
    );

    return new Uint8Array(decryptedBuffer);
};

export const decryptBlobAsString = async (
    key: CryptoKey,
    encryptedBlob: EncryptedBlobs
): Promise<string> => {
    const bytes = await decryptBlob(key, encryptedBlob);
    return new TextDecoder().decode(bytes);
};
