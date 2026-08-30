use crate::{
    base_lib::{cores::unify_types::FixedName, eff_attr::upserts::Upsert},
    common_impl::buffs::{
        buff_collections::{BuffCollection, BuffValue},
        buff_definition::{BuffMeta, BuffMountLogic},
    },
};

pub fn add_buff<const SORTED: bool, S, Ctx, Bml, F>(
    ctx: Ctx,
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff_meta: BuffMeta<SORTED, S>,
    buff_mount_logic: &Bml,
    buff_merge: F,
) where
    S: FixedName,
    Bml: BuffMountLogic<SORTED, S, Ctx>,
    F: FnOnce(&mut BuffMeta<SORTED, S>, BuffMeta<SORTED, S>),
{
    let key = buff_meta.gen_id();
    match buffs.entry(key) {
        indexmap::map::Entry::Occupied(mut entry) => {
            let buff_value = entry.get_mut();
            buff_merge(&mut buff_value.meta, buff_meta);
            buff_value.changer.reset(ctx);
        }
        indexmap::map::Entry::Vacant(entry) => {
            let buff_changer = buff_mount_logic.mount_buff(&buff_meta, ctx);
            let buff_value = BuffValue {
                meta: buff_meta,
                changer: buff_changer,
            };
            entry.insert(buff_value);
        }
    }
}
