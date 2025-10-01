use std::collections::HashMap;

use crate::{
    error::{FCallError, FCallResult},
    model::{NetdoxWriter, PluginData},
};

const CREATE_DNS_FN: &str = "netdox_create_dns";
const CREATE_NODE_FN: &str = "netdox_create_node";
const CREATE_REPORT_FN: &str = "netdox_create_report";

const CREATE_DNS_PDATA_FN: &str = "netdox_create_dns_plugin_data";
const CREATE_NODE_PDATA_FN: &str = "netdox_create_node_plugin_data";
const CREATE_PROC_NODE_PDATA_FN: &str = "netdox_create_proc_node_plugin_data";
const CREATE_REPORT_DATA_FN: &str = "netdox_create_report_data";

const CREATE_DNS_METADATA_FN: &str = "netdox_create_dns_metadata";
const CREATE_NODE_METADATA_FN: &str = "netdox_create_node_metadata";
const CREATE_PROC_NODE_METADATA_FN: &str = "netdox_create_proc_node_metadata";

impl NetdoxWriter for redis::aio::MultiplexedConnection {
    // DNS

    async fn put_dns(
        &mut self,
        plugin: &str,
        name: &str,
        rtype: Option<&str>,
        value: Option<&str>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_DNS_FN).arg(1).arg(name).arg(plugin);

        match (rtype, value) {
            (Some(rtype), Some(value)) => Ok(cmd.arg(rtype).arg(value).exec_async(self).await?),
            (None, None) => Ok(cmd.exec_async(self).await?),
            _ => Err(FCallError::WrongArgs {
                function: CREATE_DNS_FN,
                problem: "record type and value must both be provided or neiher provided.",
            }),
        }
    }

    async fn put_dns_plugin_data<'a>(
        &mut self,
        plugin: &str,
        name: &str,
        pdata_id: &str,
        data: PluginData<'a>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_DNS_PDATA_FN)
            .arg(1)
            .arg(name)
            .arg(plugin)
            .arg(pdata_id);

        data.add_as_args(&mut cmd);

        Ok(cmd.exec_async(self).await?)
    }

    async fn put_dns_metadata(
        &mut self,
        plugin: &str,
        name: &str,
        metadata: &HashMap<&str, &str>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_DNS_METADATA_FN).arg(1).arg(name).arg(plugin);

        for (key, val) in metadata {
            cmd.arg(key).arg(val);
        }

        Ok(cmd.exec_async(self).await?)
    }

    // Nodes

    async fn put_node(
        &mut self,
        plugin: &str,
        name: &str,
        dns_names: Vec<&str>,
        exclusive: bool,
        link_id: Option<&str>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_NODE_FN).arg(dns_names.len());

        for name in dns_names {
            cmd.arg(name);
        }

        cmd.arg(plugin).arg(name).arg(exclusive);

        if let Some(link_id) = link_id {
            cmd.arg(link_id);
        }

        Ok(cmd.exec_async(self).await?)
    }

    async fn put_node_plugin_data<'a>(
        &mut self,
        plugin: &str,
        dns_names: Vec<&str>,
        pdata_id: &str,
        data: PluginData<'a>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_NODE_PDATA_FN).arg(dns_names.len());

        for name in dns_names {
            cmd.arg(name);
        }

        cmd.arg(plugin).arg(pdata_id);

        data.add_as_args(&mut cmd);

        Ok(cmd.exec_async(self).await?)
    }

    async fn put_proc_node_plugin_data<'a>(
        &mut self,
        plugin: &str,
        link_id: &str,
        pdata_id: &str,
        data: PluginData<'a>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_PROC_NODE_PDATA_FN)
            .arg(1)
            .arg(link_id)
            .arg(plugin)
            .arg(pdata_id);

        data.add_as_args(&mut cmd);

        Ok(cmd.exec_async(self).await?)
    }

    async fn put_node_metadata(
        &mut self,
        plugin: &str,
        dns_names: Vec<&str>,
        metadata: &HashMap<&str, &str>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_NODE_METADATA_FN).arg(dns_names.len());
        for name in dns_names {
            cmd.arg(name);
        }
        cmd.arg(plugin);

        for (key, val) in metadata {
            cmd.arg(key).arg(val);
        }

        Ok(cmd.exec_async(self).await?)
    }

    async fn put_proc_node_metadata(
        &mut self,
        plugin: &str,
        link_id: &str,
        metadata: &HashMap<&str, &str>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_PROC_NODE_METADATA_FN)
            .arg(1)
            .arg(link_id)
            .arg(plugin);

        for (key, val) in metadata {
            cmd.arg(key).arg(val);
        }

        Ok(cmd.exec_async(self).await?)
    }

    // Reports

    async fn put_report(
        &mut self,
        plugin: &str,
        report_id: &str,
        title: &str,
        length: usize,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");

        cmd.arg(CREATE_REPORT_FN)
            .arg(1)
            .arg(report_id)
            .arg(plugin)
            .arg(title)
            .arg(length);

        Ok(cmd.exec_async(self).await?)
    }

    async fn put_report_data<'a>(
        &mut self,
        plugin: &str,
        report_id: &str,
        index: usize,
        data: PluginData<'a>,
    ) -> FCallResult<()> {
        let mut cmd = redis::cmd("FCALL");
        cmd.arg(CREATE_REPORT_DATA_FN)
            .arg(1)
            .arg(report_id)
            .arg(plugin)
            .arg(index);

        data.add_as_args(&mut cmd);

        Ok(cmd.exec_async(self).await?)
    }
}
