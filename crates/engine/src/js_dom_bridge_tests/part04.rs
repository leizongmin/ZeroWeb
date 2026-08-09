#[test]
fn test_get_computed_style_table_list_font() {
    // R2735：getComputedStyle border-spacing + list-style-image + font-size-adjust 序列化。
    let html = "<html><body>\
        <div id=\"bs-eq\" style=\"border-spacing: 5px;\"></div>\
        <div id=\"bs-diff\" style=\"border-spacing: 3px 8px;\"></div>\
        <div id=\"lsi-url\" style=\"list-style-image: url(star.png);\"></div>\
        <div id=\"fsa-num\" style=\"font-size-adjust: 0.5;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // border-spacing 默认 0px；等值单值 / 不等两值。
    assert_eq!(computed_style_property(html, "#def", "border-spacing"), "0px");
    assert_eq!(computed_style_property(html, "#bs-eq", "border-spacing"), "5px");
    assert_eq!(computed_style_property(html, "#bs-diff", "border-spacing"), "3px 8px");
    // list-style-image 默认 none；url() 引号形式。
    assert_eq!(computed_style_property(html, "#def", "list-style-image"), "none");
    assert_eq!(
        computed_style_property(html, "#lsi-url", "list-style-image"),
        "url(\"star.png\")"
    );
    // font-size-adjust 默认 none；number。
    assert_eq!(computed_style_property(html, "#def", "font-size-adjust"), "none");
    assert_eq!(computed_style_property(html, "#fsa-num", "font-size-adjust"), "0.5");
}

#[test]
fn test_get_computed_style_border_img_obj_pos_quotes() {
    // R2736：getComputedStyle border-image-source + object-position + quotes 序列化。
    let html = "<html><body>\
        <div id=\"bis-url\" style=\"border-image-source: url(border.png);\"></div>\
        <div id=\"op-kw\" style=\"object-position: top left;\"></div>\
        <div id=\"op-px\" style=\"object-position: 10px 20px;\"></div>\
        <div id=\"q-none\" style=\"quotes: none;\"></div>\
        <div id='q-pairs' style='quotes: \"\u{00ab}\" \"\u{00bb}\" \"\u{2039}\" \"\u{203a}\";'></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // border-image-source 默认 none；url() 引号形式（同 list-style-image）。
    assert_eq!(computed_style_property(html, "#def", "border-image-source"), "none");
    assert_eq!(
        computed_style_property(html, "#bis-url", "border-image-source"),
        "url(\"border.png\")"
    );
    // object-position 默认 Center→50% 50%；关键字两值 / 长度两值（复用 background-position 序列化）。
    assert_eq!(computed_style_property(html, "#def", "object-position"), "50% 50%");
    assert_eq!(computed_style_property(html, "#op-kw", "object-position"), "0% 0%");
    assert_eq!(computed_style_property(html, "#op-px", "object-position"), "10px 20px");
    // quotes 初值 auto；none；pairs→空格分隔双引号串。
    assert_eq!(computed_style_property(html, "#def", "quotes"), "auto");
    assert_eq!(computed_style_property(html, "#q-none", "quotes"), "none");
    assert_eq!(
        computed_style_property(html, "#q-pairs", "quotes"),
        "\"\u{00ab}\" \"\u{00bb}\" \"\u{2039}\" \"\u{203a}\""
    );
}

#[test]
fn test_get_computed_style_multicol_fontvar_img() {
    // R2737：getComputedStyle CSS Multi-column 簇（rule-width/style/color + count/width/fill/span）
    // + font-variant-numeric + image-rendering 序列化。column-gap 已由 R2707 长度族覆盖。
    let html = "<html><body>\
        <div id=\"cr\" style=\"column-rule: 2px dashed red;\"></div>\
        <div id=\"crt\" style=\"column-rule: thick solid blue;\"></div>\
        <div id=\"cc\" style=\"column-count: 3;\"></div>\
        <div id=\"cw\" style=\"column-width: 100px;\"></div>\
        <div id=\"cf\" style=\"column-fill: auto;\"></div>\
        <div id=\"cs\" style=\"column-span: all;\"></div>\
        <div id=\"fvn\" style=\"font-variant-numeric: tabular-nums;\"></div>\
        <div id=\"ir\" style=\"image-rendering: pixelated;\"></div>\
        <div id=\"def\" style=\"color: red;\"></div>\
        </body></html>";
    // column-rule-width：长度 2px；thick→5px（UA used px）；默认 style=none 仍 medium→3px（R2755 oracle：
    // column-rule-width 的 computed 值独立于 style，不套 border-width 的 none/hidden→0px 规则，纠正 R2737 误判）。
    assert_eq!(computed_style_property(html, "#cr", "column-rule-width"), "2px");
    assert_eq!(computed_style_property(html, "#crt", "column-rule-width"), "5px");
    assert_eq!(computed_style_property(html, "#def", "column-rule-width"), "3px");
    // column-rule-style：dashed/solid；默认 none。
    assert_eq!(computed_style_property(html, "#cr", "column-rule-style"), "dashed");
    assert_eq!(computed_style_property(html, "#crt", "column-rule-style"), "solid");
    assert_eq!(computed_style_property(html, "#def", "column-rule-style"), "none");
    // column-rule-color：显式 red/blue → rgb；默认 currentcolor → 元素 color（#def color:red）。
    assert_eq!(
        computed_style_property(html, "#cr", "column-rule-color"),
        "rgb(255, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#crt", "column-rule-color"),
        "rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#def", "column-rule-color"),
        "rgb(255, 0, 0)"
    );
    // column-count：Number(3)→"3"；默认 auto。
    assert_eq!(computed_style_property(html, "#cc", "column-count"), "3");
    assert_eq!(computed_style_property(html, "#def", "column-count"), "auto");
    // column-width：100px；默认 auto。
    assert_eq!(computed_style_property(html, "#cw", "column-width"), "100px");
    assert_eq!(computed_style_property(html, "#def", "column-width"), "auto");
    // column-fill：auto；初值 balance。
    assert_eq!(computed_style_property(html, "#cf", "column-fill"), "auto");
    assert_eq!(computed_style_property(html, "#def", "column-fill"), "balance");
    // column-span：all；初值 none。
    assert_eq!(computed_style_property(html, "#cs", "column-span"), "all");
    assert_eq!(computed_style_property(html, "#def", "column-span"), "none");
    // font-variant-numeric：tabular-nums；初值 normal。
    assert_eq!(
        computed_style_property(html, "#fvn", "font-variant-numeric"),
        "tabular-nums"
    );
    assert_eq!(computed_style_property(html, "#def", "font-variant-numeric"), "normal");
    // image-rendering：pixelated；初值 auto。
    assert_eq!(computed_style_property(html, "#ir", "image-rendering"), "pixelated");
    assert_eq!(computed_style_property(html, "#def", "image-rendering"), "auto");
}

#[test]
fn test_get_computed_style_shorthands_r2755() {
    // R2755：getComputedStyle 残余简写序列化（columns / column-rule / list-style / text-decoration）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐确切串。
    let html = "<html><body>\
        <div id=\"def\" style=\"color: red;\"></div>\
        <div id=\"cw\" style=\"column-width: 100px;\"></div>\
        <div id=\"cn\" style=\"column-count: 3;\"></div>\
        <div id=\"cb\" style=\"columns: 200px 4;\"></div>\
        <div id=\"cb2\" style=\"columns: 5;\"></div>\
        <div id=\"cr\" style=\"column-rule: thick double rgb(255, 0, 0);\"></div>\
        <div id=\"crp\" style=\"column-rule: 2px solid;\"></div>\
        <div id=\"crh\" style=\"column-rule-style: hidden;\"></div>\
        <div id=\"ls\" style=\"list-style: square inside;\"></div>\
        <div id=\"lsp\" style=\"list-style: lower-alpha outside;\"></div>\
        <div id=\"lsn\" style=\"list-style: none;\"></div>\
        <div id=\"td\" style=\"text-decoration: underline overline;\"></div>\
        <div id=\"tdl\" style=\"text-decoration: line-through;\"></div>\
        <div id=\"tdp\" style=\"text-decoration: underline dotted rgb(255, 0, 0);\"></div>\
        <div id=\"tdc\" style=\"text-decoration: underline; text-decoration-color: rgb(170, 187, 204);\"></div>\
        </body></html>";
    // columns 简写：auto 省略；全 auto→"auto"；width-only→"W"；count-only→"N"；both→"W N"。
    assert_eq!(computed_style_property(html, "#def", "columns"), "auto");
    assert_eq!(computed_style_property(html, "#cw", "columns"), "100px");
    assert_eq!(computed_style_property(html, "#cn", "columns"), "3");
    assert_eq!(computed_style_property(html, "#cb", "columns"), "200px 4");
    assert_eq!(computed_style_property(html, "#cb2", "columns"), "5");
    // column-rule 简写：style=none 省略（hidden 保留）；width 恒显；color 恒显。
    // #def 默认（style none）→"3px rgb(255, 0, 0)"（color=currentcolor→元素 red）。
    assert_eq!(
        computed_style_property(html, "#def", "column-rule"),
        "3px rgb(255, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#cr", "column-rule"),
        "5px double rgb(255, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#crp", "column-rule"),
        "2px solid rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#crh", "column-rule"),
        "3px hidden rgb(0, 0, 0)"
    );
    // list-style 简写：恒 3 段 "position image type"。
    assert_eq!(computed_style_property(html, "#def", "list-style"), "outside none disc");
    assert_eq!(computed_style_property(html, "#ls", "list-style"), "inside none square");
    assert_eq!(
        computed_style_property(html, "#lsp", "list-style"),
        "outside none lower-alpha"
    );
    assert_eq!(computed_style_property(html, "#lsn", "list-style"), "outside none none");
    // text-decoration 简写：line=none→"none"；否则 line/thickness(!auto)/style(!solid)/color(!currentcolor)。
    assert_eq!(computed_style_property(html, "#def", "text-decoration"), "none");
    assert_eq!(
        computed_style_property(html, "#td", "text-decoration"),
        "underline overline"
    );
    assert_eq!(computed_style_property(html, "#tdl", "text-decoration"), "line-through");
    assert_eq!(
        computed_style_property(html, "#tdp", "text-decoration"),
        "underline dotted rgb(255, 0, 0)"
    );
    // #tdc：line + 显式 color（非 currentcolor）；style solid / thickness auto 省略。
    assert_eq!(
        computed_style_property(html, "#tdc", "text-decoration"),
        "underline rgb(170, 187, 204)"
    );
}

#[test]
fn test_get_computed_style_transition_animation_shorthand_r2756() {
    // R2756：getComputedStyle transition / animation 简写（CSSOM 列表 zip 重组）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"def\"></div>\
        <div id=\"tn\" style=\"transition: none;\"></div>\
        <div id=\"t1\" style=\"transition: margin 2s;\"></div>\
        <div id=\"t2\" style=\"transition: margin 2s ease-in 1s;\"></div>\
        <div id=\"t5\" style=\"transition: 2s;\"></div>\
        <div id=\"tm\" style=\"transition: margin 2s ease-in 1s, padding 0.5s;\"></div>\
        <div id=\"an\" style=\"animation: none;\"></div>\
        <div id=\"a1\" style=\"animation: bounce 2s;\"></div>\
        <div id=\"a2\" style=\"animation: bounce 2s linear infinite alternate;\"></div>\
        <div id=\"ad\" style=\"animation: 2s;\"></div>\
        <div id=\"ap\" style=\"animation: bounce paused;\"></div>\
        <div id=\"anm\" style=\"animation: bounce 2s ease-in 1s, spin 1s linear 2;\"></div>\
        </body></html>";
    // transition 简写：默认（空列表）→"all"；none→"none"；省初值（property=all 仅在其余全初值时显）。
    assert_eq!(computed_style_property(html, "#def", "transition"), "all");
    assert_eq!(computed_style_property(html, "#tn", "transition"), "none");
    assert_eq!(computed_style_property(html, "#t1", "transition"), "margin 2s");
    assert_eq!(
        computed_style_property(html, "#t2", "transition"),
        "margin 2s ease-in 1s"
    );
    // #t5：property=all（初值）省略，仅 duration 显。
    assert_eq!(computed_style_property(html, "#t5", "transition"), "2s");
    // 多条目逗号连接，逐索引 zip。
    assert_eq!(
        computed_style_property(html, "#tm", "transition"),
        "margin 2s ease-in 1s, padding 0.5s"
    );
    // animation 简写：默认（空列表）→"none"；none→"none"；顺序 dur/tf/delay/iter/dir/fill/play/name 省初值。
    assert_eq!(computed_style_property(html, "#def", "animation"), "none");
    assert_eq!(computed_style_property(html, "#an", "animation"), "none");
    assert_eq!(computed_style_property(html, "#a1", "animation"), "2s bounce");
    assert_eq!(
        computed_style_property(html, "#a2", "animation"),
        "2s linear infinite alternate bounce"
    );
    // #ad：name=none（初值）省略，仅 duration 显。
    assert_eq!(computed_style_property(html, "#ad", "animation"), "2s");
    // #ap：play-state=paused 显（running 初值省），duration 0s 省。
    assert_eq!(computed_style_property(html, "#ap", "animation"), "paused bounce");
    // 多条目逗号连接，逐索引 zip。
    assert_eq!(
        computed_style_property(html, "#anm", "animation"),
        "2s ease-in 1s bounce, 1s linear 2 spin"
    );
}

#[test]
fn test_get_computed_style_background_shorthand_r2757() {
    // R2757：getComputedStyle background 简写（CSSOM 完整规范形重组，无省略）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐。
    // 注：避开 url() 图层（ZW 存相对 URL，oracle 解析绝对 URL，属 pre-existing longhand 差异）。
    // 注：attachment/box 经 **longhand** 设置——ZW 的 background 简写 parser 对含 rgb()/var() 的值
    // bail-out（整体作 color，丢 attachment），且 box token 故意 drop（R2479/R2481），故用 longhand
    // 隔离测试**序列化**正确性（本切片范围），不依赖简写 parser。
    let html = "<html><body>\
        <div id=\"def\"></div>\
        <div id=\"c\" style=\"background: rgb(255, 0, 0);\"></div>\
        <div id=\"fi\" style=\"background-color: rgb(0, 128, 0); background-attachment: fixed;\"></div>\
        <div id=\"oc\" style=\"background-origin: content-box; background-clip: padding-box;\"></div>\
        </body></html>";
    // background 简写恒完整规范形："<color> <image> <repeat> <attachment> <position> / <size> <origin> <clip>"。
    // 默认：transparent none repeat scroll 0% 0% / auto padding-box border-box。
    assert_eq!(
        computed_style_property(html, "#def", "background"),
        "rgba(0, 0, 0, 0) none repeat scroll 0% 0% / auto padding-box border-box"
    );
    // 纯色（简写声明）：color 改变，其余默认。
    assert_eq!(
        computed_style_property(html, "#c", "background"),
        "rgb(255, 0, 0) none repeat scroll 0% 0% / auto padding-box border-box"
    );
    // attachment=fixed（经 longhand 设置，测序列化）。
    assert_eq!(
        computed_style_property(html, "#fi", "background"),
        "rgb(0, 128, 0) none repeat fixed 0% 0% / auto padding-box border-box"
    );
    // origin/clip（origin 在前 clip 在后，即使相等也双显；经 longhand 设置，测序列化）。
    assert_eq!(
        computed_style_property(html, "#oc", "background"),
        "rgba(0, 0, 0, 0) none repeat scroll 0% 0% / auto content-box padding-box"
    );
}

#[test]
fn test_get_computed_style_place_shorthands_r2758() {
    // R2758：getComputedStyle place-content/items/self 简写（align+justify CSSOM 2 值最小化）。
    // 每项期望串经本地 Chromium 150 oracle 提取（--dump-dom 写 DOM 法），TDD red→green 对齐。
    // 注：place-content/items 默认值受 ZW layout-coupled 默认（justify-content FlexStart / align-items
    // Stretch vs Chromium normal）影响 diverge——测**显式设置**的值（含单值同值），序列化本身正确。
    let html = "<html><body>\
        <div id=\"pc1\" style=\"place-content: center;\"></div>\
        <div id=\"pc2\" style=\"place-content: center start;\"></div>\
        <div id=\"pc3\" style=\"place-content: space-between;\"></div>\
        <div id=\"pi1\" style=\"place-items: center;\"></div>\
        <div id=\"pi2\" style=\"place-items: center start;\"></div>\
        <div id=\"ps0\" style=\"color: red;\"></div>\
        <div id=\"ps1\" style=\"place-self: center;\"></div>\
        <div id=\"ps2\" style=\"place-self: start end;\"></div>\
        </body></html>";
    // place-content：align==justify→单值，否则 "align justify"。
    assert_eq!(computed_style_property(html, "#pc1", "place-content"), "center");
    assert_eq!(computed_style_property(html, "#pc2", "place-content"), "center start");
    assert_eq!(computed_style_property(html, "#pc3", "place-content"), "space-between");
    // place-items：同 2 值最小化。
    assert_eq!(computed_style_property(html, "#pi1", "place-items"), "center");
    assert_eq!(computed_style_property(html, "#pi2", "place-items"), "center start");
    // place-self：默认 align-self/justify-self 均 auto→"auto"（默认匹配 Chromium）。
    assert_eq!(computed_style_property(html, "#ps0", "place-self"), "auto");
    assert_eq!(computed_style_property(html, "#ps1", "place-self"), "center");
    assert_eq!(computed_style_property(html, "#ps2", "place-self"), "start end");
}

#[test]
fn test_get_computed_style_grid_lines_r2759() {
    // R2759：getComputedStyle grid 线定位 longhand（grid-column/row-start/end）+ 简写
    // （grid-column/row/area，CSSOM 最小化）。每项期望串经本地 Chromium 150 oracle 提取，TDD 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"cs\" style=\"grid-column-start: 2;\"></div>\
        <div id=\"cname\" style=\"grid-column-start: main;\"></div>\
        <div id=\"gc1\" style=\"grid-column: 2;\"></div>\
        <div id=\"gc2\" style=\"grid-column: 2 / 4;\"></div>\
        <div id=\"gc3\" style=\"grid-column: 1 / span 2;\"></div>\
        <div id=\"gc4\" style=\"grid-column: span 2;\"></div>\
        <div id=\"gr\" style=\"grid-row: span 3 / 5;\"></div>\
        <div id=\"ga1\" style=\"grid-area: 1 / 1 / 3 / 3;\"></div>\
        <div id=\"ga3\" style=\"grid-area: 2 / 3;\"></div>\
        </body></html>";
    // longhand：Auto→auto / Line(n)→n / Span(n)→span n / Name(s)→s。
    assert_eq!(computed_style_property(html, "#d", "grid-column-start"), "auto");
    assert_eq!(computed_style_property(html, "#cs", "grid-column-start"), "2");
    assert_eq!(computed_style_property(html, "#cname", "grid-column-start"), "main");
    assert_eq!(computed_style_property(html, "#gc4", "grid-column-start"), "span 2");
    // grid-column 简写：start==end→单值；end==auto 且 start 非 Name→单值；Name 保留 "name / auto"。
    assert_eq!(computed_style_property(html, "#d", "grid-column"), "auto");
    assert_eq!(computed_style_property(html, "#gc1", "grid-column"), "2");
    assert_eq!(computed_style_property(html, "#gc2", "grid-column"), "2 / 4");
    assert_eq!(computed_style_property(html, "#gc3", "grid-column"), "1 / span 2");
    assert_eq!(computed_style_property(html, "#gc4", "grid-column"), "span 2");
    assert_eq!(computed_style_property(html, "#cname", "grid-column"), "main / auto");
    // grid-row 简写：同 grid-column 规则。
    assert_eq!(computed_style_property(html, "#gr", "grid-row"), "span 3 / 5");
    // grid-area 简写：4 槽 trailing-drop 最小化。注：单值 `grid-area: header`（CSS 应四值同设）
    // 受 ZW expand_grid_area 仅设 row-start 的 pre-existing parser diverge 限——此处测 ZW 正确解析的
    // 4 值 / 2 值形式（序列化本身正确，单值 diverge 另记）。
    assert_eq!(computed_style_property(html, "#d", "grid-area"), "auto");
    assert_eq!(computed_style_property(html, "#ga1", "grid-area"), "1 / 1 / 3 / 3");
    assert_eq!(computed_style_property(html, "#ga3", "grid-area"), "2 / 3");
    // #gc1（cs=2，re/ce=auto，cs 非 Name）→grid-area drop ce/re→"auto / 2"。
    assert_eq!(computed_style_property(html, "#gc1", "grid-area"), "auto / 2");
    // #cname（cs=Name main，阻止 ce 省）→grid-area 全 4 槽 "auto / main / auto / auto"。
    assert_eq!(
        computed_style_property(html, "#cname", "grid-area"),
        "auto / main / auto / auto"
    );
    // #gr（rs=span3, re=5, ce=auto）→drop ce（cs=auto 非 Name），re=5 留→"span 3 / auto / 5"。
    assert_eq!(computed_style_property(html, "#gr", "grid-area"), "span 3 / auto / 5");
}

#[test]
fn test_get_computed_style_inset_shorthand_r2760() {
    // R2760：getComputedStyle inset 简写（top/right/bottom/left CSSOM 4 值最小化）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。ZW 解析 inset 简写
    // （parse_rect_values），序列化复用 box_4_to_css（同 margin/padding/border-radius）。
    let html = "<html><body>\
        <div id=\"i1\" style=\"inset: 10px;\"></div>\
        <div id=\"i2\" style=\"inset: 10px 20px;\"></div>\
        <div id=\"i3\" style=\"inset: 10px 20px 30px;\"></div>\
        <div id=\"i4\" style=\"inset: 10px 20px 30px 40px;\"></div>\
        <div id=\"mix\" style=\"inset: 5px 5px 5px 5px;\"></div>\
        </body></html>";
    // inset 简写 = top/right/bottom/left 的 CSSOM 4 值最小化。
    assert_eq!(computed_style_property(html, "#i1", "inset"), "10px");
    assert_eq!(computed_style_property(html, "#i2", "inset"), "10px 20px");
    assert_eq!(computed_style_property(html, "#i3", "inset"), "10px 20px 30px");
    assert_eq!(computed_style_property(html, "#i4", "inset"), "10px 20px 30px 40px");
    // 全等→单值。
    assert_eq!(computed_style_property(html, "#mix", "inset"), "5px");
    // 经 longhand 设置非等值（验证简写重组，非仅依赖 shorthand 声明）。
    let html2 = "<html><body>\
        <div id=\"lh\" style=\"top: 1px; right: 2px; bottom: 3px; left: 4px;\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html2, "#lh", "inset"), "1px 2px 3px 4px");
}

#[test]
fn test_get_computed_style_font_shorthand_r2761() {
    // R2761：getComputedStyle font 简写（CSSOM 重组省初值）+ line-height number→used px 修复。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    // 注：默认 font-family ZW 为空（vs Chromium "Times New Roman"）——pre-existing longhand diverge，
    // 故测显式 family 的 font 简写声明（序列化本身正确）。
    let html = "<html><body>\
        <div id=\"f1\" style=\"font: italic bold 14px/1.5 Arial;\"></div>\
        <div id=\"f3\" style=\"font: bold 12px sans-serif;\"></div>\
        <div id=\"f4\" style=\"font: bold 14px/2 Helvetica;\"></div>\
        <div id=\"f5\" style=\"font-family: Arial; font-size: 14px;\"></div>\
        </body></html>";
    // 经 longhand 设置（family Arial + size 14px，style/weight/line-height 全初值省）→"14px Arial"。
    assert_eq!(computed_style_property(html, "#f5", "font"), "14px Arial");
    // italic + 700(bold) + 14px + line-height 1.5→21px(14×1.5 used px) + Arial。
    assert_eq!(
        computed_style_property(html, "#f1", "font"),
        "italic 700 14px / 21px Arial"
    );
    // 700 + 12px + sans-serif（line-height normal 省）。
    assert_eq!(computed_style_property(html, "#f3", "font"), "700 12px sans-serif");
    // 700 + 14px + line-height 2→28px(14×2) + Helvetica。
    assert_eq!(
        computed_style_property(html, "#f4", "font"),
        "700 14px / 28px Helvetica"
    );
    // line-height longhand number→used px 修复（独立验证，1.5 × 默认 16px = 24px）。
    let html2 = "<html><body><div id=\"lh\" style=\"line-height: 1.5;\"></div></body></html>";
    assert_eq!(computed_style_property(html2, "#lh", "line-height"), "24px");
}

#[test]
fn test_get_computed_style_backdrop_filter_underline_offset_r2762() {
    // R2762：getComputedStyle backdrop-filter（复用 filter_to_css）+ text-underline-offset（Auto/Length）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"bf1\" style=\"backdrop-filter: blur(10px);\"></div>\
        <div id=\"bf2\" style=\"backdrop-filter: blur(5px) saturate(180%);\"></div>\
        <div id=\"tuo1\" style=\"text-underline-offset: 3px;\"></div>\
        <div id=\"tuo2\" style=\"text-underline-offset: auto;\"></div>\
        </body></html>";
    // backdrop-filter：复用 filter 序列化（空→none / 函数列表空格分隔 / saturate 百分比→数字）。
    assert_eq!(computed_style_property(html, "#d", "backdrop-filter"), "none");
    assert_eq!(computed_style_property(html, "#bf1", "backdrop-filter"), "blur(10px)");
    assert_eq!(
        computed_style_property(html, "#bf2", "backdrop-filter"),
        "blur(5px) saturate(1.8)"
    );
    // text-underline-offset：Auto→auto / Length→px。
    assert_eq!(computed_style_property(html, "#d", "text-underline-offset"), "auto");
    assert_eq!(computed_style_property(html, "#tuo1", "text-underline-offset"), "3px");
    assert_eq!(computed_style_property(html, "#tuo2", "text-underline-offset"), "auto");
}

#[test]
fn test_get_computed_style_text_emphasis_r2763() {
    // R2763：getComputedStyle text-emphasis 簇（style/color/position longhand + 简写）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"s1\" style=\"text-emphasis-style: dot;\"></div>\
        <div id=\"s3\" style=\"text-emphasis-style: open circle;\"></div>\
        <div id=\"s4\" style=\"text-emphasis-style: sesame;\"></div>\
        <div id=\"s5\" style='text-emphasis-style: \"*\";'></div>\
        <div id=\"c1\" style=\"text-emphasis-color: rgb(255, 0, 0);\"></div>\
        <div id=\"p1\" style=\"text-emphasis-position: under left;\"></div>\
        <div id=\"sh\" style=\"text-emphasis: filled circle rgb(0, 128, 0);\"></div>\
        </body></html>";
    // text-emphasis-style：char→keyword 逆映射（filled 省，open 显；string 引号化）。
    assert_eq!(computed_style_property(html, "#d", "text-emphasis-style"), "none");
    assert_eq!(computed_style_property(html, "#s1", "text-emphasis-style"), "dot");
    assert_eq!(
        computed_style_property(html, "#s3", "text-emphasis-style"),
        "open circle"
    );
    assert_eq!(computed_style_property(html, "#s4", "text-emphasis-style"), "sesame");
    assert_eq!(computed_style_property(html, "#s5", "text-emphasis-style"), "\"*\"");
    // text-emphasis-color：currentcolor→rgb（默认元素 black→rgb(0,0,0)）。
    assert_eq!(
        computed_style_property(html, "#d", "text-emphasis-color"),
        "rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#c1", "text-emphasis-color"),
        "rgb(255, 0, 0)"
    );
    // text-emphasis-position：over/under 恒显；left 显（right 初值省）。
    assert_eq!(computed_style_property(html, "#d", "text-emphasis-position"), "over");
    assert_eq!(
        computed_style_property(html, "#p1", "text-emphasis-position"),
        "under left"
    );
    // text-emphasis 简写：style + color（恒双段）。
    assert_eq!(
        computed_style_property(html, "#d", "text-emphasis"),
        "none rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#sh", "text-emphasis"),
        "circle rgb(0, 128, 0)"
    );
}

#[test]
fn test_get_computed_style_border_image_longhands_r2764() {
    // R2764：getComputedStyle border-image 切片族 longhand（slice/width/outset 4 值最小化 + repeat 2 值）。
    // 每项期望串经本地 Chromium 150 oracle 提取，TDD red→green 对齐。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"s1\" style=\"border-image-slice: 10 20 30 40;\"></div>\
        <div id=\"s2\" style=\"border-image-slice: 10% fill;\"></div>\
        <div id=\"w1\" style=\"border-image-width: 10px 20px;\"></div>\
        <div id=\"w2\" style=\"border-image-width: auto;\"></div>\
        <div id=\"r1\" style=\"border-image-repeat: round repeat;\"></div>\
        <div id=\"o1\" style=\"border-image-outset: 5px 10px;\"></div>\
        </body></html>";
    // border-image-slice：默认 100%（R2764 修 Percent）/ 4 值最小化 / fill 末尾。
    assert_eq!(computed_style_property(html, "#d", "border-image-slice"), "100%");
    assert_eq!(
        computed_style_property(html, "#s1", "border-image-slice"),
        "10 20 30 40"
    );
    assert_eq!(computed_style_property(html, "#s2", "border-image-slice"), "10% fill");
    // border-image-width：默认 1 / 4 值最小化 / auto。
    assert_eq!(computed_style_property(html, "#d", "border-image-width"), "1");
    assert_eq!(computed_style_property(html, "#w1", "border-image-width"), "10px 20px");
    assert_eq!(computed_style_property(html, "#w2", "border-image-width"), "auto");
    // border-image-outset：默认 0 / 4 值最小化。
    assert_eq!(computed_style_property(html, "#d", "border-image-outset"), "0");
    assert_eq!(computed_style_property(html, "#o1", "border-image-outset"), "5px 10px");
    // border-image-repeat：默认 stretch / 相等单值否则双值。
    assert_eq!(computed_style_property(html, "#d", "border-image-repeat"), "stretch");
    assert_eq!(
        computed_style_property(html, "#r1", "border-image-repeat"),
        "round repeat"
    );
}

#[test]
fn test_get_computed_style_border_image_shorthand_r2765() {
    // R2765：getComputedStyle border-image 简写（5 子分量 CSSOM 重组）。Chromium 150 oracle 锚定：
    // ① source==none → 整值 "none"（不论其余 slice/width/outset/repeat 是否非初值）；
    // ② source!=none → 恒全量 "<source> <slice> / <width> / <outset> <repeat>"（不省初值，width/outset
    //    各独占一个 `/` 分隔）。用 linear-gradient 源避免 url() 相对/绝对 longhand 既存 diverge。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"slc\" style=\"border-image-slice: 10;\"></div>\
        <div id=\"g\" style=\"border-image-source: linear-gradient(-45deg, red, blue);\"></div>\
        <div id=\"full\" style=\"border-image-source: linear-gradient(-45deg, red, blue);\
                               border-image-slice: 10 fill;\
                               border-image-width: 20px;\
                               border-image-outset: 5px;\
                               border-image-repeat: round;\"></div>\
        <div id=\"grep\" style=\"border-image-source: linear-gradient(-45deg, red, blue);\
                                border-image-repeat: round;\"></div>\
        </body></html>";
    let grad = "linear-gradient(-45deg, rgb(255, 0, 0), rgb(0, 0, 255))";
    // source==none（默认 / 或仅设 slice 等其余分量）→ 整值 "none"。
    assert_eq!(computed_style_property(html, "#d", "border-image"), "none");
    assert_eq!(computed_style_property(html, "#slc", "border-image"), "none");
    // source!=none：恒全量 "<source> <slice> / <width> / <outset> <repeat>"。
    assert_eq!(
        computed_style_property(html, "#g", "border-image"),
        format!("{grad} 100% / 1 / 0 stretch")
    );
    assert_eq!(
        computed_style_property(html, "#full", "border-image"),
        format!("{grad} 10 fill / 20px / 5px round")
    );
    assert_eq!(
        computed_style_property(html, "#grep", "border-image"),
        format!("{grad} 100% / 1 / 0 round")
    );
}

#[test]
fn test_get_computed_style_border_radius_shorthand() {
    // R2738：getComputedStyle border-radius 简写（CSSOM 4 值最小化）。4 角 longhand 早覆（R2707）。
    let html = "<html><body>\
        <div id=\"br1\" style=\"border-radius: 5px;\"></div>\
        <div id=\"br2\" style=\"border-radius: 5px 10px;\"></div>\
        <div id=\"br3\" style=\"border-radius: 5px 10px 15px;\"></div>\
        <div id=\"br4\" style=\"border-radius: 5px 10px 15px 20px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 全等→1 值；TL==BR&&TR==BL→2 值；TR==BL→3 值；否则 4 值（CSSOM 同 margin 语法）。
    assert_eq!(computed_style_property(html, "#br1", "border-radius"), "5px");
    assert_eq!(computed_style_property(html, "#br2", "border-radius"), "5px 10px");
    assert_eq!(computed_style_property(html, "#br3", "border-radius"), "5px 10px 15px");
    assert_eq!(
        computed_style_property(html, "#br4", "border-radius"),
        "5px 10px 15px 20px"
    );
    // 默认 4 角均 0px → 最小化 "0px"（对齐 Chromium）。
    assert_eq!(computed_style_property(html, "#def", "border-radius"), "0px");
}

#[test]
fn test_get_computed_style_box_text_shadow() {
    // R2739：getComputedStyle box-shadow + text-shadow 序列化。
    // Chromium/WPT 格式：color 在前（currentcolor 经元素 color 解析）+ 全长度（box 4 长+inset / text 3 长），
    // 多阴影逗号分隔，空→none。格式锚定 WPT box-shadow-interpolation/composition 的 expect 串。
    let html = "<html><body>\
        <div id=\"bs\" style=\"box-shadow: 5px 5px;\"></div>\
        <div id=\"bsi\" style=\"box-shadow: inset 0 0 10px red;\"></div>\
        <div id=\"bss\" style=\"box-shadow: 1px 2px 3px 4px blue;\"></div>\
        <div id=\"bsm\" style=\"box-shadow: 1px 1px red, 2px 2px blue;\"></div>\
        <div id=\"cc\" style=\"color: green; box-shadow: 5px 5px;\"></div>\
        <div id=\"ts\" style=\"text-shadow: 2px 4px;\"></div>\
        <div id=\"tsc\" style=\"text-shadow: 0 0 10px red;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // box-shadow：color 在前（无 color→currentcolor 默认元素 color=black）+ ox oy blur spread 全含；inset 在末。
    assert_eq!(
        computed_style_property(html, "#bs", "box-shadow"),
        "rgb(0, 0, 0) 5px 5px 0px 0px"
    );
    assert_eq!(
        computed_style_property(html, "#bsi", "box-shadow"),
        "rgb(255, 0, 0) 0px 0px 10px 0px inset"
    );
    assert_eq!(
        computed_style_property(html, "#bss", "box-shadow"),
        "rgb(0, 0, 255) 1px 2px 3px 4px"
    );
    // 多阴影逗号分隔。
    assert_eq!(
        computed_style_property(html, "#bsm", "box-shadow"),
        "rgb(255, 0, 0) 1px 1px 0px 0px, rgb(0, 0, 255) 2px 2px 0px 0px"
    );
    // currentcolor 解析为元素 color（green→rgb(0,128,0)）。
    assert_eq!(
        computed_style_property(html, "#cc", "box-shadow"),
        "rgb(0, 128, 0) 5px 5px 0px 0px"
    );
    // text-shadow：color 在前 + ox oy blur 3 长（无 spread/inset）。
    assert_eq!(
        computed_style_property(html, "#ts", "text-shadow"),
        "rgb(0, 0, 0) 2px 4px 0px"
    );
    assert_eq!(
        computed_style_property(html, "#tsc", "text-shadow"),
        "rgb(255, 0, 0) 0px 0px 10px"
    );
    // 默认空列表→none。
    assert_eq!(computed_style_property(html, "#def", "box-shadow"), "none");
    assert_eq!(computed_style_property(html, "#def", "text-shadow"), "none");
}

#[test]
fn test_get_computed_style_grid_tracks() {
    // R2740：getComputedStyle grid 轨道簇序列化（Option<String> 存原始 specified 值）。
    let html = "<html><body>\
        <div id=\"gtc\" style=\"grid-template-columns: 1fr 1fr 1fr;\"></div>\
        <div id=\"gtc2\" style=\"grid-template-columns: 100px minmax(200px, 1fr);\"></div>\
        <div id=\"gtr\" style=\"grid-template-rows: 50px 50px;\"></div>\
        <div id=\"gac\" style=\"grid-auto-columns: 200px;\"></div>\
        <div id=\"gar\" style=\"grid-auto-rows: 100px;\"></div>\
        <div id='gta' style='grid-template-areas: \"a b\" \"c d\";'></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // grid-template-columns/rows：Some→原样 specified 串；None→none（CSS 初值）。
    assert_eq!(
        computed_style_property(html, "#gtc", "grid-template-columns"),
        "1fr 1fr 1fr"
    );
    assert_eq!(
        computed_style_property(html, "#gtc2", "grid-template-columns"),
        "100px minmax(200px, 1fr)"
    );
    assert_eq!(computed_style_property(html, "#gtr", "grid-template-rows"), "50px 50px");
    // grid-auto-columns/rows：Some→原样；None→auto（CSS Grid §6.4 初值，非 none）。
    assert_eq!(computed_style_property(html, "#gac", "grid-auto-columns"), "200px");
    assert_eq!(computed_style_property(html, "#gar", "grid-auto-rows"), "100px");
    // grid-template-areas：Some→原样 specified 串（含引号）。
    assert_eq!(
        computed_style_property(html, "#gta", "grid-template-areas"),
        "\"a b\" \"c d\""
    );
    // 默认：grid-template-* → none；grid-auto-* → auto。
    assert_eq!(computed_style_property(html, "#def", "grid-template-columns"), "none");
    assert_eq!(computed_style_property(html, "#def", "grid-auto-columns"), "auto");
    assert_eq!(computed_style_property(html, "#def", "grid-auto-rows"), "auto");
    assert_eq!(computed_style_property(html, "#def", "grid-template-areas"), "none");
}

#[test]
fn test_get_computed_style_grid_template_shorthand_r2766() {
    // R2766：getComputedStyle grid-template 简写（rows/columns/areas 三 longhand 重组）。Chromium 150 oracle 锚定：
    // 全 none→"none"；areas==none→"<rows> / <cols>"（rows/cols 各自可 none）；areas!=none→引号区域与行尺寸逐行
    // 交错 + " / " + cols（area 数 != 行尺寸数→"" 空串，Chromium 同样不可序列化）。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"simple\" style=\"grid-template: 100px 200px / 1fr 1fr 1fr;\"></div>\
        <div id=\"cols\" style=\"grid-template-columns: 1fr 1fr;\"></div>\
        <div id=\"rows\" style=\"grid-template-rows: 100px 200px;\"></div>\
        <div id=\"areas\" style='grid-template: \"a a a\" 50px \"b b b\" 1fr \"c c c\" 2fr / 1fr 1fr 1fr;'></div>\
        </body></html>";
    // 全 none（默认）→ "none"。
    assert_eq!(computed_style_property(html, "#d", "grid-template"), "none");
    // areas==none：恒 "<rows> / <cols>"（cols 缺省→none / rows 缺省→none）。
    assert_eq!(
        computed_style_property(html, "#simple", "grid-template"),
        "100px 200px / 1fr 1fr 1fr"
    );
    assert_eq!(
        computed_style_property(html, "#cols", "grid-template"),
        "none / 1fr 1fr"
    );
    assert_eq!(
        computed_style_property(html, "#rows", "grid-template"),
        "100px 200px / none"
    );
    // areas!=none：引号区域与行尺寸逐行交错 + " / " + cols。
    assert_eq!(
        computed_style_property(html, "#areas", "grid-template"),
        "\"a a a\" 50px \"b b b\" 1fr \"c c c\" 2fr / 1fr 1fr 1fr"
    );
}

#[test]
fn test_get_computed_style_letter_spacing_normal_r2767() {
    // R2767：letter-spacing 0→normal diverge 修复。Chromium 150 oracle 把 0 值（默认 / normal /
    // 显式 0/0px）恒归一为 "normal"（normal 与 0 layout 等价）；非 0 长度才返 "Npx"。
    // ZW parse 把 normal→Px(0.0)，故 Px(0.0)→"normal" 精确对齐。word-spacing 不归一（恒 "0px"）。
    let html = "<html><body>\
        <div id=\"d\"></div>\
        <div id=\"norm\" style=\"letter-spacing: normal;\"></div>\
        <div id=\"zero\" style=\"letter-spacing: 0;\"></div>\
        <div id=\"val\" style=\"letter-spacing: 2px;\"></div>\
        <div id=\"ws\" style=\"word-spacing: normal;\"></div>\
        </body></html>";
    // letter-spacing：默认 / normal / 显式 0 → "normal"（Chromium 归一）。
    assert_eq!(computed_style_property(html, "#d", "letter-spacing"), "normal");
    assert_eq!(computed_style_property(html, "#norm", "letter-spacing"), "normal");
    assert_eq!(computed_style_property(html, "#zero", "letter-spacing"), "normal");
    // 非 0 长度 → "Npx"。
    assert_eq!(computed_style_property(html, "#val", "letter-spacing"), "2px");
    // word-spacing 不归一：normal → "0px"（与 letter-spacing 行为不同，对齐 Chromium）。
    assert_eq!(computed_style_property(html, "#ws", "word-spacing"), "0px");
    assert_eq!(computed_style_property(html, "#d", "word-spacing"), "0px");
}

#[test]
fn test_get_computed_style_containment() {
    // R2741：getComputedStyle containment 簇（content-visibility + contain-intrinsic-width/height）。
    let html = "<html><body>\
        <div id=\"cvh\" style=\"content-visibility: hidden;\"></div>\
        <div id=\"cva\" style=\"content-visibility: auto;\"></div>\
        <div id=\"ciw\" style=\"contain-intrinsic-width: 100px;\"></div>\
        <div id=\"cih\" style=\"contain-intrinsic-height: 50px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // content-visibility：visible/hidden/auto（CSS Containment 2，初值 visible）。
    assert_eq!(computed_style_property(html, "#cvh", "content-visibility"), "hidden");
    assert_eq!(computed_style_property(html, "#cva", "content-visibility"), "auto");
    assert_eq!(computed_style_property(html, "#def", "content-visibility"), "visible");
    // contain-intrinsic-width/height：None→none（初值）；Some→px。
    assert_eq!(
        computed_style_property(html, "#ciw", "contain-intrinsic-width"),
        "100px"
    );
    assert_eq!(
        computed_style_property(html, "#cih", "contain-intrinsic-height"),
        "50px"
    );
    assert_eq!(computed_style_property(html, "#def", "contain-intrinsic-width"), "none");
    assert_eq!(
        computed_style_property(html, "#def", "contain-intrinsic-height"),
        "none"
    );
}

#[test]
fn test_get_computed_style_counter_actions() {
    // R2742：getComputedStyle counter-increment / counter-reset 序列化。
    let html = "<html><body>\
        <div id=\"ci\" style=\"counter-increment: h1;\"></div>\
        <div id=\"ci2\" style=\"counter-increment: c 2;\"></div>\
        <div id=\"cr\" style=\"counter-reset: sec;\"></div>\
        <div id=\"crm\" style=\"counter-reset: a 5 b 3;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // counter-increment：空格分隔 name integer；value 省略→默认 1。
    assert_eq!(computed_style_property(html, "#ci", "counter-increment"), "h1 1");
    assert_eq!(computed_style_property(html, "#ci2", "counter-increment"), "c 2");
    // counter-reset：value 省略→默认 0；多计数器空格连接。
    assert_eq!(computed_style_property(html, "#cr", "counter-reset"), "sec 0");
    assert_eq!(computed_style_property(html, "#crm", "counter-reset"), "a 5 b 3");
    // 默认空→none。
    assert_eq!(computed_style_property(html, "#def", "counter-increment"), "none");
    assert_eq!(computed_style_property(html, "#def", "counter-reset"), "none");
}

#[test]
fn test_get_computed_style_transition_animation() {
    // R2743：getComputedStyle transition/animation 簇（10 属性，timing-function defer 到后续轮）。
    let html = "<html><body>\
        <div id=\"tp\" style=\"transition-property: margin, padding;\"></div>\
        <div id=\"tps\" style=\"transition-property: opacity;\"></div>\
        <div id=\"td\" style=\"transition-duration: 0.3s, 0.5s;\"></div>\
        <div id=\"tde\" style=\"transition-delay: 0.1s;\"></div>\
        <div id=\"an\" style=\"animation-name: fade, slide;\"></div>\
        <div id=\"ad\" style=\"animation-duration: 2s;\"></div>\
        <div id=\"adel\" style=\"animation-delay: 1s;\"></div>\
        <div id=\"aic\" style=\"animation-iteration-count: infinite;\"></div>\
        <div id=\"aicn\" style=\"animation-iteration-count: 2.5;\"></div>\
        <div id=\"adi\" style=\"animation-direction: alternate;\"></div>\
        <div id=\"afm\" style=\"animation-fill-mode: forwards;\"></div>\
        <div id=\"aps\" style=\"animation-play-state: paused;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // transition-property：逗号分隔；单值；默认 all。
    assert_eq!(
        computed_style_property(html, "#tp", "transition-property"),
        "margin, padding"
    );
    assert_eq!(computed_style_property(html, "#tps", "transition-property"), "opacity");
    assert_eq!(computed_style_property(html, "#def", "transition-property"), "all");
    // transition-duration/delay：Ns；默认 0s。
    assert_eq!(
        computed_style_property(html, "#td", "transition-duration"),
        "0.3s, 0.5s"
    );
    assert_eq!(computed_style_property(html, "#tde", "transition-delay"), "0.1s");
    assert_eq!(computed_style_property(html, "#def", "transition-duration"), "0s");
    // animation-name：逗号分隔；默认 none。
    assert_eq!(computed_style_property(html, "#an", "animation-name"), "fade, slide");
    assert_eq!(computed_style_property(html, "#def", "animation-name"), "none");
    // animation-duration/delay：Ns；默认 0s。
    assert_eq!(computed_style_property(html, "#ad", "animation-duration"), "2s");
    assert_eq!(computed_style_property(html, "#adel", "animation-delay"), "1s");
    // animation-iteration-count：infinite / 数值；默认 1。
    assert_eq!(
        computed_style_property(html, "#aic", "animation-iteration-count"),
        "infinite"
    );
    assert_eq!(
        computed_style_property(html, "#aicn", "animation-iteration-count"),
        "2.5"
    );
    assert_eq!(computed_style_property(html, "#def", "animation-iteration-count"), "1");
    // animation-direction/fill-mode/play-state：关键字；默认 normal/none/running。
    assert_eq!(
        computed_style_property(html, "#adi", "animation-direction"),
        "alternate"
    );
    assert_eq!(computed_style_property(html, "#afm", "animation-fill-mode"), "forwards");
    assert_eq!(computed_style_property(html, "#aps", "animation-play-state"), "paused");
    assert_eq!(computed_style_property(html, "#def", "animation-direction"), "normal");
    assert_eq!(computed_style_property(html, "#def", "animation-fill-mode"), "none");
    assert_eq!(computed_style_property(html, "#def", "animation-play-state"), "running");
}

#[test]
fn test_get_computed_style_timing_function() {
    // R2744：getComputedStyle transition/animation-timing-function。
    // 关键字保 keyword 不展开；cubic-bezier 4 数；steps(n) 默认 End 省略（spec-aligned，待 Chromium A/B 核实）。
    let html = "<html><body>\
        <div id=\"ease\" style=\"transition-timing-function: ease;\"></div>\
        <div id=\"lin\" style=\"transition-timing-function: linear;\"></div>\
        <div id=\"eio\" style=\"transition-timing-function: ease-in-out;\"></div>\
        <div id=\"cb\" style=\"transition-timing-function: cubic-bezier(0.25, 0.1, 0.25, 1);\"></div>\
        <div id=\"ss\" style=\"transition-timing-function: step-start;\"></div>\
        <div id=\"st\" style=\"transition-timing-function: steps(4);\"></div>\
        <div id=\"sts\" style=\"transition-timing-function: steps(4, start);\"></div>\
        <div id=\"multi\" style=\"transition-timing-function: ease-in, ease-out;\"></div>\
        <div id=\"atf\" style=\"animation-timing-function: linear;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 关键字（保原样）。
    assert_eq!(
        computed_style_property(html, "#ease", "transition-timing-function"),
        "ease"
    );
    assert_eq!(
        computed_style_property(html, "#lin", "transition-timing-function"),
        "linear"
    );
    assert_eq!(
        computed_style_property(html, "#eio", "transition-timing-function"),
        "ease-in-out"
    );
    // cubic-bezier：4 数逗号分隔（整数 1 无小数点）。
    assert_eq!(
        computed_style_property(html, "#cb", "transition-timing-function"),
        "cubic-bezier(0.25, 0.1, 0.25, 1)"
    );
    // step-start；steps(4) 默认 End 省略；steps(4, start) 含位置。
    assert_eq!(
        computed_style_property(html, "#ss", "transition-timing-function"),
        "step-start"
    );
    assert_eq!(
        computed_style_property(html, "#st", "transition-timing-function"),
        "steps(4)"
    );
    assert_eq!(
        computed_style_property(html, "#sts", "transition-timing-function"),
        "steps(4, start)"
    );
    // 多值逗号分隔。
    assert_eq!(
        computed_style_property(html, "#multi", "transition-timing-function"),
        "ease-in, ease-out"
    );
    // animation-timing-function 同结构。
    assert_eq!(
        computed_style_property(html, "#atf", "animation-timing-function"),
        "linear"
    );
    // 默认空→ease。
    assert_eq!(
        computed_style_property(html, "#def", "transition-timing-function"),
        "ease"
    );
    assert_eq!(
        computed_style_property(html, "#def", "animation-timing-function"),
        "ease"
    );
}

#[test]
fn test_get_computed_style_overflow_shorthand() {
    // R2745：getComputedStyle overflow 简写（overflow-x/y longhand 早覆）。
    let html = "<html><body>\
        <div id=\"eq\" style=\"overflow: hidden;\"></div>\
        <div id=\"ne\" style=\"overflow: hidden scroll;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // x==y→单值；x!=y→"x y"（CSS Overflow 3）；默认 visible。
    assert_eq!(computed_style_property(html, "#eq", "overflow"), "hidden");
    assert_eq!(computed_style_property(html, "#ne", "overflow"), "hidden scroll");
    assert_eq!(computed_style_property(html, "#def", "overflow"), "visible");
}

#[test]
fn test_get_computed_style_scroll_mask() {
    // R2746：getComputedStyle scroll-margin-*/scroll-padding-*（Scroll Snap 边距）+ mask-mode。
    let html = "<html><body>\
        <div id=\"sm\" style=\"scroll-margin-top: 10px; scroll-margin-right: 20px; scroll-margin-bottom: 30px; scroll-margin-left: 40px;\"></div>\
        <div id=\"sp\" style=\"scroll-padding-top: 5px; scroll-padding-left: 35px;\"></div>\
        <div id=\"mm\" style=\"mask-mode: alpha;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // scroll-margin：longhand 各 f32→px（CSS Scroll Snap 2，scroll-margin 简写未实现故逐 longhand 测）；默认 0px。
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-top"), "10px");
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-right"), "20px");
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-bottom"), "30px");
    assert_eq!(computed_style_property(html, "#sm", "scroll-margin-left"), "40px");
    assert_eq!(computed_style_property(html, "#def", "scroll-margin-top"), "0px");
    // scroll-padding：ScrollPadding Auto/Length；默认 auto。
    assert_eq!(computed_style_property(html, "#sp", "scroll-padding-top"), "5px");
    assert_eq!(computed_style_property(html, "#sp", "scroll-padding-left"), "35px");
    assert_eq!(computed_style_property(html, "#def", "scroll-padding-top"), "auto");
    // mask-mode：alpha/luminance/match-source（初值 match-source）。
    assert_eq!(computed_style_property(html, "#mm", "mask-mode"), "alpha");
    assert_eq!(computed_style_property(html, "#def", "mask-mode"), "match-source");
}

#[test]
fn test_get_computed_style_background_mask_image() {
    // R2747：getComputedStyle background-image + mask-image（None/Url 逐层；gradient defer→''）。
    let html = "<html><body>\
        <div id=\"url\" style=\"background-image: url(bg.png);\"></div>\
        <div id=\"none\" style=\"background-image: none;\"></div>\
        <div id=\"multi\" style=\"background-image: url(a.png), url(b.png);\"></div>\
        <div id=\"grad\" style=\"background-image: radial-gradient(circle, red, blue);\"></div>\
        <div id=\"mask\" style=\"mask-image: url(m.png);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // url → url("u")（同 list-style-image）；none；多层逗号分隔。
    assert_eq!(
        computed_style_property(html, "#url", "background-image"),
        "url(\"bg.png\")"
    );
    assert_eq!(computed_style_property(html, "#none", "background-image"), "none");
    assert_eq!(
        computed_style_property(html, "#multi", "background-image"),
        "url(\"a.png\"), url(\"b.png\")"
    );
    // radial-gradient(circle, ...) 层 → 序列化（R2750 radial 已实现；见 test_get_computed_style_radial_conic_gradient 全覆盖）。
    assert_eq!(
        computed_style_property(html, "#grad", "background-image"),
        "radial-gradient(circle, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // mask-image 同结构。
    assert_eq!(computed_style_property(html, "#mask", "mask-image"), "url(\"m.png\")");
    // 默认 → none。
    assert_eq!(computed_style_property(html, "#def", "background-image"), "none");
    assert_eq!(computed_style_property(html, "#def", "mask-image"), "none");
}

#[test]
fn test_get_computed_style_margin_padding_shorthand() {
    // R2748：getComputedStyle margin + padding 简写（CSSOM 4 值最小化，复用 box_4_to_css）。
    let html = "<html><body>\
        <div id=\"m1\" style=\"margin: 5px;\"></div>\
        <div id=\"m2\" style=\"margin: 5px 10px;\"></div>\
        <div id=\"m3\" style=\"margin: 5px 10px 15px;\"></div>\
        <div id=\"m4\" style=\"margin: 5px 10px 15px 20px;\"></div>\
        <div id=\"ma\" style=\"margin: auto;\"></div>\
        <div id=\"p1\" style=\"padding: 8px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // margin：全等→1 值；top==bottom&&right==left→2 值；right==left→3 值；否则 4 值。
    assert_eq!(computed_style_property(html, "#m1", "margin"), "5px");
    assert_eq!(computed_style_property(html, "#m2", "margin"), "5px 10px");
    assert_eq!(computed_style_property(html, "#m3", "margin"), "5px 10px 15px");
    assert_eq!(computed_style_property(html, "#m4", "margin"), "5px 10px 15px 20px");
    // margin: auto → auto（LengthValue::Auto 经 length_to_css）。
    assert_eq!(computed_style_property(html, "#ma", "margin"), "auto");
    // padding 同结构；默认 margin/padding 均 0px → "0px"。
    assert_eq!(computed_style_property(html, "#p1", "padding"), "8px");
    assert_eq!(computed_style_property(html, "#def", "margin"), "0px");
    assert_eq!(computed_style_property(html, "#def", "padding"), "0px");
}

#[test]
fn test_get_computed_style_linear_gradient() {
    // R2749：getComputedStyle background-image linear-gradient 层序列化（radial/conic 仍 defer）。
    let html = "<html><body>\
        <div id=\"d\" style=\"background-image: linear-gradient(to right, red, blue);\"></div>\
        <div id=\"defdir\" style=\"background-image: linear-gradient(red, blue);\"></div>\
        <div id=\"ang\" style=\"background-image: linear-gradient(45deg, red, blue);\"></div>\
        <div id=\"pos\" style=\"background-image: linear-gradient(to right, red 0%, blue 100%);\"></div>\
        <div id=\"rep\" style=\"background-image: repeating-linear-gradient(red, blue);\"></div>\
        </body></html>";
    // to right + 色标解析为 rgb。
    assert_eq!(
        computed_style_property(html, "#d", "background-image"),
        "linear-gradient(to right, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 默认方向 to bottom 省略。
    assert_eq!(
        computed_style_property(html, "#defdir", "background-image"),
        "linear-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 角度 → Xdeg。
    assert_eq!(
        computed_style_property(html, "#ang", "background-image"),
        "linear-gradient(45deg, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 色标位置。
    assert_eq!(
        computed_style_property(html, "#pos", "background-image"),
        "linear-gradient(to right, rgb(255, 0, 0) 0%, rgb(0, 0, 255) 100%)"
    );
    // repeating- 前缀。
    assert_eq!(
        computed_style_property(html, "#rep", "background-image"),
        "repeating-linear-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
}

#[test]
fn test_get_computed_style_radial_conic_gradient() {
    // R2750：getComputedStyle radial-gradient（WPT oracle 锚定）+ conic-gradient（spec-aligned）。
    let html = "<html><body>\
        <div id=\"def\" style=\"background-image: radial-gradient(red, blue);\"></div>\
        <div id=\"ctr\" style=\"background-image: radial-gradient(at center, red, blue);\"></div>\
        <div id=\"pos\" style=\"background-image: radial-gradient(at 10px 10px, red, blue);\"></div>\
        <div id=\"cir\" style=\"background-image: radial-gradient(circle, red, blue);\"></div>\
        <div id=\"fs\" style=\"background-image: radial-gradient(farthest-side, red, blue);\"></div>\
        <div id=\"cp\" style=\"background-image: radial-gradient(circle at 25% 40%, red, blue);\"></div>\
        <div id=\"cl\" style=\"background-image: radial-gradient(circle 50px, red, blue);\"></div>\
        <div id=\"cdef\" style=\"background-image: conic-gradient(red, blue);\"></div>\
        <div id=\"cfrom\" style=\"background-image: conic-gradient(from 90deg, red, blue);\"></div>\
        <div id=\"cf0\" style=\"background-image: conic-gradient(from 0deg, red, blue);\"></div>\
        <div id=\"cat\" style=\"background-image: conic-gradient(at 25% 75%, red, blue);\"></div>\
        </body></html>";
    // 默认 ellipse farthest-corner at center 全省略。
    assert_eq!(
        computed_style_property(html, "#def", "background-image"),
        "radial-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // position-首位 `at center`（默认 position）→ 省略（R2751 parser fix 支持 position-首位 config）。
    assert_eq!(
        computed_style_property(html, "#ctr", "background-image"),
        "radial-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // position-首位 非默认 → at X Y。
    assert_eq!(
        computed_style_property(html, "#pos", "background-image"),
        "radial-gradient(at 10px 10px, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // circle（默认 size）保留。
    assert_eq!(
        computed_style_property(html, "#cir", "background-image"),
        "radial-gradient(circle, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // 非默认 size 关键字 farthest-side 保留（ellipse 默认形状省略）。
    assert_eq!(
        computed_style_property(html, "#fs", "background-image"),
        "radial-gradient(farthest-side, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // circle + 非默认 position。
    assert_eq!(
        computed_style_property(html, "#cp", "background-image"),
        "radial-gradient(circle at 25% 40%, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // circle + 显式半径 → 半径（circle 省略）。
    assert_eq!(
        computed_style_property(html, "#cl", "background-image"),
        "radial-gradient(50px, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic 默认 from 0deg at center 全省略。
    assert_eq!(
        computed_style_property(html, "#cdef", "background-image"),
        "conic-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic from <angle>。
    assert_eq!(
        computed_style_property(html, "#cfrom", "background-image"),
        "conic-gradient(from 90deg, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic from 0deg（默认）→ 省略（WPT oracle 锚定）。
    assert_eq!(
        computed_style_property(html, "#cf0", "background-image"),
        "conic-gradient(rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // conic 非默认 position → at X Y（WPT oracle 锚定）。
    assert_eq!(
        computed_style_property(html, "#cat", "background-image"),
        "conic-gradient(at 25% 75%, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
}

#[test]
fn test_get_computed_style_border_image_source_gradient() {
    // R2753：border-image-source gradient 支持（旧仅 None/Url，gradient→none divergence；oracle 锚定）。
    // currentcolor 经元素 color 解析（#g 设 color:blue → rgb(0,0,255)，匹配 oracle）。
    let html = "<html><body>\
        <div id=\"g\" style=\"color: blue; border-image-source: linear-gradient(-45deg, red, currentcolor);\"></div>\
        <div id=\"r\" style=\"color: blue; border-image-source: radial-gradient(10px at 20px 30px, currentcolor, lime);\"></div>\
        <div id=\"u\" style=\"border-image-source: url(b.png);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // linear-gradient（含 -45deg 方向 + currentcolor→元素 color）。
    assert_eq!(
        computed_style_property(html, "#g", "border-image-source"),
        "linear-gradient(-45deg, rgb(255, 0, 0), rgb(0, 0, 255))"
    );
    // radial-gradient（10px 半径 + at 20px 30px + currentcolor/lime）。
    assert_eq!(
        computed_style_property(html, "#r", "border-image-source"),
        "radial-gradient(10px at 20px 30px, rgb(0, 0, 255), rgb(0, 255, 0))"
    );
    // url 仍正常；默认 none。
    assert_eq!(
        computed_style_property(html, "#u", "border-image-source"),
        "url(\"b.png\")"
    );
    assert_eq!(computed_style_property(html, "#def", "border-image-source"), "none");
}

#[test]
fn test_raf_frame_driven_on_path() {
    // R2713a：帧驱动 rAF（__ZW_RAF_FRAME_DRIVEN=true）。requestAnimationFrame 注册回调延后到
    // host render 后的 __zw_raf_tick；tick 前不 fire，tick 后按注册序 fire 并传 ts、清空队列。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // host 在 execute 前注入 env flag（模拟 worker init 读 ZW_RAF_FRAME_DRIVEN=1）。
    sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;").unwrap();
    sandbox
        .execute(
            "globalThis.__count = 0; globalThis.__ts = 'none';\
         requestAnimationFrame(function(t){ globalThis.__count++; globalThis.__ts = String(t); });\
         requestAnimationFrame(function(){ globalThis.__count++; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__count)").unwrap().value,
        "0",
        "帧驱动：tick 前回调不应 fire"
    );
    // host render 后调 __zw_raf_tick(16.7) → 按注册序 fire 两个、传 ts、清空队列。
    sandbox.execute("globalThis.__zw_raf_tick(16.7);").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__count)").unwrap().value,
        "2",
        "tick 后按注册序 fire 两个回调"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ts").unwrap().value,
        "16.7",
        "回调收到 ts 参数"
    );
}

#[test]
fn test_raf_frame_driven_cancel() {
    // R2713a：cancelAnimationFrame（ON 路径）移除待 fire 回调；tick 后不 fire。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;").unwrap();
    sandbox
        .execute(
            "globalThis.__fired = 'no';\
         var id = requestAnimationFrame(function(){ globalThis.__fired = 'yes'; });\
         cancelAnimationFrame(id);",
        )
        .unwrap();
    sandbox.execute("globalThis.__zw_raf_tick(0);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fired").unwrap().value,
        "no",
        "cancelAnimationFrame 后回调不 fire"
    );
}

#[test]
fn test_raf_sync_stub_off_path() {
    // R2713a：OFF 路径（env unset = 默认）保留同步 stub——rAF 立即同步 fire（reftest 兼容），
    // __zw_raf_tick 为 no-op。零默认行为变更的回归守护。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 不设 __ZW_RAF_FRAME_DRIVEN（默认 false）。
    sandbox
        .execute(
            "globalThis.__fired = 'no';\
         requestAnimationFrame(function(){ globalThis.__fired = 'yes'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fired").unwrap().value,
        "yes",
        "OFF 路径：rAF 立即同步 fire（reftest 兼容，零默认行为变更）"
    );
    // __zw_raf_tick OFF 路径 no-op（不应抛错、不重复 fire）。
    sandbox.execute("globalThis.__zw_raf_tick(0);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fired").unwrap().value, "yes");
}

#[test]
fn test_element_attributes_nodelist() {
    // R2699：el.attributes（NamedNodeMap 只读快照）——length/item/getNamedItem/数值索引/迭代。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\" class=\"c\" title=\"t\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // length + 数值索引 + item。
    sandbox
        .execute(
            "globalThis.__len = document.querySelector('#d').attributes.length;\n\
             globalThis.__i0 = document.querySelector('#d').attributes[0].name;\n\
             globalThis.__item1 = document.querySelector('#d').attributes.item(1).name;\n\
             globalThis.__item_oob = document.querySelector('#d').attributes.item(9);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__len").unwrap().value,
        "3",
        "attributes.length"
    );
    assert_eq!(
        sandbox.execute("globalThis.__i0").unwrap().value,
        "id",
        "attributes[0].name"
    );
    assert_eq!(
        sandbox.execute("globalThis.__item1").unwrap().value,
        "class",
        "attributes.item(1).name"
    );
    assert_eq!(
        sandbox.execute("globalThis.__item_oob === null").unwrap().value,
        "true",
        "out-of-range item → null"
    );

    // getNamedItem（命中 + value + 未命中 null）。
    sandbox
        .execute(
            "globalThis.__gn = document.querySelector('#d').attributes.getNamedItem('title').value;\n\
             globalThis.__gnn = document.querySelector('#d').attributes.getNamedItem('nope');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__gn").unwrap().value,
        "t",
        "getNamedItem('title').value"
    );
    assert_eq!(
        sandbox.execute("globalThis.__gnn === null").unwrap().value,
        "true",
        "getNamedItem 未命中 → null"
    );

    // 迭代（Symbol.iterator）→ 属性名顺序。
    sandbox
        .execute(
            "globalThis.__iter = Array.prototype.map.call(document.querySelector('#d').attributes, function(a){ return a.name; }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__iter").unwrap().value,
        "id,class,title",
        "attributes 迭代顺序"
    );
}

#[test]
fn test_set_remove_attr_syncs_cache() {
    // R2700：setAttribute/removeAttribute 须同步 class/value 客户端缓存，否则后续 classList/.value
    // 读 stale 缓存丢值（setAttribute('class','a');classList.add('b') 旧丢 'a'）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mk = |html: &str| -> (V8Sandbox, Arc<Mutex<Vec<DomMutation>>>, Arc<Mutex<String>>) {
        let mut sb = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sb.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
        let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
        register_dom_callbacks(&mut sb, &mutations, &dom_html, &page_url);
        (sb, mutations, dom_html)
    };

    // ① setAttribute('class','a') + classList.add('b') → 'a b'（旧 'base b' 丢 a）。
    let (mut sandbox, mutations, dom_html) = mk("<html><body><div id=\"d\" class=\"base\"></div></body></html>");
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.setAttribute('class', 'a');\n\
             d.classList.add('b');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        out.contains("class=\"a b\""),
        "setAttribute+classList 协作 → 'a b'\n{out}"
    );

    // ② setAttribute('value','x') + .value 读 → 'x'（旧 stale 读 'old'）。
    let (mut sandbox, _mutations, _dom_html) = mk("<html><body><input id=\"i\" value=\"old\"></body></html>");
    sandbox
        .execute(
            "document.querySelector('#i').setAttribute('value', 'x');\n\
             globalThis.__v = document.querySelector('#i').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v").unwrap().value,
        "x",
        "setAttribute('value','x') 后 .value 读见 'x'"
    );

    // ③ classList.add('a'); removeAttribute('class'); classList.add('b') → 'b'
    //    （removeAttribute 清缓存，否则 add('b') 读 stale 'base a' → 'base a b'）。
    let (mut sandbox, mutations, dom_html) = mk("<html><body><div id=\"d\" class=\"base\"></div></body></html>");
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.classList.add('a');\n\
             d.removeAttribute('class');\n\
             d.classList.add('b');",
        )
        .unwrap();
    let ms3 = mutations.lock().unwrap().clone();
    let out3 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms3).unwrap();
    assert!(
        out3.contains("class=\"b\""),
        "removeAttribute('class') 清缓存后 add('b') → 'b'\n{out3}"
    );
}

#[test]
fn test_get_computed_style_gap_shorthand() {
    // R2754：gap 简写 = row-gap/column-gap 双轴（CSS Box Alignment 3）。旧实现仅读 legacy
    // gap 字段（= row-gap），致 `gap: 5px 10px` 丢 column-gap 返 "5px"。改用 longhand 字段
    // 做 2 值最小化（row==col→单值，否则 "row col"）。Chromium oracle：单值→"5px"，双值→"5px 10px"。
    let html = "<html><body>\
        <div id=\"g1\" style=\"gap: 5px;\"></div>\
        <div id=\"g2\" style=\"gap: 5px 10px;\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html, "#g1", "gap"), "5px");
    assert_eq!(computed_style_property(html, "#g1", "row-gap"), "5px");
    assert_eq!(computed_style_property(html, "#g1", "column-gap"), "5px");
    assert_eq!(computed_style_property(html, "#g2", "gap"), "5px 10px");
    assert_eq!(computed_style_property(html, "#g2", "row-gap"), "5px");
    assert_eq!(computed_style_property(html, "#g2", "column-gap"), "10px");
}

#[test]
fn test_get_computed_style_text_decoration_longhands() {
    // R2754：text-decoration 4 longhand 早有 storage，补 getComputedStyle 序列化。
    // line（多 flag 规范序 underline overline line-through，空→none）/ style / color（currentcolor
    // 解析）/ thickness（auto/from-font/length）。Chromium oracle 锚定。
    let html = "<html><body>\
        <div id=\"td\" style=\"text-decoration: underline dotted red 2px;\"></div>\
        <div id=\"td2\" style=\"text-decoration: line-through overline;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    assert_eq!(
        computed_style_property(html, "#td", "text-decoration-line"),
        "underline"
    );
    assert_eq!(computed_style_property(html, "#td", "text-decoration-style"), "dotted");
    assert_eq!(
        computed_style_property(html, "#td", "text-decoration-color"),
        "rgb(255, 0, 0)"
    );
    assert_eq!(computed_style_property(html, "#td", "text-decoration-thickness"), "2px");
    // 多值组合按规范序重组（输入 line-through overline → overline line-through）。
    assert_eq!(
        computed_style_property(html, "#td2", "text-decoration-line"),
        "overline line-through"
    );
    // 默认值：line=none / style=solid / thickness=auto。
    assert_eq!(computed_style_property(html, "#plain", "text-decoration-line"), "none");
    assert_eq!(
        computed_style_property(html, "#plain", "text-decoration-style"),
        "solid"
    );
    assert_eq!(
        computed_style_property(html, "#plain", "text-decoration-thickness"),
        "auto"
    );
}

#[test]
fn test_get_computed_style_flex_shorthand() {
    // R2754：flex 简写 = "<grow> <shrink> <basis>"（恒 3 段）。关键：spec §7.1.1 省略 basis 时
    // flex-basis=0%（百分比），故 `flex: 1`→"1 1 0%"（Chromium oracle；旧 ZW basis="0"→"0px" diverge）。
    // none→"0 0 auto" / auto→"1 1 auto" / 显式 basis 原样。
    let html = "<html><body>\
        <div id=\"fl\" style=\"flex: 2 1 50px;\"></div>\
        <div id=\"flone\" style=\"flex: 1;\"></div>\
        <div id=\"fln\" style=\"flex: none;\"></div>\
        <div id=\"fla\" style=\"flex: auto;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html, "#fl", "flex"), "2 1 50px");
    assert_eq!(computed_style_property(html, "#flone", "flex"), "1 1 0%");
    assert_eq!(computed_style_property(html, "#flone", "flex-basis"), "0%");
    assert_eq!(computed_style_property(html, "#fln", "flex"), "0 0 auto");
    assert_eq!(computed_style_property(html, "#fla", "flex"), "1 1 auto");
    assert_eq!(computed_style_property(html, "#plain", "flex"), "0 1 auto");
}

#[test]
fn test_get_computed_style_flex_flow_shorthand() {
    // R2754：flex-flow = "<direction> <wrap>"（恒 2 段）。Chromium oracle：column wrap→"column wrap"，
    // 单值 wrap→"row wrap"（direction 缺省 row），default→"row nowrap"。
    let html = "<html><body>\
        <div id=\"ff\" style=\"flex-flow: column wrap;\"></div>\
        <div id=\"ffw\" style=\"flex-flow: wrap;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    assert_eq!(computed_style_property(html, "#ff", "flex-flow"), "column wrap");
    assert_eq!(computed_style_property(html, "#ffw", "flex-flow"), "row wrap");
    assert_eq!(computed_style_property(html, "#plain", "flex-flow"), "row nowrap");
}

#[test]
fn test_get_computed_style_outline_and_border_shorthands() {
    // R2754：outline = "<color> <style> <width>"（注意与 border 的 width-style-color 顺序相反！），
    // 恒 3 段含 none。border/per-side = "<width> <style> <color>"，全边 border 仅 4 边全等时返单边值
    // 否则 ''。outline-width 不套 border 的 none→0 规则（保留 computed medium→3px）。
    // Chromium oracle 锚定全部断言。
    let html = "<html><body>\
        <div id=\"o\" style=\"outline: 2px solid red;\"></div>\
        <div id=\"olt\" style=\"outline: thick solid #0f0;\"></div>\
        <div id=\"b\" style=\"border: 3px dashed blue;\"></div>\
        <div id=\"bt\" style=\"border-top: 3px dashed blue;\"></div>\
        <div id=\"bdiff\" style=\"border-top: 1px solid; border-bottom: 2px solid;\"></div>\
        <div id=\"plain\"></div>\
        </body></html>";
    // outline 简写（color style width 顺序）。
    assert_eq!(
        computed_style_property(html, "#o", "outline"),
        "rgb(255, 0, 0) solid 2px"
    );
    assert_eq!(
        computed_style_property(html, "#olt", "outline"),
        "rgb(0, 255, 0) solid 5px"
    );
    // outline 默认：style=none 仍保留 width medium→3px（与 border-width 不同）。
    assert_eq!(
        computed_style_property(html, "#plain", "outline"),
        "rgb(0, 0, 0) none 3px"
    );
    assert_eq!(computed_style_property(html, "#plain", "outline-width"), "3px");
    // border 简写：4 边全等 → "width style color"；不一致 → ''。
    assert_eq!(
        computed_style_property(html, "#b", "border"),
        "3px dashed rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#b", "border-top"),
        "3px dashed rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#bt", "border-top"),
        "3px dashed rgb(0, 0, 255)"
    );
    assert_eq!(
        computed_style_property(html, "#bdiff", "border-top"),
        "1px solid rgb(0, 0, 0)"
    );
    assert_eq!(
        computed_style_property(html, "#bdiff", "border-bottom"),
        "2px solid rgb(0, 0, 0)"
    );
    assert_eq!(computed_style_property(html, "#bdiff", "border"), "");
    assert_eq!(
        computed_style_property(html, "#plain", "border"),
        "0px none rgb(0, 0, 0)"
    );
}

#[test]
fn test_promise_any_all_settled_native_r2787() {
    // R2787：Promise.any / Promise.allSettled 复核（CONTINUE 指定）。ES2021 语言内置（非 Web API），
    // V8 原生提供——probe 确认 `typeof === 'function'`，无需 polyfill。本测试**锁住能力**
    //（防 V8 embed 配置 / 版本变化移除）+ 文档化语义：execute 末 `perform_microtask_checkpoint`
    // drain Promise 链 → 下 execute 可读结果。
    //   - allSettled：永不 reject，按序返 status 描述符（fulfilled→value / rejected→reason）。
    //   - any：返首个 fulfilled 值（跳过先到的 reject）；全 reject 抛 AggregateError（errors=原因数组）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 两者均为原生 function（V8 内置，非 shim 定义）。
    assert_eq!(sandbox.execute("typeof Promise.any").unwrap().value, "function");
    assert_eq!(sandbox.execute("typeof Promise.allSettled").unwrap().value, "function");

    // allSettled：混合 fulfilled/rejected → 永不 reject，按序返 status 描述符。
    sandbox
        .execute(
            "globalThis.__settled = '(pending)';\
             Promise.allSettled([Promise.resolve(1), Promise.reject('boom'), Promise.resolve(3)])\
               .then(function(r){\
                 globalThis.__settled = r.map(function(e){\
                   return e.status + ':' + (e.value !== undefined ? e.value : e.reason);\
                 }).join(',');\
               });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__settled)").unwrap().value,
        "fulfilled:1,rejected:boom,fulfilled:3"
    );

    // any：返首个 fulfilled（跳过先到的 reject）。
    sandbox
        .execute(
            "globalThis.__any = '(pending)';\
             Promise.any([Promise.reject('x'), Promise.resolve('win'), Promise.resolve('late')])\
               .then(function(v){ globalThis.__any = v; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__any)").unwrap().value, "win");

    // any：全 reject → reject AggregateError（errors=原因数组）；.catch 验证实例 + errors。
    sandbox
        .execute(
            "globalThis.__agg = '(pending)';\
             Promise.any([Promise.reject('a'), Promise.reject('b')])\
               .catch(function(e){\
                 globalThis.__agg = (e instanceof AggregateError) + ':' + e.errors.join(',');\
               });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__agg)").unwrap().value, "true:a,b");
}

#[test]
fn test_form_data_r2788() {
    // R2788：FormData（表单字段集合，表单序列化 / fetch multipart body 高频）。纯 JS，镜像
    // URLSearchParams pair-store 模式。**已知限制**：constructor `form` 参数 best-effort（renderer
    // 路径真实字段枚举 follow-up），多数库空构造再 append——本测试覆盖 manual API 全路径。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof function（全局已定义）；无 new 调用亦可构造（spec 允许）。
    assert_eq!(sandbox.execute("typeof FormData").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute("String(new FormData() instanceof FormData)")
            .unwrap()
            .value,
        "true"
    );
    // append + get/getAll/has：允许多个同名值，保插入序，get 返首个。
    sandbox
        .execute(
            "var fd = new FormData();\
             fd.append('a','1'); fd.append('b','2'); fd.append('a','3');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("fd.get('a')").unwrap().value, "1");
    assert_eq!(sandbox.execute("fd.get('z')").unwrap().value, "null");
    assert_eq!(sandbox.execute("fd.getAll('a').join(',')").unwrap().value, "1,3");
    assert_eq!(sandbox.execute("String(fd.has('b'))").unwrap().value, "true");
    assert_eq!(sandbox.execute("String(fd.has('z'))").unwrap().value, "false");
    // set：替换所有同名（保留原首次位置），无则追加。
    sandbox.execute("fd.set('a','X')").unwrap();
    assert_eq!(sandbox.execute("fd.getAll('a').join(',')").unwrap().value, "X");
    sandbox.execute("fd.set('c','new')").unwrap();
    assert_eq!(sandbox.execute("fd.get('c')").unwrap().value, "new");
    // delete：移除所有同名。
    sandbox.execute("fd.delete('b')").unwrap();
    assert_eq!(sandbox.execute("String(fd.has('b'))").unwrap().value, "false");
    // value 经 String() 归一（数字 → 字符串，spec 非 Blob 转 USVString）。
    sandbox.execute("fd.append('n', 42)").unwrap();
    assert_eq!(sandbox.execute("fd.get('n')").unwrap().value, "42");
    // 迭代协议：[Symbol.iterator]=entries → for...of / spread 取 [k,v] 对；forEach 回调序。
    assert_eq!(
        sandbox
            .execute("[...fd].map(function(p){return p[0]+'='+p[1];}).join('|')")
            .unwrap()
            .value,
        "a=X|c=new|n=42"
    );
    assert_eq!(
        sandbox
            .execute("(function(){var o=[];fd.forEach(function(v,k){o.push(k+':'+v);});return o.join(',');})()")
            .unwrap()
            .value,
        "a:X,c:new,n:42"
    );
    // keys / values 迭代器。
    assert_eq!(sandbox.execute("[...fd.keys()].join(',')").unwrap().value, "a,c,n");
    assert_eq!(sandbox.execute("[...fd.values()].join(',')").unwrap().value, "X,new,42");
}

#[test]
fn test_blob_and_object_url_r2789() {
    // R2789：Blob（不可变二进制容器）+ URL.createObjectURL/revokeObjectURL（blob: URL 注册表）。
    // 纯 JS，零 host 回调。size 按 UTF-8 字节；type 小写；text()/arrayBuffer() 返 Promise（execute 末
    // microtask checkpoint drain → 下 execute 可读）。createObjectURL 返 blob: 串并注册。
    // **已知限制**：slice 不真切字节（best-effort size clamp）；blob: URL 不被 net 解析为内容。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + instanceof；无 new 亦可构造。
    assert_eq!(sandbox.execute("typeof Blob").unwrap().value, "function");
    assert_eq!(
        sandbox.execute("String(new Blob() instanceof Blob)").unwrap().value,
        "true"
    );
    // size：空 Blob=0；string part 按 UTF-8 字节（'ZeroWeb'=7，中文 '中'=3）。
    assert_eq!(sandbox.execute("new Blob().size").unwrap().value, "0");
    assert_eq!(sandbox.execute("new Blob(['ZeroWeb']).size").unwrap().value, "7");
    assert_eq!(sandbox.execute("new Blob(['中']).size").unwrap().value, "3");
    // 多 part 求和 + ArrayBuffer part（4 字节）。
    assert_eq!(
        sandbox
            .execute("new Blob(['ab', new Uint8Array([0,0,0,0])]).size")
            .unwrap()
            .value,
        "6"
    );
    // type：小写归一；无 options → ''。
    assert_eq!(
        sandbox
            .execute("new Blob(['x'], {type:'APPLICATION/JSON'}).type")
            .unwrap()
            .value,
        "application/json"
    );
    assert_eq!(sandbox.execute("new Blob(['x']).type").unwrap().value, "");
    // text()：Promise<string>——string part 原样；多 part 拼接；execute 末 drain → 下 execute 读。
    sandbox
        .execute(
            "globalThis.__t = '(pending)';\
             new Blob(['hello',' ','world']).text().then(function(s){ globalThis.__t = s; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__t)").unwrap().value, "hello world");
    // text() 解码字节 part（TypedArray 经 TextDecoder）。
    sandbox
        .execute(
            "globalThis.__b = '(pending)';\
             new Blob([new Uint8Array([0x68,0x69])]).text().then(function(s){ globalThis.__b = s; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__b)").unwrap().value, "hi");
    // arrayBuffer()：Promise<Uint8Array>——'AB' UTF-8 = [65,66]。
    sandbox
        .execute(
            "globalThis.__ab = '(pending)';\
             new Blob(['AB']).arrayBuffer().then(function(a){ globalThis.__ab = a.length + ':' + a[0] + ',' + a[1]; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab)").unwrap().value, "2:65,66");
    // slice：best-effort size clamp（start/end 范围）+ type 重设。
    assert_eq!(
        sandbox.execute("new Blob(['ZeroWeb']).slice(1,4).size").unwrap().value,
        "3"
    );
    assert_eq!(
        sandbox
            .execute("new Blob(['ZeroWeb']).slice(0,4,'text/plain').type")
            .unwrap()
            .value,
        "text/plain"
    );
    // URL.createObjectURL：返 blob: 串 + 唯一性（两次不同）+ typeof function。
    assert_eq!(sandbox.execute("typeof URL.createObjectURL").unwrap().value, "function");
    assert_eq!(
        sandbox
            .execute("URL.createObjectURL(new Blob(['x'])).split(':')[0]")
            .unwrap()
            .value,
        "blob"
    );
    assert_eq!(
        sandbox
            .execute("URL.createObjectURL(new Blob(['a'])) !== URL.createObjectURL(new Blob(['b']))")
            .unwrap()
            .value,
        "true"
    );
    // revokeObjectURL：no-throw（不抛即视为清理成功）。
    sandbox.execute("URL.revokeObjectURL('blob:null/1-abc')").unwrap();
    assert_eq!(sandbox.execute("typeof URL.revokeObjectURL").unwrap().value, "function");
}

#[test]
fn test_anchor_javascript_target_r3057() {
    // R3057：anchor_javascript_target 解析 <a href="javascript:..."> click 的 JS 体（闭合 R3052 限制②）。
    // scheme 大小写不敏感；体 = scheme 后原始字符串（前导空白 trim），执行需原样 JS 源。

    // ① 常见 javascript: 体。
    assert_eq!(
        anchor_javascript_target("<html><body><a id='a' href='javascript:void(0)'>l</a></body></html>", "#a"),
        Some("void(0)".to_string()),
        "javascript:void(0) → Some(\"void(0)\")"
    );
    assert_eq!(
        anchor_javascript_target("<html><body><a id='f' href='javascript:doSomething()'>l</a></body></html>", "#f"),
        Some("doSomething()".to_string()),
        "javascript:doSomething() → Some(\"doSomething()\")"
    );
    assert_eq!(
        anchor_javascript_target("<html><body><a id='al' href=\"javascript:alert('hi')\">l</a></body></html>", "#al"),
        Some("alert('hi')".to_string()),
        "javascript:alert('hi') → Some(\"alert('hi')\")"
    );

    // ② scheme 大小写不敏感 + 前导空白。
    assert_eq!(
        anchor_javascript_target("<html><body><a id='u' href='JaVaScRiPt:  x()'>l</a></body></html>", "#u"),
        Some("x()".to_string()),
        "scheme 大小写不敏感 + 体前导空白 trim → Some(\"x()\")"
    );

    // ③ 空 javascript:（无体）→ Some(\"\")（执行空脚本 no-op）。
    assert_eq!(
        anchor_javascript_target("<html><body><a id='e' href='javascript:'>l</a></body></html>", "#e"),
        Some("".to_string()),
        "javascript:（空体）→ Some(\"\")"
    );

    // ④ 非 javascript: href（绝对 / 相对 / #hash / mailto:）→ None。
    assert_eq!(
        anchor_javascript_target("<html><body><a id='u' href='https://x.com/'>l</a></body></html>", "#u"),
        None,
        "绝对 URL href → None"
    );
    assert_eq!(
        anchor_javascript_target("<html><body><a id='h' href='#sec'>l</a></body></html>", "#h"),
        None,
        "#hash href → None"
    );
    assert_eq!(
        anchor_javascript_target("<html><body><a id='m' href='mailto:a@b.com'>l</a></body></html>", "#m"),
        None,
        "mailto: href → None"
    );

    // ⑤ 非 <a> / 无 href → None。
    assert_eq!(
        anchor_javascript_target("<html><body><div id='d' href='javascript:x()'>l</div></body></html>", "#d"),
        None,
        "非 <a> 元素 → None（即使 href=javascript:）"
    );
    assert_eq!(
        anchor_javascript_target("<html><body><a id='n'>l</a></body></html>", "#n"),
        None,
        "<a> 无 href → None"
    );
}

#[test]
fn test_js_cross_document_navigation_r3058() {
    // R3058：JS 跨文档导航（location.href=/assign/replace）经 NavigationBridge __zw_request_navigate 投递；
    // 同文档（hash-only）/ SPA pushState 不投递。drain 队列断言。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    let nav_bridge = crate::NavigationBridge::new();
    let nav_queue = nav_bridge.queue();
    nav_bridge.register(&mut sandbox);
    let drain = || nav_queue.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();

    // ① location.href = 跨文档 URL → 投递导航。
    sandbox.execute("location.href = 'https://other.com/x';").unwrap();
    assert_eq!(drain(), vec!["https://other.com/x".to_string()], "location.href= 跨文档 → 投递");

    // ② location.assign(跨文档) → 投递。
    sandbox.execute("location.assign('https://other.com/y');").unwrap();
    assert_eq!(drain(), vec!["https://other.com/y".to_string()], "location.assign 跨文档 → 投递");

    // ③ location.replace(跨文档) → 投递。
    sandbox.execute("location.replace('https://other.com/z');").unwrap();
    assert_eq!(drain(), vec!["https://other.com/z".to_string()], "location.replace 跨文档 → 投递");

    // ④ location.hash = '#foo'（同文档片段）→ 不投递。
    sandbox.execute("location.hash = '#foo';").unwrap();
    assert!(drain().is_empty(), "location.hash= 同文档 → 不投递导航");

    // ⑤ history.pushState（SPA 路由）→ 不投递（pushState 不经 location 导航函数）。
    sandbox.execute("history.pushState(null, '', '/spa-route');").unwrap();
    assert!(drain().is_empty(), "history.pushState SPA → 不投递导航");

    // ⑥ location.assign('#hash-only')（同文档，仅 hash 变）→ 不投递。
    sandbox.execute("location.assign('#bar');").unwrap();
    assert!(drain().is_empty(), "location.assign hash-only 同文档 → 不投递");
}
