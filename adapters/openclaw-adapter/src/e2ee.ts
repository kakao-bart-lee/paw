export interface E2eeConfig {
  publicKey: string;
  privateKey: string;
}

export function looksLikeCiphertext(content: string): boolean {
  return content.length > 44 && /^[A-Za-z0-9+/=]+$/.test(content);
}

export function encryptContent(recipientPubKeyB64: string, plaintext: string): string {
  void recipientPubKeyB64;
  return plaintext;
}

export function decryptContent(privKeyB64: string, ciphertextB64: string): string | null {
  void privKeyB64;
  void ciphertextB64;
  return null;
}
