use anyhow::Result;
use fedimint_client::ClientArc;
use fedimint_core::api::InviteCode;
use fedimint_core::config::{FederationId, FederationIdPrefix, JsonClientConfig};
use fedimint_core::db::Database;
use fedimint_core::Amount;
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;
use types::InfoResponse;

pub mod client;
pub mod db;
pub mod types;

use crate::client::LocalClientBuilder;
use crate::db::FederationConfig;

#[derive(Debug, Clone)]
pub struct MultiMint {
    db: Database,
    pub client_builder: LocalClientBuilder,
    pub clients: Arc<Mutex<BTreeMap<FederationId, ClientArc>>>,
}

impl MultiMint {
    pub async fn new(work_dir: PathBuf) -> Result<Self> {
        let db = Database::new(
            fedimint_rocksdb::RocksDb::open(work_dir.join("multimint.db"))?,
            Default::default(),
        );

        let client_builder = LocalClientBuilder::new(work_dir);

        let clients = Arc::new(Mutex::new(BTreeMap::new()));

        Self::load_clients(&mut clients.clone(), &db, &client_builder).await?;

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
    ) -> Result<()> {
        let mut clients = clients.lock().await;

        let dbtx = db.begin_transaction().await;
        let configs = client_builder.load_configs(dbtx.into_nc()).await;

        for config in configs {
            let federation_id = config.invite_code.federation_id();

            if let Ok(client) = client_builder.build(config.clone()).await {
                clients.insert(federation_id, client);
            } else {
                warn!("Failed to load client for federation: {federation_id}");
            }
        }

        Ok(())
    }

    pub async fn register_new(&mut self, invite_code: InviteCode) -> Result<FederationId> {
        let federation_id = invite_code.federation_id();
        if self
            .clients
            .lock()
            .await
            .get(&invite_code.federation_id())
            .is_some()
        {
            warn!(
                "Federation already registered: {:?}",
                invite_code.federation_id()
            );
            return Ok(federation_id);
        }

        let client_cfg = FederationConfig { invite_code };

        let client = self.client_builder.build(client_cfg.clone()).await?;
        // self.check_federation_network(&federation_config, gateway_config.network)
        //     .await?;

        self.clients.lock().await.insert(federation_id, client);

        let dbtx = self.db.begin_transaction().await;
        self.client_builder
            .save_config(client_cfg.clone(), dbtx)
            .await?;

        Ok(federation_id)
    }

    pub async fn all(&self) -> Vec<ClientArc> {
        self.clients.lock().await.values().cloned().collect()
    }

    pub async fn ids(&self) -> Vec<FederationId> {
        self.clients.lock().await.keys().cloned().collect()
    }

    pub async fn get(&self, federation_id: &FederationId) -> Option<ClientArc> {
        self.clients.lock().await.get(federation_id).cloned()
    }

    pub async fn get_by_str(&self, federation_id_str: &str) -> Option<ClientArc> {
        let federation_id = FederationId::from_str(federation_id_str).ok()?;
        self.get(&federation_id).await
    }

    pub async fn get_by_prefix(
        &self,
        federation_id_prefix: &FederationIdPrefix,
    ) -> Option<ClientArc> {
        let keys = self
            .clients
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let federation_id = keys
            .into_iter()
            .find(|id| id.to_prefix() == federation_id_prefix.clone());

        match federation_id {
            Some(federation_id) => self.get(&federation_id).await,
            None => None,
        }
    }

    pub async fn update(&self, federation_id: &FederationId, new_client: ClientArc) {
        self.clients
            .lock()
            .await
            .insert(federation_id.clone(), new_client);
    }

    pub async fn remove(&self, federation_id: &FederationId) {
        self.clients.lock().await.remove(federation_id);
    }

    pub async fn has(&self, federation_id: &FederationId) -> bool {
        self.clients.lock().await.contains_key(federation_id)
    }

    pub async fn has_by_str(&self, federation_id_str: &str) -> bool {
        let federation_id = match FederationId::from_str(federation_id_str) {
            Ok(federation_id) => federation_id,
            Err(_) => return false,
        };

        self.has(&federation_id).await
    }

    pub async fn configs(&self) -> Result<HashMap<FederationId, JsonClientConfig>> {
        let mut configs_map = HashMap::new();
        let clients = self.clients.lock().await;

        for (federation_id, client) in clients.iter() {
            let client_config = client.get_config_json();
            configs_map.insert(federation_id.clone(), client_config);
        }

        Ok(configs_map)
    }

    pub async fn ecash_balances(&self) -> Result<HashMap<FederationId, Amount>> {
        let mut balances = HashMap::new();
        let clients = self.clients.lock().await;

        for (federation_id, client) in clients.iter() {
            let balance = client.get_balance().await;
            balances.insert(federation_id.clone(), balance);
        }

        Ok(balances)
    }

    pub async fn info(&self) -> Result<HashMap<FederationId, InfoResponse>> {
        let mut info_map = HashMap::new();
        let clients = self.clients.lock().await;

        for (federation_id, client) in clients.iter() {
            let mint_client = client.get_first_module::<MintClientModule>();
            let wallet_client = client.get_first_module::<WalletClientModule>();
            let summary = mint_client
                .get_wallet_summary(
                    &mut self
                        .db
                        .begin_transaction_nc()
                        .await
                        .to_ref_with_prefix_module_id(1),
                )
                .await;

            let info = InfoResponse {
                federation_id: federation_id.clone(),
                network: wallet_client.get_network().to_string(),
                meta: client.get_config().global.meta.clone(),
                total_amount_msat: summary.total_amount(),
                total_num_notes: summary.count_items(),
                denominations_msat: summary,
            };

            info_map.insert(federation_id.clone(), info);
        }

        Ok(info_map)
    }
}
