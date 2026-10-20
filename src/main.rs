mod record;

use anyhow::{bail, Context, Result};
use record::{decode_game_record_base64, encode_game_record_base64};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Default)]
struct Records {
    latest: Option<Value>,
    replacement: Option<Value>,
    events: Vec<(&'static str, Value)>,
}

impl Records {
    fn packet(&mut self, params: &Value) -> Value {
        let packet = &params["packet"];
        if packet["direction"] != "inbound" || packet["type"] != "Res"
            || packet["method"] != ".lq.Lobby.fetchGameRecord" {
            return json!({"action":"pass"});
        }
        let replacement = self.replacement.take();
        let data = replacement.as_ref().unwrap_or(&packet["data"]);
        if let Some(encoded) = data["data"].as_str() {
            match decode_game_record_base64(encoded) {
                Ok(record) => {
                    let mut response = data.clone();
                    response["data"] = record;
                    self.latest = Some(response.clone());
                    self.events.push(("record.updated", response));
                }
                Err(error) => eprintln!("failed to decode game record: {error:#}"),
            }
        }
        if let Some(data) = replacement {
            self.events.push(("record.override", json!(false)));
            json!({"action":"edit","data":data,"expectedRevision":params["revision"]})
        } else {
            json!({"action":"pass"})
        }
    }

    fn invoke(&mut self, params: &Value) -> Result<Value> {
        match params["method"].as_str().unwrap_or_default() {
            "record.get" => Ok(json!({"record":self.latest,"armed":self.replacement.is_some()})),
            "record.override" => {
                let response = &params["params"];
                let record = response.get("data").context("fetch response data missing")?;
                let encoded = encode_game_record_base64(record)?;
                let mut replacement = response.clone();
                replacement["data"] = Value::String(encoded);
                self.replacement = Some(replacement);
                self.events.push(("record.override", json!(true)));
                Ok(json!({"ok":true}))
            }
            method => bail!("unknown record method: {method}"),
        }
    }
}

fn send(output: &mut impl Write, message: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, message)?;
    output.write_all(b"\n")?;
    output.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let input = io::stdin();
    let mut output = io::BufWriter::new(io::stdout().lock());
    let mut records = Records::default();
    for line in input.lock().lines() {
        let message: Value = serde_json::from_str(&line?)?;
        let params = &message["params"];
        let result = match message["method"].as_str() {
            Some("host.hello") => Ok(json!({"providerId":"main"})),
            Some("host.ready") => {
                send(&mut output, &json!({"jsonrpc":"2.0","id":"register","method":"module.register","params":{
                    "moduleId":"records","name":"Game records","subscriptionsRevision":1,
                    "subscriptions":[
                        {"direction":"inbound","type":"Res","method":".lq.Lobby.fetchGameRecord","operations":["read","edit"]},
                        {"direction":"outbound","type":"Req","method":".lq.Lobby.fetchGameRecord","operations":["inject"]}
                    ]
                }}))?;
                continue;
            }
            Some("packet.handle") => Ok(records.packet(params)),
            Some("frontend.invoke") => records.invoke(params),
            Some("host.shutdown") => break,
            Some("host.checkUpdate") | Some("packet.read") => continue,
            None => {
                if let Some(error) = message.get("error") { eprintln!("host request failed: {error}"); }
                continue;
            }
            Some(method) => Err(anyhow::anyhow!("unknown method: {method}")),
        };
        for (name, data) in records.events.drain(..) {
            send(&mut output, &json!({"jsonrpc":"2.0","method":"plugin.emit","params":{"name":name,"data":data}}))?;
        }
        if let Some(id) = message.get("id") {
            let response = match result {
                Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
                Err(error) => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":format!("{error:#}")}}),
            };
            send(&mut output, &response)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response() -> Value {
        json!({"head":{"uuid":"test-record"},"data":{"name":".lq.GameDetailRecords","data":{
            "version":210715,"records":[{"name":".lq.RecordNewRound","data":{"chang":0,"ju":1,"tiles0":["1m","9m"]}}],
            "actions":[{"passed":0,"result":{"name":".lq.RecordDiscardTile","data":{"seat":0,"tile":"9m"}}}]
        }}})
    }

    fn packet(response: &Value) -> Value {
        let mut data = response.clone();
        data["data"] = json!(encode_game_record_base64(&response["data"]).unwrap());
        json!({"revision":3,"packet":{"direction":"inbound","type":"Res","method":".lq.Lobby.fetchGameRecord","data":data}})
    }

    #[test]
    fn records_decode_and_encode_nested_actions() {
        let original = response();
        let encoded = encode_game_record_base64(&original["data"]).unwrap();
        let decoded = decode_game_record_base64(&encoded).unwrap();
        assert_eq!(decoded["data"]["records"][0]["data"]["tiles0"], json!(["1m","9m"]));
        assert_eq!(decoded["data"]["actions"][0]["result"]["data"]["tile"], "9m");
        assert_eq!(encode_game_record_base64(&decoded).unwrap(), encoded);
    }

    #[test]
    fn replacement_is_validated_and_applied_once() {
        let mut state = Records::default();
        assert!(state.invoke(&json!({"method":"record.override","params":{"data":{}}})).is_err());
        assert!(state.replacement.is_none());
        state.invoke(&json!({"method":"record.override","params":response()})).unwrap();
        let input = packet(&response());
        assert_eq!(state.packet(&input)["action"], "edit");
        assert_eq!(state.latest.as_ref().unwrap()["head"]["uuid"], "test-record");
        assert_eq!(state.packet(&input)["action"], "pass");
        assert!(!state.invoke(&json!({"method":"record.get"})).unwrap()["armed"].as_bool().unwrap());
    }

    #[test]
    fn unrelated_or_invalid_packets_leave_the_game_untouched() {
        let mut state = Records::default();
        state.invoke(&json!({"method":"record.override","params":response()})).unwrap();
        assert_eq!(state.packet(&json!({"packet":{"method":".lq.Lobby.login"}}))["action"], "pass");
        assert!(state.replacement.is_some());
        state.replacement = None;
        let mut input = packet(&response());
        input["packet"]["data"]["data"] = json!("invalid base64");
        assert_eq!(state.packet(&input)["action"], "pass");
        assert!(state.latest.is_none());
    }
}
