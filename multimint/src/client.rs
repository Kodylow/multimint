use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;
use anyhow::Result;

use fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
use fedimint_client::{Client, FederationInfo, get_config_from_db};
use fedimint_core::db::{
    Committable, Database, DatabaseTransaction, IDatabaseTransactionOpsCoreTyped,
};
use futures_util::StreamExt;
use rand::thread_rng;
use tracing::info;
use fedimint_ln_client::LightningClientInit;
use fedimint_mint_client::MintClientInit;
use fedimint_wallet_client::WalletClientInit;

use crate::db::{FederationConfig, FederationIdKey, FederationIdKeyPrefix};

#[derive(Debug, Clone)]
pub struct LocalClientBuilder {
    work_dir: PathBuf,
}

impl LocalClientBuilder {
    pub fn new(
        work_dir: PathBuf,

    ) -> Self {
        Self {
            work_dir,
        }
    }
}

impl LocalClientBuilder {
    #[allow(clippy::too_many_arguments)]
    pub async fn build(
        &self,
        config: FederationConfig,
    ) -> Result<fedimint_client::ClientArc> {
        let federation_id = config.invite_code.federation_id();

        let db_path = self.work_dir.join(format!("{federation_id}.db"));

        let db = Database::new(
            fedimint_rocksdb::RocksDb::open(db_path.clone())?,
            Default::default(),
        );

        let mut client_builder = Client::builder();
        // if let client_config = get_config_from_db(&db).await {
        //     let federation_info = FederationInfo::from_invite_code(config.invite_code).await?;
        //     client_builder.with_federation_info(federation_info);
        // } 
        client_builder.with_database(db.clone());
        client_builder.with_module(WalletClientInit(None));
        client_builder.with_module(MintClientInit);
        client_builder.with_module(LightningClientInit);
        client_builder.with_primary_module(1);

        let client_secret =
            match client_builder.load_decodable_client_secret().await {
                Ok(secret) => secret,
                Err(_) => {
                    info!("Generating secret and writing to client storage");
                    let secret = PlainRootSecretStrategy::random(&mut thread_rng());
                    client_builder
                        .store_encodable_client_secret(secret)
                        .await?;
                    secret
                }
            };

        let root_secret = PlainRootSecretStrategy::to_root_secret(&client_secret);

        if get_config_from_db(&db).await.is_none() {
            let federation_info = FederationInfo::from_invite_code(config.invite_code).await?;
            client_builder.with_federation_info(federation_info);
        };

        let client_res = client_builder.build(root_secret.clone()).await?;

        Ok(client_res)
    }

    pub async fn save_config(
        &self,
        config: FederationConfig,
        mut dbtx: DatabaseTransaction<'_, Committable>,
    ) -> Result<()> {
        let id = config.invite_code.federation_id();
        dbtx.insert_entry(&FederationIdKey { id }, &config).await;
        dbtx.commit_tx_result()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save config: {:?}", e))
    }

    pub async fn load_configs(&self, mut dbtx: DatabaseTransaction<'_>) -> Vec<FederationConfig> {
        dbtx.find_by_prefix(&FederationIdKeyPrefix)
            .await
            .collect::<BTreeMap<FederationIdKey, FederationConfig>>()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>()
    }
}
