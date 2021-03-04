use frame_system as system;
use frame_support::assert_ok;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::language_storage::StructTag;
use move_vm::data::*;
use move_vm_runtime::data_cache::RemoteCache;
use serde::Deserialize;
use sp_mvm::storage::MoveVmStorage;

mod common;
use common::assets::*;
use common::mock::*;
use common::utils::*;

#[derive(Deserialize)]
struct StoreU64 {
    pub val: u64,
}

fn call_publish_module(signer: <Test as system::Trait>::AccountId, bc: Vec<u8>, mod_name: &str) {
    let origin = Origin::signed(signer);
    // execute VM for publish module:
    let result = Mvm::publish(origin, bc.clone());
    eprintln!("publish_module result: {:?}", result);
    assert_ok!(result);

    // check storage:
    let module_id = ModuleId::new(to_move_addr(signer), Identifier::new(mod_name).unwrap());
    let storage = Mvm::move_vm_storage();
    let oracle = MockOracle(None);
    let state = State::new(storage, oracle);
    assert_eq!(bc, state.get_module(&module_id).unwrap().unwrap());
}

fn call_execute_script_tx_block(origin: Origin, tx: UserTx) {
    let txbc = tx.bc().to_vec();

    let result = Mvm::execute(origin, txbc);
    eprintln!("execute_script result: {:?}", result);
    assert_ok!(result);
}

fn check_storage_block(expected: u64) {
    let store = Mvm::move_vm_storage();
    let oracle = MockOracle(None);
    let state = State::new(store, oracle);
    let tag = StructTag {
        address: origin_move_addr(),
        module: Identifier::new(UserMod::Store.name()).unwrap(),
        name: Identifier::new("U64").unwrap(),
        type_params: vec![],
    };
    let blob = state
        .get_resource(&origin_move_addr(), &tag)
        .unwrap()
        .unwrap();
    let store: StoreU64 = lcs::from_bytes(&blob).unwrap();
    assert_eq!(expected, store.val);
}

#[test]
fn execute_store_block() {
    new_test_ext().execute_with(|| {
        let root = root_ps_acc();
        let origin = origin_ps_acc();
        let signer = Origin::signed(origin);
        let block = StdMod::Block;
        let store = UserMod::Store;

        call_publish_module(root, block.bc().to_vec(), block.name());
        call_publish_module(origin, store.bc().to_vec(), store.name());

        const EXPECTED: u64 = 3;
        for _ in 0..EXPECTED {
            roll_next_block();
        }
        call_execute_script_tx_block(signer, UserTx::StoreSysBlock);
        check_storage_block(EXPECTED);
    });
}

#[test]
fn execute_store_time() {
    new_test_ext().execute_with(|| {
        let root = root_ps_acc();
        let origin = origin_ps_acc();
        let signer = Origin::signed(origin);
        let time = StdMod::Time;
        let store = UserMod::Store;

        call_publish_module(root, time.bc().to_vec(), time.name());
        call_publish_module(origin, store.bc().to_vec(), store.name());

        const EXPECTED: u64 = 3;
        for _ in 0..EXPECTED {
            roll_next_block();
        }
        call_execute_script_tx_block(signer, UserTx::StoreSysTime);
        check_storage_block(EXPECTED);
    });
}