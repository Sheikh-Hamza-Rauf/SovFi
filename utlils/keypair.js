// utils/keypair.js
// Utility functions for generating and managing Solana keypairs

const { Keypair } = require('@solana/web3.js');
const fs = require('fs');

/**
 * Generate a new Solana keypair
 * @returns {Object} - Keypair information
 */
function generateKeypair() {
  const keypair = Keypair.generate();
  
  return {
    publicKey: keypair.publicKey.toBase58(),
    secretKey: Buffer.from(keypair.secretKey).toString('base64'),
    secretKeyArray: Array.from(keypair.secretKey)
  };
}

/**
 * Load keypair from a JSON file (Solana CLI format)
 * @param {string} filepath - Path to the keypair JSON file
 * @returns {Object} - Keypair information
 */
function loadKeypairFromFile(filepath) {
  const secretKey = JSON.parse(fs.readFileSync(filepath, 'utf-8'));
  const keypair = Keypair.fromSecretKey(Uint8Array.from(secretKey));
  
  return {
    publicKey: keypair.publicKey.toBase58(),
    secretKey: Buffer.from(keypair.secretKey).toString('base64'),
    secretKeyArray: Array.from(keypair.secretKey)
  };
}

/**
 * Convert base64 secret key to public key
 * @param {string} base64SecretKey - Base64 encoded secret key
 * @returns {string} - Public key as base58 string
 */
function getPublicKeyFromBase64(base64SecretKey) {
  const secretKey = Uint8Array.from(Buffer.from(base64SecretKey, 'base64'));
  const keypair = Keypair.fromSecretKey(secretKey);
  return keypair.publicKey.toBase58();
}

/**
 * Save keypair to file in Solana CLI format
 * @param {Object} keypair - Keypair object with secretKeyArray
 * @param {string} filepath - Path to save the keypair
 */
function saveKeypairToFile(keypair, filepath) {
  fs.writeFileSync(filepath, JSON.stringify(keypair.secretKeyArray));
  console.log(`Keypair saved to ${filepath}`);
}

// CLI usage
if (require.main === module) {
  const command = process.argv[2];
  
  switch (command) {
    case 'generate':
      console.log('\n=== Generating New Keypair ===\n');
      const newKeypair = generateKeypair();
      console.log('Public Key:', newKeypair.publicKey);
      console.log('Secret Key (Base64):', newKeypair.secretKey);
      console.log('\n⚠️  Store the secret key securely! Never share it!\n');
      
      // Optionally save to file
      if (process.argv[3]) {
        saveKeypairToFile(newKeypair, process.argv[3]);
      }
      break;
      
    case 'load':
      if (!process.argv[3]) {
        console.error('Usage: node keypair.js load <filepath>');
        process.exit(1);
      }
      console.log('\n=== Loading Keypair from File ===\n');
      const loadedKeypair = loadKeypairFromFile(process.argv[3]);
      console.log('Public Key:', loadedKeypair.publicKey);
      console.log('Secret Key (Base64):', loadedKeypair.secretKey);
      break;
      
    case 'pubkey':
      if (!process.argv[3]) {
        console.error('Usage: node keypair.js pubkey <base64_secret_key>');
        process.exit(1);
      }
      console.log('\n=== Getting Public Key ===\n');
      const publicKey = getPublicKeyFromBase64(process.argv[3]);
      console.log('Public Key:', publicKey);
      break;
      
    default:
      console.log(`
Solana Keypair Utility

Usage:
  node keypair.js generate [output_file]    Generate a new keypair
  node keypair.js load <filepath>           Load keypair from file
  node keypair.js pubkey <base64_key>       Get public key from secret key

Examples:
  node keypair.js generate
  node keypair.js generate ./my-keypair.json
  node keypair.js load ~/.config/solana/id.json
  node keypair.js pubkey "YOUR_BASE64_SECRET_KEY"
      `);
  }
}

module.exports = {
  generateKeypair,
  loadKeypairFromFile,
  getPublicKeyFromBase64,
  saveKeypairToFile
};s