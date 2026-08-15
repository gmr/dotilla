use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use super::types::{Label, Table};
use super::{errors, namespace};

#[derive(AvroSchema, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[avro(namespace = "org.dotilla")]
pub struct Edge {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub labels: Vec<Label>,
    pub properties: Table,
}

#[derive(AvroSchema, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[avro(namespace = "org.dotilla")]
pub struct Node {
    pub id: u64,
    pub labels: Vec<Label>,
    pub properties: Table,
}

impl Node {
    pub async fn create(
        ns: &namespace::Namespace,
        labels: &[Label],
        properties: &Table,
    ) -> Result<Self, errors::Error> {
        let id = ns.get_next_id("nodes").await?;
        let node = Self {
            id,
            labels: labels.to_vec(),
            properties: properties.clone(),
        };
        ns.nodes.put_item(&format!("{id}"), &node).await?;
        Ok(node)
    }

    pub async fn delete(ns: &namespace::Namespace, id: u64) -> Result<(), errors::Error> {
        ns.nodes.remove_item(&format!("{id}")).await
    }

    pub async fn get(ns: &namespace::Namespace, id: u64) -> Result<Self, errors::Error> {
        ns.nodes.get_item(&format!("{id}")).await
    }
}

pub async fn update_node() {}

pub async fn upsert_node() {}

pub async fn list_nodes() {}

pub async fn search_nodes() {}

pub async fn create_edge() {}

pub async fn delete_edge() {}

pub async fn get_edge() {}

pub async fn update_edge() {}

pub async fn upsert_edge() {}

pub async fn list_edges() {}

pub async fn search_edges() {}

pub async fn get_neighbors() {}

pub async fn traverse() {}

pub async fn find_paths() {}

pub async fn find_shortest_path() {}

pub async fn match_subgraph() {}

pub async fn check_node_exists() {}

pub async fn check_edge_exists() {}

pub async fn bulk_create_nodes() {}

pub async fn bulk_delete_nodes() {}

pub async fn bulk_update_nodes() {}

pub async fn bulk_create_edges() {}

pub async fn bulk_delete_edges() {}

pub async fn bulk_update_edges() {}

pub async fn create_index() {}

pub async fn drop_index() {}

pub async fn start_transaction() {}

pub async fn commit_transaction() {}

pub async fn rollback_transaction() {}

pub async fn get_graph_schema() {}

pub async fn get_graph_stats() {}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::storage::{namespace, types};
    use crate::test_helpers::TestContext;
    use test_context::test_context;

    #[test_context(TestContext)]
    #[tokio::test]
    async fn test_create_get_delete_node(ctx: &mut TestContext) {
        let namespace = namespace::Namespace::create(&ctx.state.database, "test", None, None, None)
            .await
            .unwrap();
        assert!(namespace.name.to_string() == "test".to_string());

        let labels = vec![types::Label("Foo".to_string())];
        let mut properties = types::Table::default();
        properties.insert("foo".to_string(), types::Value::String("bar".to_string()));

        let node = Node::create(&namespace, &labels, &properties)
            .await
            .unwrap();

        assert!(node.labels.len() == 1);
        assert!(node.labels[0].0 == "Foo");
        assert!(
            node.properties.get(&"foo".to_string())
                == Some(&types::Value::String("bar".to_string()))
        );

        let fetched: Node = Node::get(&namespace, node.id).await.unwrap();

        assert!(fetched.id == node.id);
        assert!(fetched.labels.len() == 1);
        assert!(fetched.labels[0].0 == "Foo");
        assert!(
            fetched.properties.get(&"foo".to_string())
                == Some(&types::Value::String("bar".to_string()))
        );

        Node::delete(&namespace, node.id).await.unwrap();
        let result = Node::get(&namespace, node.id).await;
        assert!(matches!(result, Err(errors::Error::NotFound)));
    }
}
