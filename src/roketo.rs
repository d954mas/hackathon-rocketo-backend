use near_sdk::{AccountId, Promise};

use crate::external::ext_roketo;

pub(crate) fn get_account_outgoing_streams(
    account_id: AccountId,
    roketo_acc: AccountId,
) -> Promise {
    ext_roketo::ext(roketo_acc).get_account_outgoing_streams(account_id, None, None)
}
