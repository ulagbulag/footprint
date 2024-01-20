use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use dash_pipe_provider::{
    storage::StorageIO, FunctionContext, PipeArgs, PipeMessage, PipeMessages,
};
use footprint_api::ObjectLocation;
use footprint_provider_sewio_uwb::Metrics;
use serde::{Deserialize, Serialize};

fn main() {
    PipeArgs::<Function>::from_env().loop_forever()
}

#[derive(Clone, Debug, Serialize, Deserialize, Parser)]
struct FunctionArgs {}

#[derive(Debug)]
struct Function {
    metrics: Metrics,
}

#[async_trait]
impl ::dash_pipe_provider::FunctionBuilder for Function {
    type Args = FunctionArgs;

    async fn try_new(
        _args: &<Self as ::dash_pipe_provider::FunctionBuilder>::Args,
        _ctx: &mut FunctionContext,
        _storage: &Arc<StorageIO>,
    ) -> Result<Self> {
        Ok(Self {
            metrics: Metrics::new().await?,
        })
    }
}

#[async_trait]
impl ::dash_pipe_provider::Function for Function {
    type Input = ();
    type Output = ObjectLocation;

    async fn tick(
        &mut self,
        _inputs: PipeMessages<<Self as ::dash_pipe_provider::Function>::Input>,
    ) -> Result<PipeMessages<<Self as ::dash_pipe_provider::Function>::Output>> {
        Ok(PipeMessages::Single(PipeMessage::new(
            self.metrics.next().await?,
        )))
    }
}
