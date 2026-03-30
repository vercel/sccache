// Copyright 2016 Mozilla Foundation
// Copyright 2017 David Michael Barr <b@rr-dav.id.au>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::Duration;

use opendal::Operator;
use opendal::layers::LoggingLayer;
use opendal::services::Memcached;

use crate::errors::*;

/// Resolve hostname in a memcached endpoint URL to an IP address.
/// The new opendal memcached service uses SocketAddr::parse internally which
/// doesn't support DNS hostnames, only IP:port. This works around that by
/// resolving tcp://hostname:port to tcp://ip:port.
fn resolve_memcached_endpoint(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("tcp://") {
        if let Ok(addrs) = std::net::ToSocketAddrs::to_socket_addrs(&rest) {
            if let Some(addr) = addrs.into_iter().next() {
                return format!("tcp://{}", addr);
            }
        }
    }
    url.to_string()
}

#[derive(Clone)]
pub struct MemcachedCache;

impl MemcachedCache {
    pub fn build(
        url: &str,
        username: Option<&str>,
        password: Option<&str>,
        key_prefix: &str,
        expiration: u32,
    ) -> Result<Operator> {
        // The new opendal memcached service uses SocketAddr::parse which doesn't
        // support hostnames. Resolve hostname to IP if the endpoint uses tcp://.
        let url = resolve_memcached_endpoint(url);
        let mut builder = Memcached::default().endpoint(&url);

        if let Some(username) = username {
            builder = builder.username(username);
        }
        if let Some(password) = password {
            builder = builder.password(password);
        }

        builder = builder
            .root(key_prefix)
            .default_ttl(Duration::from_secs(expiration.into()));

        let op = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        Ok(op)
    }
}
