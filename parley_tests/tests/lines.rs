// Copyright 2026 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Line layout tests.
//!
//! These tests validate the vertical positioning of lines, including metrics like
//! ascent, descent, and line box height.

use crate::test_name;
use crate::util::{ColorBrush, TestEnv};
use parley::{
    Affinity, Alignment, AlignmentOptions, BoundingBox, Brush, Cursor, InlineBox, InlineBoxKind,
    Layout, LineHeight, Selection, StyleProperty,
};
use peniko::kurbo::Size;

const TEXT: &str = "Some text here. Let's make\n\
        it a bit longer so that\n \
        we have more lines.\n\
        And also some latin text for\n\
        this spot right here.\n\
        This is underline and\n\
        strikethrough text and some\n\
        extra more text\n\
        and some extra more text\nand some extra more text\n\
        and some extra more text\nand some extra more text\n\
        and some extra more text\nand some extra more text";

const LINE_COUNT: f32 = 14.;

/// Returns the integral max advance that these tests are designed around.
fn max_advance(font_size: f32) -> f32 {
    (200. / 16. * font_size).ceil()
}

/// Returns the precise ascent and descent of Roboto.
fn roboto_ascent_descent(font_size: f32) -> (f32, f32) {
    // We calculate it based on these 16px values
    let ascent = 14.84375;
    let descent = 3.90625;
    (ascent / 16. * font_size, descent / 16. * font_size)
}

/// Returns the expected outputs for ascent, descent, and line box height.
fn ascent_descent_box_height(font_size: f32, line_height_px: f32) -> (f32, f32, f32) {
    let (ascent, descent) = roboto_ascent_descent(font_size);
    // Ascent and descent must be rounded separately to match the line box height of Chrome.
    // See lines_integral_line_height_ascent_descent_rounding() for more details.
    let ascent_descent = ascent.round() + descent.round();
    // Line box height does not get reduced by negative leading, so clamp leading to zero.
    let line_box_height = ascent_descent + (line_height_px.round() - ascent_descent).max(0.);
    (ascent, descent, line_box_height)
}

/// Returns selection geometry such that every line is covered.
fn get_selections<B: Brush>(layout: &Layout<B>) -> Vec<(BoundingBox, usize)> {
    let selection_parts = [
        ("Some text here.", 4),
        ("a bit longer", 8),
        ("we have more", 5),
        ("also some latin", 9),
        ("this spot", 7),
        ("is under", 6),
        ("ough text", 4),
        ("more", 4),
        ("me ex", 5),
        ("some", 4),
        ("me ex", 5),
        ("some", 4),
        ("me ex", 5),
        ("some", 4),
    ];

    let mut selections = Vec::new();
    let mut idx = 0;
    for (substr, len) in selection_parts {
        let i = idx + TEXT[idx..].find(substr).unwrap();
        let j = i + len;
        idx = j;
        let selection = Selection::new(
            Cursor::from_byte_index(layout, i, Affinity::Downstream),
            Cursor::from_byte_index(layout, j, Affinity::Downstream),
        );
        selections.extend(selection.geometry(layout));
    }
    selections
}

/// Returns the layout.
fn build_layout<A: Into<Option<f32>>>(
    env: &mut TestEnv,
    font_size: f32,
    line_height: f32,
    max_advance: A,
) -> Layout<ColorBrush> {
    let mut builder = env.ranged_builder(TEXT);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(LineHeight::Absolute(line_height));

    let underline_style = StyleProperty::Underline(true);
    let strikethrough_style = StyleProperty::Strikethrough(true);

    // Set the underline & strikethrough style
    let pos_u = TEXT.find("underline").unwrap();
    builder.push(underline_style, pos_u..pos_u + "underline".len());
    let pos_s = TEXT.find("strikethrough").unwrap();
    builder.push(strikethrough_style, pos_s..pos_s + "strikethrough".len());

    builder.push_inline_box(InlineBox {
        id: 0,
        kind: InlineBoxKind::InFlow,
        index: 40,
        width: 50.0,
        height: 5.0,
    });
    builder.push_inline_box(InlineBox {
        id: 1,
        kind: InlineBoxKind::InFlow,
        index: 51,
        width: 50.0,
        height: 3.0,
    });

    let mut layout = builder.build(TEXT);
    layout.break_all_lines(max_advance.into());
    layout
}

/// Returns the test environment, the layout, and the selections.
fn compute(
    test_name: &str,
    font_size: f32,
    line_height_px: f32,
) -> (TestEnv, Layout<ColorBrush>, Vec<(BoundingBox, usize)>) {
    // Use max advance as the target width to ensure consistency in case of early line breaks
    let width = max_advance(font_size);
    // Calculate precise total height based on requested line height,
    // then round it to match Chrome.
    let height = (LINE_COUNT * line_height_px).round();
    // Compute the layout
    let size = Size::new(width as f64, height as f64);
    let mut env = TestEnv::new(test_name, size);
    let layout = build_layout(&mut env, font_size, line_height_px, width);
    let selections = get_selections(&layout);
    (env, layout, selections)
}

/// Assert expectations that are common across tests.
fn assert_common_truths(
    layout: &Layout<ColorBrush>,
    ascent: f32,
    descent: f32,
    line_box_height: f32,
    line_height_px: f32,
) {
    let layout_height = LINE_COUNT * line_height_px;
    assert_eq!(
        layout.height(),
        layout_height,
        "expected layout height {layout_height}"
    );
    for line in layout.lines() {
        let metrics = line.metrics();
        assert_eq!(metrics.ascent, ascent, "expected ascent {ascent}");
        assert_eq!(metrics.descent, descent, "expected descent {descent}");
        assert_eq!(
            metrics.block_max_coord - metrics.block_min_coord,
            line_box_height,
            "expected line box height {line_box_height}"
        );
        assert_eq!(
            metrics.line_height, line_height_px,
            "expected line height {line_height_px}"
        );
        assert_eq!(metrics.baseline.fract(), 0., "expected integral baseline");
    }
}

/// Test the happy path of integral line height with no leading.
#[test]
fn lines_integral_line_height_zero_leading() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 19.0;

    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name!(), font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            ascent.fract() >= 0.5,
            "expected ascent {ascent} to round up"
        );
        assert!(
            descent.fract() >= 0.5,
            "expected descent {descent} to round up"
        );
        let leading = metrics.leading - (1. - ascent.fract()) - (1. - descent.fract());
        assert_eq!(leading, 0., "expected zero leading");
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test integral line height that gives a negative leading of -1.
///
/// The line box height must not be reduced by the negative leading.
#[test]
fn lines_integral_line_height_minus_one_leading() {
    // Inputs
    let font_size = 17.0;
    let line_height_px = 19.0;

    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name!(), font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            ascent.fract() >= 0.5,
            "expected ascent {ascent} to round up"
        );
        assert!(
            descent.fract() < 0.5,
            "expected descent {descent} to round down"
        );
        let leading = metrics.leading - (1. - ascent.fract()) + descent.fract();
        assert_eq!(leading, -1., "expected -1 leading");
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test integral line height that gives a positive leading of 1.
///
/// The line box height must be increased by the leading on the correct side of the baseline.
#[test]
fn lines_integral_line_height_plus_one_leading() {
    // Inputs
    let font_size = 15.0;
    let line_height_px = 19.0;

    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name!(), font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            ascent.fract() >= 0.5,
            "expected ascent {ascent} to round up"
        );
        assert!(
            descent.fract() >= 0.5,
            "expected descent {descent} to round up"
        );
        let leading = metrics.leading - (1. - ascent.fract()) - (1. - descent.fract());
        assert_eq!(leading, 1., "expected +1 leading");

        let above = line.metrics().baseline - line.metrics().block_min_coord;
        assert_eq!(
            above,
            ascent.round(),
            "expected above to be exactly rounded ascent {}",
            ascent.round()
        );
        let below = line.metrics().block_max_coord - line.metrics().baseline;
        assert_eq!(
            below,
            descent.round() + 1.,
            "expected below to be exactly rounded descent {} + 1. = {}",
            descent.round(),
            descent.round() + 1.
        );
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test that ascent and descent are rounded separately and then summed.
///
/// With Roboto 19.0px the ascent and descent are 17.626953 and 4.638672.
/// When rounding before summing we get 23, when rounding after summing we get 22.
/// Chrome renders the selection box with a height of 23px.
///
/// Roboto 20.0px would be another example with 18.554688 and 4.8828125.
#[test]
fn lines_integral_line_height_ascent_descent_rounding() {
    // Inputs
    let font_size = 19.0;
    let line_height_px = 20.0; // Just something lower than 23.0 (ascent+descent)

    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name!(), font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    for line in layout.lines() {
        let ascent_descent_round_before_sum =
            line.metrics().ascent.round() + line.metrics().descent.round();
        let ascent_descent_round_after_sum =
            (line.metrics().ascent + line.metrics().descent).round();
        assert_ne!(
            ascent_descent_round_before_sum, ascent_descent_round_after_sum,
            "expected ascent and descent to be such that the ordering of round and sum matters"
        );
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test handling of line height that rounds up both individually and also as the total layout sum.
#[test]
fn lines_line_height_rounds_up() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 20.7; // Greater than ascent + descent

    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name!(), font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    assert!(
        layout.height().fract() >= 0.5,
        "expected layout height to be fractional and round up"
    );
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            ascent + descent < metrics.line_height,
            "expected line height {} to be greater than {} (ascent {} + descent {})",
            metrics.line_height,
            ascent + descent,
            ascent,
            descent,
        );
        assert!(
            metrics.line_height.fract() >= 0.5,
            "expected line height {} to be fractional and round up",
            metrics.line_height,
        );
        assert_eq!(
            metrics.line_height.ceil(),
            line_box_height,
            "expected line box height {} to equal line height {} rounded up",
            line_box_height,
            metrics.line_height,
        );
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test handling of line height that rounds down both individually and also as the total layout sum.
#[test]
fn lines_line_height_rounds_down() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 20.3; // Greater than ascent + descent

    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name!(), font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    assert!(
        layout.height().fract() > 0. && layout.height().fract() < 0.5,
        "expected layout height to be fractional and round down"
    );
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            ascent + descent < metrics.line_height,
            "expected line height {} to be greater than {} (ascent {} + descent {})",
            metrics.line_height,
            ascent + descent,
            ascent,
            descent,
        );
        assert!(
            metrics.line_height.fract() > 0. && metrics.line_height.fract() < 0.5,
            "expected line height {} to be fractional and round down",
            metrics.line_height,
        );
        assert_eq!(
            metrics.line_height.floor(),
            line_box_height,
            "expected line box height {} to equal line height {} rounded down",
            line_box_height,
            metrics.line_height,
        );
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test fractional line height with a negative leading.
///
/// The line box height must not be reduced by the negative leading.
fn lines_fractional_line_height_negative_leading_internal(
    test_name: &str,
    font_size: f32,
    line_height_px: f32,
) {
    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name, font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            metrics.leading < 0.,
            "expected negative leading, but got {}",
            metrics.leading
        );
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test fractional line height with a negative leading.
///
/// The line box height must not be reduced by the negative leading.
#[test]
fn lines_fractional_line_height_negative_leading() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 12.75;

    // Run the test
    lines_fractional_line_height_negative_leading_internal(test_name!(), font_size, line_height_px);
}

/// Test fractional line height with a big negative leading.
///
/// The line box height must not be reduced by the negative leading.
///
/// NOTE: Going even lower (than the 0.675em in this test) will start divergence from Chrome.
#[test]
fn lines_fractional_line_height_big_negative_leading() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 10.8;

    // Run the test
    lines_fractional_line_height_negative_leading_internal(test_name!(), font_size, line_height_px);
}

/// Test fractional line height with a positive leading.
///
/// The line box height must be increased by the leading,
/// divided correctly between the sides of the baseline.
fn lines_fractional_line_height_positive_leading_internal(
    test_name: &str,
    font_size: f32,
    line_height_px: f32,
) {
    // Expected outputs
    let (ascent, descent, line_box_height) = ascent_descent_box_height(font_size, line_height_px);

    // Compute
    let (mut env, layout, selections) = compute(test_name, font_size, line_height_px);

    // Confirm metrics
    assert_common_truths(&layout, ascent, descent, line_box_height, line_height_px);
    for line in layout.lines() {
        let metrics = line.metrics();
        assert!(
            metrics.leading > 0.,
            "expected positive leading, but got {}",
            metrics.leading
        );

        let above = metrics.baseline - metrics.block_min_coord;
        let below = metrics.block_max_coord - metrics.baseline;
        let above_leading = above - ascent.round();
        let below_leading = below - descent.round();
        assert!(
            above_leading < below_leading,
            "expected above leading {above_leading} to be less than below leading {below_leading}"
        );
    }

    // Verify visuals
    env.render_and_check_snapshot(&layout, None, &selections);
}

/// Test fractional line height with a positive leading.
///
/// The line box height must be increased by the leading,
/// divided correctly between the sides of the baseline.
#[test]
fn lines_fractional_line_height_positive_leading() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 24.1;

    // Run the test
    lines_fractional_line_height_positive_leading_internal(test_name!(), font_size, line_height_px);
}

/// Test fractional line height with a big positive leading.
///
/// The line box height must be increased by the leading,
/// divided correctly between the sides of the baseline.
#[test]
fn lines_fractional_line_height_big_positive_leading() {
    // Inputs
    let font_size = 16.0;
    let line_height_px = 46.66;

    lines_fractional_line_height_positive_leading_internal(test_name!(), font_size, line_height_px);
}

#[test]
fn lines_line_height_metrics_relative() {
    let mut env = TestEnv::new(test_name!(), None);

    let mut builder = env.ranged_builder(TEXT);
    builder.push_default(LineHeight::MetricsRelative(1.1));

    let mut layout = builder.build(TEXT);

    layout.break_all_lines(None);
    layout.align(Alignment::Start, AlignmentOptions::default());
    env.check_layout_snapshot(&layout);
}

#[test]
fn lines_line_height_size_relative() {
    let mut env = TestEnv::new(test_name!(), None);

    let mut builder = env.ranged_builder(TEXT);
    builder.push_default(LineHeight::FontSizeRelative(1.2));

    let mut layout = builder.build(TEXT);

    layout.break_all_lines(None);
    layout.align(Alignment::Start, AlignmentOptions::default());
    env.check_layout_snapshot(&layout);
}

#[test]
fn lines_line_height_absolute() {
    let mut env = TestEnv::new(test_name!(), None);

    let mut builder = env.ranged_builder(TEXT);
    builder.push_default(LineHeight::Absolute(20.0));

    let mut layout = builder.build(TEXT);

    layout.break_all_lines(None);
    layout.align(Alignment::Start, AlignmentOptions::default());
    env.check_layout_snapshot(&layout);
}

// ---------------------------------------------------------------------------
// `LineHeight::Normal`: the line box is sized from every font used on the line.
// ---------------------------------------------------------------------------

/// The precise ascent and descent of Noto Kufi Arabic, the second family in the test
/// font stack. `hhea` is 1282 / -615 per 1000 upem, and `USE_TYPO_METRICS` is set with
/// the same values, so both the `FreeType` and the Blink selection agree.
///
/// (`USE_TYPO_METRICS` is `OS/2` `fsSelection` bit 7.)
fn noto_kufi_ascent_descent(font_size: f32) -> (f32, f32) {
    (1.282 * font_size, 0.615 * font_size)
}

/// Lay `text` out on a single line with the given `line_height` and return its metrics.
fn single_line_metrics(
    env: &mut TestEnv,
    text: &str,
    font_size: f32,
    line_height: LineHeight,
) -> parley::LineMetrics {
    let mut builder = env.ranged_builder(text);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(line_height);
    let mut layout: Layout<ColorBrush> = builder.build(text);
    layout.break_all_lines(None);
    let lines: Vec<_> = layout.lines().collect();
    assert_eq!(lines.len(), 1, "expected {text:?} to fit on one line");
    *lines[0].metrics()
}

/// `line-height: normal` sizes the line box from the union of the fonts on the line.
///
/// CSS Inline Layout 3 [5.3] sizes a line box "to exactly include the aligned layout bounds
/// of all its inline-level boxes", and states that "metrics from fonts other than the first
/// available font only impact the layout bounds of an inline box with `line-height: normal`".
///
/// Chromium does this in `InlineBoxState::AccumulateUsedFonts`, which unites the `FontHeight`
/// of every font in the run's `ShapeResult::UsedFonts()` into the box's metrics, gated on
/// `include_used_fonts`, which `InlineBoxState::ComputeTextMetrics` sets from
/// `styleref.LineHeight().IsAuto()`.
///
/// At 20px Roboto is 18.555 / 4.883 -> 19 + 5 = 24 and Noto Kufi Arabic is 25.64 / 12.3 ->
/// 26 + 12 = 38. A line drawing from both is 38, not 24.
///
/// [5.3]: https://drafts.csswg.org/css-inline-3/#inline-height
#[test]
fn lines_normal_line_height_unions_the_fonts_on_the_line() {
    let font_size = 20.;
    let mut env = TestEnv::new(test_name!(), None);

    let (roboto_ascent, roboto_descent) = roboto_ascent_descent(font_size);
    let (kufi_ascent, kufi_descent) = noto_kufi_ascent_descent(font_size);
    let strut = LineHeight::Normal {
        strut_ascent: roboto_ascent,
        strut_descent: roboto_descent,
    };

    // Latin only: the first available font is the only font used.
    let latin = single_line_metrics(&mut env, "Latin", font_size, strut);
    assert_eq!(
        latin.line_height,
        roboto_ascent.round() + roboto_descent.round()
    );
    assert_eq!(latin.line_height, 24.);

    // The same style, with one Arabic word that Roboto does not cover. Noto Kufi Arabic is
    // taller in both directions, so the line box grows to its bounds.
    let mixed = single_line_metrics(
        &mut env,
        "Latin \u{645}\u{631}\u{62d}\u{628}\u{627}",
        font_size,
        strut,
    );
    assert_eq!(
        mixed.line_height,
        kufi_ascent.round() + kufi_descent.round()
    );
    assert_eq!(mixed.line_height, 38.);
}

/// The union is taken per metric, not per height.
///
/// css-inline-3 5.3 describes the layout bounds as an ascent `A` above and a descent `D`
/// below the baseline; Chromium unites `FontHeight` values, which are `(ascent, descent)`
/// pairs. Taking the taller of the contributors' *heights* is a different, smaller number
/// whenever one contributor has the greater ascent and another the greater descent.
///
/// A strut of 30 / 1 is 31 tall and Roboto at 20px is 18.555 / 4.883, 24 tall; the union of
/// the pair is 30 + 5 = 35, which is taller than either.
#[test]
fn lines_normal_line_height_unions_ascent_and_descent_separately() {
    let font_size = 20.;
    let mut env = TestEnv::new(test_name!(), None);

    let (roboto_ascent, roboto_descent) = roboto_ascent_descent(font_size);
    let metrics = single_line_metrics(
        &mut env,
        "Latin",
        font_size,
        LineHeight::Normal {
            strut_ascent: 30.,
            strut_descent: 1.,
        },
    );

    let taller_of_the_two_heights =
        (30.0_f32.round() + 1.0_f32.round()).max(roboto_ascent.round() + roboto_descent.round());
    assert_eq!(taller_of_the_two_heights, 31.);
    assert_eq!(metrics.line_height, 35.);
}

/// Every `line-height` other than `normal` is a fixed height a fallback font must not grow.
///
/// css-inline-3 5.3: "metrics from fonts other than the first available font **only** impact
/// the layout bounds of an inline box with `line-height: normal`".
#[test]
fn lines_fixed_line_height_is_not_grown_by_a_fallback_font() {
    let font_size = 20.;
    let mut env = TestEnv::new(test_name!(), None);
    let arabic = "Latin \u{645}\u{631}\u{62d}\u{628}\u{627}";

    for line_height in [LineHeight::Absolute(24.), LineHeight::FontSizeRelative(1.2)] {
        let latin = single_line_metrics(&mut env, "Latin", font_size, line_height);
        let mixed = single_line_metrics(&mut env, arabic, font_size, line_height);
        assert_eq!(latin.line_height, 24.);
        assert_eq!(
            mixed.line_height, 24.,
            "{line_height:?} is a fixed height, so a taller fallback font must not grow it"
        );
    }
}

/// The growth is per line box, not per span.
///
/// A fallback glyph on one line of a wrapped paragraph must not grow the others. An
/// implementation that resolves one taller `line-height` for a whole span because some glyph
/// in it falls back over-approximates here.
#[test]
fn lines_only_the_line_using_the_fallback_font_grows() {
    let font_size = 20.;
    let mut env = TestEnv::new(test_name!(), None);

    let (roboto_ascent, roboto_descent) = roboto_ascent_descent(font_size);
    let (kufi_ascent, kufi_descent) = noto_kufi_ascent_descent(font_size);
    let roboto_line = roboto_ascent.round() + roboto_descent.round();
    let kufi_line = kufi_ascent.round() + kufi_descent.round();

    let text = "\u{645}\u{631}\u{62d}\u{628}\u{627}\nLatin";
    let mut builder = env.ranged_builder(text);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(LineHeight::Normal {
        strut_ascent: roboto_ascent,
        strut_descent: roboto_descent,
    });
    let mut layout: Layout<ColorBrush> = builder.build(text);
    layout.break_all_lines(None);

    let heights: Vec<f32> = layout.lines().map(|l| l.metrics().line_height).collect();
    assert_eq!(
        heights,
        vec![kufi_line, roboto_line],
        "only the Arabic line takes Noto Kufi Arabic's bounds"
    );
    assert_eq!(heights, vec![38., 24.]);
}

/// A run's own `line-height` decides whether *its* fonts may grow the line box.
///
/// css-inline-3 5.3 is per inline box: "when its computed line-height is not normal, its
/// layout bounds are derived solely from metrics of its first available font". Chromium
/// enforces that per box -- `include_used_fonts` gates `AccumulateUsedFonts` inside one
/// `InlineBoxState`, and each already-gated box then bubbles up through
/// `parent_box.metrics.Unite(box->metrics)`.
///
/// So a span with a fixed `line-height` contributes exactly that height, and its font metrics
/// must not reach the `normal` resolution of an unrelated sibling on the same line.
///
/// Here a 10px `normal` run (9.277 / 2.441 -> 9 + 2 = 11) sits beside a 100px run pinned to
/// `Absolute(5.)`. The line is 11: the 100px run asks for 5, the small one for 11. Letting the
/// 100px run's 92.773 / 24.414 metrics into the `normal` union would give 93 + 24 = 117.
#[test]
fn lines_a_fixed_line_height_run_does_not_grow_a_normal_sibling() {
    let small_font_size = 10.;
    let mut env = TestEnv::new(test_name!(), None);

    let (small_ascent, small_descent) = roboto_ascent_descent(small_font_size);

    let text = "a BIG";
    let big = text.find("BIG").unwrap();
    let mut builder = env.ranged_builder(text);
    builder.push_default(StyleProperty::FontSize(small_font_size));
    builder.push(
        StyleProperty::LineHeight(LineHeight::Normal {
            strut_ascent: small_ascent,
            strut_descent: small_descent,
        }),
        0..big,
    );
    // A sibling inline box: much bigger font, and a `line-height` that is not `normal`.
    builder.push(StyleProperty::FontSize(100.), big..text.len());
    builder.push(
        StyleProperty::LineHeight(LineHeight::Absolute(5.)),
        big..text.len(),
    );

    let mut layout: Layout<ColorBrush> = builder.build(text);
    layout.break_all_lines(None);
    let lines: Vec<_> = layout.lines().collect();
    assert_eq!(lines.len(), 1);
    let metrics = *lines[0].metrics();

    // Sanity: the two runs really do have the styles this test is about.
    let runs: Vec<_> = lines[0].runs().collect();
    assert_eq!(runs.len(), 2);
    assert!(
        runs[0].metrics().normal_strut.is_some(),
        "the small run is `normal`"
    );
    assert!(
        runs[1].metrics().normal_strut.is_none(),
        "the big run has a fixed line height"
    );
    assert!(runs[1].metrics().ascent > 90.);

    // The 100px run's own metrics are on the line -- they place the baseline -- but they do
    // not resolve the `normal` sibling's line height.
    assert!(
        metrics.ascent > 90.,
        "expected the 100px run's ascent on the line, got {}",
        metrics.ascent
    );
    assert_eq!(
        metrics.line_height,
        small_ascent.round() + small_descent.round(),
        "the `normal` run resolves from its own fonts only"
    );
    assert_eq!(metrics.line_height, 11.);
}
