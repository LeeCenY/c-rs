mod tests {
    use std::sync::Arc;

    use futures::SinkExt;

    use crate::{
        app::{
            dispatcher::{Dispatcher, DispatcherImpl, statistics_manager},
            dns::MockClashResolver,
            outbound::MockOutboundManager,
            router::MockRouter,
        },
        proxy::datagram::UdpPacket,
        session::Session,
    };

    #[derive(Debug)]
    /// input tx -> output rx
    /// output tx -> input rx
    struct DummyUdpInbound {
        left_rx: tokio::sync::mpsc::Receiver<UdpPacket>,
        left_tx: tokio_util::sync::PollSender<UdpPacket>,

        right_rx: tokio::sync::mpsc::Receiver<UdpPacket>,
        right_tx: tokio_util::sync::PollSender<UdpPacket>,
    }

    impl DummyUdpInbound {
        fn new() -> Self {
            let left = tokio::sync::mpsc::channel(1024);
            let right = tokio::sync::mpsc::channel(1024);

            Self {
                left_rx: left.1,
                left_tx: tokio_util::sync::PollSender::new(left.0),

                right_rx: right.1,
                right_tx: tokio_util::sync::PollSender::new(right.0),
            }
        }
    }

    impl futures::Sink<UdpPacket> for DummyUdpInbound {
        type Error = std::io::Error;

        fn poll_ready(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            let this = self.get_mut();
            this.left_tx.poll_ready_unpin(cx).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "DummyUdpInbound left_tx poll_ready failed",
                )
            })
        }

        fn start_send(
            self: std::pin::Pin<&mut Self>,
            item: UdpPacket,
        ) -> Result<(), Self::Error> {
            let this = self.get_mut();
            this.left_tx.start_send_unpin(item).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "DummyUdpInbound left_tx start_send failed",
                )
            })
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            let this = self.get_mut();
            this.left_tx.poll_flush_unpin(cx).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "DummyUdpInbound left_tx poll_flush failed",
                )
            })
        }

        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            let this = self.get_mut();
            this.left_tx.poll_close_unpin(cx).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "DummyUdpInbound left_tx poll_close failed",
                )
            })
        }
    }

    impl futures::Stream for DummyUdpInbound {
        type Item = UdpPacket;

        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            let this = self.get_mut();
            this.right_rx.poll_recv(cx)
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_dispatcher_udp() {
        let mock_outbound_manager = MockOutboundManager::new();
        let mock_router = MockRouter::new();
        let mock_resolver = MockClashResolver::new();
        let stats_manager = statistics_manager::Manager::new();

        let dispather = DispatcherImpl::new(
            Arc::new(mock_outbound_manager),
            Arc::new(mock_router),
            Arc::new(mock_resolver),
            crate::config::def::RunMode::Direct,
            stats_manager,
            None,
        );

        let mode = dispather.get_mode().await;
        assert_eq!(mode, crate::config::def::RunMode::Direct);

        dispather.set_mode(crate::config::def::RunMode::Rule).await;
        let mode = dispather.get_mode().await;
        assert_eq!(mode, crate::config::def::RunMode::Rule);

        let sess = Session {
            network: crate::session::Network::Udp,
            typ: crate::session::Type::Tun,
            source: "127.0.0.1:1234".parse().unwrap(),
            destination: "dns.google.com:53".parse().unwrap(),
            resolved_ip: None,
            so_mark: None,
            iface: None,
            asn: None,
        };
        let dummy_inbound = DummyUdpInbound::new();

        let closer = dispather
            .dispatch_datagram(sess, Box::new(dummy_inbound))
            .await;
    }
}
