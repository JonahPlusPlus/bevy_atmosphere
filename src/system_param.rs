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

impl<'w, T: Atmospheric> Deref for Atmosphere<'w, T> {
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
    value: ResMut<'w, AtmosphereModel>, // could keep pointer, but rather just avoid unsafe
    _marker: std::marker::PhantomData<T>,
}

impl<'w, T: Atmospheric> Deref for AtmosphereMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.to_ref().unwrap()
    }
}

impl<'w, T: Atmospheric> DerefMut for AtmosphereMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.to_mut().unwrap()
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
        let value: ResMut<'w, AtmosphereModel> =
            <ResMut<'w, AtmosphereModel> as SystemParam>::get_param(
                state,
                system_meta,
                world,
                change_tick,
            );
        value
            .to_ref::<T>()
            .expect("Wrong type of `Atmospheric` model found");
        Self::Item {
            value,
            _marker: std::marker::PhantomData,
        }
    }
}
