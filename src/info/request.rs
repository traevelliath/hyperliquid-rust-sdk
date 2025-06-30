#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleSnapshotRequest<'a> {
    coin: &'a str,
    interval: crate::ws::Interval,
    start_time: u64,
    end_time: u64,
}

impl<'a> CandleSnapshotRequest<'a> {
    pub(crate) fn new(
        coin: &'a str,
        interval: crate::ws::Interval,
        start_time: u64,
        end_time: u64,
    ) -> Self {
        Self {
            coin,
            interval,
            start_time,
            end_time,
        }
    }
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum InfoRequest<'a> {
    #[serde(rename = "clearinghouseState")]
    UserState {
        user: ethers::types::H160,
    },
    #[serde(rename = "batchClearinghouseStates")]
    UserStates {
        users: Vec<ethers::types::H160>,
    },
    #[serde(rename = "spotClearinghouseState")]
    UserTokenBalances {
        user: ethers::types::H160,
    },
    UserFees {
        user: ethers::types::H160,
    },
    OpenOrders {
        user: ethers::types::H160,
    },
    OrderStatus {
        user: ethers::types::H160,
        oid: u64,
    },
    Meta,
    SpotMeta,
    SpotMetaAndAssetCtxs,
    AllMids,
    UserFills {
        user: ethers::types::H160,
    },
    #[serde(rename_all = "camelCase")]
    FundingHistory {
        coin: &'a str,
        start_time: u64,
        end_time: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    UserFunding {
        user: ethers::types::H160,
        start_time: u64,
        end_time: Option<u64>,
    },
    L2Book {
        coin: &'a str,
    },
    RecentTrades {
        coin: &'a str,
    },
    #[serde(rename_all = "camelCase")]
    CandleSnapshot {
        req: CandleSnapshotRequest<'a>,
    },
    Referral {
        user: ethers::types::H160,
    },
    HistoricalOrders {
        user: ethers::types::H160,
    },
}
