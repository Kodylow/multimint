use anyhow::Result;
use fedimint_client::ClientArc;
use fedimint_core::api::InviteCode;
use fedimint_core::config::FederationId;
use fedimint_core::db::Database;
use tokio::sync::Mutex;
use tracing::warn;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

pub mod db;
pub mod client;

use crate::client::LocalClientBuilder;
use crate::db::FederationConfig;


#[derive(Debug, Clone)]
pub struct MultiMint {
    db: Database,
    pub client_builder: LocalClientBuilder,
    pub clients: Arc<Mutex<BTreeMap<FederationId, ClientArc>>>
}

impl MultiMint {
    pub async fn new(
        work_dir: PathBuf,
    ) -> Result<Self> {
        let db = Database::new(
            fedimint_rocksdb::RocksDb::open(work_dir.join("multimint.db"))?,
            Default::default(),
        );

        let client_builder = LocalClientBuilder::new(
            work_dir,
        );

        let mut clients = Arc::new(Mutex::new(BTreeMap::new()));

        Self::load_clients(&mut clients, &db, &client_builder).await;

        Ok(Self {
            db: db,
            client_builder: client_builder,
            clients,
        })
    }

    async fn load_clients(
        clients: &mut Arc<Mutex<BTreeMap<FederationId, ClientArc>>>,
        db: &Database,
        client_builder: &LocalClientBuilder,
    ) {
        let mut clients = clients.lock().await;

        let dbtx = db.begin_transaction().await;
        let configs = client_builder.load_configs(dbtx.into_nc()).await;

        for config in configs {
            let federation_id = config.invite_code.federation_id();

            if let Ok(client) = client_builder
                .build(config.clone())
                .await
            {
                clients.insert(federation_id, client);
                
            } else {
                warn!("Failed to load client for federation: {federation_id}");
            }
        }
    }

    pub async fn register_new(&mut self, invite_code: InviteCode) -> Result<()> {
        if self
                .clients
                .lock()
                .await
                .get(&invite_code.federation_id())
                .is_some()
            {
                warn!("Federation already registered: {:?}", invite_code.federation_id());
                return Ok(());
            }

            let federation_id = invite_code.federation_id();
            let client_cfg = FederationConfig {
                invite_code,
            };

            let client = self
                .client_builder
                .build(client_cfg.clone())
                .await?;
            // self.check_federation_network(&federation_config, gateway_config.network)
            //     .await?;

            self.clients.lock().await.insert(federation_id, client);

            let dbtx = self.db.begin_transaction().await;
            self.client_builder
                .save_config(client_cfg.clone(), dbtx)
                .await?;

            Ok(())
    }

    pub async fn get_client(&self, federation_id: &FederationId) -> Option<ClientArc> {
        self.clients.lock().await.get(federation_id).cloned()
    }
}
