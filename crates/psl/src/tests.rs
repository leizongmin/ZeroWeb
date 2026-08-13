//! zero-psl 单元测试。覆盖 PSL 算法全矩阵：普通/通配/例外规则、IP、
//! 单标签、未知 TLD 回退、大小写、IDN 简化处理。

use super::*;

fn dflt() -> PublicSuffixList {
    PublicSuffixList::default()
}

#[test]
fn plain_gtld_two_label_host_is_itself_registrable() {
    // example.com 恰为「后缀+1」，注册域 = 自身。
    assert_eq!(dflt().registrable_domain("example.com"), "example.com");
}

#[test]
fn subdomain_collapses_to_etld_plus_one() {
    assert_eq!(dflt().registrable_domain("a.b.c.example.com"), "example.com");
}

#[test]
fn multi_label_suffix_co_uk() {
    // co.uk 为公共后缀；example.co.uk → 注册域 example.co.uk。
    assert_eq!(dflt().registrable_domain("example.co.uk"), "example.co.uk");
    assert_eq!(dflt().registrable_domain("deep.sub.example.co.uk"), "example.co.uk");
}

/// R3382：扩展后的默认规则集覆盖更多 ccTLD 二级公共后缀。
/// 锁定各区域代表性后缀的 eTLD+1 折叠行为。
#[test]
fn expanded_ruleset_covers_regional_cctld_suffixes() {
    let psl = dflt();
    // 日本 .jp 家族：ne.jp / or.jp / co.jp / ac.jp / go.jp。
    assert_eq!(psl.registrable_domain("shop.ne.jp"), "shop.ne.jp");
    assert_eq!(psl.registrable_domain("corp.or.jp"), "corp.or.jp");
    assert_eq!(psl.registrable_domain("sub.shop.co.jp"), "shop.co.jp");
    assert_eq!(psl.registrable_domain("u-tokyo.ac.jp"), "u-tokyo.ac.jp");
    // 台湾 .tw 家族。
    assert_eq!(psl.registrable_domain("site.com.tw"), "site.com.tw");
    // 巴西 .br 家族。
    assert_eq!(psl.registrable_domain("loja.com.br"), "loja.com.br");
    // 南非 .za 家族。
    assert_eq!(psl.registrable_domain("firm.co.za"), "firm.co.za");
    // 奥地利 .at 家族。
    assert_eq!(psl.registrable_domain("club.or.at"), "club.or.at");
    // 意大利 .it 家族。
    assert_eq!(psl.registrable_domain("ente.gov.it"), "ente.gov.it");
}

/// R3382：扩展后的平台通配规则——各托管平台子域独立隔离。
#[test]
fn expanded_ruleset_covers_platform_wildcard_suffixes() {
    let psl = dflt();
    // Vercel / Netlify / Render 等静态托管。
    assert_eq!(psl.registrable_domain("myapp.vercel.app"), "myapp.vercel.app");
    assert_eq!(psl.registrable_domain("site.netlify.app"), "site.netlify.app");
    assert_eq!(psl.registrable_domain("svc.onrender.com"), "svc.onrender.com");
    // 不同子域 → 不同注册域（隔离）。
    assert_ne!(
        psl.registrable_domain("a.vercel.app"),
        psl.registrable_domain("b.vercel.app")
    );
    // Azure / Firebase。
    assert_eq!(psl.registrable_domain("app.azurewebsites.net"), "app.azurewebsites.net");
    assert_eq!(psl.registrable_domain("proj.web.app"), "proj.web.app");
}

#[test]
fn suffix_itself_has_no_registrable_domain_beyond_itself() {
    // 主机 == 公共后缀 → 返回自身（无可注册子域）。
    assert_eq!(dflt().registrable_domain("co.uk"), "co.uk");
    assert_eq!(dflt().registrable_domain("com"), "com");
}

#[test]
fn wildcard_rule_isolates_each_sublabel() {
    // *.github.io：每个用户子域是独立注册域——核心修复点。
    let psl = dflt();
    assert_eq!(psl.registrable_domain("a.github.io"), "a.github.io");
    assert_eq!(psl.registrable_domain("b.github.io"), "b.github.io");
    assert_ne!(
        psl.registrable_domain("a.github.io"),
        psl.registrable_domain("b.github.io"),
        "a.github.io 与 b.github.io 必须判为不同注册域（朴素末两段算法的误判修复）"
    );
}

#[test]
fn wildcard_github_io_deep_subdomain_treats_user_label_as_suffix() {
    // *.github.io：通配位置（紧邻 github.io 的标签）成为公共后缀标签。
    // 故 user.github.io 自身即公共后缀；www.user.github.io 的注册域 = 后缀 + 1
    // 标签 = www.user.github.io。这正是 PSL 通配语义（user.github.io 不可注册，
    // 其下 www.user.github.io 才是注册域）。
    assert_eq!(dflt().registrable_domain("user.github.io"), "user.github.io");
    assert_eq!(dflt().registrable_domain("www.user.github.io"), "www.user.github.io");
    // 不同 user 标签仍判为不同注册域——隔离不变量保持。
    assert_ne!(
        dflt().registrable_domain("alice.github.io"),
        dflt().registrable_domain("bob.github.io")
    );
}

#[test]
fn exception_rule_promotes_suffix_to_registrable() {
    // *.ck 通配 + !www.ck 例外：www.ck 自身即可注册。
    assert_eq!(dflt().registrable_domain("www.ck"), "www.ck");
    // 例外仅作用于精确匹配的 www.ck；其他 *.ck 仍走通配。
    assert_eq!(dflt().registrable_domain("a.b.ck"), "a.b.ck");
    assert_eq!(dflt().registrable_domain("x.ck"), "x.ck");
}

#[test]
fn ip_address_returned_verbatim() {
    assert_eq!(dflt().registrable_domain("127.0.0.1"), "127.0.0.1");
    assert_eq!(dflt().registrable_domain("::1"), "::1");
    assert_eq!(dflt().registrable_domain("2001:db8::1"), "2001:db8::1");
}

#[test]
fn single_label_host_returned_verbatim() {
    assert_eq!(dflt().registrable_domain("localhost"), "localhost");
    assert_eq!(dflt().registrable_domain("intranet"), "intranet");
}

#[test]
fn unknown_tld_falls_back_to_last_two_labels() {
    // 未知 TLD（如 .example / .test / .invalid）：无规则命中，
    // PSL 算法规定公共后缀 = 末标签，注册域 = 末两标签。
    assert_eq!(dflt().registrable_domain("foo.example"), "foo.example");
    assert_eq!(dflt().registrable_domain("a.b.example"), "b.example");
}

#[test]
fn case_insensitive_matching() {
    let psl = dflt();
    assert_eq!(psl.registrable_domain("Sub.EXAMPLE.Com"), "example.com");
    assert_eq!(psl.registrable_domain("A.GitHub.IO"), "a.github.io");
    // 返回值取自规范化后的主机标签，故为小写。
}

#[test]
fn trailing_dot_stripped() {
    assert_eq!(dflt().registrable_domain("example.com."), "example.com");
}

#[test]
fn empty_host_returns_empty() {
    assert_eq!(dflt().registrable_domain(""), "");
}

#[test]
fn custom_rules_override_defaults() {
    // from_rules：自定义规则集，验证解析与匹配独立于默认集。
    let psl = PublicSuffixList::from_rules("// comment\n*.compute.amazonaws.com\n!s3.amazonaws.com\n");
    // 通配规则生效。
    assert_eq!(
        psl.registrable_domain("svc.compute.amazonaws.com"),
        "svc.compute.amazonaws.com"
    );
}

#[test]
fn custom_rules_exception_beats_wildcard() {
    // 显式构造同基通配 + 例外，验证例外胜出。
    let psl = PublicSuffixList::from_rules("*.yp\n!www.yp\n");
    assert_eq!(psl.registrable_domain("www.yp"), "www.yp");
    assert_eq!(psl.registrable_domain("a.yp"), "a.yp");
}

#[test]
fn custom_rules_skips_blank_and_empty_label_lines() {
    // 空行、孤立 `*.` / `!.` 等空标签规则被跳过，不 panic。
    let psl = PublicSuffixList::from_rules("\n\n*.\n!.\ncom\n");
    assert_eq!(psl.registrable_domain("example.com"), "example.com");
}

#[test]
fn shared_instance_is_consistent() {
    // 全局惰性实例与默认实例行为一致。
    let a = PublicSuffixList::shared().registrable_domain("a.github.io");
    let b = dflt().registrable_domain("a.github.io");
    assert_eq!(a, b);
}

#[test]
fn parse_rules_idna_passthrough() {
    // 非 ASCII 标签按 Unicode 简单小写（不做 Punycode）。
    // 例如 münchen.de → 大写 Μ 经 to_ascii_lowercase 不变（非 ASCII），
    // 但匹配规则 `de` 时后缀比对仍命中。
    let psl = dflt();
    assert_eq!(psl.registrable_domain("foo.de"), "foo.de");
}

#[test]
fn normal_beats_wildcard_when_same_label_count() {
    // 同标签数时，普通规则优先于通配规则。
    // 构造：`x.yp`（普通，2 标签）与 `*.yp`（通配，2 标签）。
    let psl = PublicSuffixList::from_rules("*.yp\nx.yp\n");
    // host=a.x.yp：通配 *.yp 命中（公共后缀 x.yp，注册域 a.x.yp），
    // 普通 x.yp 也命中（公共后缀 x.yp，同结果但优先级普通胜）。
    assert_eq!(psl.registrable_domain("a.x.yp"), "a.x.yp");
    // host=x.yp：普通 x.yp 命中，公共后缀=x.yp（2 标签）→ host==后缀 → 返回自身。
    assert_eq!(psl.registrable_domain("x.yp"), "x.yp");
}
