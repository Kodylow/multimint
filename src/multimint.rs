use anyhow::Result;
use fedimint_core::api::InviteCode;
use fedimint_core::config::FederationId;
use std::collections::BTreeMap;
use std::path::PathBuf;

use fedimint_client::{derivable_secret::DerivableSecret, ClientArc};

use fedimint_client::{get_config_from_db, FederationInfo};
use fedimint_core::{db::Database};
use fedimint_ln_client::LightningClientInit;
use fedimint_mint_client::MintClientInit;
use fedimint_wallet_client::WalletClientInit;


#[derive(Debug, Clone)]
pub struct MultiMint {
    pub mint_map: BTreeMap<FederationId, ClientArc>
}

impl MultiMint {
    pub fn new() -> Result<Self> {
        // Load existing fedimint clients
        // Return self
        Ok(Self {
            mint_map: BTreeMap::new()
        })
    }

    fn load_existing(self) -> Result<Self> {
        // Load existing fedimint clients
        // Return self
        Ok(self)
    }

    pub fn register(self, invite_code: InviteCode) -> Self {
        // Register new FederationId and ClientArc
        // Add into map
        // Return self
        self
    }

    pub async fn load_fedimint_client(
        mut self,
        invite_code: InviteCode,
        fm_db_path: PathBuf,
        root_secret: DerivableSecret,
    ) -> Result<Self> {
        let db = Database::new(
            fedimint_rocksdb::RocksDb::open(fm_db_path.clone())?,
            Default::default(),
        );
        let mut client_builder = fedimint_client::Client::builder();
        if get_config_from_db(&db).await.is_none() {
            let federation_info = FederationInfo::from_invite_code(invite_code).await?;
            client_builder.with_federation_info(federation_info);
        };
        client_builder.with_database(db);
        client_builder.with_module(WalletClientInit(None));
        client_builder.with_module(MintClientInit);
        client_builder.with_module(LightningClientInit);
        client_builder.with_primary_module(1);
        let client_res = client_builder.build(root_secret.clone()).await?;

        self.mint_map.insert(client_res.federation_id(), client_res);

        Ok(self)
    }
}
