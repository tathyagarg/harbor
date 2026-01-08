#![allow(dead_code)]

use std::fmt::Debug;

use crate::font::otf_dtypes::*;
use crate::font::tables::TableTrait;

#[derive(Clone, Default, Debug)]
pub struct LongHorMetric {
    /// Advance width, in font design units.
    pub advance_width: UFWORD,

    /// Glyph left side bearing, in font design units.
    pub lsb: FWORD,
}

#[derive(Clone, Default)]
pub struct HMtxTable {
    /// Paired advance width and left side bearing values for each glyph.
    /// Records are indexed by glyph ID.
    pub h_metrics: Vec<LongHorMetric>,

    /// Left side bearings for glyph IDs greater than or equal to numberOfHMetrics.
    pub left_side_bearings: Vec<FWORD>,

    _num_glyphs: Option<usize>,
    _num_h_metrics: Option<usize>,
}

impl Debug for HMtxTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HMtxTable")
            .field(
                "h_metrics",
                &&self.h_metrics[..std::cmp::min(4, self.h_metrics.len())],
            )
            .field("h_metrics_len", &self.h_metrics.len())
            .field(
                "left_side_bearings",
                &&self.left_side_bearings[..std::cmp::min(4, self.left_side_bearings.len())],
            )
            .field("left_side_bearings_len", &self.left_side_bearings.len())
            .finish()
    }
}

impl HMtxTable {
    pub fn set_num_glyphs(&mut self, num_glyphs: usize) {
        self._num_glyphs = Some(num_glyphs);
    }

    pub fn set_num_h_metrics(&mut self, num_h_metrics: usize) {
        self._num_h_metrics = Some(num_h_metrics);
    }
}

impl TableTrait for HMtxTable {
    fn parse(_data: &[u8]) -> Self
    where
        Self: Sized,
    {
        panic!(
            "HMtxTable parsing requires number of glyphs and number of hMetrics to be set before parsing."
        );
    }

    fn construct(&mut self, data: &[u8]) {
        assert!(self._num_glyphs.is_some(), "Number of glyphs not set.");
        assert!(self._num_h_metrics.is_some(), "Number of hMetrics not set.");

        let num_glyphs = self._num_glyphs.unwrap();
        let num_h_metrics = self._num_h_metrics.unwrap();

        let mut h_metrics = Vec::<LongHorMetric>::with_capacity(num_h_metrics);
        let mut left_side_bearings = Vec::<FWORD>::with_capacity(num_glyphs - num_h_metrics);

        let mut offset = 0;

        for _ in 0..num_h_metrics {
            let advance_width = uint16::from_data(&data[offset..]);
            offset += 2;

            let lsb = int16::from_data(&data[offset..]);
            offset += 2;

            h_metrics.push(LongHorMetric { advance_width, lsb });
        }

        for _ in num_h_metrics..num_glyphs {
            let lsb = int16::from_data(&data[offset..]);
            offset += 2;

            left_side_bearings.push(lsb);
        }

        self.h_metrics = h_metrics;
        self.left_side_bearings = left_side_bearings;
    }
}
