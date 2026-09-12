import { exportKeyToArray } from "./index";

const EMOJIS = [
    "🐶",
    "🐱",
    "🦊",
    "🐼",
    "🐨",
    "🦁",
    "🐯",
    "🐰",
    "🦄",
    "🐙",
    "🦀",
    "🐬",
    "🐳",
    "🦅",
    "🦉",
    "🐝",

    "🍎",
    "🍐",
    "🍊",
    "🍋",
    "🍌",
    "🍉",
    "🍇",
    "🍓",
    "🍒",
    "🥑",
    "🍍",
    "🥥",
    "🥝",
    "🍅",
    "🍆",
    "🌽",
];

export async function generateEcdhKeyPair(): Promise<CryptoKeyPair> {
    return await window.crypto.subtle.generateKey({ name: "X25519" }, true, [
        "deriveKey",
        "deriveBits",
    ]);
}

export async function exportPublicKey(
    publicKey: CryptoKey
): Promise<Uint8Array> {
    const raw = await window.crypto.subtle.exportKey("raw", publicKey);
    return new Uint8Array(raw);
}

export async function deriveSharedSecretKey(
    privateKey: CryptoKey,
    peerPublicKeyBytes: Uint8Array
): Promise<CryptoKey> {
    const peerPublicKey = await window.crypto.subtle.importKey(
        "raw",
        new Uint8Array(peerPublicKeyBytes),
        { name: "X25519" },
        true,
        []
    );

    const sharedBits = await window.crypto.subtle.deriveBits(
        {
            name: "X25519",
            public: peerPublicKey,
        },
        privateKey,
        256
    );

    return await window.crypto.subtle.importKey(
        "raw",
        sharedBits,
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
}

export async function generateSasEmojis(
    sharedSecretKey: CryptoKey
): Promise<string[]> {
    const rawKey = await exportKeyToArray(sharedSecretKey);
    const hashBuffer = await window.crypto.subtle.digest(
        "SHA-256",
        new Uint8Array(rawKey)
    );
    const hashBytes = new Uint8Array(hashBuffer);

    const emojis: string[] = [];
    for (let i = 0; i < 6; i++) {
        const index = hashBytes[i] % EMOJIS.length;
        emojis.push(EMOJIS[index]);
    }
    return emojis;
}

export async function encryptFolderKey(
    sharedSecretKey: CryptoKey,
    folderKey: CryptoKey
): Promise<Uint8Array> {
    const folderKeyRaw = await exportKeyToArray(folderKey);
    const iv = window.crypto.getRandomValues(new Uint8Array(12));

    const ciphertext = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv },
        sharedSecretKey,
        new Uint8Array(folderKeyRaw)
    );

    const ciphertextBytes = new Uint8Array(ciphertext);
    const result = new Uint8Array(12 + ciphertextBytes.length);
    result.set(iv, 0);
    result.set(ciphertextBytes, 12);
    return result;
}

export async function decryptFolderKey(
    sharedSecretKey: CryptoKey,
    encryptedData: Uint8Array
): Promise<CryptoKey> {
    if (encryptedData.length < 12) {
        throw new Error("Invalid encrypted folder key length");
    }
    const iv = encryptedData.slice(0, 12);
    const ciphertext = encryptedData.slice(12);

    const decryptedBuffer = await window.crypto.subtle.decrypt(
        { name: "AES-GCM", iv },
        sharedSecretKey,
        new Uint8Array(ciphertext)
    );

    return await window.crypto.subtle.importKey(
        "raw",
        decryptedBuffer,
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
}
