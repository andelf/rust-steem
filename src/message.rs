use bytes::Buf;
use serde_json::json;
use std::net::Ipv4Addr;
use std::str;

/*
trx_message_type = 1000,
block_message_type = 1001,
core_message_type_first = 5000,
item_ids_inventory_message_type = 5001,
blockchain_item_ids_inventory_message_type = 5002,
fetch_blockchain_item_ids_message_type = 5003,
fetch_items_message_type = 5004,
item_not_available_message_type = 5005,
hello_message_type = 5006,
connection_accepted_message_type = 5007,
connection_rejected_message_type = 5008,
address_request_message_type = 5009,
address_message_type = 5010,
closing_connection_message_type = 5011,
current_time_request_message_type = 5012,
current_time_reply_message_type = 5013,
check_firewall_message_type = 5014,
check_firewall_reply_message_type = 5015,
get_current_connections_request_message_type = 5016,
get_current_connections_reply_message_type = 5017,
core_message_type_last = 5099,
*/

#[repr(u32)]
pub enum Message {
    Hello(Hello),
    ConnectionAccepted,
    AddressRequest,
    Address(Vec<AddressInfo>),
}

pub struct Hello {
    user_agent: String,         // "Steem Reference Implementation"
    core_protocol_version: u32, // LE, 106
    inbound_address: Ipv4Addr,  // a.b.c.d => [d, c, b, a]
    inbound_port: u16,          // LE
    outbound_port: u16,         // LE
    // i.e. node_id
    node_public_key: [u8; 33],      // [0x02/0x03, ...]
    signed_shared_secret: [u8; 65], //
    user_data: serde_json::Value,
    /* std::string                user_agent;  Steem Reference Implementation
    uint32_t                   core_protocol_version;
    fc::ip::address            inbound_address;
    uint16_t                   inbound_port;
    uint16_t                   outbound_port;
    node_id_t                  node_public_key;
    fc::ecc::compact_signature signed_shared_secret;
    fc::variant_object         user_data; */
}

pub struct AddressInfo {
    remote_endpoint: [u8; 8],
    last_seen_time: u32,
    latency: i64, // ms
    node_id: [u8; 33],
    direction: u8,  // PeerConnectionDirection, // u8
    firewalled: u8, // FirewalledState,        // u8
}

pub fn parse_variant(data: &[u8]) -> serde_json::Value {
    let mut ret = json!({});

    let mut buf = data;

    assert_eq!(buf.get_u8(), 0x09); // \t, the table

    while buf.has_remaining() {
        let key_len = buf.get_u8() as usize;

        let key = str::from_utf8(&buf[..key_len]).expect("json key must be of utf8 encoding");
        buf.advance(key_len);
        match buf.get_u8() {
            0x05 => {
                let str_len = buf.get_u8() as usize;
                let value = String::from_utf8_lossy(&buf[..str_len]).to_owned();
                ret[key] = json!(value);
                buf.advance(str_len);
            }
            0x02 => {
                let value = buf.get_u64_le();
                ret[key] = json!(value);
            }
            tp => {
                println!("have not parse key: {:?} type={}", key, tp);
                unimplemented!()
            }
        }
    }

    ret
}
