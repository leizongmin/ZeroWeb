//! DownloadManager 单元测试。

use crate::*;

// ── DownloadManager 测试 ──

#[test]
fn test_download_manager_new() {
    let dm = DownloadManager::new();
    assert!(dm.is_empty());
    assert_eq!(dm.len(), 0);
}

#[test]
fn test_download_start() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    assert_eq!(dm.len(), 1);
    let entry = dm.get(id).unwrap();
    assert_eq!(entry.url(), "https://example.com/file.zip");
    assert_eq!(entry.filename(), "file.zip");
    assert_eq!(entry.state(), DownloadState::Pending);
    assert_eq!(entry.downloaded_bytes(), 0);
    assert!(entry.total_bytes().is_none());
}

#[test]
fn test_download_update_progress() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.update_progress(id, 500, Some(1000));
    let entry = dm.get(id).unwrap();
    assert_eq!(entry.state(), DownloadState::Downloading);
    assert_eq!(entry.downloaded_bytes(), 500);
    assert_eq!(entry.total_bytes(), Some(1000));
    assert!((entry.progress() - 0.5).abs() < 0.01);
}

#[test]
fn test_download_mark_completed() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.update_progress(id, 1000, Some(1000));
    dm.mark_completed(id);
    let entry = dm.get(id).unwrap();
    assert!(entry.is_completed());
    assert_eq!(entry.state(), DownloadState::Completed);
}

#[test]
fn test_download_pause_resume() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.update_progress(id, 500, Some(1000));
    dm.pause(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Paused);
    dm.resume(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Downloading);
}

#[test]
fn test_download_cancel() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.cancel(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Cancelled);
}

#[test]
fn test_download_mark_failed() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.mark_failed(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Failed);
}

#[test]
fn test_download_remove_completed() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    assert!(!dm.remove(id), "Cannot remove active download");
    dm.mark_completed(id);
    assert!(dm.remove(id), "Should remove completed download");
    assert!(dm.is_empty());
}

#[test]
fn test_download_clear_completed() {
    let mut dm = DownloadManager::new();
    let id1 = dm.start_download("https://a.com/file1.zip", "file1.zip");
    let id2 = dm.start_download("https://b.com/file2.zip", "file2.zip");
    let id3 = dm.start_download("https://c.com/file3.zip", "file3.zip");
    dm.mark_completed(id1);
    dm.cancel(id2);
    // id3 is still pending
    dm.clear_completed();
    assert_eq!(dm.len(), 1);
    assert!(dm.get(id3).is_some());
}

#[test]
fn test_download_active_count() {
    let mut dm = DownloadManager::new();
    let id1 = dm.start_download("https://a.com/f1", "f1");
    let _id2 = dm.start_download("https://b.com/f2", "f2");
    assert_eq!(dm.active_count(), 2);
    // Now mark one as completed
    dm.update_progress(id1, 100, Some(100));
    dm.mark_completed(id1);
    assert_eq!(dm.active_count(), 1);
}

#[test]
fn test_download_progress_unknown_size() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.update_progress(id, 500, None);
    let entry = dm.get(id).unwrap();
    assert_eq!(entry.progress(), 0.0, "Unknown size should return 0 progress");
}

#[test]
fn test_download_is_active() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    assert!(dm.get(id).unwrap().is_active());
    dm.pause(id);
    assert!(!dm.get(id).unwrap().is_active());
}

// ── 下载管理器边界测试 ──

#[test]
fn test_download_pause_completed_is_noop() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.mark_completed(id);
    dm.pause(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Completed);
}

#[test]
fn test_download_resume_not_paused_is_noop() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.resume(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Pending);
}

#[test]
fn test_download_cancel_completed_is_noop() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.mark_completed(id);
    dm.cancel(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Completed);
}

#[test]
fn test_download_mark_failed_completed() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    dm.mark_completed(id);
    // 标记已完成的下载为失败（网络错误后覆盖）
    dm.mark_failed(id);
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Failed);
}

#[test]
fn test_download_remove_active_is_noop() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    assert!(!dm.remove(id), "不能移除活跃下载");
    assert_eq!(dm.len(), 1);
}

#[test]
fn test_download_remove_nonexistent() {
    let mut dm = DownloadManager::new();
    assert!(!dm.remove(DownloadId(999)));
}

#[test]
fn test_download_multiple_downloads_active_count() {
    let mut dm = DownloadManager::new();
    let id1 = dm.start_download("https://a.com/a.zip", "a.zip");
    let id2 = dm.start_download("https://b.com/b.zip", "b.zip");
    let id3 = dm.start_download("https://c.com/c.zip", "c.zip");
    assert_eq!(dm.active_count(), 3);
    dm.mark_completed(id1);
    assert_eq!(dm.active_count(), 2);
    dm.cancel(id2);
    assert_eq!(dm.active_count(), 1);
    dm.pause(id3);
    assert_eq!(dm.active_count(), 0, "暂停的下载不是活跃的");
}

#[test]
fn test_download_update_progress_transitions_pending_to_downloading() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://example.com/file.zip", "file.zip");
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Pending);
    dm.update_progress(id, 1024, Some(4096));
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Downloading);
    assert_eq!(dm.get(id).unwrap().downloaded_bytes(), 1024);
    assert_eq!(dm.get(id).unwrap().total_bytes(), Some(4096));
}

#[test]
fn test_download_clear_completed_keeps_active() {
    let mut dm = DownloadManager::new();
    let id1 = dm.start_download("https://a.com/a.zip", "a.zip");
    let id2 = dm.start_download("https://b.com/b.zip", "b.zip");
    dm.mark_completed(id1);
    dm.clear_completed();
    assert_eq!(dm.len(), 1);
    assert_eq!(dm.get(id2).unwrap().state(), DownloadState::Pending);
}

// ── Download 边界测试 ──

#[test]
fn test_download_progress_zero_total() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://x.com/f", "f.bin");
    dm.update_progress(id, 500, Some(0));
    // 零 total_bytes 不应 panic
    assert_eq!(dm.get(id).unwrap().state(), DownloadState::Downloading);
}

#[test]
fn test_download_remove_failed() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://x.com/bad", "bad.dat");
    dm.mark_failed(id);
    assert!(dm.remove(id), "应能移除 failed 下载");
    assert!(dm.get(id).is_none());
}

// ── Download 迭代与边界 ──

#[test]
/// 测试 DownloadManager::iter() 迭代多个下载。
fn test_download_iter_multiple() {
    let mut dm = DownloadManager::new();
    dm.start_download("https://a.com/f1", "f1.bin");
    dm.start_download("https://b.com/f2", "f2.bin");
    dm.start_download("https://c.com/f3", "f3.bin");
    let urls: Vec<&str> = dm.iter().map(|d| d.url()).collect();
    assert_eq!(urls.len(), 3);
    assert_eq!(urls[0], "https://a.com/f1");
    assert_eq!(urls[2], "https://c.com/f3");
}

#[test]
/// 测试 DownloadManager::default() 产生空管理器。
fn test_download_default() {
    let dm = DownloadManager::default();
    assert!(dm.is_empty());
    assert_eq!(dm.len(), 0);
}

#[test]
/// 测试 DownloadEntry::progress() 在 total=0 时返回 0.0。
fn test_download_progress_return_value_zero_total() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://x.com/f", "f.bin");
    dm.update_progress(id, 500, Some(0));
    assert_eq!(dm.get(id).unwrap().progress(), 0.0);
}

#[test]
/// 测试 update_progress 对不存在的 DownloadId 静默忽略。
fn test_download_update_progress_nonexistent_id() {
    let mut dm = DownloadManager::new();
    dm.update_progress(DownloadId(99999), 100, Some(200));
    assert!(dm.is_empty());
}

#[test]
/// 测试 mark_completed 设置 downloaded_bytes = total_bytes。
fn test_download_mark_completed_sets_bytes() {
    let mut dm = DownloadManager::new();
    let id = dm.start_download("https://x.com/f", "f.bin");
    dm.update_progress(id, 500, Some(1000));
    dm.mark_completed(id);
    assert!(dm.get(id).unwrap().is_completed());
}

#[test]
/// 测试 DownloadManager 移除不存在的下载不 panic。
fn test_download_manager_remove_nonexistent() {
    let mut dm = DownloadManager::new();
    let fake_id = DownloadId(99999);
    dm.remove(fake_id); // 不存在的 id
    assert_eq!(dm.active_count(), 0);
}
