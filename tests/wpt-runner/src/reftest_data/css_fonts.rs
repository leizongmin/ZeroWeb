use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── 1. font-size: absolute keywords, px, em, percentage (7 cases) ──

    // font-size: small
    InlineReftestDef {
        id: "css-fonts/font-size-small",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:small\">Small text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:small\">Small text</div></body></html>",
        is_match: true,
    },
    // font-size: medium
    InlineReftestDef {
        id: "css-fonts/font-size-medium",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:medium\">Medium text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:medium\">Medium text</div></body></html>",
        is_match: true,
    },
    // font-size: large
    InlineReftestDef {
        id: "css-fonts/font-size-large",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:large\">Large text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:large\">Large text</div></body></html>",
        is_match: true,
    },
    // font-size: x-large
    InlineReftestDef {
        id: "css-fonts/font-size-x-large",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:x-large\">Extra large text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:x-large\">Extra large text</div></body></html>",
        is_match: true,
    },
    // font-size: 24px
    InlineReftestDef {
        id: "css-fonts/font-size-24px",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px\">24px text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px\">24px text</div></body></html>",
        is_match: true,
    },
    // font-size: 1.5em
    InlineReftestDef {
        id: "css-fonts/font-size-em",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:1.5em\">One point five em text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:1.5em\">One point five em text</div></body></html>",
        is_match: true,
    },
    // font-size: 150%
    InlineReftestDef {
        id: "css-fonts/font-size-percent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:150%\">150 percent text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:150%\">150 percent text</div></body></html>",
        is_match: true,
    },
    // ── 2. font-weight: normal, bold, 100-900 (7 cases) ──

    // font-weight: normal
    InlineReftestDef {
        id: "css-fonts/font-weight-normal",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:normal\">Normal weight</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:normal\">Normal weight</div></body></html>",
        is_match: true,
    },
    // font-weight: bold
    InlineReftestDef {
        id: "css-fonts/font-weight-bold",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold\">Bold weight</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold\">Bold weight</div></body></html>",
        is_match: true,
    },
    // font-weight: 100 (thin)
    InlineReftestDef {
        id: "css-fonts/font-weight-100",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:100\">Thin weight</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:100\">Thin weight</div></body></html>",
        is_match: true,
    },
    // font-weight: 300 (light)
    InlineReftestDef {
        id: "css-fonts/font-weight-300",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:300\">Light weight</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:300\">Light weight</div></body></html>",
        is_match: true,
    },
    // font-weight: 400 (normal)
    InlineReftestDef {
        id: "css-fonts/font-weight-400",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:400\">Weight 400</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:400\">Weight 400</div></body></html>",
        is_match: true,
    },
    // font-weight: 700 (bold)
    InlineReftestDef {
        id: "css-fonts/font-weight-700",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:700\">Weight 700</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:700\">Weight 700</div></body></html>",
        is_match: true,
    },
    // font-weight: 900 (black)
    InlineReftestDef {
        id: "css-fonts/font-weight-900",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:900\">Black weight</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:900\">Black weight</div></body></html>",
        is_match: true,
    },
    // ── 3. font-style: normal, italic, oblique (5 cases) ──

    // font-style: normal
    InlineReftestDef {
        id: "css-fonts/font-style-normal",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-style:normal\">Normal style</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-style:normal\">Normal style</div></body></html>",
        is_match: true,
    },
    // font-style: italic
    InlineReftestDef {
        id: "css-fonts/font-style-italic",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic\">Italic style</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic\">Italic style</div></body></html>",
        is_match: true,
    },
    // font-style: oblique
    InlineReftestDef {
        id: "css-fonts/font-style-oblique",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-style:oblique\">Oblique style</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-style:oblique\">Oblique style</div></body></html>",
        is_match: true,
    },
    // font-style: italic on bold text
    InlineReftestDef {
        id: "css-fonts/font-style-italic-bold",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic;font-weight:bold\">Bold italic</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic;font-weight:bold\">Bold italic</div></body></html>",
        is_match: true,
    },
    // font-style: normal override italic parent
    InlineReftestDef {
        id: "css-fonts/font-style-normal-override",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic\"><span style=\"font-style:normal\">Reset to normal</span></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic\"><span style=\"font-style:normal\">Reset to normal</span></div></body></html>",
        is_match: true,
    },
    // ── 4. font-family: generic families (5 cases) ──

    // font-family: serif
    InlineReftestDef {
        id: "css-fonts/font-family-serif",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-family:serif\">Serif text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-family:serif\">Serif text</div></body></html>",
        is_match: true,
    },
    // font-family: sans-serif
    InlineReftestDef {
        id: "css-fonts/font-family-sans-serif",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-family:sans-serif\">Sans serif text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-family:sans-serif\">Sans serif text</div></body></html>",
        is_match: true,
    },
    // font-family: monospace
    InlineReftestDef {
        id: "css-fonts/font-family-monospace",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-family:monospace\">Monospace text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-family:monospace\">Monospace text</div></body></html>",
        is_match: true,
    },
    // font-family: cursive
    InlineReftestDef {
        id: "css-fonts/font-family-cursive",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-family:cursive\">Cursive text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-family:cursive\">Cursive text</div></body></html>",
        is_match: true,
    },
    // font-family: fantasy
    InlineReftestDef {
        id: "css-fonts/font-family-fantasy",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-family:fantasy\">Fantasy text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-family:fantasy\">Fantasy text</div></body></html>",
        is_match: true,
    },
    // ── 5. font shorthand: various combinations (5 cases) ──

    // font shorthand: size and family
    InlineReftestDef {
        id: "css-fonts/font-shorthand-size-family",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font:16px serif\">Shorthand size family</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font:16px serif\">Shorthand size family</div></body></html>",
        is_match: true,
    },
    // font shorthand: italic size family
    InlineReftestDef {
        id: "css-fonts/font-shorthand-italic",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font:italic 16px sans-serif\">Shorthand italic</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font:italic 16px sans-serif\">Shorthand italic</div></body></html>",
        is_match: true,
    },
    // font shorthand: bold size family
    InlineReftestDef {
        id: "css-fonts/font-shorthand-bold",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font:bold 20px monospace\">Shorthand bold</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font:bold 20px monospace\">Shorthand bold</div></body></html>",
        is_match: true,
    },
    // font shorthand: bold italic size family
    InlineReftestDef {
        id: "css-fonts/font-shorthand-bold-italic",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font:bold italic 18px serif\">Shorthand bold italic</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font:bold italic 18px serif\">Shorthand bold italic</div></body></html>",
        is_match: true,
    },
    // font shorthand: weight size/line-height family
    InlineReftestDef {
        id: "css-fonts/font-shorthand-weight-size-lh",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font:bold 14px/1.8 sans-serif\">Shorthand with line height</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font:bold 14px/1.8 sans-serif\">Shorthand with line height</div></body></html>",
        is_match: true,
    },
    // ── 6. line-height with fonts: normal, numeric, px, percentage (5 cases) ──

    // line-height: normal with explicit font-size
    InlineReftestDef {
        id: "css-fonts/line-height-normal",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px;line-height:normal\">Line height normal</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px;line-height:normal\">Line height normal</div></body></html>",
        is_match: true,
    },
    // line-height: 2 (numeric multiplier)
    InlineReftestDef {
        id: "css-fonts/line-height-numeric",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:2\">Double line height<br>Second line</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:2\">Double line height<br>Second line</div></body></html>",
        is_match: true,
    },
    // line-height: 32px (absolute)
    InlineReftestDef {
        id: "css-fonts/line-height-px",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:32px\">32px line height<br>Second line</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:32px\">32px line height<br>Second line</div></body></html>",
        is_match: true,
    },
    // line-height: 200% (percentage)
    InlineReftestDef {
        id: "css-fonts/line-height-percent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:200%\">200 percent line height<br>Second line</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:200%\">200 percent line height<br>Second line</div></body></html>",
        is_match: true,
    },
    // line-height: 1.5 with large font
    InlineReftestDef {
        id: "css-fonts/line-height-large-font",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:28px;line-height:1.5\">Big text with line height<br>Second line</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:28px;line-height:1.5\">Big text with line height<br>Second line</div></body></html>",
        is_match: true,
    },
    // ── 7. letter-spacing with fonts: combined effects (5 cases) ──

    // letter-spacing: 3px with large font
    InlineReftestDef {
        id: "css-fonts/letter-spacing-large-font",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px;letter-spacing:3px\">Large spaced text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px;letter-spacing:3px\">Large spaced text</div></body></html>",
        is_match: true,
    },
    // letter-spacing: 5px with bold
    InlineReftestDef {
        id: "css-fonts/letter-spacing-bold",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold;letter-spacing:5px\">Bold spaced text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold;letter-spacing:5px\">Bold spaced text</div></body></html>",
        is_match: true,
    },
    // letter-spacing: 0px (explicit reset)
    InlineReftestDef {
        id: "css-fonts/letter-spacing-zero",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;letter-spacing:0px\">Zero spacing</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;letter-spacing:0px\">Zero spacing</div></body></html>",
        is_match: true,
    },
    // letter-spacing: normal (default)
    InlineReftestDef {
        id: "css-fonts/letter-spacing-normal",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;letter-spacing:normal\">Normal spacing</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;letter-spacing:normal\">Normal spacing</div></body></html>",
        is_match: true,
    },
    // letter-spacing: 4px with italic serif
    InlineReftestDef {
        id: "css-fonts/letter-spacing-italic-serif",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic;font-family:serif;letter-spacing:4px\">Italic serif spaced</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-style:italic;font-family:serif;letter-spacing:4px\">Italic serif spaced</div></body></html>",
        is_match: true,
    },
    // ── 8. Font sizing in flex/grid containers (5 cases) ──

    // font-size in flex row container
    InlineReftestDef {
        id: "css-fonts/font-size-in-flex-row",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"font-size:20px\">Large</div><div style=\"font-size:12px\">Small</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"font-size:20px\">Large</div><div style=\"font-size:12px\">Small</div></div></body></html>",
        is_match: true,
    },
    // font-size in flex column container
    InlineReftestDef {
        id: "css-fonts/font-size-in-flex-col",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:200px\"><div style=\"font-size:18px\">Row one</div><div style=\"font-size:14px\">Row two</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:200px\"><div style=\"font-size:18px\">Row one</div><div style=\"font-size:14px\">Row two</div></div></body></html>",
        is_match: true,
    },
    // font-size in grid container
    InlineReftestDef {
        id: "css-fonts/font-size-in-grid",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px\"><div style=\"font-size:16px\">Cell A</div><div style=\"font-size:24px\">Cell B</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px\"><div style=\"font-size:16px\">Cell A</div><div style=\"font-size:24px\">Cell B</div></div></body></html>",
        is_match: true,
    },
    // bold text in flex container
    InlineReftestDef {
        id: "css-fonts/bold-in-flex",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"font-weight:bold\">Bold</div><div style=\"font-weight:normal\">Normal</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"font-weight:bold\">Bold</div><div style=\"font-weight:normal\">Normal</div></div></body></html>",
        is_match: true,
    },
    // font shorthand in grid container
    InlineReftestDef {
        id: "css-fonts/shorthand-in-grid",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:150px 150px;width:300px\"><div style=\"font:bold 16px serif\">Grid one</div><div style=\"font:italic 14px sans-serif\">Grid two</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:150px 150px;width:300px\"><div style=\"font:bold 16px serif\">Grid one</div><div style=\"font:italic 14px sans-serif\">Grid two</div></div></body></html>",
        is_match: true,
    },
    // ── 9. Nested font inheritance (6 cases) ──

    // font-size inheritance through nested divs
    InlineReftestDef {
        id: "css-fonts/nested-font-size-inherit",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px\"><div>Child inherits 20px</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px\"><div>Child inherits 20px</div></div></body></html>",
        is_match: true,
    },
    // font-weight inheritance through nested elements
    InlineReftestDef {
        id: "css-fonts/nested-font-weight-inherit",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold\"><div>Inherited bold</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold\"><div>Inherited bold</div></div></body></html>",
        is_match: true,
    },
    // font-family inheritance through nested elements
    InlineReftestDef {
        id: "css-fonts/nested-font-family-inherit",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-family:monospace\"><div>Inherited monospace</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-family:monospace\"><div>Inherited monospace</div></div></body></html>",
        is_match: true,
    },
    // font-size em inheritance (compound)
    InlineReftestDef {
        id: "css-fonts/nested-em-compound",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px\"><div style=\"font-size:1.5em\">Compound em sizing</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px\"><div style=\"font-size:1.5em\">Compound em sizing</div></div></body></html>",
        is_match: true,
    },
    // line-height inheritance through nested elements
    InlineReftestDef {
        id: "css-fonts/nested-line-height-inherit",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:2\"><div>Inherited line height<br>Second line</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;line-height:2\"><div>Inherited line height<br>Second line</div></div></body></html>",
        is_match: true,
    },
    // override inherited font-size in child
    InlineReftestDef {
        id: "css-fonts/nested-font-size-override",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px\"><div style=\"font-size:10px\">Overridden to 10px</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:20px\"><div style=\"font-size:10px\">Overridden to 10px</div></div></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}
