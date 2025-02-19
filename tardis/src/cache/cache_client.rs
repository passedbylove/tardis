use std::collections::HashMap;

use futures_util::lock::{Mutex, MutexGuard};
use log::{error, trace};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError, RedisResult};
use url::Url;

use crate::basic::error::TardisError;
use crate::basic::result::TardisResult;
use crate::config::config_dto::FrameworkConfig;
use crate::log::info;

/// Distributed cache handle / 分布式缓存操作
///
/// Encapsulates common Redis operations.
///
/// 封装了Redis的常用操作.
///
/// # Steps to use / 使用步骤
///
/// 1. Create the cache configuration / 创建缓存配置, @see [CacheConfig](crate::basic::config::CacheConfig)
///
/// 4. Use `TardisCacheClient` to operate cache / 使用 `TardisCacheClient` 操作缓存, E.g:
/// ```ignore
/// use tardis::TardisFuns;
/// assert_eq!(TardisFuns::cache().get("test_key").await.unwrap(), None);
/// client.set("test_key", "测试").await.unwrap();
/// assert_eq!(TardisFuns::cache().get("test_key").await.unwrap(), "测试");
/// assert!(TardisFuns::cache().set_nx("test_key2", "测试2").await.unwrap());
/// assert!(!TardisFuns::cache().set_nx("test_key2", "测试2").await.unwrap());
/// ```
pub struct TardisCacheClient {
    con: Mutex<MultiplexedConnection>,
}

impl TardisCacheClient {
    /// Initialize configuration from the cache configuration object / 从缓存配置对象中初始化配置
    pub async fn init_by_conf(conf: &FrameworkConfig) -> TardisResult<HashMap<String, TardisCacheClient>> {
        let mut clients = HashMap::new();
        clients.insert("".to_string(), TardisCacheClient::init(&conf.cache.url).await?);
        for (k, v) in &conf.cache.modules {
            clients.insert(k.to_string(), TardisCacheClient::init(&v.url).await?);
        }
        Ok(clients)
    }

    /// Initialize configuration / 初始化配置
    pub async fn init(str_url: &str) -> TardisResult<TardisCacheClient> {
        let url = Url::parse(str_url).map_err(|_| TardisError::format_error(&format!("[Tardis.CacheClient] Invalid url {}", str_url), "406-tardis-cache-url-error"))?;
        info!(
            "[Tardis.CacheClient] Initializing, host:{}, port:{}, db:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            if url.path().is_empty() { "" } else { &url.path()[1..] },
        );
        let client = redis::Client::open(str_url)?;
        let con = client.get_multiplexed_tokio_connection().await?;
        info!(
            "[Tardis.CacheClient] Initialized, host:{}, port:{}, db:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            if url.path().is_empty() { "" } else { &url.path()[1..] },
        );
        Ok(TardisCacheClient { con: Mutex::new(con) })
    }

    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] set, key:{}, value:{}", key, value);
        (*self.con.lock().await).set(key, value).await
    }

    pub async fn set_ex(&self, key: &str, value: &str, ex_sec: usize) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] set_ex, key:{}, value:{}, ex_sec:{}", key, value, ex_sec);
        (*self.con.lock().await).set_ex(key, value, ex_sec).await
    }

    pub async fn set_nx(&self, key: &str, value: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] set_nx, key:{}, value:{}", key, value);
        (*self.con.lock().await).set_nx(key, value).await
    }

    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        trace!("[Tardis.CacheClient] get, key:{}", key);
        (*self.con.lock().await).get(key).await
    }

    pub async fn getset(&self, key: &str, value: &str) -> RedisResult<Option<String>> {
        trace!("[Tardis.CacheClient] getset, key:{}, value:{}", key, value);
        (*self.con.lock().await).getset(key, value).await
    }

    pub async fn incr(&self, key: &str, delta: isize) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] incr, key:{}, delta:{}", key, delta);
        (*self.con.lock().await).incr(key, delta).await
    }

    pub async fn del(&self, key: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] del, key:{}", key);
        (*self.con.lock().await).del(key).await
    }

    pub async fn del_confirm(&self, key: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] del_confirm, key:{}", key);
        if let Err(e) = self.del(key).await {
            return Err(e);
        }
        loop {
            match self.exists(key).await {
                Ok(false) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                _ => {}
            }
        }
    }

    pub async fn exists(&self, key: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] exists, key:{}", key);
        (*self.con.lock().await).exists(key).await
    }

    pub async fn expire(&self, key: &str, ex_sec: usize) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] expire, key:{}, ex_sec:{}", key, ex_sec);
        (*self.con.lock().await).expire(key, ex_sec).await
    }

    pub async fn expire_at(&self, key: &str, timestamp_sec: usize) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] expire_at, key:{}, timestamp_sec:{}", key, timestamp_sec);
        (*self.con.lock().await).expire_at(key, timestamp_sec).await
    }

    pub async fn ttl(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] ttl, key:{}", key);
        (*self.con.lock().await).ttl(key).await
    }

    // list operations

    pub async fn lpush(&self, key: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] lpush, key:{}, value:{}", key, value);
        (*self.con.lock().await).lpush(key, value).await
    }

    pub async fn lrangeall(&self, key: &str) -> RedisResult<Vec<String>> {
        trace!("[Tardis.CacheClient] lrangeall, key:{}", key);
        (*self.con.lock().await).lrange(key, 0, -1).await
    }

    pub async fn llen(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] llen, key:{}", key);
        (*self.con.lock().await).llen(key).await
    }

    // hash operations

    pub async fn hget(&self, key: &str, field: &str) -> RedisResult<Option<String>> {
        trace!("[Tardis.CacheClient] hget, key:{}, field:{}", key, field);
        (*self.con.lock().await).hget(key, field).await
    }

    pub async fn hset(&self, key: &str, field: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] hset, key:{}, field:{}, value:{}", key, field, value);
        (*self.con.lock().await).hset(key, field, value).await
    }

    pub async fn hset_nx(&self, key: &str, field: &str, value: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] hset_nx, key:{}, field:{}, value:{}", key, field, value);
        (*self.con.lock().await).hset_nx(key, field, value).await
    }

    pub async fn hdel(&self, key: &str, field: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] hdel, key:{}, field:{}", key, field);
        (*self.con.lock().await).hdel(key, field).await
    }

    pub async fn hdel_confirm(&self, key: &str, field: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] hdel_confirm, key:{}, field:{}", key, field);
        if let Err(e) = self.hdel(key, field).await {
            return Err(e);
        }
        loop {
            match self.hexists(key, field).await {
                Ok(false) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                _ => {}
            }
        }
    }

    pub async fn hincr(&self, key: &str, field: &str, delta: isize) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] hincr, key:{}, field:{}, delta:{}", key, field, delta);
        (*self.con.lock().await).hincr(key, field, delta).await
    }

    pub async fn hexists(&self, key: &str, field: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] hexists, key:{}, field:{}", key, field);
        (*self.con.lock().await).hexists(key, field).await
    }

    pub async fn hkeys(&self, key: &str) -> RedisResult<Vec<String>> {
        trace!("[Tardis.CacheClient] hkeys, key:{}", key);
        (*self.con.lock().await).hkeys(key).await
    }

    pub async fn hvals(&self, key: &str) -> RedisResult<Vec<String>> {
        trace!("[Tardis.CacheClient] hvals, key:{}", key);
        (*self.con.lock().await).hvals(key).await
    }

    pub async fn hgetall(&self, key: &str) -> RedisResult<HashMap<String, String>> {
        trace!("[Tardis.CacheClient] hgetall, key:{}", key);
        (*self.con.lock().await).hgetall(key).await
    }

    pub async fn hlen(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] hlen, key:{}", key);
        (*self.con.lock().await).hlen(key).await
    }

    // bitmap operations

    pub async fn setbit(&self, key: &str, offset: usize, value: bool) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] setbit, key:{}, offset:{}, value:{}", key, offset, value);
        (*self.con.lock().await).setbit(key, offset, value).await
    }

    pub async fn getbit(&self, key: &str, offset: usize) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] getbit, key:{}, offset:{}", key, offset);
        (*self.con.lock().await).getbit(key, offset).await
    }

    pub async fn bitcount(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] bitcount, key:{}", key);
        (*self.con.lock().await).bitcount(key).await
    }

    pub async fn bitcount_range_by_byte(&self, key: &str, start: usize, end: usize) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] bitcount_range_by_byte, key:{}, start:{}, end:{}", key, start, end);
        (*self.con.lock().await).bitcount_range(key, start, end).await
    }

    /// Supported from version redis 7.0.0
    pub async fn bitcount_range_by_bit(&self, key: &str, start: usize, end: usize) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] bitcount_range_by_bit, key:{}, start:{}, end:{}", key, start, end);
        match redis::cmd("BITCOUNT").arg(key).arg(start).arg(end).arg("BIT").query_async(&mut (*self.con.lock().await)).await {
            Ok(count) => Ok(count),
            Err(e) => Err(e),
        }
    }

    // other operations

    pub async fn flushdb(&self) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] flushdb");
        match redis::cmd("FLUSHDB").query_async(&mut (*self.con.lock().await)).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn flushall(&self) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] flushall");
        match redis::cmd("FLUSHALL").query_async(&mut (*self.con.lock().await)).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // custom
    pub async fn cmd(&self) -> MutexGuard<'_, MultiplexedConnection> {
        self.con.lock().await
    }
}

impl From<RedisError> for TardisError {
    fn from(error: RedisError) -> Self {
        error!("[Tardis.CacheClient] [{}]{},", error.code().unwrap_or(""), error.detail().unwrap_or(""));
        TardisError::wrap(&format!("[Tardis.CacheClient] {:?}", error), "-1-tardis-cache-error")
    }
}
