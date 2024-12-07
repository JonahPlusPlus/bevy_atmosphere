//! Provides system params for easy reading/modifying of [`Atmospheric`] models.

use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::{
        component::{ComponentId, Tick},
        system::{ReadOnlySystemParam, SystemMeta, SystemParam},
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    prelude::*,
};

use crate::{model::Atmospheric, prelude::AtmosphereModel};

/// Accessor for reading from an [`Atmospheric`] model.
pub struct Atmosphere<'w, T: Atmospheric> {
    value: &'w T,
}

// SAFETY: Res only reads a single World resource
unsafe impl<T: Atmospheric> ReadOnlySystemParam for Atmosphere<'_, T> {}

impl<T: Atmospheric> Deref for Atmosphere<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

unsafe impl<T: Atmospheric> SystemParam for Atmosphere<'_, T> {
    type State = ComponentId;
    type Item<'w, 's> = Atmosphere<'w, T>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        Res::<AtmosphereModel>::init_state(world, system_meta)
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'w>,
        change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        let atmosphere_model = <Res<AtmosphereModel> as SystemParam>::get_param(
            state,
            system_meta,
            world,
            change_tick,
        )
        .into_inner();
        let value = atmosphere_model
            .to_ref::<T>()
            .expect("Wrong type of `Atmospheric` model found");
        Self::Item { value }
    }
}

/// Accessor for writing to an [`Atmospheric`] model.
pub struct AtmosphereMut<'w, T: Atmospheric> {
    value: &'w mut T,
}

impl<T: Atmospheric> Deref for AtmosphereMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: Atmospheric> DerefMut for AtmosphereMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

unsafe impl<T: Atmospheric> SystemParam for AtmosphereMut<'_, T> {
    type State = ComponentId;
    type Item<'w, 's> = AtmosphereMut<'w, T>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        ResMut::<AtmosphereModel>::init_state(world, system_meta)
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'w>,
        change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        let atmosphere_model = <ResMut<AtmosphereModel> as SystemParam>::get_param(
            state,
            system_meta,
            world,
            change_tick,
        )
        .into_inner();
        let value = atmosphere_model
            .to_mut::<T>()
            .expect("Wrong type of `Atmospheric` model found");
        Self::Item { value }
    }
}
