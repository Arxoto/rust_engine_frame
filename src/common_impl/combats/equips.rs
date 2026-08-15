use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr_prop::{
        attr_eff::AttrEffect,
        upsert_container::{Upsert, UpsertContainer},
    },
};

pub struct EquipWeapon<S: FixedName> {
    name: S,
    /// 武器锋利度
    sharp: f64,
    /// 武器质量
    mass: f64,
}

pub struct EquipArmor<S: FixedName> {
    name: S,
    /// 盔甲坚韧
    hard: f64,
    /// 盔甲柔韧
    soft: f64,
    /// 盔甲质量
    mass: f64,
}

impl<S: FixedName> EquipWeapon<S> {
    pub fn new(name: S, sharp: f64, mass: f64) -> Self {
        Self { name, sharp, mass }
    }

    pub fn new_from<T: Into<S>>(name: T, sharp: f64, mass: f64) -> Self {
        let name: S = name.into();
        Self { name, sharp, mass }
    }

    pub fn equip(
        &self,
        from_char_name: &S,
        weapon_sharp_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
        weapon_mass_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    ) {
        equip_system::add_attr_eff(from_char_name, &self.name, self.sharp, weapon_sharp_effs);
        equip_system::add_attr_eff(from_char_name, &self.name, self.mass, weapon_mass_effs);
    }

    pub fn take_off(
        &self,
        from_char_name: &S,
        weapon_sharp_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
        weapon_mass_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    ) {
        let attr_eff_id = equip_system::gen_attr_eff_id(from_char_name, &self.name);
        weapon_sharp_effs.delete_ele(|e| e.matched_id(&attr_eff_id));
        weapon_mass_effs.delete_ele(|e| e.matched_id(&attr_eff_id));
    }
}

impl<S: FixedName> EquipArmor<S> {
    pub fn new(name: S, hard: f64, soft: f64, mass: f64) -> Self {
        Self {
            name,
            hard,
            soft,
            mass,
        }
    }

    pub fn new_from<T: Into<S>>(name: T, hard: f64, soft: f64, mass: f64) -> Self {
        let name: S = name.into();
        Self {
            name,
            hard,
            soft,
            mass,
        }
    }

    pub fn equip(
        &self,
        from_char_name: &S,
        armor_hard_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
        armor_soft_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
        armor_mass_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    ) {
        equip_system::add_attr_eff(from_char_name, &self.name, self.hard, armor_hard_effs);
        equip_system::add_attr_eff(from_char_name, &self.name, self.soft, armor_soft_effs);
        equip_system::add_attr_eff(from_char_name, &self.name, self.mass, armor_mass_effs);
    }

    pub fn take_off(
        &self,
        from_char_name: &S,
        armor_hard_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
        armor_soft_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
        armor_mass_effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    ) {
        let attr_eff_id = equip_system::gen_attr_eff_id(from_char_name, &self.name);
        armor_hard_effs.delete_ele(|e| e.matched_id(&attr_eff_id));
        armor_soft_effs.delete_ele(|e| e.matched_id(&attr_eff_id));
        armor_mass_effs.delete_ele(|e| e.matched_id(&attr_eff_id));
    }
}

pub mod equip_system {
    use crate::base_lib::{
        cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
        eff_attr_prop::{
            attr_eff::{AttrEffId, AttrEffect, AttrEffectType},
            effects::Effect,
            upsert_container::{Upsert, UpsertContainer},
        },
    };

    pub fn gen_attr_eff_id<S: FixedName>(from_name: &S, equip_name: &S) -> AttrEffId<S> {
        AttrEffId {
            eff: equip_name.clone(),
            from: from_name.clone(),
        }
    }

    /// 触发装备的属性效果
    ///
    /// 由于 [`AttrEffect`] 是不允许堆叠的，因此若要实现双持武器，设置不同的装备名称
    pub fn add_attr_eff<S: FixedName>(
        from_name: &S,
        equip_name: &S,
        equip_value: f64,
        effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    ) {
        let attr_eff = AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::new(from_name.clone(), equip_name.clone(), equip_value),
            StaticTimer::inf(),
        );
        effs.upsert_ele(attr_eff, |old, new| {
            Upsert::replace(old, new);
        });
    }
}
