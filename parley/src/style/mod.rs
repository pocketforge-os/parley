// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Rich styling support.

mod brush;
mod font;
mod styleset;

use alloc::borrow::Cow;

pub use brush::*;
pub use font::{
    FontFamily, FontFamilyName, FontFeature, FontFeatures, FontStyle, FontVariation,
    FontVariations, FontWeight, FontWidth, GenericFamily,
};
pub use fontique::Language;
pub use parlance::{OverflowWrap, TextWrapMode, WordBreak};
pub use styleset::StyleSet;

use crate::util::nearly_eq;

#[derive(Debug, Clone, Copy)]
pub enum WhiteSpaceCollapse {
    Collapse,
    Preserve,
}

/// The height that this text takes up. The default is `MetricsRelative(1.0)`, which is the given
/// font's preferred line height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    /// The line's height is a multiple of the "line height" defined by the font's metrics--the sum
    /// of the ascender height, descender height, and line gap / leading.
    MetricsRelative(f32),
    /// Line height specified as a multiple of the font size. This is how the CSS `line-height`
    /// property behaves if given a unitless number. Useful if you're using system-defined generic
    /// font families and want the line heights to be consistent across platforms.
    FontSizeRelative(f32),
    /// Line height specified in absolute units. This can be useful for ensuring all lines are
    /// spaced a whole number of pixels apart, or fitting lines into a given layout container
    /// height.
    Absolute(f32),
    /// The line is sized to contain the ascent and descent of *every* font used on it, not
    /// only of the first available font. This is how the CSS `line-height: normal` keyword
    /// behaves.
    ///
    /// CSS Inline Layout 3 [§5.3] sizes a line box "to exactly include the aligned layout
    /// bounds of all its inline-level boxes", where those bounds are an ascent `A` above and
    /// a descent `D` below the baseline, and states that "metrics from fonts other than the
    /// first available font only impact the layout bounds of an inline box with
    /// `line-height: normal`". CSS 2.1 [§10.8.1] says the same thing less precisely: "when an
    /// element contains text that is rendered in more than one font, user agents may determine
    /// the `normal` line-height value according to the largest font size".
    ///
    /// Every other variant is a fixed height that fallback fonts must *not* grow, so this is
    /// the only variant that consults the fonts a run actually resolved to.
    ///
    /// The fields are the *strut*: the ascent and descent of the style's first available font
    /// at its used size, as positive distances from the baseline. The strut contributes to the
    /// line box even on a line where no glyph is drawn from that font, so it is supplied by the
    /// caller — which is the only party that knows the style's family list — rather than
    /// rediscovered per run.
    ///
    /// [§5.3]: https://drafts.csswg.org/css-inline-3/#inline-height
    /// [§10.8.1]: https://www.w3.org/TR/CSS21/visudet.html#line-height
    Normal {
        /// Ascent of the first available font, a positive distance above the baseline.
        strut_ascent: f32,
        /// Descent of the first available font, a positive distance below the baseline.
        strut_descent: f32,
    },
}

impl Default for LineHeight {
    fn default() -> Self {
        Self::MetricsRelative(1.0)
    }
}

impl LineHeight {
    pub(crate) fn nearly_eq(self, other: Self) -> bool {
        match (self, other) {
            (Self::MetricsRelative(a), Self::MetricsRelative(b))
            | (Self::FontSizeRelative(a), Self::FontSizeRelative(b))
            | (Self::Absolute(a), Self::Absolute(b)) => nearly_eq(a, b),
            (
                Self::Normal {
                    strut_ascent: a_asc,
                    strut_descent: a_desc,
                },
                Self::Normal {
                    strut_ascent: b_asc,
                    strut_descent: b_desc,
                },
            ) => nearly_eq(a_asc, b_asc) && nearly_eq(a_desc, b_desc),
            _ => false,
        }
    }

    pub(crate) fn scale(self, scale: f32) -> Self {
        match self {
            Self::Absolute(value) => Self::Absolute(value * scale),
            // The strut is in the same absolute units as `Absolute`.
            Self::Normal {
                strut_ascent,
                strut_descent,
            } => Self::Normal {
                strut_ascent: strut_ascent * scale,
                strut_descent: strut_descent * scale,
            },
            // The other variants are relative to the font size, so scaling here needn't do anything
            value => value,
        }
    }
}

/// Properties that define a style.
#[derive(Clone, PartialEq, Debug)]
pub enum StyleProperty<'a, B: Brush> {
    /// CSS `font-family` property value.
    FontFamily(FontFamily<'a>),
    /// Font size.
    FontSize(f32),
    /// Font width.
    FontWidth(FontWidth),
    /// Font style.
    FontStyle(FontStyle),
    /// Font weight.
    FontWeight(FontWeight),
    /// Font variation settings.
    FontVariations(FontVariations<'a>),
    /// Font feature settings.
    FontFeatures(FontFeatures<'a>),
    /// Locale.
    Locale(Option<Language>),
    /// Brush for rendering text.
    Brush(B),
    /// Underline decoration.
    Underline(bool),
    /// Offset of the underline decoration.
    UnderlineOffset(Option<f32>),
    /// Size of the underline decoration.
    UnderlineSize(Option<f32>),
    /// Brush for rendering the underline decoration.
    UnderlineBrush(Option<B>),
    /// Strikethrough decoration.
    Strikethrough(bool),
    /// Offset of the strikethrough decoration.
    StrikethroughOffset(Option<f32>),
    /// Size of the strikethrough decoration.
    StrikethroughSize(Option<f32>),
    /// Brush for rendering the strikethrough decoration.
    StrikethroughBrush(Option<B>),
    /// Line height.
    LineHeight(LineHeight),
    /// Extra spacing between words.
    WordSpacing(f32),
    /// Extra spacing between letters.
    LetterSpacing(f32),
    /// Control over where words can wrap.
    WordBreak(WordBreak),
    /// Control over "emergency" line-breaking.
    OverflowWrap(OverflowWrap),
    /// Control over non-"emergency" line-breaking.
    TextWrapMode(TextWrapMode),
}

/// Unresolved styles.
#[derive(Clone, PartialEq, Debug)]
pub struct TextStyle<'family, 'settings, B: Brush> {
    /// CSS `font-family` property value.
    pub font_family: FontFamily<'family>,
    /// Font size.
    pub font_size: f32,
    /// Font width.
    pub font_width: FontWidth,
    /// Font style.
    pub font_style: FontStyle,
    /// Font weight.
    pub font_weight: FontWeight,
    /// Font variation settings.
    pub font_variations: FontVariations<'settings>,
    /// Font feature settings.
    pub font_features: FontFeatures<'settings>,
    /// Locale.
    pub locale: Option<Language>,
    /// Brush for rendering text.
    pub brush: B,
    /// Underline decoration.
    pub has_underline: bool,
    /// Offset of the underline decoration.
    pub underline_offset: Option<f32>,
    /// Size of the underline decoration.
    pub underline_size: Option<f32>,
    /// Brush for rendering the underline decoration.
    pub underline_brush: Option<B>,
    /// Strikethrough decoration.
    pub has_strikethrough: bool,
    /// Offset of the strikethrough decoration.
    pub strikethrough_offset: Option<f32>,
    /// Size of the strikethrough decoration.
    pub strikethrough_size: Option<f32>,
    /// Brush for rendering the strikethrough decoration.
    pub strikethrough_brush: Option<B>,
    /// Line height.
    pub line_height: LineHeight,
    /// Extra spacing between words.
    pub word_spacing: f32,
    /// Extra spacing between letters.
    pub letter_spacing: f32,
    /// Control over where words can wrap.
    pub word_break: WordBreak,
    /// Control over "emergency" line-breaking.
    pub overflow_wrap: OverflowWrap,
    /// Control over non-"emergency" line-breaking.
    pub text_wrap_mode: TextWrapMode,
}

impl<B: Brush> Default for TextStyle<'static, 'static, B> {
    fn default() -> Self {
        TextStyle {
            font_family: FontFamily::Source(Cow::Borrowed("sans-serif")),
            font_size: 16.0,
            font_width: FontWidth::default(),
            font_style: FontStyle::default(),
            font_weight: FontWeight::default(),
            font_variations: FontVariations::empty(),
            font_features: FontFeatures::empty(),
            locale: None,
            brush: B::default(),
            has_underline: false,
            underline_offset: None,
            underline_size: None,
            underline_brush: None,
            has_strikethrough: false,
            strikethrough_offset: None,
            strikethrough_size: None,
            strikethrough_brush: None,
            line_height: LineHeight::default(),
            word_spacing: 0.0,
            letter_spacing: 0.0,
            word_break: WordBreak::default(),
            overflow_wrap: OverflowWrap::default(),
            text_wrap_mode: TextWrapMode::default(),
        }
    }
}

impl<'a, B: Brush> From<FontFamily<'a>> for StyleProperty<'a, B> {
    fn from(value: FontFamily<'a>) -> Self {
        StyleProperty::FontFamily(value)
    }
}

impl<'a, B: Brush> From<&'a [FontFamilyName<'a>]> for StyleProperty<'a, B> {
    fn from(value: &'a [FontFamilyName<'a>]) -> Self {
        StyleProperty::FontFamily(value.into())
    }
}

impl<'a, B: Brush> From<FontFamilyName<'a>> for StyleProperty<'a, B> {
    fn from(value: FontFamilyName<'a>) -> Self {
        StyleProperty::FontFamily(value.into())
    }
}

impl<'a, B: Brush> From<FontVariations<'a>> for StyleProperty<'a, B> {
    fn from(value: FontVariations<'a>) -> Self {
        StyleProperty::FontVariations(value)
    }
}

impl<'a, B: Brush> From<FontFeatures<'a>> for StyleProperty<'a, B> {
    fn from(value: FontFeatures<'a>) -> Self {
        StyleProperty::FontFeatures(value)
    }
}

impl<B: Brush> From<GenericFamily> for StyleProperty<'_, B> {
    fn from(f: GenericFamily) -> Self {
        StyleProperty::FontFamily(f.into())
    }
}

impl<B: Brush> From<LineHeight> for StyleProperty<'_, B> {
    fn from(value: LineHeight) -> Self {
        StyleProperty::LineHeight(value)
    }
}

impl<B: Brush> From<FontWidth> for StyleProperty<'_, B> {
    fn from(value: FontWidth) -> Self {
        StyleProperty::FontWidth(value)
    }
}

impl<B: Brush> From<FontStyle> for StyleProperty<'_, B> {
    fn from(value: FontStyle) -> Self {
        StyleProperty::FontStyle(value)
    }
}

impl<B: Brush> From<FontWeight> for StyleProperty<'_, B> {
    fn from(value: FontWeight) -> Self {
        StyleProperty::FontWeight(value)
    }
}

impl<B: Brush> From<WordBreak> for StyleProperty<'_, B> {
    fn from(value: WordBreak) -> Self {
        StyleProperty::WordBreak(value)
    }
}

impl<B: Brush> From<OverflowWrap> for StyleProperty<'_, B> {
    fn from(value: OverflowWrap) -> Self {
        StyleProperty::OverflowWrap(value)
    }
}

impl<B: Brush> From<TextWrapMode> for StyleProperty<'_, B> {
    fn from(value: TextWrapMode) -> Self {
        StyleProperty::TextWrapMode(value)
    }
}
