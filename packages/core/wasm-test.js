// @ts-check

const core = require('./pkg/agikit_core');

const encryptionKey = core.getXorEncryptionKey();
console.log('Encryption Key:', encryptionKey);

const plaintext = 'Hello world!';
const xored = core.xorBuffer(Buffer.from(plaintext, 'ascii'), core.getXorEncryptionKey());
console.log('XORed:', xored);

const unxored = core.xorBuffer(xored, core.getXorEncryptionKey());
console.log('Unxored:', unxored);
console.log('Unxored as string:', Buffer.from(unxored).toString('ascii'));
