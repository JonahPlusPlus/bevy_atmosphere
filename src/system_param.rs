//! System params for easy reading/modifying of [`Atmospheric`] models.

use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use bevy::{ecs::system::{SystemParam, SystemMeta, SystemParamFetch, SystemParamState, ResState, ResMutState, ReadOnlySystemParamFetch}, prelude::*};

use crate::{model::Atmospheric, prelude::AtmosphereModel};

/// Accessor for reading from an [`Atmospheric`] model.
pub struct Atmosphere<'w, T: Atmospheric> {
    value: &'w T,
}

// SAFETY: Res only reads a single World resource
unsafe impl<T: Atmospheric> ReadOnlySystemParamFetch for AtmosphereState<T> {}

impl<'w, T: Atmospheric> Deref for Atmosphere<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

#[doc(hidden)]
pub struct AtmosphereState<T: Atmospheric> {
    res_state: ResState<AtmosphereModel>,
    _marker: PhantomData<T>,
}

impl<'w, T: Atmospheric> SystemParam for Atmosphere<'w, T> {
    type Fetch = AtmosphereState<T>;
}

// SAFETY: Atmosphere ComponentId and ArchetypeComponentId access is applied to SystemMeta. If this Atmosphere
// conflicts with any prior access, a panic will occur.
unsafe impl<T: Atmospheric> SystemParamState for AtmosphereState<T> {
    fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        Self {
            res_state: ResState::init(world, system_meta),
            _marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Atmospheric> SystemParamFetch<'w, 's> for AtmosphereState<T> {
    type Item = Atmosphere<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        let atmosphere_model = <<Res<AtmosphereModel> as SystemParam>::Fetch as SystemParamFetch>::get_param(&mut state.res_state, system_meta, world, change_tick).into_inner();
        let value = atmosphere_model.to_ref::<T>().expect("Wrong type of `Atmospheric` model found");
        Self::Item {
            value,
        }
    }
}

/// Accessor for writing to an [`Atmospheric`] model.
pub struct AtmosphereMut<'w, T: Atmospheric> {
    value: &'w mut T,
}

impl<'w, T: Atmospheric> Deref for AtmosphereMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Atmospheric> DerefMut for AtmosphereMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

#[doc(hidden)]
pub struct AtmosphereMutState<T: Atmospheric> {
    res_state: ResMutState<AtmosphereModel>,
    _marker: PhantomData<T>,
}

impl<'w, T: Atmospheric> SystemParam for AtmosphereMut<'w, T> {
    type Fetch = AtmosphereMutState<T>;
}

// SAFETY: Atmosphere ComponentId and ArchetypeComponentId access is applied to SystemMeta. If this Atmosphere
// conflicts with any prior access, a panic will occur.
unsafe impl<T: Atmospheric> SystemParamState for AtmosphereMutState<T> {
    fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        Self {
            res_state: ResMutState::init(world, system_meta),
            _marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Atmospheric> SystemParamFetch<'w, 's> for AtmosphereMutState<T> {
    type Item = AtmosphereMut<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        let atmosphere_model = <<ResMut<AtmosphereModel> as SystemParam>::Fetch as SystemParamFetch>::get_param(&mut state.res_state, system_meta, world, change_tick).into_inner();
        let value = atmosphere_model.to_mut::<T>().expect("Wrong type of `Atmospheric` model found");
        Self::Item {
            value,
        }
    }
}
