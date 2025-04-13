mod dispatcher_impl;
mod statistics_manager;
mod tracked;

#[cfg(test)]
mod dipatcher_impl_test;

use async_trait::async_trait;
pub use dispatcher_impl::DispatcherImpl;
pub use statistics_manager::Manager as StatisticsManager;

pub use tracked::{
    BoxedChainedDatagram, BoxedChainedStream, ChainedDatagram,
    ChainedDatagramWrapper, ChainedStream, ChainedStreamWrapper, TrackCopy,
    TrackedStream,
};

use crate::{
    config::def::RunMode,
    proxy::{AnyInboundDatagram, ClientStream},
    session::Session,
};

#[async_trait]
pub trait Dispatcher: Send + Sync + std::fmt::Debug {
    async fn get_mode(&self) -> RunMode;
    async fn set_mode(&self, mode: RunMode);
    async fn dispatch_stream(&self, sess: Session, lhs: Box<dyn ClientStream>);
    async fn dispatch_datagram(
        &self,
        sess: Session,
        udp_inbound: AnyInboundDatagram,
    ) -> tokio::sync::oneshot::Sender<u8>;
}
