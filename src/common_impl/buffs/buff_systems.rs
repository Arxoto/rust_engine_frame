use crate::{
    base_lib::{
        cores::{timers::tick_timer::TickTimer, unify_types::FixedName},
        eff_attr::upserts::Upsert,
    },
    common_impl::buffs::{
        buff_collections::{BuffCollection, BuffEntity, BuffRuntime},
        buff_definition::{BuffMeta, BuffMountLogic},
    },
};

pub fn add_stat_buff<const SORTED: bool, S, Ctx, Bml, F>(
    ctx: Ctx,
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff_meta: BuffMeta<SORTED, S>,
    buff_mount_logic: &Bml,
    buff_merge: F,
    buff_timer: TickTimer,
) where
    S: FixedName,
    Bml: BuffMountLogic<SORTED, S, Ctx>,
    F: FnOnce(&mut BuffMeta<SORTED, S>, BuffMeta<SORTED, S>),
{
    let key = buff_meta.gen_id();
    match buffs.entry(key) {
        indexmap::map::Entry::Occupied(mut entry) => {
            let buff_entity = entry.get_mut();
            if let BuffRuntime::Stat(_, buff_changer) = &buff_entity.runtime {
                buff_merge(&mut buff_entity.meta, buff_meta);
                buff_changer.reset(&buff_entity.meta, ctx);
            }
        }
        indexmap::map::Entry::Vacant(entry) => {
            let buff_changer = buff_mount_logic.mount_buff(&buff_meta, ctx);
            let buff_entity = BuffEntity {
                meta: buff_meta,
                runtime: BuffRuntime::Stat(buff_timer, buff_changer),
            };
            entry.insert(buff_entity);
        }
    }
}

// todo fn for trigger
