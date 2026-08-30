use crate::{
    base_lib::{cores::unify_types::FixedName, eff_attr::upsert_container::Upsert},
    common_impl::buffs::{
        buff_collections::BuffCollection,
        buff_definition::{BuffLogic, BuffMeta},
    },
};

pub fn add_buff<const SORTED: bool, S, Ctx, BL, F>(
    ctx: Ctx,
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff: BuffMeta<SORTED, S>,
    buff_logic: &BL,
    buff_merge: F,
) where
    S: FixedName,
    BL: BuffLogic<SORTED, S, Ctx>,
    F: FnOnce(&mut BuffMeta<SORTED, S>, BuffMeta<SORTED, S>),
{
    let key = buff.gen_id();
    match buffs.entry(key) {
        indexmap::map::Entry::Occupied(mut entry) => {
            buff_merge(&mut entry.get_mut().meta, buff);
            // todo refresh
        }
        indexmap::map::Entry::Vacant(entry) => todo!(),
    }
}
