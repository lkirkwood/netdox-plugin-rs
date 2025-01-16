use async_trait::async_trait;
use redis::{self, cmd, AsyncCommands};
use std::collections::{HashMap, HashSet};

use crate::{
    error::{FCallError, FCallResult},
    model::{NetdoxReader, Node},
};

const QUALIFY_DNS_NAME_FN: &str = "netdox_qualify_dns_names";

const META_KEY: &str = "meta";
const DNS_KEY: &str = "dns";
const NODES_KEY: &str = "nodes";
const PROC_NODES_KEY: &str = "proc_nodes";
const DEFAULT_NETWORK_KEY: &str = "default_network";

// Implementing the trait for redis::aio::MultiplexedConnection
#[async_trait]
impl NetdoxReader for redis::aio::MultiplexedConnection {
    /// Gets the default network.
    async fn get_default_network(&mut self) -> FCallResult<String> {
        Ok(self.get(DEFAULT_NETWORK_KEY).await?)
    }

    /// Qualifies a list of DNS names with the default network.
    async fn qualify_dns_names(&mut self, names: Vec<String>) -> FCallResult<Vec<String>> {
        Ok(cmd(QUALIFY_DNS_NAME_FN)
            .arg(names.len() as u32)
            .arg(&names)
            .query_async(self)
            .await?)
    }

    /// Get all DNS names that have been registered.
    async fn get_dns_names(&mut self) -> FCallResult<HashSet<String>> {
        Ok(self.smembers(DNS_KEY).await?)
    }

    /// Get all nodes in the database.
    async fn get_nodes(&mut self) -> FCallResult<Vec<Node>> {
        let mut nodes = Vec::new();
        let link_ids: Vec<String> = self.smembers(PROC_NODES_KEY).await?;
        for link_id in link_ids {
            nodes.push(self.get_node(&link_id).await?);
        }
        Ok(nodes)
    }

    /// Get a node by its link ID.
    async fn get_node(&mut self, link_id: &str) -> FCallResult<Node> {
        let name: String = self.get(format!("{PROC_NODES_KEY};{link_id}")).await?;
        let alt_names: HashSet<String> = self
            .smembers(format!("{PROC_NODES_KEY};{link_id};alt_names"))
            .await?;
        let dns_names: HashSet<String> = self
            .smembers(format!("{PROC_NODES_KEY};{link_id};dns_names"))
            .await?;
        let raw_ids: HashSet<String> = self
            .smembers(format!("{PROC_NODES_KEY};{link_id};raw_ids"))
            .await?;
        let plugins: HashSet<String> = self
            .smembers(format!("{PROC_NODES_KEY};{link_id};plugins"))
            .await?;

        Ok(Node {
            name,
            link_id: link_id.to_string(),
            alt_names,
            dns_names,
            raw_ids,
            plugins,
        })
    }

    /// Get metadata for a DNS name.
    async fn get_dns_metadata(&mut self, name: &str) -> FCallResult<HashMap<String, String>> {
        let qualified_name = match self
            .qualify_dns_names(vec![name.to_string()])
            .await?
            .into_iter()
            .next()
        {
            Some(qn) => qn,
            None => {
                return Err(FCallError::Logic(
                    "Tried to qualify one DNS name but got zero back.",
                ))
            }
        };

        Ok(self
            .hgetall(format!("{META_KEY};{DNS_KEY};{qualified_name}"))
            .await?)
    }

    /// Get metadata for a node.
    async fn get_node_metadata(&mut self, node: &Node) -> FCallResult<HashMap<String, String>> {
        let mut meta = HashMap::new();
        for raw_id in &node.raw_ids {
            let raw_meta: HashMap<String, String> = self
                .hgetall(format!("{META_KEY};{NODES_KEY};{raw_id}"))
                .await?;
            meta.extend(raw_meta);
        }
        let proc_meta: HashMap<String, String> = self
            .hgetall(format!("{META_KEY};{PROC_NODES_KEY};{}", node.link_id))
            .await?;
        meta.extend(proc_meta);
        Ok(meta)
    }
}
