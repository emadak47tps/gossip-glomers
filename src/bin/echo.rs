use anyhow::Context;
use gossip_glomers::{main_loop, Init, Message, Node};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoNode {
    id: usize,
}

impl Node<(), Payload> for EchoNode {
    fn from_init(_state: (), _init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self { id: 0 })
    }

    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Echo { echo } => {
                reply.body.payload = Payload::EchoOk { echo };
                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialise reply to JSON for stdout failed")?;
                stdout.write_all(b"\n").context("write trailing lines")?;
            }
            Payload::EchoOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, EchoNode, _>(())
}
