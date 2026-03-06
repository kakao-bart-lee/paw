export interface E2eeConfig {
    publicKey: string;
    privateKey: string;
}
export declare function looksLikeCiphertext(content: string): boolean;
export declare function encryptContent(recipientPubKeyB64: string, plaintext: string): string;
export declare function decryptContent(privKeyB64: string, ciphertextB64: string): string | null;
