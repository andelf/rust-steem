# rust-steem

开个天坑吧

## Handshake

```text
<= secp256k1 public key
=> secp256k1 public key
(pubkey tweak)
<= encrypted message
=> encrypted message
```

## Message encrpytion

AES-256-CBC with ZeroPadding. Every message is aligned to 16 bytes boundary.

AES key via `secp256k1_ec_pubkey_tweak_mul`.

AES IV via a modified version of cityhash.

### Cityhash modification

```cpp
static uint128 CityMurmur(const char *s, size_t len, uint128 seed) {
  /*
  uint64 a = Uint128Low64(seed);
  uint64 b = Uint128High64(seed);
  */
  uint64 a = Uint128High64(seed);
  uint64 b = Uint128Low64(seed);
  // ...
}
```

```cpp
static uint64 HashLen16(uint64 u, uint64 v) {
  // return Hash128to64(uint128(u, v));
  return Hash128to64(uint128(v, u));
}
```

## Message encoding

### Message Packet

```text
+--------------+--------------+---------+
| 0 ........ 3 | 4 ........ 7 | 8 ..... |
+---------------------------------------+
| Payload Size | Message Type | Payload |
+--------------+--------------+---------+
```

### Message Types

Message type encoded as LE u32.

```cpp
// via core_message.hpp
enum core_message_type_enum
{
  trx_message_type                             = 1000,
  block_message_type                           = 1001,
  core_message_type_first                      = 5000,
  item_ids_inventory_message_type              = 5001,
  blockchain_item_ids_inventory_message_type   = 5002,
  fetch_blockchain_item_ids_message_type       = 5003,
  fetch_items_message_type                     = 5004,
  item_not_available_message_type              = 5005,
  hello_message_type                           = 5006,
  connection_accepted_message_type             = 5007,
  connection_rejected_message_type             = 5008,
  address_request_message_type                 = 5009,
  address_message_type                         = 5010,
  closing_connection_message_type              = 5011,
  current_time_request_message_type            = 5012,
  current_time_reply_message_type              = 5013,
  check_firewall_message_type                  = 5014,
  check_firewall_reply_message_type            = 5015,
  get_current_connections_request_message_type = 5016,
  get_current_connections_reply_message_type   = 5017,
  core_message_type_last                       = 5099
};
```

### Field encoding

Memory layout as raw bytes.

### Variant encoding

little endian

0x09  \t => variant
0xXX length of key
0x02 u64 value
0x05 string value, with length prefix

## Message Handling

`libraries/net/node.cpp`.

```text
# handshake & peer discovery

<= hello_message
(validating node_public_key, signature, block_number, chain_id, is_self, allowed_peers list. failed.)
=> connection_rejected_message

<= hello_message
(firewalled ? _ : save to db)
=> connection_accepted_message
<= address_request_message
=> address_message
(save address to db)

# sync

<= fetch_blockchain_item_ids_message
=> blockchain_item_ids_inventory_message

# advertise_inventory_loop

=> item_ids_inventory_message

<= fetch_items_message
=> [block_message / item_not_available_message]
```
