# Arka - ERC-4337 Paymaster Backend

A lightweight ERC-4337 paymaster backend service implemented in Rust using the jsonrpsee crate. This service provides an RPC interface for applications to request transaction sponsorship via the ERC-4337 Account Abstraction protocol.

## Features

- JSON-RPC server with methods for transaction sponsorship
- ERC-4337 compliant paymaster operations
- Signature verification
- Time-based operation validity

## Prerequisites

- Rust toolchain (1.70+)
- Access to an Ethereum node with JSON-RPC support
- Private key with funds for sponsoring transactions

## Getting Started

### Installation

Clone the repository and build the application:

```bash
git clone https://github.com/ch4r10t33r/arka-light.git
cd arka-light
cargo build --release
```

### Configuration

You can configure the service using command-line arguments

### Running the Service

```bash
cargo run --release -- --rpc-server-addr 127.0.0.1:8545
```

Or with explicit command-line arguments:

```bash
cargo run --release -- \
  --rpc-server-addr 127.0.0.1:8545 \
  --private-key your_private_key_here \
  --chain-id 1 \
  --eth-rpc-url https://your-ethereum-node-url
```

## API Reference

The service exposes the following JSON-RPC methods:

### `pm_sponsorUserOperation`

Requests the paymaster to sponsor a user operation.

**Parameters:**
- `userOp`: An ERC-4337 UserOperation object

**Returns:**
- `paymasterAndData`: Bytes to be included in the UserOperation

Example:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "pm_sponsor",
  "params": [{
    "sender": "0x...",
    "nonce": "0x1",
    "initCode": "0x",
    "callData": "0x...",
    "callGasLimit": "0x...",
    "verificationGasLimit": "0x...",
    "preVerificationGas": "0x...",
    "maxFeePerGas": "0x...",
    "maxPriorityFeePerGas": "0x...",
    "paymasterAndData": "0x",
    "signature": "0x..."
  }]
}
```

## ERC-4337 Compliance

This implementation follows the ERC-4337 standard for Account Abstraction. The `paymasterAndData` field is structured as:

```
paymasterAddress (20 bytes) + validUntil (32 bytes) + validAfter (32 bytes) + signature
```

## Security Considerations

- The private key used by the paymaster should be properly secured
- Consider implementing additional validation rules for user operations
- Set appropriate gas limits to prevent excessive costs
- Monitor the paymaster balance to ensure it can cover operation costs

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.