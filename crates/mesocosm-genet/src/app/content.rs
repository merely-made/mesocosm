// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Admit immutable voxel content before founding, and restore recorded bytes.

use mesocosm_mesh::{VolumeMap, content::ContentPack};
use mesocosm_runtime::Runtime;

use super::HostConfig;

pub(super) fn start(
    config: &HostConfig,
) -> Result<(Runtime, Option<ContentPack>, VolumeMap), String> {
    let founding = config.effective_body_layout().founding();
    let pack = match &config.replay {
        Some(trace) => trace.content.clone(),
        None if config.generated_content => Some(
            ContentPack::generate(founding.palette())
                .map_err(|why| format!("generation refused: {why:?}"))?,
        ),
        None => None,
    };
    if let Some(pack) = pack {
        let volumes = pack
            .resolve()
            .map_err(|why| format!("pack refused: {why:?}"))?;
        let runtime = Runtime::with_founding_palette(
            config.seed,
            config.organisms,
            config.ticks_per_second,
            founding,
            pack.palette,
        )
        .map_err(|why| format!("palette refused: {why:?}"))?;
        Ok((runtime, Some(pack), volumes))
    } else {
        let runtime = Runtime::with_founding_palette(
            config.seed,
            config.organisms,
            config.ticks_per_second,
            founding,
            founding.palette(),
        )
        .map_err(|why| format!("founding refused: {why:?}"))?;
        let volumes = crate::fixture::volumes_for(runtime.world());
        Ok((runtime, None, volumes))
    }
}

#[cfg(test)]
mod tests;
