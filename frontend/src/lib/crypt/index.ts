import  {AlgorithmSchema, type EncryptedBlobs, EncryptedBlobsSchema, EncryptedVaultSchema, VersionSchema} from "$lib/grpc/gen/folder/v1/encryption_pb";
import {create} from "@bufbuild/protobuf";

export async function generateCryptoKey(): Promise<CryptoKey> {
    return await window.crypto.subtle.generateKey(
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
}

export async function exportKey(key: CryptoKey): Promise<string> {
    const exportedKey = await window.crypto.subtle.exportKey("raw", key);
    const bytes = new Uint8Array(exportedKey);

    return bytes.toBase64({ alphabet: "base64url", omitPadding: true });
}

function bufferToBase64(buffer: ArrayBuffer | Uint8Array): string {
    const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    let binary = "";
    for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
}

export async function importKeyFromUrlSafe(urlSafeString: string): Promise<CryptoKey> {
    const bytes = Uint8Array.fromBase64(urlSafeString, { alphabet: "base64url" });

    return await window.crypto.subtle.importKey(
        "raw",
        bytes,
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
}

export async function encryptBlob(
    key: CryptoKey,
    array: Uint8Array<ArrayBuffer>
): Promise<EncryptedBlobs> {
    const iv = window.crypto.getRandomValues(new Uint8Array(12));

    const encryptedName = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv },
        key,
        array
    )

    const encryptedBytes = new Uint8Array(encryptedName);
    const ciphertext = encryptedBytes.slice(0, encryptedBytes.length - 16);
    const tag = encryptedBytes.slice(encryptedBytes.length - 16);

    const version = create(VersionSchema, {
        value: 1
    })

    const algo = create(AlgorithmSchema, {
        value: "aes-256-gcm"
    });

    const encryptedVault = create(EncryptedVaultSchema, {
        iv: bufferToBase64(iv),
        tag: bufferToBase64(tag),
        version,
        algo,
    });

    return create(EncryptedBlobsSchema, {
        meta: encryptedVault,
        data: ciphertext
    })
}

function base64ToBuffer(base64: string): Uint8Array {
    return Uint8Array.fromBase64(base64, { alphabet: "base64" });
}

export async function decryptBlob(
    key: CryptoKey,
    encryptedBlob: EncryptedBlobs
): Promise<Uint8Array> {
    if (!encryptedBlob.meta) {
        throw new Error("Missing encrypted blob metadata");
    }

    const { iv: ivBase64, tag: tagBase64 } = encryptedBlob.meta;
    const ciphertext = encryptedBlob.data;

    if (!ivBase64 || !tagBase64 || !ciphertext) {
        throw new Error("Invalid encrypted blob structure: missing iv, tag or data");
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
}

export async function decryptBlobAsString(
    key: CryptoKey,
    encryptedBlob: EncryptedBlobs
): Promise<string> {
    const bytes = await decryptBlob(key, encryptedBlob);
    return new TextDecoder().decode(bytes);
}