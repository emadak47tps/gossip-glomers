use anyhow::Context;
use gossip_glomers::{main_loop, Init, Message, Node};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BroadcastNode {
    node: String,
    messages: Vec<usize>,
    topology: HashMap<String, Vec<String>>,
    id: usize,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            node: init.node_id,
            messages: Vec::new(),
            topology: HashMap::new(),
            id: 0,
        })
    }

    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Broadcast { message } => {
                reply.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialise reply to JSON for stdout failed")?;
                stdout.write_all(b"\n").context("write trailing lines")?;

                self.messages.push(message);
            }
            Payload::Read => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialise reply to JSON for stdout failed")?;
                stdout.write_all(b"\n").context("write trailing lines")?;
            }
            Payload::Topology { topology } => {
                reply.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialise reply to JSON for stdout failed")?;
                stdout.write_all(b"\n").context("write trailing lines")?;

                self.topology = topology;
            }
            Payload::ReadOk { .. } | Payload::BroadcastOk | Payload::TopologyOk => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
