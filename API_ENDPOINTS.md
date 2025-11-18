# SovFi Oracle API Endpoints for Postman

Base URL: `http://localhost:3000`

---

## Health Check

### Check Server Status
```
GET http://localhost:3000/api/health
```

---

## Initialization APIs

### 1. Initialize Oracle Program
```
POST http://localhost:3000/api/initialize
```
**Body (JSON):**
```json
{
  "authoritySecretKey": "base64_encoded_secret_key",
  "tokenMintAddress": "TokenMintPublicKeyHere",
  "rewardRate": "1000000",
  "proposalThreshold": "100000",
  "votingPeriod": "86400",
  "quorumPercentage": 51,
  "timelockDuration": "172800",
  "totalSupply": "1000000000"
}
```

### 2. Create Product Feed
```
POST http://localhost:3000/api/products/create
```
**Body (JSON):**
```json
{
  "authoritySecretKey": "base64_encoded_secret_key",
  "symbol": "BTC/USD",
  "assetType": "crypto",
  "description": "Bitcoin to USD price feed",
  "priceType": "price",
  "minPublishers": 3,
  "exponent": -8
}
```

---

## Publisher Management APIs

### 3. Add New Publisher
```
POST http://localhost:3000/api/publishers/add
```
**Body (JSON):**
```json
{
  "payerSecretKey": "base64_encoded_payer_secret_key",
  "publisherAuthoritySecretKey": "base64_encoded_publisher_secret_key",
  "name": "Publisher Name",
  "initialStake": "1000000",
  "tokenMintAddress": "TokenMintPublicKeyHere"
}
```

### 4. Get Publisher Info
```
GET http://localhost:3000/api/publishers/{publisherAddress}
```
**Example:**
```
GET http://localhost:3000/api/publishers/5ne7phA48Jrvpn39AtupB8ZkCCAy8gLTfpGihZkuDbbh
```

### 5. Stake Tokens
```
POST http://localhost:3000/api/publishers/stake
```
**Body (JSON):**
```json
{
  "publisherAuthoritySecretKey": "base64_encoded_secret_key",
  "amount": "500000",
  "tokenMintAddress": "TokenMintPublicKeyHere"
}
```

### 6. Unstake Tokens
```
POST http://localhost:3000/api/publishers/unstake
```
**Body (JSON):**
```json
{
  "publisherAuthoritySecretKey": "base64_encoded_secret_key",
  "amount": "250000"
}
```

### 7. Withdraw Unbonded Tokens
```
POST http://localhost:3000/api/publishers/withdraw-unbonded
```
**Body (JSON):**
```json
{
  "publisherAuthoritySecretKey": "base64_encoded_secret_key",
  "tokenMintAddress": "TokenMintPublicKeyHere"
}
```

---

## Price Update APIs

### 8. Update Price
```
POST http://localhost:3000/api/prices/update
```
**Body (JSON):**
```json
{
  "publisherAuthoritySecretKey": "base64_encoded_secret_key",
  "symbol": "BTC/USD",
  "price": "45000000000000",
  "confidence": "100000000"
}
```

### 9. Get Price for Symbol
```
GET http://localhost:3000/api/prices/{symbol}
```
**Example:**
```
GET http://localhost:3000/api/prices/BTC/USD
```

---

## Governance APIs

### 10. Create Proposal
```
POST http://localhost:3000/api/governance/proposals/create
```
**Body (JSON):**
```json
{
  "proposerSecretKey": "base64_encoded_secret_key",
  "proposalType": {
    "type": "UpdateRewardRate",
    "newRate": "2000000"
  },
  "description": "Increase reward rate to attract more publishers",
  "tokenMintAddress": "TokenMintPublicKeyHere"
}
```

### 11. Get Proposal Details
```
GET http://localhost:3000/api/governance/proposals/{proposalId}
```
**Example:**
```
GET http://localhost:3000/api/governance/proposals/1
```

### 12. Vote on Proposal
```
POST http://localhost:3000/api/governance/proposals/{proposalId}/vote
```
**Example:**
```
POST http://localhost:3000/api/governance/proposals/1/vote
```
**Body (JSON):**
```json
{
  "voterSecretKey": "base64_encoded_secret_key",
  "vote": "yes",
  "tokenMintAddress": "TokenMintPublicKeyHere"
}
```
**Vote options:** `yes`, `no`, `abstain`

### 13. Execute Proposal
```
POST http://localhost:3000/api/governance/proposals/{proposalId}/execute
```
**Example:**
```
POST http://localhost:3000/api/governance/proposals/1/execute
```

### 14. Execute Governance Action
```
POST http://localhost:3000/api/governance/proposals/{proposalId}/execute-action
```
**Example:**
```
POST http://localhost:3000/api/governance/proposals/1/execute-action
```
**Body (JSON):**
```json
{
  "authoritySecretKey": "base64_encoded_secret_key"
}
```

---

## Emergency APIs

### 15. Emergency Pause
```
POST http://localhost:3000/api/emergency/pause
```
**Body (JSON):**
```json
{
  "authoritySecretKey": "base64_encoded_secret_key"
}
```

### 16. Emergency Unpause
```
POST http://localhost:3000/api/emergency/unpause
```
**Body (JSON):**
```json
{
  "authoritySecretKey": "base64_encoded_secret_key"
}
```

---

## Notes

1. **Replace placeholders:**
   - `base64_encoded_secret_key` - Your actual base64 encoded secret key
   - `TokenMintPublicKeyHere` - Your token mint public key
   - `{publisherAddress}` - Actual publisher public key
   - `{symbol}` - Price feed symbol (e.g., BTC/USD)
   - `{proposalId}` - Proposal ID number

2. **Headers for all POST requests:**
   ```
   Content-Type: application/json
   ```

3. **Test the health endpoint first** to ensure the server is running properly.

4. **Default Port:** 3000 (can be changed via PORT environment variable)

5. **RPC URL:** Defaults to Solana Devnet (`https://api.devnet.solana.com`)

---

## Postman Collection Import

You can import these endpoints directly into Postman or create a collection manually using the URLs above.
