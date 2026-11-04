use whatsapp_api::signal::{app_state_collection::{CollectionState, patch::OutgoingMutation}, app_state_crypto::{MutationKeys, MutationOperation}};
fn mutation(operation: MutationOperation) -> OutgoingMutation {
    OutgoingMutation { operation, index: br#"["fixture","same-index"]"#.to_vec(), action: Some(vec![8, 1]), version: 1 }
}
fn main() {
    let a = MutationKeys::derive(&[1; 32]).unwrap();
    let b = MutationKeys::derive(&[2; 32]).unwrap();
    let first = CollectionState::empty("regular").unwrap().build_patch(&a, [0,1,0,0,0,1], 0, &[mutation(MutationOperation::Set)]).unwrap().state;
    let changed = first.build_patch(&b, [0,1,0,0,0,2], 0, &[mutation(MutationOperation::Set)]).unwrap().state;
    let remove = first.build_patch(&b, [0,1,0,0,0,2], 0, &[mutation(MutationOperation::Remove)]);
    println!("same logical index, distinct mutation keys: wire index MAC differs={}", a.index_mac(br#"["fixture","same-index"]"#).unwrap()!=b.index_mac(br#"["fixture","same-index"]"#).unwrap());
    println!("first SET rows={}, replacement SET rows={}", first.len(), changed.len());
    println!("cross-key REMOVE result={}", match remove { Ok(_) => "ok".to_owned(), Err(e) => e.to_string() });
}
