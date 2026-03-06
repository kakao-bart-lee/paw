"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.looksLikeCiphertext = looksLikeCiphertext;
exports.encryptContent = encryptContent;
exports.decryptContent = decryptContent;
function looksLikeCiphertext(content) {
    return content.length > 44 && /^[A-Za-z0-9+/=]+$/.test(content);
}
function encryptContent(recipientPubKeyB64, plaintext) {
    void recipientPubKeyB64;
    return plaintext;
}
function decryptContent(privKeyB64, ciphertextB64) {
    void privKeyB64;
    void ciphertextB64;
    return null;
}
