use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;

use crate::proxy::{AnyOutboundHandler, selector::ThreadSafeSelectorControl};

use super::remote_content_manager::providers::proxy_provider::ThreadSafeProxyProvider;

pub mod manager;

mod utils;

#[cfg(test)]
use mockall::automock;
#[cfg_attr(test, automock)]
#[async_trait]
pub trait OutboundManager: Send + Sync {
    fn get_outbound(&self, name: &str) -> Option<AnyOutboundHandler>;

    fn get_proxy_providers(&self) -> HashMap<String, ThreadSafeProxyProvider>;
    fn get_proxy_provider(&self, name: &str) -> Option<ThreadSafeProxyProvider>;

    async fn url_test(
        &self,
        proxy: AnyOutboundHandler,
        url: &str,
        timeout: Duration,
    ) -> std::io::Result<(u16, u16)>;

    fn get_selector_control(&self, name: &str) -> Option<ThreadSafeSelectorControl>;

    // for API to display proxies info
    async fn get_proxies(
        &self,
    ) -> HashMap<String, Box<dyn erased_serde::Serialize + Send>>;
    async fn get_proxy(
        &self,
        proxy: &AnyOutboundHandler,
    ) -> HashMap<String, Box<dyn erased_serde::Serialize + Send>>;
}
