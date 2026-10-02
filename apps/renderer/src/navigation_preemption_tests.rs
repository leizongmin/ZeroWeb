//! ZRG-2026-10-03-01 回归钉：导航抢占提升时丢弃陈旧加载命令。
//!
//! 04b4f8ed2 fix#14 的抢占只提升了 Navigate，仍按 FIFO 派发先于它入队的
//! LoadHtml——旧文档 load 覆盖在途新文档 pending load，其迟到生命周期报告被宿主
//! pending 校验拒收，导航永不提交、零帧产出（parity smoke 首帧 4/4 超时）。
//! 本组测试钉住 [`RendererRuntime::promote_pending_navigation`] 的三点契约：
//! 陈旧加载命令丢弃、非加载消息（输入等）保序保留、无导航时原样返回。

use super::*;

fn nav(url: &str, epoch: u64) -> IpcMessage {
    IpcMessage {
        id: 0,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: url.to_string(),
            referrer: None,
            navigation_epoch: epoch,
        }),
    }
}

fn load_html(url: &str, epoch: u64) -> IpcMessage {
    IpcMessage {
        id: 0,
        kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
            html: format!("<html>{url}</html>"),
            css: None,
            url: Some(url.to_string()),
            navigation_epoch: epoch,
        }),
    }
}

fn key_event() -> IpcMessage {
    IpcMessage {
        id: 0,
        kind: IpcMessageKind::GoBack,
    }
}

fn url_of(msg: &IpcMessage) -> &str {
    match &msg.kind {
        IpcMessageKind::Navigate(p) => &p.url,
        IpcMessageKind::LoadHtml(p) => p.url.as_deref().unwrap_or(""),
        _ => "",
    }
}

/// smoke 复现形：队首 LoadHtml(welcome)，积压中 arriving Navigate(fixture) —— fixture
/// 胜出，welcome LoadHtml 按陈旧丢弃（否则覆盖在途 fixture 加载 → 零帧超时）。
#[test]
fn promoted_navigation_drops_stale_queued_load() {
    let mut deferred = VecDeque::from([load_html("zero://newtab", 1)]);
    let head = Some(load_html("about:blank", 0));
    let next =
        RendererRuntime::promote_pending_navigation(head, &mut deferred, || vec![nav("file:///fixture.html", 2)]);
    let next = next.expect("Navigate 应被提升为下一派发对象");
    assert_eq!(url_of(&next), "file:///fixture.html");
    assert!(
        deferred.is_empty(),
        "陈旧 welcome LoadHtml 应被丢弃，实际 {:?}",
        deferred.iter().map(url_of).collect::<Vec<_>>()
    );
}

/// 输入等非加载消息不被丢弃，且保持相对顺序位于提升导航之后继续处理（fix#14 语义）。
#[test]
fn promoted_navigation_preserves_non_load_messages_in_order() {
    let mut deferred = VecDeque::from([key_event(), load_html("zero://newtab", 1)]);
    let head = Some(key_event());
    let next =
        RendererRuntime::promote_pending_navigation(head, &mut deferred, || vec![nav("file:///fixture.html", 2)]);
    assert_eq!(url_of(next.as_ref().expect("导航被提升")), "file:///fixture.html");
    let kept: Vec<&str> = deferred.iter().map(url_of).collect();
    assert_eq!(kept, vec!["", ""], "两个非加载消息应保留");
    assert!(
        matches!(deferred[0].kind, IpcMessageKind::GoBack) && matches!(deferred[1].kind, IpcMessageKind::GoBack),
        "保留消息相对顺序不变"
    );
}

/// 积压与入站均无导航：head 原样返回，排干消息追加不丢。
#[test]
fn no_pending_navigation_returns_head_and_keeps_drained() {
    let mut deferred = VecDeque::from([key_event()]);
    let head = Some(load_html("zero://newtab", 1));
    let next = RendererRuntime::promote_pending_navigation(head, &mut deferred, || vec![key_event()]);
    assert!(matches!(
        next.as_ref().expect("head 原样").kind,
        IpcMessageKind::LoadHtml(_)
    ));
    assert_eq!(deferred.len(), 2, "排干消息应追加至积压尾");
}

/// 队首已是导航命令：不抢占、不排干、原样返回（后续导航 FIFO 依序处理）。
#[test]
fn navigation_head_short_circuits_without_draining() {
    let mut deferred = VecDeque::from([load_html("zero://newtab", 1)]);
    let head = Some(nav("file:///first.html", 1));
    let mut drained = 0;
    let next = RendererRuntime::promote_pending_navigation(head, &mut deferred, || {
        drained += 1;
        Vec::new()
    });
    assert_eq!(url_of(&next.expect("head 原样")), "file:///first.html");
    assert_eq!(drained, 0, "队首即导航时不排干入站");
    assert_eq!(deferred.len(), 1, "积压保持原样");
}

/// head 为空且积压中有导航：导航应从积压中提升（非阻塞排干后 recv 空转的窗口）。
#[test]
fn empty_head_promotes_navigation_from_deferred() {
    let mut deferred = VecDeque::from([load_html("zero://newtab", 1), nav("file:///f.html", 2)]);
    let next = RendererRuntime::promote_pending_navigation(None, &mut deferred, Vec::new);
    assert_eq!(url_of(&next.expect("积压中导航被提升")), "file:///f.html");
    assert!(deferred.is_empty(), "陈旧 welcome LoadHtml 随提升一并丢弃");
}
