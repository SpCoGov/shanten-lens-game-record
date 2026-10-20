use anyhow::{bail, ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor, SerializeOptions};
use serde_json::Value;
use std::sync::LazyLock;

const DESCRIPTORS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/liqi_desc.bin"));
static POOL: LazyLock<DescriptorPool> = LazyLock::new(|| DescriptorPool::decode(DESCRIPTORS).expect("invalid liqi descriptors"));

#[derive(Clone, PartialEq, Message)]
struct Wrapper {
    #[prost(string, tag = "1")] name: String,
    #[prost(bytes = "vec", tag = "2")] data: Vec<u8>,
}

fn message_descriptor(name: &str) -> Result<MessageDescriptor> {
    POOL.get_message_by_name(name.trim_start_matches('.')).with_context(|| format!("unknown message {name}"))
}

pub fn decode_game_record_base64(encoded: &str) -> Result<Value> {
    let mut record = decode_wrapper(&BASE64.decode(encoded)?)?;
    ensure!(
        record.get("name").and_then(Value::as_str) == Some(".lq.GameDetailRecords"),
        "expected .lq.GameDetailRecords"
    );
    let Some(data) = record.get_mut("data") else {
        bail!("game record data missing");
    };
    if let Some(records) = data.get_mut("records").and_then(Value::as_array_mut) {
        for item in records {
            if let Some(encoded) = item.as_str().filter(|value| !value.is_empty()) {
                if let Ok(bytes) = BASE64.decode(encoded) {
                    if let Ok(decoded) = decode_wrapper(&bytes) {
                        *item = decoded;
                    }
                }
            }
        }
    }
    if let Some(actions) = data.get_mut("actions").and_then(Value::as_array_mut) {
        for action in actions {
            let Some(result) = action.get_mut("result") else {
                continue;
            };
            let Some(encoded) = result.as_str().filter(|value| !value.is_empty()) else {
                continue;
            };
            if let Ok(bytes) = BASE64.decode(encoded) {
                if let Ok(decoded) = decode_wrapper(&bytes) {
                    *result = decoded;
                }
            }
        }
    }
    Ok(record)
}

pub fn encode_game_record_base64(record: &Value) -> Result<String> {
    let mut record = record.clone();
    ensure!(
        record.get("name").and_then(Value::as_str) == Some(".lq.GameDetailRecords"),
        "expected .lq.GameDetailRecords"
    );
    let data = record.get_mut("data").context("game record data missing")?;
    if let Some(records) = data.get_mut("records").and_then(Value::as_array_mut) {
        for (index, item) in records.iter_mut().enumerate() {
            if item.is_object() {
                *item =
                    Value::String(BASE64.encode(encode_wrapper(item).with_context(|| {
                        format!("failed to encode game record records[{index}]")
                    })?));
            }
        }
    }
    if let Some(actions) = data.get_mut("actions").and_then(Value::as_array_mut) {
        for (index, action) in actions.iter_mut().enumerate() {
            let Some(result) = action.get_mut("result") else {
                continue;
            };
            if result.is_object() {
                *result =
                    Value::String(BASE64.encode(encode_wrapper(result).with_context(|| {
                        format!("failed to encode game record actions[{index}].result")
                    })?));
            }
        }
    }
    Ok(BASE64.encode(encode_wrapper(&record)?))
}

fn encode_wrapper(value: &Value) -> Result<Vec<u8>> {
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .context("wrapper name missing")?;
    let data = value.get("data").context("wrapper data missing")?;
    let message = dynamic_from_json(message_descriptor(name)?, data)
        .with_context(|| format!("invalid data for {name}"))?;
    Ok(Wrapper {
        name: name.to_owned(),
        data: message.encode_to_vec(),
    }
    .encode_to_vec())
}

fn decode_wrapper(bytes: &[u8]) -> Result<Value> {
    let wrapper = Wrapper::decode(bytes)?;
    let message =
        DynamicMessage::decode(message_descriptor(&wrapper.name)?, wrapper.data.as_slice())?;
    Ok(serde_json::json!({
        "name": wrapper.name,
        "data": dynamic_to_json(&message)?,
    }))
}

fn dynamic_to_json(message: &DynamicMessage) -> Result<Value> {
    let options = SerializeOptions::new()
        .stringify_64_bit_integers(false)
        // Keep the packet JSON compatible with Python MessageToDict and the
        // existing frontend/pipeline contract, which use lowerCamelCase.
        .use_proto_field_name(false)
        .skip_default_fields(false);
    let mut serializer = serde_json::Serializer::new(Vec::new());
    message.serialize_with_options(&mut serializer, &options)?;
    Ok(serde_json::from_slice(&serializer.into_inner())?)
}

fn dynamic_from_json(descriptor: MessageDescriptor, value: &Value) -> Result<DynamicMessage> {
    let text = serde_json::to_vec(value)?;
    let mut deserializer = serde_json::Deserializer::from_slice(&text);
    Ok(DynamicMessage::deserialize(descriptor, &mut deserializer)?)
}
