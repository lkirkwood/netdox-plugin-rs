use std::{
    collections::{HashMap, HashSet},
    future::Future,
};

use redis::{Cmd, ToRedisArgs};
use serde::Deserialize;

use crate::error::FCallResult;

// CLI

/// Struct for modeling the redis connection details argument each plugin receives.
#[derive(Debug, Deserialize)]
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

impl RedisArgs {
    /// Return a redis client object using these connection  details.
    pub fn to_client(self) -> FCallResult<redis::Client> {
        let client =
            redis::Client::open(format!("redis://{}:{}/{}", self.host, self.port, self.db))?;

        if let Some(username) = self.username {
            redis::cmd("AUTH")
                .arg(username)
                .arg(self.password.unwrap())
                .exec(&mut client.get_connection()?)?;
        }

        Ok(client)
    }
}

// Data

/// Models a datum that can be attached to an object.
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

/// The different content types a string datum can be tagged as.
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

/// Models a node.
pub struct Node {
    pub name: String,
    pub link_id: String,
    pub alt_names: HashSet<String>,
    pub dns_names: HashSet<String>,
    pub raw_ids: HashSet<String>,
    pub plugins: HashSet<String>,
}

// Behaviour

/// Defines the read API.
pub trait NetdoxReader {
    /// Fetch the default network namespace.
    fn get_default_network(&mut self) -> impl Future<Output = FCallResult<String>> + Send;
    /// Prepend the default network qualifier to a list of DNS names.
    fn qualify_dns_names(
        &mut self,
        names: Vec<String>,
    ) -> impl Future<Output = FCallResult<Vec<String>>> + Send;
    /// Fetch all the DNS names from the datastore.
    fn get_dns_names(&mut self) -> impl Future<Output = FCallResult<HashSet<String>>> + Send;
    /// Fetch all the nodes from the datastore.
    fn get_nodes(&mut self) -> impl Future<Output = FCallResult<Vec<Node>>> + Send;
    /// Fetch a processed node using its link ID.
    fn get_node(&mut self, link_id: &str) -> impl Future<Output = FCallResult<Node>> + Send;
    /// Fetch the metadata for a DNS name.
    fn get_dns_metadata(
        &mut self,
        name: &str,
    ) -> impl Future<Output = FCallResult<HashMap<String, String>>> + Send;
    /// Fetch the metadata for a node.
    fn get_node_metadata(
        &mut self,
        node: &Node,
    ) -> impl Future<Output = FCallResult<HashMap<String, String>>> + Send;
}

/// Defines the write API.
pub trait NetdoxWriter {
    /// Create a DNS name and optionally attach a record.
    fn put_dns(
        &mut self,
        plugin: &str,
        name: &str,
        rtype: Option<&str>,
        value: Option<&str>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach plugin data to a DNS name.
    fn put_dns_plugin_data<'a>(
        &mut self,
        plugin: &str,
        name: &str,
        pdata_id: &str,
        data: PluginData<'a>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach metadata to a DNS name.
    fn put_dns_metadata(
        &mut self,
        plugin: &str,
        name: &str,
        metadata: &HashMap<&str, &str>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Create a node.
    fn put_node(
        &mut self,
        plugin: &str,
        name: &str,
        dns_names: Vec<&str>,
        exclusive: bool,
        link_id: Option<&str>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach plugin data to a node.
    fn put_node_plugin_data<'a>(
        &mut self,
        plugin: &str,
        dns_names: Vec<&str>,
        pdata_id: &str,
        data: PluginData<'a>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach plugin data to a processed node by link ID.
    fn put_proc_node_plugin_data<'a>(
        &mut self,
        plugin: &str,
        link_id: &str,
        pdata_id: &str,
        data: PluginData<'a>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach metadata to a node.
    fn put_node_metadata(
        &mut self,
        plugin: &str,
        dns_names: Vec<&str>,
        metadata: &HashMap<&str, &str>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach metadata to a processed node by link ID.
    fn put_proc_node_metadata(
        &mut self,
        plugin: &str,
        link_id: &str,
        metadata: &HashMap<&str, &str>,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Create a report.
    fn put_report(
        &mut self,
        plugin: &str,
        report_id: &str,
        title: &str,
        length: usize,
    ) -> impl Future<Output = FCallResult<()>> + Send;

    /// Attach data to a report.
    fn put_report_data<'a>(
        &mut self,
        plugin: &str,
        report_id: &str,
        index: usize,
        data: PluginData<'a>,
    ) -> impl Future<Output = FCallResult<()>> + Send;
}
