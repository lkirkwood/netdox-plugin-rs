use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use redis::{Cmd, ToRedisArgs};
use serde::Deserialize;

use crate::error::FCallResult;

// CLI

#[derive(Deserialize)]
pub struct RedisArgs {
    /// Hostname of the redis server to use.
    pub host: String,
    /// Port of the redis server to use.
    pub port: usize,
    /// Logical database in the redis instance to use.
    pub db: usize,
    /// Username to use when authenticating with redis - if any.
    pub username: Option<String>,
    /// Password to use when authenticating with redis - if any.
    pub password: Option<String>,
}

// Data

pub enum PluginData<'a> {
    Hash {
        title: &'a str,
        items: HashMap<&'a str, &'a str>,
    },
    List {
        title: &'a str,
        items: Vec<(&'a str, &'a str, &'a str)>,
    },
    String {
        title: &'a str,
        content_type: StringContentType,
        content: &'a str,
    },
    Table {
        title: &'a str,
        num_columns: usize,
        rows: Vec<Vec<&'a str>>,
    },
}

impl<'a> PluginData<'a> {
    /// Adds the necessary args to a redis command in order to complete
    /// a plugin data creation fcall with this data.
    pub fn add_as_args(&'a self, cmd: &mut Cmd) {
        match self {
            PluginData::Hash { title, items } => {
                cmd.arg("hash").arg(title);
                for (key, val) in items {
                    cmd.arg(key).arg(val);
                }
            }
            PluginData::List { title, items } => {
                cmd.arg("list").arg(title).arg(items);
            }
            PluginData::String {
                title,
                content_type,
                content,
            } => {
                cmd.arg("string").arg(title).arg(content_type).arg(content);
            }
            PluginData::Table {
                title,
                num_columns,
                rows,
            } => {
                cmd.arg("table").arg(title).arg(num_columns);
                for row in rows {
                    for col in row {
                        cmd.arg(col);
                    }
                }
            }
        };
    }
}

pub enum StringContentType {
    HTML,
    Markdown,
    Plain,
}

impl ToRedisArgs for StringContentType {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            StringContentType::HTML => out.write_arg("html".as_bytes()),
            StringContentType::Markdown => out.write_arg("markdown".as_bytes()),
            StringContentType::Plain => out.write_arg("plain".as_bytes()),
        }
    }
}

pub struct Node {
    pub name: String,
    pub link_id: String,
    pub alt_names: HashSet<String>,
    pub dns_names: HashSet<String>,
    pub raw_ids: HashSet<String>,
    pub plugins: HashSet<String>,
}

// Behaviour

#[async_trait]
pub trait NetdoxReader {
    async fn get_default_network(&mut self) -> FCallResult<String>;
    async fn qualify_dns_names(&mut self, names: Vec<String>) -> FCallResult<Vec<String>>;
    async fn get_dns_names(&mut self) -> FCallResult<HashSet<String>>;
    async fn get_nodes(&mut self) -> FCallResult<Vec<Node>>;
    async fn get_node(&mut self, link_id: &str) -> FCallResult<Node>;
    async fn get_dns_metadata(&mut self, name: &str) -> FCallResult<HashMap<String, String>>;
    async fn get_node_metadata(&mut self, node: &Node) -> FCallResult<HashMap<String, String>>;
}

#[async_trait]
pub trait NetdoxWriter {
    async fn put_dns(
        &mut self,
        plugin: &str,
        name: &str,
        rtype: Option<&str>,
        value: Option<&str>,
    ) -> FCallResult<()>;

    async fn put_dns_plugin_data(
        &mut self,
        plugin: &str,
        name: &str,
        pdata_id: &str,
        data: PluginData<'async_trait>,
    ) -> FCallResult<()>;

    async fn put_dns_metadata(
        &mut self,
        plugin: &str,
        name: &str,
        metadata: &HashMap<&str, &str>,
    ) -> FCallResult<()>;

    async fn put_node(
        &mut self,
        plugin: &str,
        name: &str,
        dns_names: Vec<&str>,
        exclusive: bool,
        link_id: Option<&str>,
    ) -> FCallResult<()>;

    async fn put_node_plugin_data(
        &mut self,
        plugin: &str,
        dns_names: Vec<&str>,
        pdata_id: &str,
        data: PluginData<'async_trait>,
    ) -> FCallResult<()>;

    async fn put_proc_node_plugin_data(
        &mut self,
        plugin: &str,
        link_id: &str,
        pdata_id: &str,
        data: PluginData<'async_trait>,
    ) -> FCallResult<()>;

    async fn put_node_metadata(
        &mut self,
        plugin: &str,
        dns_names: Vec<&str>,
        metadata: &HashMap<&str, &str>,
    ) -> FCallResult<()>;

    async fn put_proc_node_metadata(
        &mut self,
        plugin: &str,
        link_id: &str,
        metadata: &HashMap<&str, &str>,
    ) -> FCallResult<()>;

    async fn put_report(
        &mut self,
        plugin: &str,
        report_id: &str,
        title: &str,
        length: usize,
    ) -> FCallResult<()>;

    async fn put_report_data(
        &mut self,
        plugin: &str,
        report_id: &str,
        index: usize,
        data: PluginData<'async_trait>,
    ) -> FCallResult<()>;
}
